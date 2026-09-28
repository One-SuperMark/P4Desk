import XCTest
@testable import P4DeskCore

final class ProtocolTests: XCTestCase {
    func testCRCAndHeaderBytes() throws {
        XCTAssertEqual(CRC16.checksum(Data("123456789".utf8)), 0x29b1)
        let packet = Packet(kind: .control, sequence: 513, payload: Data("{}".utf8))
        let data = [UInt8](try packet.encoded())
        XCTAssertEqual(Array(data[2...11]), [16, 1, 0, 0, 0, 0, 0, 0, 0, 0])
        XCTAssertEqual(Array(data[12...15]), [1, 10, 0, 0])
    }
    func testAllFragmentationAndCoalescing() throws {
        let packets = [Packet(kind: .control, sequence: 1, payload: Data("{}".utf8)),
                       Packet(kind: .resource, sequence: 2, payload: Data([0, 0, 0, 0, 0x41]))]
        let wire = try packets.reduce(Data()) { try $0 + $1.encoded() }
        for split in 0...wire.count {
            var parser = PacketParser()
            XCTAssertEqual(parser.feed(wire.prefix(split)) + parser.feed(wire.dropFirst(split)), packets)
        }
        var parser = PacketParser(), result: [Packet] = []
        for byte in wire { result += parser.feed(Data([byte])) }
        XCTAssertEqual(result, packets)
    }
    func testBadCRCAndAllocationBound() throws {
        var wire = try Packet(kind: .control, sequence: 4, payload: Data("{}".utf8)).encoded()
        wire[16] ^= 1
        var parser = PacketParser()
        XCTAssertTrue(parser.feed(wire).isEmpty)
        XCTAssertEqual(parser.rejected, 1)
        XCTAssertThrowsError(try Packet(kind: .resource, sequence: 0, payload: Data(count: 32773)).encoded())
        XCTAssertThrowsError(try Packet(kind: .control, sequence: 1024, payload: Data([1])).encoded())
    }
    func testTimeoutClearsPartialFrame() throws {
        let packet = Packet(kind: .control, sequence: 9, payload: Data("{}".utf8))
        let wire = try packet.encoded()
        var parser = PacketParser()
        XCTAssertTrue(parser.feed(wire.prefix(10), now: 100).isEmpty)
        XCTAssertEqual(parser.feed(wire, now: 103), [packet])
    }
    func testSharedHelloFixture() throws {
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let url = root.appendingPathComponent("tests/fixtures/hello-v1.json")
        let fixture = try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as! [String: Any]
        // The root fixture carries the cross-language canonical bytes.
        let key = ["packet_hex", "wire_hex", "hex", "wire"].first { fixture[$0] is String }
        guard let key, let hex = fixture[key] as? String else { XCTFail("wire hex missing"); return }
        let cleaned = hex.filter { !$0.isWhitespace }
        var bytes: [UInt8] = [], index = cleaned.startIndex
        while index < cleaned.endIndex {
            let end = cleaned.index(index, offsetBy: 2)
            bytes.append(try XCTUnwrap(UInt8(cleaned[index..<end], radix: 16))); index = end
        }
        var parser = PacketParser()
        let parsed = parser.feed(Data(bytes))
        XCTAssertEqual(parsed.count, 1)
        XCTAssertEqual(parsed.first?.kind, .control)
        XCTAssertEqual(try parsed.first?.encoded(), Data(bytes))
        XCTAssertEqual(try JSONControl.object(parsed[0].payload)["op"] as? String, "hello")
    }
}
