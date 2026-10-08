import AppKit
import P4DeskNative
import P4DeskCore
import CoreVideo

enum Diagnostics {
    @MainActor static func configureMonitorFromMac() -> Int32 {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        var code: Int32 = 1
        Task { @MainActor in
            let model = DeskModel.shared
            model.suppressNotices = true
            model.start()
            for _ in 0..<100 {
                if model.connected { break }
                try? await Task.sleep(nanoseconds: 100_000_000)
            }
            if model.connected {
                model.importMonitorConfiguration()
                if !model.monitorKey.isEmpty { await model.configureMonitor() }
                if model.monitorConfigurationSucceeded { code = 0 }
            }
            print("{\"probe\":\"monitor_configure\",\"connected\":\(model.connected),\"configured\":\(code == 0)}")
            await model.shutdown()
            stop(app)
        }
        app.run()
        return code
    }
    @MainActor static func uploadFile(localPath: String, destination: String) -> Int32 {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        var code: Int32 = 1
        Task { @MainActor in
            let model = DeskModel.shared
            model.suppressNotices = true
            model.start()
            for _ in 0..<100 {
                if model.connected { break }
                try? await Task.sleep(nanoseconds: 100_000_000)
            }
            do {
                guard model.connected else { throw DeskError.usbDisconnected }
                let manifest = try await model.uploadFileForDiagnostics(URL(fileURLWithPath: localPath), destination: destination)
                print("{\"probe\":\"file_upload\",\"connected\":true,\"committed\":true,\"font_installed\":\(model.fileFontInstalled),\"missing\":\(model.fileFontMissingCount),\"skipped\":\(model.fileFontSkipped),\"bytes\":\(manifest.length),\"sha256\":\"\(manifest.sha256)\"}")
                code = 0
            } catch {
                // File paths, names and Foundation error descriptions are private.
                print("{\"probe\":\"file_upload\",\"connected\":\(model.connected),\"committed\":false,\"error_code\":\"\(safeErrorCode(error))\",\"font_installed\":\(model.fileFontInstalled),\"missing\":\(model.fileFontMissingCount),\"skipped\":\(model.fileFontSkipped)}")
            }
            await model.shutdown()
            stop(app)
        }
        app.run()
        return code
    }
    @MainActor static func syncCurrent() -> Int32 {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        var code: Int32 = 1
        Task { @MainActor in
            let model = DeskModel.shared
            model.suppressNotices = true
            model.start()
            for _ in 0..<100 {
                if model.connected { break }
                try? await Task.sleep(nanoseconds: 100_000_000)
            }
            await model.sync()
            let result = model.lastSyncResult
            print("{\"probe\":\"sync_current\",\"connected\":\(model.connected),\"committed\":\(result.committed),\"error_code\":\"\(result.errorCode)\",\"font_bytes\":\(result.fontBytes),\"generation\":\(result.generation)}")
            code = result.committed ? 0 : 1
            await model.shutdown()
            stop(app)
        }
        app.run()
        return code
    }
    static func safeErrorCode(_ error: Error) -> String {
        if let error = error as? DeviceFailure { return error.code }
        if let error = error as? FileTransferError { return error.diagnosticCode }
        if error is CancellationError { return "cancelled" }
        if let error = error as? DeskError {
            switch error {
            case .usbDisconnected: return "usb_disconnected"
            case .usbQueueUnavailable: return "usb_queue_unavailable"
            case .timeout: return "timeout"
            case .deviceRejected: return "device_rejected"
            case .invalidDevice: return "invalid_device"
            case .permission: return "permission"
            case .displayUnavailable: return "display_unavailable"
            case .captureUnavailable: return "capture_unavailable"
            case .encodingFailed: return "encoding_failed"
            case .fontUnavailable: return "font_unavailable"
            case .fontBakeFailed: return "font_bake_failed"
            case .invalidFont: return "invalid_font"
            }
        }
        if let error = error as? ProtocolError {
            switch error {
            case .invalidHeader: return "invalid_header"
            case .invalidLength: return "invalid_length"
            case .invalidCRC: return "invalid_crc"
            case .invalidJSON: return "invalid_json"
            case .invalidSnapshot: return "invalid_snapshot"
            }
        }
        return "operation_failed"
    }
    @MainActor static func listFiles(path: String) -> Int32 {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        var code: Int32 = 1
        Task { @MainActor in
            let model = DeskModel.shared
            model.suppressNotices = true
            model.start()
            for _ in 0..<100 {
                if model.connected { break }
                try? await Task.sleep(nanoseconds: 100_000_000)
            }
            do {
                guard model.connected else { throw DeskError.usbDisconnected }
                guard model.fileTransferSupported else { throw FileTransferError.unsupported }
                guard model.sdReady else { throw FileTransferError.storageUnavailable }
                try RemoteFilePath.validate(path)
                let reply = try await model.fileControl("file_list", ["path": path, "offset": UInt32(0), "limit": UInt16(32)], expected: "file_listing")
                let listing = try RemoteFileListing.decode(reply, expectedPath: path)
                print("{\"probe\":\"file_list\",\"connected\":true,\"listed\":true,\"entries\":\(listing.entries.count),\"total\":\(listing.total),\"truncated\":\(listing.truncated),\"read_only\":\(listing.readOnly)}")
                code = 0
            } catch {
                print("{\"probe\":\"file_list\",\"connected\":\(model.connected),\"listed\":false,\"error_code\":\"\(safeErrorCode(error))\"}")
            }
            await model.shutdown()
            stop(app)
        }
        app.run()
        return code
    }
    static func virtualDisplay() -> Int32 {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        var result: Int32 = 1
        // Use AppKit's actual event loop. Sleeping the main thread can delay display
        // registration and must not turn a successful private RPC into a false pass.
        DispatchQueue.main.async {
            var error = [CChar](repeating: 0, count: 256)
            guard let handle = P4DisplayCreate(1024, 600, &error, error.count) else {
                print("{\"probe\":\"virtual_display\",\"created\":false,\"private_symbols\":\(P4DisplaySymbolsAvailable()),\"capture_requested\":false}")
                stop(app)
                return
            }
            let id = P4DisplayID(handle)
            var deadline = Date().addingTimeInterval(5)
            var removing = false, valid = false
            var width = 0, height = 0
            var active = false, online = false
            var external: [String: Any]?
            var observing = false
            var timer: Timer?
            timer = Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { _ in
                if !removing {
                    // Recent CoreGraphics caches CGDisplayMode objects per process;
                    // a display created after AppKit's first query can have a nil mode.
                    // The active list and framebuffer size remain live and are the
                    // actual registration gate, independent of that cache.
                    width = CGDisplayPixelsWide(id); height = CGDisplayPixelsHigh(id)
                    active = activeDisplays().contains(id); online = onlineDisplays().contains(id)
                    let locallyRegistered = active && online && width == 1024 && height == 600
                    if locallyRegistered, !observing {
                        observing = true
                        DispatchQueue.global(qos: .utility).async {
                            let observation = freshDisplayObservation(id)
                            DispatchQueue.main.async { external = observation }
                        }
                    }
                    valid = locallyRegistered && external?["active"] as? Bool == true && external?["online"] as? Bool == true
                        && external?["mode_width"] as? Int == 1024 && external?["mode_height"] as? Int == 600
                        && external?["pixel_width"] as? Int == 1024 && external?["pixel_height"] as? Int == 600
                    if valid || Date() >= deadline {
                        P4DisplayDestroy(handle)
                        removing = true
                        deadline = Date().addingTimeInterval(3)
                    }
                } else if !onlineDisplays().contains(id) || Date() >= deadline {
                    let removed = !onlineDisplays().contains(id)
                    timer?.invalidate(); timer = nil
                    result = valid && removed ? 0 : 1
                    print("{\"probe\":\"virtual_display\",\"created\":\(valid),\"removed\":\(removed),\"active\":\(active),\"online\":\(online),\"width\":\(width),\"height\":\(height),\"capture_requested\":false}")
                    if let external, let json = try? JSONSerialization.data(withJSONObject: external, options: [.sortedKeys]),
                       let text = String(data: json, encoding: .utf8) { print(text) }
                    stop(app)
                }
            }
        }
        app.run()
        return result
    }
    private static func freshDisplayObservation(_ id: CGDirectDisplayID) -> [String: Any]? {
        guard let executable = Bundle.main.executableURL else { return nil }
        let process = Process(), output = Pipe()
        process.executableURL = executable
        process.arguments = ["--inspect-display", String(id)]
        process.standardOutput = output; process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            let deadline = Date().addingTimeInterval(2)
            while process.isRunning, Date() < deadline { Thread.sleep(forTimeInterval: 0.025) }
            if process.isRunning { process.terminate() }
            process.waitUntilExit()
            let data = output.fileHandleForReading.readDataToEndOfFile()
            guard data.count <= 4096 else { return nil }
            return try JSONSerialization.jsonObject(with: data) as? [String: Any]
        } catch { return nil }
    }
    static func inspectDisplay(_ id: CGDirectDisplayID) -> Int32 {
        let active = activeDisplays().contains(id), online = onlineDisplays().contains(id)
        let mode = CGDisplayCopyDisplayMode(id)
        print("{\"active\":\(active),\"online\":\(online),\"width\":\(CGDisplayPixelsWide(id)),\"height\":\(CGDisplayPixelsHigh(id)),\"mode_width\":\(mode?.width ?? 0),\"mode_height\":\(mode?.height ?? 0),\"pixel_width\":\(mode?.pixelWidth ?? 0),\"pixel_height\":\(mode?.pixelHeight ?? 0)}")
        return 0
    }
    private static func activeDisplays() -> [CGDirectDisplayID] {
        var ids = [CGDirectDisplayID](repeating: 0, count: 32), count: UInt32 = 0
        guard CGGetActiveDisplayList(32, &ids, &count) == .success else { return [] }
        return Array(ids.prefix(Int(count)))
    }
    private static func onlineDisplays() -> [CGDirectDisplayID] {
        var ids = [CGDirectDisplayID](repeating: 0, count: 32), count: UInt32 = 0
        guard CGGetOnlineDisplayList(32, &ids, &count) == .success else { return [] }
        return Array(ids.prefix(Int(count)))
    }
    private static func stop(_ app: NSApplication) {
        app.stop(nil)
        if let event = NSEvent.otherEvent(with: .applicationDefined, location: .zero, modifierFlags: [],
            timestamp: 0, windowNumber: 0, context: nil, subtype: 0, data1: 0, data2: 0) {
            app.postEvent(event, atStart: false)
        }
    }
    static func jpeg() -> Int32 {
        var buffer: CVPixelBuffer?
        guard CVPixelBufferCreate(kCFAllocatorDefault, 1024, 600, kCVPixelFormatType_32BGRA,
            [kCVPixelBufferIOSurfacePropertiesKey: [:]] as CFDictionary, &buffer) == kCVReturnSuccess, let buffer else { return 1 }
        CVPixelBufferLockBaseAddress(buffer, [])
        if let base = CVPixelBufferGetBaseAddress(buffer) { memset(base, 0x7f, CVPixelBufferGetBytesPerRow(buffer) * 600) }
        CVPixelBufferUnlockBaseAddress(buffer, [])
        let encoder = JPEGEncoder()
        var complete = false, valid = false, bytes = 0
        var backend = "unknown"
        encoder.onBackend = { name in DispatchQueue.main.async { backend = name } }
        encoder.onJPEG = { frame in DispatchQueue.main.async { valid = JPEGEncoder.isBaselineJPEG(frame.data); bytes = frame.data.count; complete = true } }
        encoder.onError = { DispatchQueue.main.async { complete = true } }
        encoder.start(); encoder.submit(buffer)
        waitUntil(timeout: 5) { complete }
        encoder.stop()
        print("{\"probe\":\"jpeg\",\"baseline\":\(valid),\"bytes\":\(bytes),\"backend\":\"\(backend)\",\"source\":\"synthetic\"}")
        return complete && valid ? 0 : 1
    }
    static func selfTest() -> Int32 {
        do {
            guard CRC16.checksum(Data("123456789".utf8)) == 0x29b1 else { return 1 }
            let packet = Packet(kind: .control, sequence: 7, payload: try JSONControl.data(["op": "hello", "request_id": 7, "version": 1]))
            let wire = try packet.encoded()
            var parser = PacketParser(), output: [Packet] = []
            for byte in wire { output += parser.feed(Data([byte])) }
            guard output == [packet] else { return 1 }
            var gestures = GestureMachine()
            let bounds = CGRect(x: -1024, y: -600, width: 1024, height: 600)
            _ = gestures.update([TouchPoint(id: 0, x: 0, y: 0)], stampUS: 0, bounds: bounds)
            let click = gestures.update([], stampUS: 1, bounds: bounds)
            guard click == [.down(CGPoint(x: -1024, y: -600)), .up(CGPoint(x: -1024, y: -600))] else { return 1 }
            print("{\"self_test\":true,\"crc\":true,\"fragmentation\":true,\"negative_origin\":true}")
            return 0
        } catch { print("{\"self_test\":false}"); return 1 }
    }
    private static func waitUntil(timeout: Double = 3, _ condition: () -> Bool) {
        let end = Date().addingTimeInterval(timeout)
        while !condition(), Date() < end { RunLoop.current.run(until: Date().addingTimeInterval(0.05)) }
    }
}

@main
enum P4DeskLauncher {
    @MainActor static func main() {
        if CommandLine.arguments.contains("--sync-current") { exit(Diagnostics.syncCurrent()) }
        if let index = CommandLine.arguments.firstIndex(of: "--list-files") {
            guard CommandLine.arguments.count > index + 1 else {
                print("{\"probe\":\"file_list\",\"listed\":false,\"arguments_valid\":false}")
                exit(2)
            }
            exit(Diagnostics.listFiles(path: CommandLine.arguments[index + 1]))
        }
        if let index = CommandLine.arguments.firstIndex(of: "--upload-file") {
            guard CommandLine.arguments.count > index + 1,
                  let target = CommandLine.arguments.firstIndex(of: "--destination"),
                  CommandLine.arguments.count > target + 1 else {
                print("{\"probe\":\"file_upload\",\"committed\":false,\"arguments_valid\":false}")
                exit(2)
            }
            exit(Diagnostics.uploadFile(localPath: CommandLine.arguments[index + 1], destination: CommandLine.arguments[target + 1]))
        }
        if let index = CommandLine.arguments.firstIndex(of: "--inspect-display"), CommandLine.arguments.count > index + 1,
           let id = UInt32(CommandLine.arguments[index + 1]) { exit(Diagnostics.inspectDisplay(id)) }
        if CommandLine.arguments.contains("--configure-monitor-from-mac") { exit(Diagnostics.configureMonitorFromMac()) }
        if CommandLine.arguments.contains("--probe-virtual-display") { exit(Diagnostics.virtualDisplay()) }
        if CommandLine.arguments.contains("--probe-jpeg") { exit(Diagnostics.jpeg()) }
        if CommandLine.arguments.contains("--self-test") { exit(Diagnostics.selfTest()) }
        P4DeskApp.main()
    }
}
