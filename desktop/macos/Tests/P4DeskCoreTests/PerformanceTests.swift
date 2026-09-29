import XCTest
@testable import P4DeskCore

final class PerformanceTests: XCTestCase {
    func testStagesP95AndActualPresentedFPS() {
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        for index in 0..<20 {
            let base = UInt64(index) * 100_000_000
            var frame = FrameTiming(sequence: UInt16(index), capturedNS: base,
                encodeStartedNS: base + 5_000_000, encodedNS: base + 15_000_000,
                enqueuedNS: base + 20_000_000, captureUsesPresentationTimestamp: true,
                jpegBytes: UInt64(100_000 + index * 1000))
            frame.usbSentNS = base + 40_000_000
            frame.usbTransferUS = 12_000
            frame.receiptNS = base + UInt64(50 + index) * 1_000_000
            frame.deviceMetrics = DeviceFrameMetrics(fields: ["jpeg_bytes": frame.jpegBytes,
                "decode_us": 1000, "copy_us": 200, "present_us": 33_333])
            window.recordTransfer(frame)
            XCTAssertTrue(window.record(frame))
        }
        let report = window.report(nowNS: 2_000_000_000)
        XCTAssertEqual(report.sampleCount, 20)
        XCTAssertEqual(report.usbCompletedSamples, 20)
        XCTAssertEqual(report.captureToReceiptP95MS, 68)
        XCTAssertEqual(report.jpegEncodeP95MS, 10)
        XCTAssertEqual(report.queueToUSBSentP95MS, 20)
        XCTAssertEqual(report.usbQueueWaitP95MS, 8)
        XCTAssertEqual(report.usbTransferP95MS, 12)
        XCTAssertEqual(report.deviceDecodeP95MS, 1)
        XCTAssertEqual(report.deviceCopyP95MS, 0.2)
        XCTAssertEqual(report.devicePresentP95MS, 33.333)
        XCTAssertEqual(report.jpegMeanBytes, 109_500)
        XCTAssertEqual(report.jpegP95Bytes, 118_000)
        XCTAssertEqual(report.deviceJPEGByteMismatchCount, 0)
        XCTAssertEqual(report.deviceTimingSamples, 20)
        XCTAssertEqual(report.transmittedSampleCount, 20)
        XCTAssertEqual(report.jpegWireBytesPerSecond, 1_095_160)
        XCTAssertEqual(report.effectivePresentedFPS, 10, accuracy: 0.00001)
        // Frame order and monotonic clock matter: a negative stage cannot enter statistics.
        var invalid = FrameTiming(sequence: 50, capturedNS: 20, encodeStartedNS: 10, encodedNS: 30,
                                  enqueuedNS: 40, captureUsesPresentationTimestamp: false)
        invalid.receiptNS = 50
        XCTAssertFalse(window.record(invalid))
    }
    func testSlidingExpiryAndLateUSBCompletion() {
        var window = PerformanceWindow(seconds: 1, capacity: 2)
        var frame = FrameTiming(sequence: 1, capturedNS: 0, encodeStartedNS: 0, encodedNS: 0,
                                enqueuedNS: 10_000_000, captureUsesPresentationTimestamp: false)
        frame.receiptNS = 50_000_000
        XCTAssertTrue(window.record(frame))
        window.markSent(sequence: 1, timeNS: 30_000_000)
        XCTAssertEqual(window.report(nowNS: 50_000_000).usbCompletedSamples, 1)
        XCTAssertEqual(window.report(nowNS: 50_000_000).queueToUSBSentP95MS, 20)
        XCTAssertEqual(window.report(nowNS: 1_050_000_001).sampleCount, 0)
    }

    func testSENTCallbackAfterACKStillCountsSuccessfulUSBBytes() {
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        var frame = FrameTiming(sequence: 1, capturedNS: 0, encodeStartedNS: 1_000_000, encodedNS: 2_000_000,
            enqueuedNS: 10_000_000, captureUsesPresentationTimestamp: false, jpegBytes: 65_536)
        frame.receiptNS = 50_000_000
        XCTAssertTrue(window.record(frame))
        // IN and OUT callbacks may be delivered in this order although bytes already reached the board.
        window.markSent(sequence: 1, timeNS: 55_000_000, transferUS: 40_000)
        let report = window.report(nowNS: 1_000_000_000)
        XCTAssertEqual(report.usbCompletedSamples, 1)
        XCTAssertEqual(report.transmittedSampleCount, 1)
        XCTAssertEqual(report.jpegWireBytesPerSecond, 65_552)
        XCTAssertEqual(report.usbTransferP95MS, 40)
        XCTAssertNil(report.usbSentToReceiptP95MS) // Never publish a negative duration.
    }

    func testIdleTimeAndStopUpdateRatesWithoutNewACK() {
        var window = PerformanceWindow(seconds: 2)
        window.reset(nowNS: 0)
        for index in 0..<20 {
            let time = UInt64(index) * 50_000_000
            var frame = FrameTiming(sequence: UInt16(index), capturedNS: time, encodeStartedNS: time,
                encodedNS: time, enqueuedNS: time, captureUsesPresentationTimestamp: false, jpegBytes: 1000)
            frame.usbSentNS = time + 1_000_000; frame.usbTransferUS = 1000
            frame.receiptNS = time + 2_000_000
            window.recordTransfer(frame)
            XCTAssertTrue(window.record(frame))
        }
        XCTAssertEqual(window.report(nowNS: 1_000_000_000).effectivePresentedFPS, 20)
        XCTAssertEqual(window.report(nowNS: 1_500_000_000).effectivePresentedFPS, 20 / 1.5, accuracy: 0.00001)
        var stopped = window
        stopped.stop()
        let stoppedReport = stopped.report(nowNS: 1_000_000_000)
        XCTAssertEqual(stoppedReport.effectivePresentedFPS, 0)
        XCTAssertEqual(stoppedReport.jpegWireBytesPerSecond, 0)
        XCTAssertEqual(stoppedReport.sampleCount, 20) // Keep samples, never freeze an old FPS.
        XCTAssertNotNil(stoppedReport.captureToReceiptP95MS)
        let expired = window.report(nowNS: 3_000_000_000)
        XCTAssertEqual(expired.effectivePresentedFPS, 0)
        XCTAssertEqual(expired.jpegWireBytesPerSecond, 0)
        XCTAssertEqual(expired.sampleCount, 0)
    }

    func testUSBThroughputIncludesSentFramesWithoutPresentationACK() {
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        var frame = FrameTiming(sequence: 1, capturedNS: 100_000_000, encodeStartedNS: 110_000_000,
            encodedNS: 120_000_000, enqueuedNS: 140_000_000, captureUsesPresentationTimestamp: false,
            jpegBytes: 200_000)
        frame.usbSentNS = 200_000_000; frame.usbTransferUS = 50_000
        window.recordTransfer(frame)
        window.recordTransfer(frame) // Completion duplicates must not inflate throughput.
        let report = window.report(nowNS: 1_000_000_000)
        XCTAssertEqual(report.sampleCount, 0)
        XCTAssertEqual(report.transmittedSampleCount, 1)
        XCTAssertEqual(report.jpegMeanBytes, 200_000) // JPEG size excludes header.
        XCTAssertEqual(report.jpegWireBytesPerSecond, 200_016) // USB throughput includes it.
        XCTAssertEqual(report.queueToUSBSentP95MS, 60)
        XCTAssertEqual(report.usbQueueWaitP95MS, 10)
        XCTAssertEqual(report.usbTransferP95MS, 50)
        XCTAssertEqual(report.effectivePresentedFPS, 0)
        frame.sequence = 2; frame.usbTransferUS = UInt64.max
        window.recordTransfer(frame)
        XCTAssertNil(window.report(nowNS: 1_000_000_000).transfers.last?.transferMS)
    }

    func testOptionalBoardMetricsRemainCompatibleAndPreserveUnsignedIntegers() throws {
        let old = DeviceFrameMetrics(fields: ["op": "frame_presented", "session": 1, "sequence": 1023, "device_us": 55])
        XCTAssertNil(old.jpegBytes); XCTAssertNil(old.decodeUS); XCTAssertNil(old.copyUS); XCTAssertNil(old.presentUS)
        let maximum = try JSONSerialization.jsonObject(with: Data("{\"jpeg_bytes\":18446744073709551615,\"decode_us\":18446744073709551615,\"copy_us\":18446744073709551615,\"present_us\":18446744073709551615}".utf8)) as! [String: Any]
        let values = DeviceFrameMetrics(fields: maximum)
        XCTAssertEqual(values.jpegBytes, UInt64.max); XCTAssertEqual(values.decodeUS, UInt64.max)
        XCTAssertEqual(values.copyUS, UInt64.max); XCTAssertEqual(values.presentUS, UInt64.max)
        let absent = DeviceFrameMetrics(fields: ["jpeg_bytes": 0, "decode_us": false, "copy_us": -1, "present_us": 1.5])
        XCTAssertNil(absent.jpegBytes); XCTAssertNil(absent.decodeUS); XCTAssertNil(absent.copyUS); XCTAssertNil(absent.presentUS)
        XCTAssertNil(ControlNumber.unsigned(NSNumber(value: Double(UInt64.max))))
        XCTAssertEqual(ControlNumber.unsigned(NSNumber(value: 0.0)), 0)
    }

    func testDirectDecodeZeroCopyIsMeasuredWhileMissingLegacyFieldIsUnknown() throws {
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        var frame = FrameTiming(sequence: 1, capturedNS: 0, encodeStartedNS: 1_000_000,
            encodedNS: 2_000_000, enqueuedNS: 3_000_000,
            captureUsesPresentationTimestamp: false, jpegBytes: 32_768)
        frame.receiptNS = 40_000_000
        frame.deviceMetrics = DeviceFrameMetrics(fields: ["op": "frame_presented", "device_us": 55])
        XCTAssertTrue(window.record(frame))
        let legacy = window.report(nowNS: 50_000_000)
        XCTAssertEqual(legacy.deviceTimingSamples, 0)
        XCTAssertNil(legacy.deviceCopyP95MS)
        XCTAssertNil(legacy.frames.first?.deviceMetrics?.copyUS)

        window.reset(nowNS: 0)
        let fields = try JSONSerialization.jsonObject(with: Data(
            "{\"jpeg_bytes\":32768,\"decode_us\":6000,\"copy_us\":0,\"present_us\":16000}".utf8)) as! [String: Any]
        frame.deviceMetrics = DeviceFrameMetrics(fields: fields)
        XCTAssertEqual(frame.deviceMetrics?.copyUS, 0)
        XCTAssertTrue(window.record(frame))
        let direct = window.report(nowNS: 50_000_000)
        XCTAssertEqual(direct.deviceTimingSamples, 1)
        XCTAssertEqual(direct.deviceCopyP95MS, 0)
        XCTAssertEqual(direct.deviceDecodeP95MS, 6)
        XCTAssertEqual(direct.devicePresentP95MS, 16)
        XCTAssertEqual(direct.deviceJPEGByteMismatchCount, 0)
    }

    func testCopyMetricsAcceptNonnegativeIntegersAndRejectInvalidNumbers() {
        let zero = DeviceFrameMetrics(fields: ["jpeg_bytes": 0, "decode_us": 0, "copy_us": 0, "present_us": 0])
        XCTAssertEqual(zero.copyUS, 0)
        XCTAssertNil(zero.jpegBytes); XCTAssertNil(zero.decodeUS); XCTAssertNil(zero.presentUS)
        XCTAssertEqual(DeviceFrameMetrics(fields: ["copy_us": UInt64.max]).copyUS, UInt64.max)
        XCTAssertEqual(DeviceFrameMetrics(fields: ["copy_us": NSNumber(value: 0.0)]).copyUS, 0)

        let invalid: [Any] = [true, false, -1, -0.5, 1.5, NSNumber(value: Double.nan),
            NSNumber(value: Double.infinity), NSNumber(value: -Double.infinity),
            NSNumber(value: Double(UInt64.max)), NSNull(), "0"]
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        for (index, value) in invalid.enumerated() {
            let metrics = DeviceFrameMetrics(fields: ["decode_us": 1000, "copy_us": value, "present_us": 1000])
            XCTAssertNil(metrics.copyUS, "invalid copy_us at index \(index)")
            var frame = FrameTiming(sequence: UInt16(index), capturedNS: 0, encodeStartedNS: 0,
                encodedNS: 0, enqueuedNS: 0, captureUsesPresentationTimestamp: false)
            frame.receiptNS = UInt64(index + 1) * 1_000_000
            frame.deviceMetrics = metrics
            XCTAssertTrue(window.record(frame))
        }
        let report = window.report(nowNS: 20_000_000)
        XCTAssertEqual(report.deviceTimingSamples, 0)
        XCTAssertNil(report.deviceCopyP95MS)
    }

    func testThirtySecondWindowRetainsSixtyFPSAcrossWireSequenceWrap() {
        var window = PerformanceWindow()
        window.reset(nowNS: 0)
        for index in 0..<1800 {
            let time = UInt64(index) * 16_666_666
            var frame = FrameTiming(sequence: UInt16(index & 1023), capturedNS: time, encodeStartedNS: time,
                encodedNS: time, enqueuedNS: time, captureUsesPresentationTimestamp: false, jpegBytes: 1000)
            frame.usbSentNS = time + 1000; frame.receiptNS = time + 2000
            window.recordTransfer(frame)
            XCTAssertTrue(window.record(frame))
        }
        let report = window.report(nowNS: 30_000_000_000)
        XCTAssertEqual(report.sampleCount, 1800)
        XCTAssertEqual(report.transmittedSampleCount, 1800)
        XCTAssertEqual(report.frames[1023].sequence, 1023)
        XCTAssertEqual(report.frames[1024].sequence, 0)
        XCTAssertEqual(report.effectivePresentedFPS, 60)
    }
}
