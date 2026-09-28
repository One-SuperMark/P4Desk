import AppKit
import ApplicationServices
import P4DeskCore

@MainActor
final class MacInput {
    private var gestures = GestureMachine()
    private let source = CGEventSource(stateID: .hidSystemState)
    private var sequenceFilter = TouchSequenceFilter()
    var isAllowed: Bool { AXIsProcessTrusted() }
    func requestPermission() {
        AXIsProcessTrustedWithOptions([kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary)
    }
    func touch(points: [TouchPoint], stampUS: UInt64, sequence: UInt16, displayID: CGDirectDisplayID) {
        guard isAllowed else { release(); return }
        guard sequenceFilter.accept(sequence) else { return }
        post(gestures.update(points, stampUS: stampUS, bounds: CGDisplayBounds(displayID)))
    }
    func release() { post(gestures.cancel()); sequenceFilter.reset() }
    private func post(_ commands: [PointerEvent]) {
        for command in commands {
            let type: CGEventType
            let point: CGPoint
            switch command {
            case .move(let p): type = .mouseMoved; point = p
            case .down(let p): type = .leftMouseDown; point = p
            case .drag(let p): type = .leftMouseDragged; point = p
            case .up(let p): type = .leftMouseUp; point = p
            case .scroll(let x, let y):
                let dx = Int32(max(-120, min(120, x.rounded())))
                let dy = Int32(max(-120, min(120, y.rounded())))
                if dx != 0 || dy != 0 {
                    CGEvent(scrollWheelEvent2Source: source, units: .pixel, wheelCount: 2,
                            wheel1: dy, wheel2: dx, wheel3: 0)?.post(tap: .cghidEventTap)
                }
                continue
            }
            let event = CGEvent(mouseEventSource: source, mouseType: type, mouseCursorPosition: point, mouseButton: .left)
            if type == .leftMouseDown || type == .leftMouseUp { event?.setIntegerValueField(.mouseEventClickState, value: 1) }
            event?.post(tap: .cghidEventTap)
        }
    }
    func shortcut(key: UInt16, modifiers: UInt32) throws {
        guard isAllowed else { throw DeskError.permission("辅助功能") }
        guard let down = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: false) else { throw DeskError.deviceRejected("快捷键") }
        let flags = CGEventFlags(rawValue: UInt64(modifiers))
        down.flags = flags; up.flags = flags
        down.post(tap: .cghidEventTap); up.post(tap: .cghidEventTap)
    }
}
