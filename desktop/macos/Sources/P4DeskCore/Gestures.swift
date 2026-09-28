import Foundation
import CoreGraphics

public struct TouchPoint: Equatable {
    public var id: UInt8
    public var x: Double
    public var y: Double
    public init(id: UInt8, x: Double, y: Double) { self.id = id; self.x = x; self.y = y }
}
public enum PointerEvent: Equatable {
    case move(CGPoint), down(CGPoint), drag(CGPoint), up(CGPoint), scroll(Double, Double)
}

/// One-finger tap/drag and two-finger scroll, with delayed down to avoid a click on two-finger entry.
public struct GestureMachine {
    private enum State { case idle, one, two, suppressed }
    private var state: State = .idle
    private var primary: UInt8 = 0
    private var origin = CGPoint.zero
    private var last = CGPoint.zero
    private var centroid = CGPoint.zero
    private var started: UInt64 = 0
    private var pressed = false
    public init() {}
    public mutating func cancel() -> [PointerEvent] {
        let events: [PointerEvent] = pressed ? [.up(last)] : []
        self = GestureMachine()
        return events
    }
    public mutating func update(_ points: [TouchPoint], stampUS: UInt64, bounds: CGRect) -> [PointerEvent] {
        guard points.count <= 5, Set(points.map(\.id)).count == points.count,
              bounds.width > 0, bounds.height > 0,
              points.allSatisfy({ $0.x >= 0 && $0.x <= 1023 && $0.y >= 0 && $0.y <= 599 }) else { return cancel() }
        func position(_ point: TouchPoint) -> CGPoint {
            CGPoint(x: bounds.minX + point.x / 1023 * (bounds.width - 1),
                    y: bounds.minY + point.y / 599 * (bounds.height - 1))
        }
        if points.isEmpty {
            var events: [PointerEvent] = []
            if state == .one { events = pressed ? [.up(last)] : [.down(last), .up(last)] }
            self = GestureMachine()
            return events
        }
        if points.count >= 3 {
            let events: [PointerEvent] = pressed ? [.up(last)] : []
            pressed = false; state = .suppressed
            return events
        }
        if state == .suppressed { return [] }
        if points.count == 2 {
            let a = position(points[0]), b = position(points[1])
            let middle = CGPoint(x: (a.x + b.x) / 2, y: (a.y + b.y) / 2)
            if state != .two {
                let events: [PointerEvent] = pressed ? [.up(last)] : []
                pressed = false; state = .two; centroid = middle
                return events
            }
            let dx = middle.x - centroid.x, dy = middle.y - centroid.y
            centroid = middle
            return [.scroll(dx, dy)]
        }
        // After a scroll, wait for every finger to lift before another click.
        if state == .two { return [] }
        let point = points[0], mapped = position(point)
        if state == .idle {
            state = .one; primary = point.id; origin = mapped; last = mapped; started = stampUS
            return [.move(mapped)]
        }
        guard point.id == primary else { let events = cancel(); state = .suppressed; return events }
        last = mapped
        if !pressed, hypot(mapped.x - origin.x, mapped.y - origin.y) >= 4 || (stampUS >= started && stampUS - started >= 200_000) {
            pressed = true
            return [.down(origin), .drag(mapped)]
        }
        return [pressed ? .drag(mapped) : .move(mapped)]
    }
}
