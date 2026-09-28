import Foundation

/// Monotonic metadata only. No pixel data, user text, device identity, or wall-clock values.
public struct FrameTiming: Codable, Equatable {
    public var sequence: UInt16
    public var capturedNS: UInt64
    public var encodeStartedNS: UInt64
    public var encodedNS: UInt64
    public var enqueuedNS: UInt64
    public var usbSentNS: UInt64?
    public var receiptNS: UInt64?
    public var devicePresentedUS: UInt64?
    public var captureUsesPresentationTimestamp: Bool
    public init(sequence: UInt16, capturedNS: UInt64, encodeStartedNS: UInt64, encodedNS: UInt64,
                enqueuedNS: UInt64, captureUsesPresentationTimestamp: Bool) {
        self.sequence = sequence; self.capturedNS = capturedNS; self.encodeStartedNS = encodeStartedNS
        self.encodedNS = encodedNS; self.enqueuedNS = enqueuedNS
        self.captureUsesPresentationTimestamp = captureUsesPresentationTimestamp
    }
    public var queueToReceiptMS: Double? { receiptNS.flatMap { Self.ms(enqueuedNS, $0) } }
    fileprivate static func ms(_ start: UInt64, _ end: UInt64) -> Double? {
        end >= start ? Double(end - start) / 1_000_000 : nil
    }
    fileprivate var ordered: Bool {
        guard capturedNS <= encodeStartedNS, encodeStartedNS <= encodedNS, encodedNS <= enqueuedNS,
              let receiptNS, enqueuedNS <= receiptNS else { return false }
        if let usbSentNS { return enqueuedNS <= usbSentNS && usbSentNS <= receiptNS }
        return true
    }
}

public struct PerformanceReport: Codable {
    public var windowSeconds: Double
    public var sampleCount: Int
    public var usbCompletedSamples: Int
    public var effectivePresentedFPS: Double
    public var captureToReceiptP95MS: Double?
    public var captureToEncodeStartP95MS: Double?
    public var jpegEncodeP95MS: Double?
    public var encodedToEnqueueP95MS: Double?
    public var queueToUSBSentP95MS: Double?
    public var usbSentToReceiptP95MS: Double?
    public var queueToReceiptP95MS: Double?
    public var frames: [FrameTiming]
}

public struct PerformanceWindow {
    private var frames: [FrameTiming] = []
    private let capacity: Int
    private let windowNS: UInt64
    public init(seconds: UInt64 = 30, capacity: Int = 900) {
        windowNS = seconds * 1_000_000_000; self.capacity = capacity
    }
    public mutating func reset() { frames.removeAll(keepingCapacity: true) }
    @discardableResult
    public mutating func record(_ frame: FrameTiming) -> Bool {
        guard frame.ordered, let time = frame.receiptNS else { return false }
        frames.removeAll { time >= ($0.receiptNS ?? 0) && time - ($0.receiptNS ?? 0) > windowNS }
        frames.append(frame)
        if frames.count > capacity { frames.removeFirst(frames.count - capacity) }
        return true
    }
    public mutating func markSent(sequence: UInt16, timeNS: UInt64) {
        guard let index = frames.lastIndex(where: { $0.sequence == sequence }),
              let receipt = frames[index].receiptNS, timeNS >= frames[index].enqueuedNS, timeNS <= receipt else { return }
        frames[index].usbSentNS = timeNS
    }
    public func report(nowNS: UInt64) -> PerformanceReport {
        let recent = frames.filter { guard let t = $0.receiptNS else { return false }; return nowNS >= t && nowNS - t <= windowNS }
        let duration = recent.first?.receiptNS.flatMap { first in recent.last?.receiptNS.flatMap { FrameTiming.ms(first, $0) } } ?? 0
        let fps = duration > 0 ? Double(max(0, recent.count - 1)) * 1000 / duration : 0
        func p95(_ values: [Double]) -> Double? {
            guard !values.isEmpty else { return nil }
            let sorted = values.sorted(), index = max(0, Int(ceil(Double(values.count) * 0.95)) - 1)
            return sorted[index]
        }
        return PerformanceReport(windowSeconds: Double(windowNS) / 1_000_000_000, sampleCount: recent.count,
            usbCompletedSamples: recent.filter { $0.usbSentNS != nil }.count, effectivePresentedFPS: fps,
            captureToReceiptP95MS: p95(recent.compactMap { f in f.receiptNS.flatMap { FrameTiming.ms(f.capturedNS, $0) } }),
            captureToEncodeStartP95MS: p95(recent.compactMap { FrameTiming.ms($0.capturedNS, $0.encodeStartedNS) }),
            jpegEncodeP95MS: p95(recent.compactMap { FrameTiming.ms($0.encodeStartedNS, $0.encodedNS) }),
            encodedToEnqueueP95MS: p95(recent.compactMap { FrameTiming.ms($0.encodedNS, $0.enqueuedNS) }),
            queueToUSBSentP95MS: p95(recent.compactMap { f in f.usbSentNS.flatMap { FrameTiming.ms(f.enqueuedNS, $0) } }),
            usbSentToReceiptP95MS: p95(recent.compactMap { f in f.usbSentNS.flatMap { sent in f.receiptNS.flatMap { FrameTiming.ms(sent, $0) } } }),
            queueToReceiptP95MS: p95(recent.compactMap { $0.queueToReceiptMS }), frames: recent)
    }
}
