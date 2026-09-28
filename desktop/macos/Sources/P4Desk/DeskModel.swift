import SwiftUI
import AppKit
import ApplicationServices
import UniformTypeIdentifiers
import P4DeskNative
import P4DeskCore

@MainActor
final class DeskModel: ObservableObject {
    static let shared = DeskModel()
    @Published var snapshot: Snapshot
    @Published var connected = false
    @Published var connectionStatus = "未连接 USB HS（Type-A）"
    @Published var displayActive = false
    @Published var changingMode = false
    @Published var syncing = false
    @Published var syncProgress = 0.0
    @Published var dirty = true
    @Published var sdReady = false
    @Published var screenAllowed = false
    @Published var inputAllowed = false
    @Published var message = "请用数据线连接板上 Type-A USB-OTG 大接口（USB HS 数据口）。Type-C 小接口用于供电／烧录调试，不能连接此副屏协议。"
    @Published var codec = "尚未启动"
    @Published var presentedFrames = 0
    @Published var presentationMS: Double?
    @Published var captureToReceiptP95MS: Double?
    @Published var effectivePresentedFPS = 0.0
    @Published var performanceSampleCount = 0
    @Published var fontPath = ""
    @Published var fontToolPath = ""

    private let transport = USBSession()
    private let input = MacInput()
    private var parser = PacketParser()
    private var usbOpen = false
    private var connectionEpoch: UInt64 = 0
    private var deviceActions: [String: DeskAction] = [:]
    private var capture: DisplayCapture?
    private var displayHandle: UnsafeMutableRawPointer?
    private var activeSession: UInt32 = 0
    private var videoEnabled = false
    private var videoSequence: UInt16 = 0
    private var frameEnqueued: [UInt16: FrameTiming] = [:]
    private var frameTokens: [UInt32: UInt16] = [:]
    private var nextFrameToken: UInt32 = 1
    private var performance = PerformanceWindow()
    private var firstJPEG: EncodedFrame?
    private var jpegWait: (UUID, CheckedContinuation<EncodedFrame, Error>)?
    private var presentedWait: (UUID, CheckedContinuation<Void, Error>)?
    private var transition = 0
    private var editRevision: UInt64 = 0
    private var requestSequence: UInt16 = 1
    private struct Pending {
        var identity: UUID
        var expected: String
        var continuation: CheckedContinuation<[String: Any], Error>
    }
    private var pending: [UInt16: Pending] = [:]
    private var heartbeatTask: Task<Void, Never>?
    private var fontTask: Task<Data, Error>?
    private var saveTask: Task<Void, Never>?
    private var lastDeviceContact = Date.timeIntervalSinceReferenceDate
    private var sleeping = false
    private var stopping = false
    private var observers: [NSObjectProtocol] = []
    private let storeURL: URL

    private init() {
        let directory = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("P4Desk", isDirectory: true)
        storeURL = directory.appendingPathComponent("state.json")
        let initial = Snapshot(notes: [Note(title: "桌面便签")], buttons: [
            DeskButton(label: "复制", action: .shortcut(8, flags: 1 << 20)),
            DeskButton(label: "粘贴", action: .shortcut(9, flags: 1 << 20)),
            DeskButton(label: "访达", action: .application("/System/Library/CoreServices/Finder.app")),
            DeskButton(label: "播放/暂停", action: .media(0xcd)),
            DeskButton(label: "音量 +", action: .media(0xe9)),
            DeskButton(label: "音量 −", action: .media(0xea))
        ])
        if let data = try? Data(contentsOf: storeURL), let saved = try? JSONDecoder().decode(Snapshot.self, from: data),
           (try? saved.validated()) != nil { snapshot = saved }
        else { snapshot = initial }
        let preferences = UserDefaults.standard
        fontPath = preferences.string(forKey: "fontPath") ?? Bundle.main.url(forResource: "NotoSansSC-Regular", withExtension: "otf")?.path ?? ""
        fontToolPath = preferences.string(forKey: "fontToolPath") ?? ProcessInfo.processInfo.environment["P4DESK_FONTPACK_BIN"]
            ?? Bundle.main.url(forResource: "p4desk-fontpack", withExtension: nil)?.path ?? ""
    }
    func start() {
        guard !stopping else { return }
        refreshPermissions()
        transport.onEvent = { [weak self] event in self?.usbEvent(event) }
        transport.start()
        let center = NSWorkspace.shared.notificationCenter
        observers.append(center.addObserver(forName: NSWorkspace.willSleepNotification, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                self.sleeping = true
                await self.endDisplay(sendPad: self.usbOpen)
            }
        })
        observers.append(center.addObserver(forName: NSWorkspace.didWakeNotification, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in
                guard let self else { return }
                self.sleeping = false; self.transport.reconnect(); self.refreshPermissions()
            }
        })
        observers.append(NotificationCenter.default.addObserver(forName: NSApplication.didBecomeActiveNotification, object: nil, queue: .main) { [weak self] _ in
            Task { @MainActor in self?.refreshPermissions() }
        })
    }
    func refreshPermissions() {
        screenAllowed = CGPreflightScreenCaptureAccess()
        inputAllowed = input.isAllowed
    }
    func requestScreenPermission() { screenAllowed = CGRequestScreenCaptureAccess(); refreshPermissions() }
    func requestInputPermission() { input.requestPermission(); refreshPermissions() }
    func openPrivacy(_ pane: String) {
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?\(pane)") { NSWorkspace.shared.open(url) }
    }
    func reconnect() { transport.reconnect() }
    private func usbEvent(_ event: USBSession.Event) {
        switch event {
        case .connected:
            connectionEpoch &+= 1
            let epoch = connectionEpoch
            usbOpen = true; connected = false; parser.reset(); connectionStatus = "USB 已打开，正在握手"
            lastDeviceContact = Date.timeIntervalSinceReferenceDate
            startHeartbeat()
            Task { await establish(epoch: epoch) }
        case .disconnected:
            connectionEpoch &+= 1
            usbOpen = false; connected = false; sdReady = false
            deviceActions.removeAll()
            connectionStatus = "USB 已断开"; parser.reset(); failPending(DeskError.usbDisconnected)
            heartbeatTask?.cancel(); heartbeatTask = nil
            fontTask?.cancel()
            Task { await endDisplay(sendPad: false) }
        case .bytes(let data):
            for packet in parser.feed(data) {
                if packet.kind == .control, let fields = try? JSONControl.object(packet.payload) { receive(fields, sequence: packet.sequence) }
            }
        case .sent(let token, let timeNS):
            guard let sequence = frameTokens.removeValue(forKey: token) else { break }
            if var timing = frameEnqueued[sequence] { timing.usbSentNS = timeNS; frameEnqueued[sequence] = timing }
            else { performance.markSent(sequence: sequence, timeNS: timeNS) }
        case .error(let reason):
            connectionStatus = "USB 未能打开"
            message = "\(reason) 请确认已允许 USB 配件访问，连接板上 Type-A USB-OTG 大接口（USB HS 数据口），并关闭占用设备的其他应用。Type-C 小接口用于供电／烧录调试。"
        }
    }
    private func currentConnection(_ epoch: UInt64) -> Bool { usbOpen && connectionEpoch == epoch && !stopping }
    private func establish(epoch: UInt64) async {
        do {
            let caps = try await request("hello", ["version": 1], expected: "caps", timeout: 2)
            guard currentConnection(epoch) else { return }
            guard number(caps, "version") == 1, number(caps, "width") == 1024,
                  number(caps, "height") == 600, (number(caps, "max_jpeg") ?? 0) >= 1_048_576,
                  (number(caps, "max_control") ?? 0) >= 65_536 else { throw DeskError.invalidDevice }
            sdReady = caps["sd_ready"] as? Bool ?? false
            _ = try await request("time_sync", ["unix_ms": Int64(Date().timeIntervalSince1970 * 1000),
                                                "timezone_minutes": TimeZone.current.secondsFromGMT() / 60], timeout: 2)
            guard currentConnection(epoch) else { return }
            _ = try await request("get_state", [:], expected: "state", timeout: 2)
            guard currentConnection(epoch) else { return }
            // Every new USB connection begins from Pad, including reconnects after host sleep.
            _ = try await request("set_mode", ["mode": "pad", "session": UInt32(0)], timeout: 2)
            guard currentConnection(epoch) else { return }
            connected = true; connectionStatus = "已连接 P4 Desk"
            message = sdReady ? "Pad 可用；编辑后同步便签、按钮与中文字库。" : "已连接；TF 卡未就绪，暂不能保存配置。"
        } catch {
            guard currentConnection(epoch) else { return }
            report(error)
            connected = false; connectionStatus = "设备握手失败"
            await endDisplay(sendPad: usbOpen)
            // Keep the actual USB ownership visible. User can retry after firmware/TF correction.
        }
    }
    private func startHeartbeat() {
        heartbeatTask?.cancel()
        let epoch = connectionEpoch
        heartbeatTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self, self.currentConnection(epoch) else { return }
                if !self.sleeping {
                    if Date.timeIntervalSinceReferenceDate - self.lastDeviceContact > 4 {
                        self.message = "设备心跳超时，副屏已停止。"
                        await self.endDisplay(sendPad: false)
                        self.transport.reconnect()
                        return
                    }
                    Task { [weak self] in
                        guard let self, self.currentConnection(epoch) else { return }
                        _ = try? await self.request("heartbeat", [:], timeout: 1.8)
                    }
                }
                try? await Task.sleep(nanoseconds: 1_000_000_000)
            }
        }
    }
    private func nextRequest() throws -> UInt16 {
        for _ in 0..<1024 {
            let candidate = requestSequence; requestSequence = (requestSequence + 1) & 1023
            if pending[candidate] == nil { return candidate }
        }
        throw DeskError.usbQueueUnavailable
    }
    private func request(_ op: String, _ fields: [String: Any], expected: String? = nil, timeout: Double = 3) async throws -> [String: Any] {
        guard usbOpen else { throw DeskError.usbDisconnected }
        let id = try nextRequest()
        var object = fields; object["op"] = op; object["request_id"] = id
        let packet = Packet(kind: .control, sequence: id, payload: try JSONControl.data(object))
        return try await awaitReply(packet, id: id, expected: expected ?? op, timeout: timeout)
    }
    private func awaitReply(_ packet: Packet, id: UInt16, expected: String, timeout: Double) async throws -> [String: Any] {
        let identity = UUID()
        return try await withCheckedThrowingContinuation { continuation in
            pending[id] = Pending(identity: identity, expected: expected, continuation: continuation)
            do { try transport.send(packet) }
            catch { pending.removeValue(forKey: id)?.continuation.resume(throwing: error); return }
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: UInt64(timeout * 1_000_000_000))
                guard let self, let current = self.pending[id], current.identity == identity else { return }
                self.pending.removeValue(forKey: id)?.continuation.resume(throwing: DeskError.timeout(expected))
            }
        }
    }
    private func receive(_ fields: [String: Any], sequence: UInt16) {
        guard let op = fields["op"] as? String else { return }
        lastDeviceContact = Date.timeIntervalSinceReferenceDate
        let id = number(fields, "request_id").flatMap { $0 <= 1023 ? UInt16($0) : nil }
        if op == "state", let object = fields["state"], JSONSerialization.isValidJSONObject(object) {
            do {
                let device = try JSONDecoder().decode(Snapshot.self, from: JSONSerialization.data(withJSONObject: object))
                var merged = snapshot; try merged.mergeDevice(device)
                deviceActions = Dictionary(uniqueKeysWithValues: device.buttons.map { ($0.id, $0.action) })
                if merged != snapshot { snapshot = merged; changed() }
            } catch { report(error); if let id { pending.removeValue(forKey: id)?.continuation.resume(throwing: DeskError.invalidDevice) }; return }
        }
        if let id, id == sequence, let waiting = pending[id] {
            if op == waiting.expected || (op == "ack" && fields["acknowledged"] as? String == waiting.expected) {
                pending.removeValue(forKey: id)
                if op == "ack", fields["ok"] as? Bool != true { waiting.continuation.resume(throwing: DeskError.deviceRejected(waiting.expected)) }
                else { waiting.continuation.resume(returning: fields) }
            }
        }
        switch op {
        case "touch":
            guard displayActive, number(fields, "session") == UInt64(activeSession),
                  let raw = fields["points"] as? [[String: Any]], raw.count <= 5,
                  let stamp = number(fields, "stamp_us"), let seq = number(fields, "sequence"), seq <= 65_535,
                  let handle = displayHandle else { return }
            let points = raw.compactMap { value -> TouchPoint? in
                guard let id = number(value, "id"), id <= 255, let x = number(value, "x"), x <= 1023,
                      let y = number(value, "y"), y <= 599 else { return nil }
                return TouchPoint(id: UInt8(id), x: Double(x), y: Double(y))
            }
            guard points.count == raw.count else { input.release(); return }
            input.touch(points: points, stampUS: stamp, sequence: UInt16(seq), displayID: P4DisplayID(handle))
        case "frame_presented":
            guard videoEnabled, number(fields, "session") == UInt64(activeSession),
                  let seq = number(fields, "sequence"), seq < 1024,
                  var timing = frameEnqueued.removeValue(forKey: UInt16(seq)) else { return }
            timing.receiptNS = DispatchTime.now().uptimeNanoseconds
            timing.devicePresentedUS = number(fields, "device_us")
            presentationMS = timing.queueToReceiptMS
            _ = performance.record(timing)
            let report = performance.report(nowNS: timing.receiptNS!)
            captureToReceiptP95MS = report.captureToReceiptP95MS
            effectivePresentedFPS = report.effectivePresentedFPS
            performanceSampleCount = report.sampleCount
            presentedFrames += 1
            if let waiting = presentedWait { presentedWait = nil; waiting.1.resume() }
        case "action":
            guard connected, !stopping, !sleeping, let id = fields["action_id"] as? String,
                  let action = deviceActions[id] else { return }
            Task { await run(action) }
        case "request_mode":
            if fields["mode"] as? String == "display" { Task { await beginDisplay() } }
            else if fields["mode"] as? String == "pad" { Task { await endDisplay(sendPad: usbOpen) } }
        case "request_time_sync":
            Task {
                do {
                    _ = try await request("time_sync", ["unix_ms": Int64(Date().timeIntervalSince1970 * 1000),
                                                        "timezone_minutes": TimeZone.current.secondsFromGMT() / 60])
                } catch { report(error) }
            }
        case "status":
            sdReady = fields["sd_ready"] as? Bool ?? sdReady
            if fields["mode"] as? String == "pad", videoEnabled { Task { await endDisplay(sendPad: false) } }
        default: break
        }
    }
    private func number(_ object: [String: Any], _ key: String) -> UInt64? {
        guard let value = object[key] as? NSNumber, CFGetTypeID(value) != CFBooleanGetTypeID(), value.doubleValue >= 0,
              value.doubleValue.rounded(.towardZero) == value.doubleValue else { return nil }
        return value.uint64Value
    }
    private func failPending(_ error: Error) {
        let waiting = pending.values; pending.removeAll()
        for item in waiting { item.continuation.resume(throwing: error) }
    }

    func beginDisplay() async {
        guard connected, !changingMode, !displayActive, !sleeping, !stopping else { return }
        refreshPermissions()
        if !screenAllowed { requestScreenPermission() }
        guard screenAllowed else { report(DeskError.permission("屏幕录制")); return }
        changingMode = true; transition += 1
        let operation = transition
        var errorBuffer = [CChar](repeating: 0, count: 256)
        guard let handle = P4DisplayCreate(1024, 600, &errorBuffer, errorBuffer.count) else {
            changingMode = false; report(DeskError.displayUnavailable); return
        }
        displayHandle = handle; firstJPEG = nil
        let stream = DisplayCapture(); capture = stream
        stream.onJPEG = { [weak self] data in
            Task { @MainActor in self?.encoded(data, operation: operation) }
        }
        stream.onFailure = { [weak self] in
            Task { @MainActor in
                guard let self, self.transition == operation else { return }
                self.report(DeskError.encodingFailed); await self.endDisplay(sendPad: self.usbOpen)
            }
        }
        stream.onBackend = { [weak self] backend in Task { @MainActor in
            guard let self, self.transition == operation else { return }; self.codec = backend
        } }
        do {
            try await stream.start(displayID: P4DisplayID(handle))
            guard transition == operation else { return }
            let first = try await awaitJPEG()
            guard transition == operation, connected else { throw CancellationError() }
            activeSession = UInt32.random(in: 1...UInt32.max)
            _ = try await request("set_mode", ["mode": "display", "session": activeSession], timeout: 3)
            guard transition == operation else { return }
            videoEnabled = true; videoSequence = 0; presentedFrames = 0; frameEnqueued.removeAll(); frameTokens.removeAll()
            performance.reset(); captureToReceiptP95MS = nil; presentationMS = nil
            effectivePresentedFPS = 0; performanceSampleCount = 0
            try await waitForPresentation(first)
            guard transition == operation else { return }
            displayActive = true; changingMode = false
            message = inputAllowed ? "USB 副屏已呈现，支持单指点击/拖动与双指滚动。" : "USB 副屏已呈现；开启辅助功能后可触摸操作。"
        } catch {
            if transition == operation { report(error); await endDisplay(sendPad: usbOpen) }
        }
    }
    private func encoded(_ jpeg: EncodedFrame, operation: Int) {
        guard transition == operation, displayHandle != nil, jpeg.data.count <= 1_048_576 else { return }
        if firstJPEG == nil { firstJPEG = jpeg }
        if let waiting = jpegWait { jpegWait = nil; waiting.1.resume(returning: jpeg) }
        if videoEnabled { do { try sendJPEG(jpeg) } catch { Task { await endDisplay(sendPad: false) }; report(error) } }
    }
    private func awaitJPEG() async throws -> EncodedFrame {
        if let firstJPEG { return firstJPEG }
        let identity = UUID()
        return try await withCheckedThrowingContinuation { continuation in
            jpegWait = (identity, continuation)
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: 4_000_000_000)
                guard let self, self.jpegWait?.0 == identity else { return }
                let current = self.jpegWait; self.jpegWait = nil
                current?.1.resume(throwing: DeskError.captureUnavailable)
            }
        }
    }
    private func sendJPEG(_ frame: EncodedFrame) throws {
        let sequence = videoSequence; videoSequence = (videoSequence + 1) & 1023
        let time = DispatchTime.now().uptimeNanoseconds
        let token = 0x80000000 | nextFrameToken
        nextFrameToken = (nextFrameToken &+ 1) & 0x7fffffff
        try transport.send(Packet(kind: .jpeg, sequence: sequence, payload: frame.data), token: token)
        frameTokens[token] = sequence
        if frameTokens.count > 256 {
            for obsolete in frameTokens.keys.sorted().prefix(frameTokens.count - 256) { frameTokens.removeValue(forKey: obsolete) }
        }
        frameEnqueued[sequence] = FrameTiming(sequence: sequence, capturedNS: frame.capturedNS,
            encodeStartedNS: frame.encodeStartedNS, encodedNS: frame.encodedNS, enqueuedNS: time,
            captureUsesPresentationTimestamp: frame.captureUsesPresentationTimestamp)
        if frameEnqueued.count > 120 {
            frameEnqueued = frameEnqueued.filter { time >= $0.value.enqueuedNS && time - $0.value.enqueuedNS < 4_000_000_000 }
        }
    }
    private func waitForPresentation(_ first: EncodedFrame) async throws {
        let identity = UUID()
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            presentedWait = (identity, continuation)
            do { try sendJPEG(first) }
            catch { presentedWait = nil; continuation.resume(throwing: error); return }
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: 4_000_000_000)
                guard let self, self.presentedWait?.0 == identity else { return }
                let current = self.presentedWait; self.presentedWait = nil
                current?.1.resume(throwing: DeskError.timeout("LCD 呈现回执"))
            }
        }
    }
    func endDisplay(sendPad: Bool) async {
        transition += 1; let operation = transition
        let epoch = connectionEpoch
        changingMode = true; videoEnabled = false; displayActive = false
        input.release(); transport.clearVideo()
        let waitingJPEG = jpegWait; jpegWait = nil; waitingJPEG?.1.resume(throwing: CancellationError())
        let waitingFrame = presentedWait; presentedWait = nil; waitingFrame?.1.resume(throwing: CancellationError())
        let stream = capture; capture = nil
        let handle = displayHandle; displayHandle = nil
        activeSession = 0; firstJPEG = nil; frameEnqueued.removeAll(); frameTokens.removeAll()
        if let stream { await stream.stop() }
        if let handle { P4DisplayDestroy(handle) }
        if sendPad, usbOpen, connectionEpoch == epoch { _ = try? await request("set_mode", ["mode": "pad", "session": UInt32(0)], timeout: 1.2) }
        if transition == operation { changingMode = false; codec = "尚未启动" }
    }
    func run(_ action: DeskAction) async {
        do {
            switch action.kind {
            case "shortcut":
                guard let key = action.keyCode, let flags = action.modifiers else { throw DeskError.deviceRejected("快捷键") }
                try input.shortcut(key: key, modifiers: flags)
            case "application":
                guard let path = action.bundlePath, FileManager.default.fileExists(atPath: path) else { throw DeskError.deviceRejected("应用启动") }
                let configuration = NSWorkspace.OpenConfiguration(); configuration.activates = true
                _ = try await NSWorkspace.shared.openApplication(at: URL(fileURLWithPath: path), configuration: configuration)
            case "media": break // Firmware emits the standard HID Consumer report. Never duplicate it on macOS.
            default: throw DeskError.deviceRejected("动作类型")
            }
        } catch { report(error) }
    }

    func addNote() {
        guard snapshot.notes.count < 32 else { message = "便签最多 32 条。"; return }
        snapshot.notes.insert(Note(), at: 0); changed()
    }
    func deleteNote(_ id: String) {
        snapshot.notes.removeAll { $0.id == id }
        snapshot.deletedNoteIDs = Array(Set(snapshot.deletedNoteIDs).union([id])).sorted()
        changed()
    }
    func editNote(_ id: String, title: String? = nil, body: String? = nil) {
        guard let index = snapshot.notes.firstIndex(where: { $0.id == id }) else { return }
        if let title { snapshot.notes[index].title = String(title.unicodeScalars.prefix(128)) }
        if let body { snapshot.notes[index].body = String(body.unicodeScalars.prefix(2000)) }
        snapshot.notes[index].updatedMS = UInt64(Date().timeIntervalSince1970 * 1000)
        changed()
    }
    func addButton() {
        guard snapshot.buttons.count < 48 else { message = "按钮最多 48 个。"; return }
        snapshot.buttons.append(DeskButton()); changed()
    }
    func deleteButton(_ id: String) { snapshot.buttons.removeAll { $0.id == id }; changed() }
    func editButton(_ id: String, label: String? = nil, action: DeskAction? = nil) {
        guard let index = snapshot.buttons.firstIndex(where: { $0.id == id }) else { return }
        if let label { snapshot.buttons[index].label = String(label.unicodeScalars.prefix(64)) }
        if let action { snapshot.buttons[index].action = action }
        changed()
    }
    private func changed() {
        editRevision += 1; dirty = true
        saveTask?.cancel()
        saveTask = Task { [weak self] in
            do { try await Task.sleep(nanoseconds: 350_000_000) } catch { return }
            self?.save()
        }
    }
    private func save() {
        do {
            try snapshot.validated()
            try FileManager.default.createDirectory(at: storeURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try JSONEncoder().encode(snapshot).write(to: storeURL, options: .atomic)
        } catch { report(error) }
    }
    func selectFont() {
        let panel = NSOpenPanel(); panel.canChooseDirectories = false; panel.allowsMultipleSelection = false
        panel.allowedContentTypes = [.init(filenameExtension: "ttf")!, .init(filenameExtension: "otf")!]
        if panel.runModal() == .OK, let url = panel.url { fontPath = url.path; UserDefaults.standard.set(fontPath, forKey: "fontPath") }
    }
    func selectFontTool() {
        let panel = NSOpenPanel(); panel.canChooseDirectories = false; panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            fontToolPath = url.path; UserDefaults.standard.set(fontToolPath, forKey: "fontToolPath")
        }
    }
    func exportPerformance() {
        let panel = NSSavePanel(); panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "P4Desk-performance.json"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let encoder = JSONEncoder(); encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            encoder.keyEncodingStrategy = .convertToSnakeCase
            let metrics = try JSONSerialization.jsonObject(with: encoder.encode(performance.report(nowNS: DispatchTime.now().uptimeNanoseconds)))
            let report: [String: Any] = ["schema": "p4desk.host-performance.v1", "os": ProcessInfo.processInfo.operatingSystemVersionString,
                "display_width": 1024, "display_height": 600, "encoder": codec, "display_active": displayActive,
                "presented_total": presentedFrames,
                "measurement": "ScreenCaptureKit presentation timestamp (or capture callback) to host receipt of device LCD presentation ACK; includes USB return transit. Physical LCD latency is not measured.",
                "metrics": metrics]
            try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys]).write(to: url, options: .atomic)
            message = "已导出性能元数据；文件不包含画面、便签或按钮内容。"
        } catch { message = "性能诊断文件未能保存。" }
    }
    func sync() async {
        guard connected, !syncing else { return }
        guard sdReady else { message = "TF 卡未就绪，无法同步。"; return }
        syncing = true; syncProgress = 0
        let epoch = connectionEpoch
        var started = false
        defer { syncing = false; fontTask = nil }
        do {
            _ = try await request("get_state", [:], expected: "state")
            guard currentConnection(epoch) else { throw DeskError.usbDisconnected }
            var state = snapshot
            let (nextGeneration, overflow) = snapshot.generation.addingReportingOverflow(1)
            guard !overflow else { throw ProtocolError.invalidSnapshot("配置代次超出范围，无法继续同步。") }
            state.generation = max(nextGeneration, UInt64(Date().timeIntervalSince1970 * 1000))
            try state.validated()
            let revision = editRevision, tool = URL(fileURLWithPath: fontToolPath), font = URL(fileURLWithPath: fontPath)
            message = "正在生成本次配置的中文字库…"
            let bake = Task.detached(priority: .utility) { try FontPackage.bake(snapshot: state, toolURL: tool, fontURL: font) }
            fontTask = bake
            let data = try await bake.value
            guard connected, currentConnection(epoch) else { throw DeskError.usbDisconnected }
            let stateObject = try JSONSerialization.jsonObject(with: JSONEncoder().encode(state))
            _ = try await request("sync_begin", ["generation": state.generation, "state": stateObject,
                                                "font_length": data.count, "font_sha256": FontPackage.hash(data)], timeout: 5)
            started = true
            var offset = 0
            while offset < data.count {
                guard connected, currentConnection(epoch) else { throw DeskError.usbDisconnected }
                let count = min(32768, data.count - offset), id = try nextRequest()
                let value = UInt32(offset)
                var resource = Data((0..<4).map { UInt8(truncatingIfNeeded: value >> ($0 * 8)) })
                resource.append(data.subdata(in: offset..<(offset + count)))
                _ = try await awaitReply(Packet(kind: .resource, sequence: id, payload: resource), id: id, expected: "resource", timeout: 5)
                offset += count; syncProgress = Double(offset) / Double(data.count)
                message = "正在同步字体与配置：\(Int(syncProgress * 100))%"
            }
            guard currentConnection(epoch) else { throw DeskError.usbDisconnected }
            _ = try await request("sync_commit", ["generation": state.generation], timeout: 12)
            guard currentConnection(epoch) else { throw DeskError.usbDisconnected }
            deviceActions = Dictionary(uniqueKeysWithValues: state.buttons.map { ($0.id, $0.action) })
            snapshot.generation = max(snapshot.generation, state.generation)
            dirty = editRevision != revision
            save()
            message = dirty ? "本次同步已提交；新的编辑尚未同步。" : "同步已提交，设备已切换到新的便签、按钮与字库。"
        } catch {
            if started, currentConnection(epoch) { _ = try? await request("sync_abort", [:], timeout: 1.5) }
            report(error)
        }
    }
    func report(_ error: Error) {
        if let error = error as? ProtocolError, case .invalidSnapshot(let text) = error { message = text }
        else if error is CancellationError { return }
        else { message = (error as? DeskError)?.errorDescription ?? "操作失败，请检查设备、权限与资源。" }
    }
    func shutdown() async {
        stopping = true; heartbeatTask?.cancel(); fontTask?.cancel(); saveTask?.cancel()
        await endDisplay(sendPad: usbOpen)
        failPending(DeskError.usbDisconnected)
        transport.stop(); usbOpen = false; connected = false; save()
        for observer in observers { NSWorkspace.shared.notificationCenter.removeObserver(observer); NotificationCenter.default.removeObserver(observer) }
        observers.removeAll()
    }
}
