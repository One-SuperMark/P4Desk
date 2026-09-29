import AppKit
import CoreGraphics
import Foundation

// Compile with swiftc -O -parse-as-library. This draws an owned benchmark window
// on the existing P4 Desk display; it never captures pixels or accesses USB.
@MainActor
final class PatternView: NSView {
    var tick = 0
    private(set) var draws = 0
    private let titleFont: NSFont
    private let rowFont: NSFont
    private let titleText: NSAttributedString
    private let diagnosticRows: [NSAttributedString]
    private let frameLabels: [NSAttributedString]
    private let rowAttributes: [NSAttributedString.Key: Any]
    private let blue = NSColor(calibratedRed: 0.1, green: 0.45, blue: 0.8, alpha: 1)
    private let bottomColors: [NSColor]

    init(frame: NSRect, requestedFPS: Int, duration: Double) {
        // Keep the fonts and their attributed strings strongly owned for the
        // whole run. Do not recreate temporary font dictionaries in draw().
        let titleFont = NSFont.systemFont(ofSize: 28, weight: .semibold)
        let rowFont = NSFont.monospacedSystemFont(ofSize: 17, weight: .regular)
        self.titleFont = titleFont; self.rowFont = rowFont
        titleText = NSAttributedString(string: "P4 Desk USB motion benchmark",
            attributes: [.font: titleFont, .foregroundColor: NSColor.black])
        let attributes: [NSAttributedString.Key: Any] = [.font: rowFont, .foregroundColor: NSColor.black]
        rowAttributes = attributes
        diagnosticRows = (0..<16).map { row in
            NSAttributedString(string: "Diagnostic row \(row + 1)    1024 x 600 / RGB / JPEG / \(requestedFPS) FPS",
                               attributes: attributes)
        }
        frameLabels = (0...(Int(ceil(duration * Double(requestedFPS))) + 2)).map {
            NSAttributedString(string: "Frame \($0)", attributes: attributes)
        }
        bottomColors = (0..<8).map {
            NSColor(calibratedHue: CGFloat($0) / 8, saturation: 0.8, brightness: 0.85, alpha: 1)
        }
        super.init(frame: frame)
    }
    required init?(coder: NSCoder) { nil }
    override var isOpaque: Bool { true }
    override func draw(_ dirtyRect: NSRect) {
        draws += 1
        NSColor.white.setFill(); bounds.fill()
        titleText.draw(at: NSPoint(x: 35, y: 550))
        for row in 0..<16 { diagnosticRows[row].draw(at: NSPoint(x: 40, y: 495 - row * 25)) }
        blue.setFill()
        NSRect(x: CGFloat((tick * 7) % 900), y: 30, width: 105, height: 475).fill()
        for col in 0..<8 {
            bottomColors[col].setFill()
            NSRect(x: CGFloat(col * 128), y: 0, width: 128, height: 24).fill()
        }
        let counter = tick < frameLabels.count ? frameLabels[tick]
            : NSAttributedString(string: "Frame \(tick)", attributes: rowAttributes)
        counter.draw(at: NSPoint(x: 800, y: 550))
    }
}

@MainActor
final class PatternBenchmark {
    private let app: NSApplication
    private let view: PatternView
    private let window: NSWindow
    private let metadataURL: URL
    private let requestedFPS: Int
    private let duration = 45.0
    private var started: UInt64 = 0
    private var placementMatches = false
    private var timer: Timer?
    private var finished = false
    private var completedSuccessfully = false

    init(app: NSApplication, screen: NSScreen, metadataURL: URL, requestedFPS: Int) {
        self.app = app; self.metadataURL = metadataURL; self.requestedFPS = requestedFPS
        view = PatternView(frame: NSRect(origin: .zero, size: screen.frame.size),
                           requestedFPS: requestedFPS, duration: duration)
        window = NSWindow(contentRect: screen.frame, styleMask: [.borderless], backing: .buffered, defer: false, screen: screen)
        window.setFrame(screen.frame, display: true)
        window.isReleasedWhenClosed = false
        window.title = "P4Desk owned motion benchmark"
        window.contentView = view
        window.level = .floating
        window.ignoresMouseEvents = true
        window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        placementMatches = window.frame == screen.frame && window.screen === screen
    }
    func run() throws -> Int32 {
        try FileManager.default.createDirectory(at: metadataURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        window.orderFrontRegardless()
        started = DispatchTime.now().uptimeNanoseconds
        try writeMetadata(commonMetadata())
        let timer = Timer(timeInterval: 1.0 / Double(requestedFPS), repeats: true) { [weak self] _ in
            // This timer belongs exclusively to the main run loop.
            MainActor.assumeIsolated { self?.advance() }
        }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
        print("owned_pattern_started width=1024 height=600 requested_fps=\(requestedFPS) duration_s=\(duration)")
        // Explicitly retain the controller, fonts, view and window across the
        // event loop, including optimized standalone compilation.
        withExtendedLifetime(self) { app.run() }
        timer.invalidate(); self.timer = nil
        return completedSuccessfully ? 0 : 1
    }
    private func advance() {
        guard !finished else { return }
        let elapsed = Double(DispatchTime.now().uptimeNanoseconds - started) / 1_000_000_000
        guard elapsed < duration else { finish(); return }
        view.tick += 1; view.needsDisplay = true
    }
    private func commonMetadata() -> [String: Any] {
        ["started_ns": started, "requested_fps": requestedFPS, "duration_s": duration,
         "width": 1024, "height": 600, "placement_matches_target": placementMatches]
    }
    private func writeMetadata(_ metadata: [String: Any]) throws {
        try JSONSerialization.data(withJSONObject: metadata, options: [.sortedKeys]).write(to: metadataURL, options: .atomic)
    }
    private func finish() {
        finished = true; timer?.invalidate(); timer = nil
        var metadata = commonMetadata()
        metadata["finished_ns"] = DispatchTime.now().uptimeNanoseconds
        metadata["updates"] = view.tick; metadata["draws"] = view.draws
        // Commit the end marker before tearing down AppKit objects. Cleanup
        // must not remove the benchmark's independently verifiable duration.
        do {
            try writeMetadata(metadata)
            completedSuccessfully = true
            print("owned_pattern_complete duration_s=\(duration) updates=\(view.tick) draws=\(view.draws)")
        } catch { print("owned_pattern_metadata_write_failed") }
        window.orderOut(nil); window.contentView = nil; window.close()
        app.stop(nil)
        if let event = NSEvent.otherEvent(with: .applicationDefined, location: .zero, modifierFlags: [],
            timestamp: 0, windowNumber: 0, context: nil, subtype: 0, data1: 0, data2: 0) {
            app.postEvent(event, atStart: false)
        }
    }
}

@main
enum OwnedPatternMain {
    @MainActor
    static func main() {
        let arguments = Array(CommandLine.arguments.dropFirst())
        let requestedFPS = Int(arguments.dropFirst().first ?? "60") ?? 60
        guard requestedFPS == 30 || requestedFPS == 60 else { print("requested_fps_must_be_30_or_60"); exit(1) }
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        let candidates = NSScreen.screens.filter { screen in
            guard let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber else { return false }
            let display = CGDirectDisplayID(number.uint32Value)
            return CGDisplayPixelsWide(display) == 1024 && CGDisplayPixelsHigh(display) == 600 &&
                ((CGDisplayVendorNumber(display) == 0x5044 && CGDisplayModelNumber(display) == 7) || screen.localizedName == "P4 Desk")
        }
        guard candidates.count == 1, let screen = candidates.first else { print("owned_virtual_display_unavailable"); exit(1) }
        let metadataURL = URL(fileURLWithPath: arguments.first ?? ".cache/performance-pattern.json")
        let benchmark = PatternBenchmark(app: app, screen: screen, metadataURL: metadataURL, requestedFPS: requestedFPS)
        do { exit(try benchmark.run()) }
        catch { print("owned_pattern_setup_failed"); exit(1) }
    }
}
