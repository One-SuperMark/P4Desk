import AppKit
import CoreGraphics
import Foundation

// Compile as a standalone program. It draws owned content on the existing
// P4 Desk display and never reads screen pixels, USB, notes, or shortcuts.
@MainActor
final class QualityPatternView: NSView {
    private let fonts: [NSFont]
    private let rows: [NSAttributedString]
    private let title: NSAttributedString
    private let labels: [NSAttributedString]
    private let swatches: [NSColor]
    private let gradient: NSGradient
    private let gray: NSGradient

    override init(frame: NSRect) {
        fonts = [12, 14, 16, 20, 28].map { NSFont.systemFont(ofSize: CGFloat($0)) }
        rows = fonts.prefix(4).map { font in
            NSAttributedString(string: "清晰度测试  中文边缘  ABCDE  0123456789  Rust / macOS / USB",
                attributes: [.font: font, .foregroundColor: NSColor.black])
        }
        title = NSAttributedString(string: "P4 Desk · 清晰度与颜色测试",
            attributes: [.font: fonts[4], .foregroundColor: NSColor.black])
        let names = ["红", "橙", "黄", "绿", "蓝", "紫"]
        let labelFont = fonts[2]
        labels = names.map {
            NSAttributedString(string: $0, attributes: [.font: labelFont, .foregroundColor: NSColor.black])
        }
        swatches = [
            NSColor(srgbRed: 0.95, green: 0.15, blue: 0.12, alpha: 1),
            NSColor(srgbRed: 1, green: 0.55, blue: 0.08, alpha: 1),
            NSColor(srgbRed: 1, green: 0.85, blue: 0.05, alpha: 1),
            NSColor(srgbRed: 0.15, green: 0.72, blue: 0.28, alpha: 1),
            NSColor(srgbRed: 0.12, green: 0.38, blue: 0.94, alpha: 1),
            NSColor(srgbRed: 0.62, green: 0.25, blue: 0.88, alpha: 1)
        ]
        gradient = NSGradient(starting: NSColor(srgbRed: 1, green: 0.22, blue: 0.02, alpha: 1),
            ending: NSColor(srgbRed: 1, green: 0.8, blue: 0.16, alpha: 1))!
        gray = NSGradient(starting: .black, ending: .white)!
        super.init(frame: frame)
    }
    required init?(coder: NSCoder) { nil }
    override var isOpaque: Bool { true }
    override func draw(_ dirtyRect: NSRect) {
        NSColor.white.setFill(); bounds.fill()
        title.draw(at: NSPoint(x: 30, y: 540))
        for (index, row) in rows.enumerated() {
            row.draw(at: NSPoint(x: 32, y: 494 - index * 32))
        }
        for (index, color) in swatches.enumerated() {
            color.setFill()
            NSRect(x: 32 + index * 160, y: 262, width: 146, height: 105).fill()
            labels[index].draw(at: NSPoint(x: 92 + index * 160, y: 235))
        }
        gradient.draw(in: NSRect(x: 32, y: 155, width: 960, height: 58), angle: 0)
        gray.draw(in: NSRect(x: 32, y: 84, width: 960, height: 45), angle: 0)
        // One-pixel alternating colored lines expose chroma subsampling.
        for x in 0..<960 {
            (x.isMultiple(of: 2) ? swatches[0] : swatches[4]).setFill()
            NSRect(x: 32 + x, y: 36, width: 1, height: 24).fill()
        }
        // The visible bottom 24 rows include eight neutral levels.
        let grayLevels: [CGFloat] = [0, 16, 64, 128, 192, 235, 248, 255]
        for (index, value) in grayLevels.enumerated() {
            let component = value / 255
            NSColor(srgbRed: component, green: component, blue: component, alpha: 1).setFill()
            NSRect(x: index * 128, y: 0, width: 128, height: 24).fill()
        }
    }
}

@main
enum QualityPatternMain {
    @MainActor
    static func main() {
        let seconds = Double(CommandLine.arguments.dropFirst().first ?? "300") ?? 300
        guard seconds >= 10, seconds <= 1800 else { print("duration_out_of_range"); exit(1) }
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        let candidates = NSScreen.screens.filter { screen in
            guard let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber else { return false }
            let display = CGDirectDisplayID(number.uint32Value)
            return CGDisplayPixelsWide(display) == 1024 && CGDisplayPixelsHigh(display) == 600 &&
                ((CGDisplayVendorNumber(display) == 0x5044 && CGDisplayModelNumber(display) == 7) || screen.localizedName == "P4 Desk")
        }
        guard candidates.count == 1, let screen = candidates.first else { print("owned_virtual_display_unavailable"); exit(1) }
        let view = QualityPatternView(frame: NSRect(origin: .zero, size: screen.frame.size))
        let window = NSWindow(contentRect: screen.frame, styleMask: [.borderless], backing: .buffered, defer: false, screen: screen)
        window.setFrame(screen.frame, display: true)
        window.isReleasedWhenClosed = false
        window.title = "P4Desk owned quality pattern"
        window.contentView = view
        window.level = .floating
        window.ignoresMouseEvents = true
        window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        guard window.frame == screen.frame, window.screen === screen else { print("pattern_placement_failed"); exit(1) }
        window.orderFrontRegardless()
        print("owned_quality_pattern_started width=1024 height=600 duration_s=\(seconds)")
        let timer = Timer.scheduledTimer(withTimeInterval: seconds, repeats: false) { _ in
            MainActor.assumeIsolated {
                window.orderOut(nil); window.contentView = nil; window.close()
                print("owned_quality_pattern_complete")
                app.stop(nil)
                if let event = NSEvent.otherEvent(with: .applicationDefined, location: .zero, modifierFlags: [],
                    timestamp: 0, windowNumber: 0, context: nil, subtype: 0, data1: 0, data2: 0) {
                    app.postEvent(event, atStart: false)
                }
            }
        }
        withExtendedLifetime((view, window, timer)) { app.run() }
        timer.invalidate()
    }
}
