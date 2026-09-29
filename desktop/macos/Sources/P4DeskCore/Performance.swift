import Foundation
import CoreFoundation

/// Accept JSON integers without rounding UInt64.max through a Double or treating Bool as 0/1.
public enum ControlNumber {
    public static func unsigned(_ value: Any?) -> UInt64? {
        guard let number = value as? NSNumber, CFGetTypeID(number) != CFBooleanGetTypeID() else { return nil }
        if let integer = UInt64(number.stringValue) { return integer }
        let floating = number.doubleValue
        guard floating.isFinite, floating >= 0, floating.rounded(.towardZero) == floating,
              floating < 18_446_744_073_709_551_616.0 else { return nil }
        return UInt64(floating)
    }
}

/// Optional additions to frame_presented. Older firmware supplies none of these values.
public struct DeviceFrameMetrics: Codable, Equatable {
    public let jpegBytes: UInt64?
    public let decodeUS: UInt64?
    public let copyUS: UInt64?
    public let presentUS: UInt64?
    public init(fields: [String: Any]) {
        func positive(_ key: String) -> UInt64? {
            ControlNumber.unsigned(fields[key]).flatMap { $0 > 0 ? $0 : nil }
        }
        jpegBytes = positive("jpeg_bytes"); decodeUS = positive("decode_us")
        // Direct JPEG-to-LCD decoding performs no copy. An explicit zero is a
        // measured stage; a missing or invalid field remains unknown.
        copyUS = ControlNumber.unsigned(fields["copy_us"]); presentUS = positive("present_us")
    }
}

/// Monotonic metadata only. No pixel data, user text, device identity, or wall-clock values.
public struct FrameTiming: Codable, Equatable {
    public var sequence: UInt16
    public var capturedNS: UInt64
    public var encodeStartedNS: UInt64
    public var encodedNS: UInt64
    public var enqueuedNS: UInt64
    /// JPEG payload bytes, excluding the 16-byte P4Desk header.
    public var jpegBytes: UInt64
    public var usbSentNS: UInt64?
    /// Native duration from submitting the first OUT chunk to completing the last chunk.
    public var usbTransferUS: UInt64?
    public var receiptNS: UInt64?
    public var devicePresentedUS: UInt64?
    public var deviceMetrics: DeviceFrameMetrics?
    public var captureUsesPresentationTimestamp: Bool
    public var captureTimestampSource: CaptureTimestampSource?
    /// ImageIO settings and parsed SOF metadata for this frame, not the latest session setting.
    public var jpegQuality: Double?
    public var jpegSampling: JPEGSampling?
    public var jpegEncodingAttempts: Int?
    public var jpegPayloadFallback: Bool?
    public init(sequence: UInt16, capturedNS: UInt64, encodeStartedNS: UInt64, encodedNS: UInt64,
                enqueuedNS: UInt64, captureUsesPresentationTimestamp: Bool, jpegBytes: UInt64 = 0,
                captureTimestampSource: CaptureTimestampSource? = nil, jpegQuality: Double? = nil,
                jpegSampling: JPEGSampling? = nil, jpegEncodingAttempts: Int? = nil, jpegPayloadFallback: Bool? = nil) {
        self.sequence = sequence; self.capturedNS = capturedNS; self.encodeStartedNS = encodeStartedNS
        self.encodedNS = encodedNS; self.enqueuedNS = enqueuedNS; self.jpegBytes = jpegBytes
        self.captureUsesPresentationTimestamp = captureUsesPresentationTimestamp
        self.captureTimestampSource = captureTimestampSource
        self.jpegQuality = jpegQuality; self.jpegSampling = jpegSampling
        self.jpegEncodingAttempts = jpegEncodingAttempts; self.jpegPayloadFallback = jpegPayloadFallback
    }
    public var queueToReceiptMS: Double? { receiptNS.flatMap { Self.ms(enqueuedNS, $0) } }
    public var usbTransferMS: Double? {
        guard let usbTransferUS, let usbSentNS, let elapsed = Self.ms(enqueuedNS, usbSentNS),
              Double(usbTransferUS) / 1000 <= elapsed + 0.001 else { return nil }
        return Double(usbTransferUS) / 1000
    }
    public var usbQueueWaitMS: Double? {
        guard let usbSentNS, let elapsed = Self.ms(enqueuedNS, usbSentNS), let usbTransferMS else { return nil }
        return max(0, elapsed - usbTransferMS)
    }
    fileprivate static func ms(_ start: UInt64, _ end: UInt64) -> Double? {
        end >= start ? Double(end - start) / 1_000_000 : nil
    }
    fileprivate var ordered: Bool {
        guard capturedNS <= encodeStartedNS, encodeStartedNS <= encodedNS, encodedNS <= enqueuedNS,
              let receiptNS, enqueuedNS <= receiptNS else { return false }
        if let usbSentNS { return enqueuedNS <= usbSentNS }
        return true
    }
}

public struct FrameTransfer: Codable, Equatable {
    public var sequence: UInt16
    public var enqueuedNS: UInt64
    public var sentNS: UInt64
    public var jpegBytes: UInt64
    public var transferUS: UInt64?
    public var wireBytes: UInt64 { jpegBytes + 16 }
    public var transferMS: Double? {
        guard let transferUS, let total = FrameTiming.ms(enqueuedNS, sentNS),
              Double(transferUS) / 1000 <= total + 0.001 else { return nil }
        return Double(transferUS) / 1000
    }
    public var queueWaitMS: Double? {
        guard let transferMS, let total = FrameTiming.ms(enqueuedNS, sentNS) else { return nil }
        return max(0, total - transferMS)
    }
}

public struct PerformanceReport: Codable {
    public var windowSeconds: Double
    public var observationSeconds: Double
    public var sampleCount: Int
    public var usbCompletedSamples: Int
    public var transmittedSampleCount: Int
    public var deviceTimingSamples: Int
    public var effectivePresentedFPS: Double
    /// Successful JPEG packets including 16-byte headers; excludes control/HID traffic.
    public var jpegWireBytesPerSecond: Double
    public var jpegMeanBytes: Double?
    public var jpegP95Bytes: Double?
    public var deviceJPEGByteMismatchCount: Int
    public var captureToReceiptP95MS: Double?
    public var captureToEncodeStartP95MS: Double?
    public var jpegEncodeP95MS: Double?
    public var encodedToEnqueueP95MS: Double?
    public var queueToUSBSentP95MS: Double?
    public var usbQueueWaitP95MS: Double?
    public var usbTransferP95MS: Double?
    public var usbSentToReceiptP95MS: Double?
    public var queueToReceiptP95MS: Double?
    public var deviceDecodeP95MS: Double?
    public var deviceCopyP95MS: Double?
    public var devicePresentP95MS: Double?
    public var frames: [FrameTiming]
    public var transfers: [FrameTransfer]
}

public struct PerformanceWindow {
    private var frames: [FrameTiming] = []
    private var transfers: [FrameTransfer] = []
    private let capacity: Int
    private let windowNS: UInt64
    private var startedNS: UInt64?
    private var stopped = false
    public init(seconds: UInt64 = 30, capacity: Int = 1800) {
        precondition(seconds > 0 && seconds <= UInt64.max / 1_000_000_000 && capacity > 0)
        windowNS = seconds * 1_000_000_000; self.capacity = capacity
    }
    public mutating func reset(nowNS: UInt64? = nil) {
        frames.removeAll(keepingCapacity: true); transfers.removeAll(keepingCapacity: true)
        startedNS = nowNS; stopped = false
    }
    /// Preserve latency samples while making a stopped stream's rate immediately zero.
    public mutating func stop() { stopped = true }
    private mutating func observe(_ timeNS: UInt64) {
        if startedNS == nil { startedNS = timeNS }
    }
    @discardableResult
    public mutating func record(_ frame: FrameTiming) -> Bool {
        guard !stopped, frame.ordered, let time = frame.receiptNS else { return false }
        observe(frame.enqueuedNS)
        frames.removeAll { time >= ($0.receiptNS ?? 0) && time - ($0.receiptNS ?? 0) > windowNS }
        frames.append(frame)
        if frames.count > capacity { frames.removeFirst(frames.count - capacity) }
        return true
    }
    /// Records every completed JPEG send, including frames the panel later supersedes.
    public mutating func recordTransfer(_ frame: FrameTiming) {
        guard !stopped, let time = frame.usbSentNS, time >= frame.enqueuedNS,
              frame.jpegBytes > 0, frame.jpegBytes <= 1_048_576 else { return }
        observe(frame.enqueuedNS)
        transfers.removeAll { time >= $0.sentNS && time - $0.sentNS > windowNS }
        guard !transfers.contains(where: { $0.sequence == frame.sequence && $0.sentNS == time }) else { return }
        transfers.append(FrameTransfer(sequence: frame.sequence, enqueuedNS: frame.enqueuedNS, sentNS: time,
                                       jpegBytes: frame.jpegBytes, transferUS: frame.usbTransferUS))
        if transfers.count > capacity { transfers.removeFirst(transfers.count - capacity) }
    }
    public mutating func markSent(sequence: UInt16, timeNS: UInt64, transferUS: UInt64? = nil) {
        guard let index = frames.lastIndex(where: { $0.sequence == sequence }),
              timeNS >= frames[index].enqueuedNS else { return }
        frames[index].usbSentNS = timeNS; frames[index].usbTransferUS = transferUS
        recordTransfer(frames[index])
    }
    public func report(nowNS: UInt64) -> PerformanceReport {
        let recent = frames.filter { guard let t = $0.receiptNS else { return false }; return nowNS >= t && nowNS - t <= windowNS }
        let sent = transfers.filter { nowNS >= $0.sentNS && nowNS - $0.sentNS <= windowNS }
        let elapsedNS = startedNS.map { nowNS >= $0 ? min(nowNS - $0, windowNS) : 0 } ?? 0
        let seconds = Double(elapsedNS) / 1_000_000_000
        // Include time since the last ACK, so idle displays do not retain an old burst's FPS.
        let fps = !stopped && seconds > 0 ? Double(recent.count) / seconds : 0
        let byteRate = !stopped && seconds > 0 ? sent.reduce(0.0) { $0 + Double($1.wireBytes) } / seconds : 0
        func p95(_ values: [Double]) -> Double? {
            guard !values.isEmpty else { return nil }
            let sorted = values.sorted(), index = max(0, Int(ceil(Double(values.count) * 0.95)) - 1)
            return sorted[index]
        }
        let sizes = sent.map { Double($0.jpegBytes) }
        let measured = recent.filter { f in
            guard let m = f.deviceMetrics else { return false }
            return m.decodeUS != nil && m.copyUS != nil && m.presentUS != nil
        }
        return PerformanceReport(windowSeconds: Double(windowNS) / 1_000_000_000, observationSeconds: seconds,
            sampleCount: recent.count, usbCompletedSamples: recent.filter { $0.usbSentNS != nil }.count,
            transmittedSampleCount: sent.count, deviceTimingSamples: measured.count, effectivePresentedFPS: fps,
            jpegWireBytesPerSecond: byteRate, jpegMeanBytes: sizes.isEmpty ? nil : sizes.reduce(0, +) / Double(sizes.count),
            jpegP95Bytes: p95(sizes), deviceJPEGByteMismatchCount: recent.filter { f in
                guard let device = f.deviceMetrics?.jpegBytes else { return false }; return device != f.jpegBytes
            }.count,
            captureToReceiptP95MS: p95(recent.compactMap { f in f.receiptNS.flatMap { FrameTiming.ms(f.capturedNS, $0) } }),
            captureToEncodeStartP95MS: p95(recent.compactMap { FrameTiming.ms($0.capturedNS, $0.encodeStartedNS) }),
            jpegEncodeP95MS: p95(recent.compactMap { FrameTiming.ms($0.encodeStartedNS, $0.encodedNS) }),
            encodedToEnqueueP95MS: p95(recent.compactMap { FrameTiming.ms($0.encodedNS, $0.enqueuedNS) }),
            queueToUSBSentP95MS: p95(sent.isEmpty
                ? recent.compactMap { f in f.usbSentNS.flatMap { FrameTiming.ms(f.enqueuedNS, $0) } }
                : sent.compactMap { FrameTiming.ms($0.enqueuedNS, $0.sentNS) }),
            usbQueueWaitP95MS: p95(sent.compactMap { $0.queueWaitMS }),
            usbTransferP95MS: p95(sent.compactMap { $0.transferMS }),
            usbSentToReceiptP95MS: p95(recent.compactMap { f in f.usbSentNS.flatMap { sent in f.receiptNS.flatMap { FrameTiming.ms(sent, $0) } } }),
            queueToReceiptP95MS: p95(recent.compactMap { $0.queueToReceiptMS }),
            deviceDecodeP95MS: p95(recent.compactMap { $0.deviceMetrics?.decodeUS.map { Double($0) / 1000 } }),
            deviceCopyP95MS: p95(recent.compactMap { $0.deviceMetrics?.copyUS.map { Double($0) / 1000 } }),
            devicePresentP95MS: p95(recent.compactMap { $0.deviceMetrics?.presentUS.map { Double($0) / 1000 } }),
            frames: recent, transfers: sent)
    }
}
