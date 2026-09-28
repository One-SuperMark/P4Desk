import XCTest
import CoreGraphics
@testable import P4DeskCore

final class TouchSequenceTests: XCTestCase {
    func testJSONSequenceContinuesAcrossWireHeaderBoundary() {
        var filter = TouchSequenceFilter()
        XCTAssertTrue(filter.accept(1022))
        XCTAssertTrue(filter.accept(1023))
        XCTAssertTrue(filter.accept(1024))
        XCTAssertTrue(filter.accept(1025))
        // Masking the JSON sequence to 10 bits would incorrectly turn new samples into stale ones.
        XCTAssertFalse(filter.accept(1))
        XCTAssertTrue(filter.accept(1026))
    }
    func testUInt16WrapIsForward() {
        var filter = TouchSequenceFilter()
        XCTAssertTrue(filter.accept(65534))
        XCTAssertTrue(filter.accept(65535))
        XCTAssertTrue(filter.accept(0))
        XCTAssertTrue(filter.accept(1))
    }
    func testDuplicateAndOldSequenceDoNotAdvanceState() {
        var filter = TouchSequenceFilter()
        XCTAssertTrue(filter.accept(20))
        XCTAssertFalse(filter.accept(20))
        XCTAssertFalse(filter.accept(19))
        XCTAssertFalse(filter.accept(65535))
        XCTAssertFalse(filter.accept(20 &+ 0x8000))
        XCTAssertTrue(filter.accept(21))
        XCTAssertFalse(filter.accept(20))
        XCTAssertTrue(filter.accept(22))
    }
    func testReleaseResetAllowsFirstSampleOfNewSession() {
        var filter = TouchSequenceFilter()
        XCTAssertTrue(filter.accept(5000))
        XCTAssertFalse(filter.accept(4))
        filter.reset()
        XCTAssertTrue(filter.accept(4))
        XCTAssertFalse(filter.accept(4))
        XCTAssertTrue(filter.accept(5))
    }
    func testPriorityReleaseRejectsOlderTouchesAcrossWrap() {
        var filter = TouchSequenceFilter(), gestures = GestureMachine()
        let bounds = CGRect(x: 0, y: 0, width: 1024, height: 600)
        func process(_ sequence: UInt16, _ points: [TouchPoint], _ time: UInt64) -> [PointerEvent] {
            guard filter.accept(sequence) else { return [] }
            return gestures.update(points, stampUS: time, bounds: bounds)
        }
        XCTAssertEqual(process(65534, [TouchPoint(id: 1, x: 10, y: 10)], 0), [.move(CGPoint(x: 10, y: 10))])
        XCTAssertEqual(process(65535, [TouchPoint(id: 1, x: 40, y: 20)], 20_000),
                       [.down(CGPoint(x: 10, y: 10)), .drag(CGPoint(x: 40, y: 20))])
        XCTAssertEqual(process(0, [], 40_000), [.up(CGPoint(x: 40, y: 20))])
        // The high-priority release can overtake queued touch samples. They must never restart a drag.
        XCTAssertTrue(process(65535, [TouchPoint(id: 1, x: 60, y: 20)], 20_000).isEmpty)
        XCTAssertTrue(process(65534, [TouchPoint(id: 1, x: 10, y: 10)], 0).isEmpty)
        XCTAssertEqual(process(1, [TouchPoint(id: 1, x: 100, y: 100)], 60_000), [.move(CGPoint(x: 100, y: 100))])
        XCTAssertEqual(process(2, [], 80_000), [.down(CGPoint(x: 100, y: 100)), .up(CGPoint(x: 100, y: 100))])
    }
}
