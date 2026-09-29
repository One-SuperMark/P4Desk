import Foundation

/// A queue-owned gate with two frames of burst allowance. Timestamp jitter at
/// the capture interval boundary must not halve an otherwise steady 60 Hz feed.
public struct CaptureRateGate {
    private static let frameCredit: UInt64 = 1_000_000_000
    private static let maximumCredit: UInt64 = 2_000_000_000
    private let framesPerSecond: UInt64
    private var active = false
    private var lastObservedNS: UInt64?
    private var credit: UInt64 = 0

    public init(framesPerSecond: UInt64 = 60) {
        precondition(framesPerSecond > 0 && framesPerSecond <= Self.frameCredit)
        self.framesPerSecond = framesPerSecond
    }
    public mutating func start() {
        active = true; lastObservedNS = nil; credit = Self.maximumCredit
    }
    public mutating func stop() {
        active = false; lastObservedNS = nil; credit = 0
    }
    public mutating func accept(presentationNS: UInt64) -> Bool {
        guard active else { return false }
        if let previous = lastObservedNS {
            guard presentationNS > previous else { return false }
            let elapsed = presentationNS - previous
            let needed = Self.maximumCredit - credit
            let refillNS = needed / framesPerSecond + (needed % framesPerSecond == 0 ? 0 : 1)
            if elapsed >= refillNS {
                credit = Self.maximumCredit
            } else {
                // elapsed is now below the amount needed to fill at most two
                // frames. Neither an absolute timestamp nor an unbounded gap
                // is multiplied, even when presentationNS is UInt64.max.
                credit += elapsed * framesPerSecond
            }
        }
        // Remember every fresh observation, including rate-limited samples,
        // so a rejected timestamp cannot refill the same elapsed time twice.
        lastObservedNS = presentationNS
        guard credit >= Self.frameCredit else { return false }
        credit -= Self.frameCredit
        return true
    }
}
