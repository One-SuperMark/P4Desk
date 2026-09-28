import XCTest
import CoreGraphics
@testable import P4DeskCore

final class GestureAndStateTests: XCTestCase {
    private let bounds = CGRect(x: -1024, y: -600, width: 1024, height: 600)
    func testTapAndNegativeOrigin() {
        var machine = GestureMachine()
        XCTAssertEqual(machine.update([TouchPoint(id: 1, x: 0, y: 0)], stampUS: 0, bounds: bounds), [.move(CGPoint(x: -1024, y: -600))])
        XCTAssertEqual(machine.update([], stampUS: 30_000, bounds: bounds), [.down(CGPoint(x: -1024, y: -600)), .up(CGPoint(x: -1024, y: -600))])
    }
    func testDragAndCancelAlwaysRelease() {
        var machine = GestureMachine()
        _ = machine.update([TouchPoint(id: 1, x: 10, y: 10)], stampUS: 0, bounds: bounds)
        let events = machine.update([TouchPoint(id: 1, x: 30, y: 20)], stampUS: 40_000, bounds: bounds)
        XCTAssertEqual(events, [.down(CGPoint(x: -1014, y: -590)), .drag(CGPoint(x: -994, y: -580))])
        XCTAssertEqual(machine.cancel(), [.up(CGPoint(x: -994, y: -580))])
        XCTAssertTrue(machine.cancel().isEmpty)
    }
    func testTwoFingerScrollDoesNotClickAndWaitsForAllLift() {
        var machine = GestureMachine()
        _ = machine.update([TouchPoint(id: 1, x: 100, y: 100)], stampUS: 0, bounds: bounds)
        XCTAssertTrue(machine.update([TouchPoint(id: 1, x: 100, y: 100), TouchPoint(id: 2, x: 200, y: 100)], stampUS: 20_000, bounds: bounds).isEmpty)
        XCTAssertEqual(machine.update([TouchPoint(id: 1, x: 105, y: 120), TouchPoint(id: 2, x: 205, y: 120)], stampUS: 40_000, bounds: bounds), [.scroll(5, 20)])
        XCTAssertTrue(machine.update([TouchPoint(id: 1, x: 105, y: 120)], stampUS: 60_000, bounds: bounds).isEmpty)
        XCTAssertTrue(machine.update([], stampUS: 80_000, bounds: bounds).isEmpty)
    }
    func testThreeFingersSuppressAndReleaseDrag() {
        var machine = GestureMachine()
        _ = machine.update([TouchPoint(id: 1, x: 0, y: 0)], stampUS: 0, bounds: bounds)
        _ = machine.update([TouchPoint(id: 1, x: 10, y: 10)], stampUS: 20_000, bounds: bounds)
        XCTAssertEqual(machine.update([TouchPoint(id: 1, x: 10, y: 10), TouchPoint(id: 2, x: 20, y: 20), TouchPoint(id: 3, x: 30, y: 30)], stampUS: 40_000, bounds: bounds), [.up(CGPoint(x: -1014, y: -590))])
        XCTAssertTrue(machine.update([TouchPoint(id: 1, x: 10, y: 10)], stampUS: 60_000, bounds: bounds).isEmpty)
        XCTAssertTrue(machine.update([], stampUS: 80_000, bounds: bounds).isEmpty)
    }
    func testOfflineDeletionWinsOverNewerBody() throws {
        let deleted = Note(id: "note-a", title: "测试", body: "内容", updatedMS: 200)
        var local = Snapshot(notes: [deleted], deletedNoteIDs: [])
        let device = Snapshot(generation: 5, deletedNoteIDs: ["note-a"])
        try local.mergeDevice(device)
        XCTAssertTrue(local.notes.isEmpty)
        XCTAssertEqual(local.deletedNoteIDs, ["note-a"])
        try local.mergeDevice(Snapshot(generation: 6, notes: [Note(id: "note-a", updatedMS: 999)]))
        XCTAssertTrue(local.notes.isEmpty)
    }
    func testNewestNoteWinsAndButtonsStayOnHost() throws {
        var local = Snapshot(notes: [Note(id: "a", title: "old", updatedMS: 1)], buttons: [DeskButton(id: "button", label: "主机按钮")])
        try local.mergeDevice(Snapshot(generation: 2, notes: [Note(id: "a", title: "new", updatedMS: 2)]))
        XCTAssertEqual(local.notes.first?.title, "new")
        XCTAssertEqual(local.buttons.count, 1)
    }
    func testSnapshotWireKeysAndLimits() throws {
        let snapshot = Snapshot(notes: [Note(id: "id", title: "中文", updatedMS: 1)], buttons: [DeskButton(id: "b", action: .media(0xcd))])
        try snapshot.validated()
        let object = try JSONSerialization.jsonObject(with: JSONEncoder().encode(snapshot)) as! [String: Any]
        XCTAssertNotNil(object["deleted_note_ids"])
        XCTAssertEqual((object["notes"] as? [[String: Any]])?.first?["updated_ms"] as? Int, 1)
        XCTAssertThrowsError(try Snapshot(notes: [Note(id: "非法 ID")]).validated())
        XCTAssertThrowsError(try Snapshot(notes: [Note(id: "id", body: String(repeating: "x", count: 2001))]).validated())
    }
}
