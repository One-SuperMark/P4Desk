/// Orders touch JSON's full UInt16 sequence independently of the wire header's 10 bits.
/// Distances at least half the counter range are stale or ambiguous and are rejected.
public struct TouchSequenceFilter {
    private var last: UInt16?
    public init() {}
    public mutating func accept(_ sequence: UInt16) -> Bool {
        if let last {
            let distance = sequence &- last
            guard distance != 0, distance < 0x8000 else { return false }
        }
        last = sequence
        return true
    }
    public mutating func reset() { last = nil }
}
