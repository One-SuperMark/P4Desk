import CoreMedia

public enum CaptureTimestampSource: String, Codable {
    case displayTime = "display_time"
    case samplePTS = "sample_pts"
    case callback
}

public struct CaptureTimestamp: Equatable {
    public let capturedNS: UInt64
    public let source: CaptureTimestampSource
    public var usesTimestamp: Bool { source != .callback }

    public init(capturedNS: UInt64, source: CaptureTimestampSource) {
        self.capturedNS = capturedNS; self.source = source
    }
}

/// Resolves SCK timestamps in the host monotonic clock. displayTime is native
/// mach units, whereas receivedNS is DispatchTime uptime in nanoseconds.
public enum CaptureTimestampResolver {
    public static func resolve(displayTime: UInt64?, samplePTS: CMTime, receivedNS: UInt64) -> CaptureTimestamp {
        if let displayTime, displayTime > 0 {
            let hostTime = CMClockMakeHostTimeFromSystemUnits(displayTime)
            if let ns = validHostNanoseconds(hostTime, receivedNS: receivedNS) {
                return CaptureTimestamp(capturedNS: ns, source: .displayTime)
            }
        }
        if let ns = validHostNanoseconds(samplePTS, receivedNS: receivedNS) {
            return CaptureTimestamp(capturedNS: ns, source: .samplePTS)
        }
        return CaptureTimestamp(capturedNS: receivedNS, source: .callback)
    }

    private static func validHostNanoseconds(_ time: CMTime, receivedNS: UInt64) -> UInt64? {
        // CMTime has no clock identifier. The caller supplies SCK's host PTS;
        // a different epoch must never be subtracted from the host callback.
        guard time.isNumeric, time.value > 0, time.timescale > 0, time.epoch == 0 else { return nil }
        let converted = CMTimeConvertScale(time, timescale: 1_000_000_000, method: .default)
        guard converted.isNumeric, converted.value > 0, converted.timescale == 1_000_000_000,
              converted.epoch == 0 else { return nil }
        let ns = UInt64(converted.value)
        // Only subtract after conversion into the same unit and checking the
        // ordering. Raw mach ticks must never enter this nanosecond difference.
        guard ns <= receivedNS, receivedNS - ns < 5_000_000_000 else { return nil }
        return ns
    }
}
