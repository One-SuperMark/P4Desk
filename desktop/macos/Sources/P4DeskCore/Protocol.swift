import Foundation

public enum ProtocolError: Error, Equatable {
    case invalidHeader, invalidLength, invalidCRC, invalidJSON, invalidSnapshot(String)
}

public enum PacketKind: UInt8 { case jpeg = 3, control = 16, resource = 17 }
public struct Packet: Equatable {
    public var kind: PacketKind
    public var sequence: UInt16
    public var payload: Data
    public init(kind: PacketKind, sequence: UInt16, payload: Data) {
        self.kind = kind; self.sequence = sequence; self.payload = payload
    }
    public func encoded() throws -> Data {
        let limit = kind == .jpeg ? 1_048_576 : kind == .control ? 65_536 : 32_772
        guard sequence < 1024, !payload.isEmpty, payload.count <= limit,
              kind != .resource || payload.count >= 5 else { throw ProtocolError.invalidLength }
        var bytes = [UInt8](repeating: 0, count: 16)
        if kind != .jpeg {
            let crc = CRC16.checksum(payload)
            bytes[0] = UInt8(truncatingIfNeeded: crc); bytes[1] = UInt8(truncatingIfNeeded: crc >> 8)
            bytes[3] = 1
        }
        bytes[2] = kind.rawValue
        if kind == .jpeg { bytes[8] = 0; bytes[9] = 4; bytes[10] = 88; bytes[11] = 2 }
        let packed = UInt32(sequence) | (UInt32(payload.count) << 10)
        for n in 0..<4 { bytes[12 + n] = UInt8(truncatingIfNeeded: packed >> (n * 8)) }
        return Data(bytes) + payload
    }
}

public enum CRC16 {
    public static func checksum(_ data: Data) -> UInt16 {
        var crc: UInt16 = 0xffff
        for byte in data {
            crc ^= UInt16(byte) << 8
            for _ in 0..<8 { crc = (crc & 0x8000) != 0 ? (crc &<< 1) ^ 0x1021 : crc &<< 1 }
        }
        return crc
    }
}

/// Incremental parser accepts USB fragmentation/coalescing, bounds allocations and never emits invalid CRC.
public struct PacketParser {
    private var bytes: [UInt8] = []
    private var lastFeed: TimeInterval = 0
    public private(set) var rejected = 0
    public init() {}
    public mutating func reset() { bytes.removeAll(keepingCapacity: false); lastFeed = 0 }
    public mutating func feed(_ data: Data, now: TimeInterval = Date.timeIntervalSinceReferenceDate) -> [Packet] {
        if lastFeed != 0, now - lastFeed > 2 { reset() }
        lastFeed = now
        guard data.count <= 1_048_592, bytes.count + data.count <= 2_097_184 else { rejected += 1; reset(); return [] }
        bytes.append(contentsOf: data)
        var messages: [Packet] = []
        while bytes.count >= 16 {
            guard let kind = PacketKind(rawValue: bytes[2]), bytes[3] & ~UInt8(1) == 0 else {
                rejected += 1; bytes.removeAll(keepingCapacity: false); break
            }
            let packed = u32(bytes, 12), length = Int(packed >> 10), sequence = UInt16(packed & 1023)
            let limit = kind == .jpeg ? 1_048_576 : kind == .control ? 65_536 : 32_772
            let validGeometry = kind == .jpeg
                ? (u16(bytes, 4) == 0 && u16(bytes, 6) == 0 && u16(bytes, 8) == 1024 && u16(bytes, 10) == 600)
                : bytes[4..<12].allSatisfy { $0 == 0 }
            guard length > 0, length <= limit, validGeometry,
                  kind != .resource || length >= 5, kind == .jpeg || bytes[3] == 1 else {
                rejected += 1; bytes.removeAll(keepingCapacity: false); break
            }
            guard bytes.count >= 16 + length else { break }
            let payload = Data(bytes[16..<(16 + length)])
            let crcValid = bytes[3] == 0 || CRC16.checksum(payload) == u16(bytes, 0)
            bytes.removeFirst(16 + length)
            if crcValid { messages.append(Packet(kind: kind, sequence: sequence, payload: payload)) }
            else { rejected += 1 }
        }
        return messages
    }
    private func u16(_ b: [UInt8], _ offset: Int) -> UInt16 { UInt16(b[offset]) | UInt16(b[offset + 1]) << 8 }
    private func u32(_ b: [UInt8], _ offset: Int) -> UInt32 {
        (0..<4).reduce(0) { $0 | UInt32(b[offset + $1]) << ($1 * 8) }
    }
}

public enum JSONControl {
    public static func data(_ fields: [String: Any]) throws -> Data {
        guard JSONSerialization.isValidJSONObject(fields) else { throw ProtocolError.invalidJSON }
        let data = try JSONSerialization.data(withJSONObject: fields, options: [.sortedKeys])
        guard data.count <= 65_536 else { throw ProtocolError.invalidLength }
        return data
    }
    public static func object(_ data: Data) throws -> [String: Any] {
        guard data.count <= 65_536, let object = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              object["op"] is String else { throw ProtocolError.invalidJSON }
        return object
    }
}
