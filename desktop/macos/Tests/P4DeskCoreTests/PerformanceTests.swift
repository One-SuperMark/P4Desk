import XCTest
@testable import P4DeskCore

final class PerformanceTests: XCTestCase {
    func testStagesP95AndActualPresentedFPS() {
        var window = PerformanceWindow()
        for index in 0..<20 {
            let base = UInt64(index) * 100_000_000
            var frame = FrameTiming(sequence: UInt16(index), capturedNS: base,
                encodeStartedNS: base + 5_000_000, encodedNS: base + 15_000_000,
                enqueuedNS: base + 20_000_000, captureUsesPresentationTimestamp: true)
            frame.usbSentNS = base + 40_000_000
            frame.receiptNS = base + UInt64(50 + index) * 1_000_000
            XCTAssertTrue(window.record(frame))
        }
        let report = window.report(nowNS: 2_000_000_000)
        XCTAssertEqual(report.sampleCount, 20)
        XCTAssertEqual(report.usbCompletedSamples, 20)
        XCTAssertEqual(report.captureToReceiptP95MS, 68)
        XCTAssertEqual(report.jpegEncodeP95MS, 10)
        XCTAssertEqual(report.queueToUSBSentP95MS, 20)
        XCTAssertEqual(report.effectivePresentedFPS, 19_000 / 1919.0, accuracy: 0.00001)
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
}
