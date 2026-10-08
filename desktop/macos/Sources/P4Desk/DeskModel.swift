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
    @Published var connectionStatus = "未连接 USB（Type-A）"
    @Published var usbSpeedMbps: UInt64 = 0
    @Published var displayActive = false
    @Published var changingMode = false
    @Published var syncing = false
    @Published private var noticeQueue = OperationNoticeQueue()
    var activeNotice: OperationNotice? { noticeQueue.current }
    var suppressNotices = false
    private(set) var lastSyncResult = SyncResult()
    @Published var syncProgress = 0.0
    @Published var dirty = true
    @Published var sdReady = false
    @Published var screenAllowed = false
    @Published var inputAllowed = false
    @Published var message = "请用数据线连接板上 Type-A USB-OTG 大接口。Type-C 小接口用于供电／烧录调试，不能连接此副屏协议。"
    @Published var codec = "尚未启动"
    @Published var presentedFrames = 0
    @Published var presentationMS: Double?
    @Published var captureToReceiptP95MS: Double?
    @Published var effectivePresentedFPS = 0.0
    @Published var performanceSampleCount = 0
    @Published var performanceMetrics: PerformanceReport?
    @Published var encoderQueueStatistics: LatestFrameQueueStatistics?
    @Published var jpegDeliveryStatistics: LatestFrameQueueStatistics?
    @Published var encoderDiagnostics: JPEGEncoderDiagnostics?
    @Published var monitorSite = ""
    @Published var monitorKey = ""
    @Published var monitorBusy = false
    @Published var monitorMessage = "连接 USB 后可为设备配置独立 Wi-Fi 监控"
    @Published var monitorConfigured = false
    private(set) var monitorConfigurationSucceeded = false
    @Published var fileTransferSupported = false
    @Published var fileBusy = false
    @Published var fileUploading = false
    @Published var fileDirectory = "Downloads"
    @Published var fileEntries: [RemoteFileEntry] = []
    @Published var fileTotal: UInt32 = 0
    @Published var fileHasMore = false
    @Published var fileReadOnly = false
    @Published var fileProgress = 0.0
    @Published var fileTransferredBytes: UInt64 = 0
    @Published var fileTotalBytes: UInt64 = 0
    @Published var fileGlyphWarning = ""
    @Published var fileFontInstalled = false
    @Published var fileFontMissingCount = 0
    @Published var fileFontSkipped = false
    @Published var fileMessage = "连接 USB 后可传文件到 TF 卡"
    @Published var fileResults: [FileTransferResult] = []
    var fileNextOffset: UInt32 = 0
    var fileOperationIdentity: UUID?
    var fileOperationTask: Task<Void, Never>?
    @Published var fontPath = ""
    @Published var fontToolPath = ""

    private let transport = USBSession()
    private let input = MacInput()
    private var parser = PacketParser()
    private var usbOpen = false
    private var connectionEpoch: UInt64 = 0
    private var deviceActions: [String: DeskAction] = [:]
    private var capture: DisplayCapture?
    private var encodedDelivery: LatestFrameDelivery<EncodedFrame>?
    private var displayHandle: UnsafeMutableRawPointer?
    private var activeSession: UInt32 = 0
    private var directJPEGRotationDegrees = 0
    private var videoEnabled = false
    private var videoSequence: UInt16 = 0
    private var frameEnqueued: [UInt16: FrameTiming] = [:]
    private var frameTokens: [UInt32: UInt16] = [:]
    private var nextFrameToken: UInt32 = 1
    private var performance = PerformanceWindow()
    private var encodedTotal: UInt64 = 0
    private var enqueuedTotal: UInt64 = 0
    private var usbCompletedTotal: UInt64 = 0
    private var presentedTotal = 0
    private var latestPresentationMS: Double?
    private var firstJPEG: EncodedFrame?
    private var jpegWait: (UUID, CheckedContinuation<EncodedFrame, Error>)?
    private var presentedWait: (UUID, CheckedContinuation<Void, Error>)?
    private var transition = 0
    private var editRevision: UInt64 = 0
    private var requestSequence: UInt16 = 1
    private struct Pending {
        var identity: UUID
        var expected: String
        var acknowledged: String
        var continuation: CheckedContinuation<[String: Any], Error>
    }
    private var pending: [UInt16: Pending] = [:]
    private var heartbeatTask: Task<Void, Never>?
    private var performanceTask: Task<Void, Never>?
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
        fontPath = FontResources.resolvePath(
            savedPath: preferences.string(forKey: "fontPath"),
            bundledURL: Bundle.main.url(forResource: "HarmonyOS_Sans_SC_Regular", withExtension: "ttf")
        )
        fontToolPath = preferences.string(forKey: "fontToolPath") ?? ProcessInfo.processInfo.environment["P4DESK_FONTPACK_BIN"]
            ?? Bundle.main.url(forResource: "p4desk-fontpack", withExtension: nil)?.path ?? ""
    }
    func start() {
        guard !stopping else { return }
        refreshPermissions()
        transport.onEvent = { [weak self] event in self?.usbEvent(event) }
        transport.start()
        performanceTask?.cancel()
        performanceTask = Task { [weak self] in
            while !Task.isCancelled {
                // Keep collecting ACK metadata in the background. Sorting and
                // publishing the full report is only needed by a visible editor.
                if DeskEditorWindow.shared.isVisible { self?.refreshPerformance() }
                do { try await Task.sleep(nanoseconds: 1_000_000_000) }
                catch { return }
            }
        }
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
    var usbSpeedDescription: String {
        guard usbOpen else { return "尚未连接" }
        switch usbSpeedMbps {
        case 12: return "Full Speed · 12 Mbps"
        case 480: return "High Speed · 480 Mbps"
        case 5000: return "SuperSpeed · 5 Gbps"
        case 10000: return "SuperSpeed+ · 10 Gbps"
        default: return "已连接 · 协商速率未知"
        }
    }
    var usbBandwidthHint: String? {
        guard usbOpen, usbSpeedMbps == 12 else { return nil }
        return "当前协商为 Full Speed（12 Mbps），理论带宽上限为 1.5 MB/s，实际还要扣除 USB 开销。整幅 JPEG 的大小会限制动态窗口帧率；使用 HS 接口不代表已协商到 480 Mbps。"
    }
    func refreshPerformance() {
        let report = performance.report(nowNS: DispatchTime.now().uptimeNanoseconds)
        performanceMetrics = report
        captureToReceiptP95MS = report.captureToReceiptP95MS
        effectivePresentedFPS = report.effectivePresentedFPS
        performanceSampleCount = report.sampleCount
        presentedFrames = presentedTotal
        presentationMS = latestPresentationMS
        if let capture { encoderQueueStatistics = capture.encoderStatistics }
        if let delivery = encodedDelivery {
            let statistics = delivery.statistics
            jpegDeliveryStatistics = statistics; encodedTotal = statistics.submitted
        }
    }
    private func usbEvent(_ event: USBSession.Event) {
        switch event {
        case .connected(let speedMbps):
            connectionEpoch &+= 1
            let epoch = connectionEpoch
            usbOpen = true; usbSpeedMbps = speedMbps; connected = false; parser.reset(); connectionStatus = "USB 已打开，正在握手"
            lastDeviceContact = Date.timeIntervalSinceReferenceDate
            startHeartbeat()
            Task { await establish(epoch: epoch) }
        case .disconnected:
            connectionEpoch &+= 1
            usbOpen = false; usbSpeedMbps = 0; connected = false; sdReady = false; directJPEGRotationDegrees = 0
            fileTransferSupported = false
            invalidateFileOperation(message: "连接 USB 后可浏览 TF 卡与传输文件")
            deviceActions.removeAll()
            connectionStatus = "USB 已断开"; parser.reset(); failPending(DeskError.usbDisconnected)
            heartbeatTask?.cancel(); heartbeatTask = nil
            fontTask?.cancel()
            Task { await endDisplay(sendPad: false) }
        case .bytes(let data):
            for packet in parser.feed(data) {
                if packet.kind == .control, let fields = try? JSONControl.object(packet.payload) { receive(fields, sequence: packet.sequence) }
            }
        case .sent(let token, let timeNS, let transferUS):
            guard let sequence = frameTokens.removeValue(forKey: token) else { break }
            usbCompletedTotal &+= 1
            if var timing = frameEnqueued[sequence] {
                timing.usbSentNS = timeNS; timing.usbTransferUS = transferUS
                frameEnqueued[sequence] = timing; performance.recordTransfer(timing)
            } else { performance.markSent(sequence: sequence, timeNS: timeNS, transferUS: transferUS) }
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
            fileTransferSupported = caps["file_transfer"] as? Bool ?? false
            fileMessage = fileTransferSupported ? "选择目标文件夹，上传后可在板上文件管理查看" : "设备固件不支持文件传输，请升级固件"
            directJPEGRotationDegrees = number(caps, "direct_jpeg_rotation_degrees") == 180 ? 180 : 0
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
            connected = false; connectionStatus = "设备握手失败"
            // Reconnect attempts are connection state, not completed user
            // operations. Do not repeatedly open alerts in the background.
            message = "设备握手未完成，请检查 USB、设备固件和 TF 卡状态。"
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
    func importMonitorConfiguration() {
        do {
            let c = try MonitorConfiguration.readLocal()
            monitorSite = c.site; monitorKey = c.key
            setNotice(title: "本机配置已读取", message: "点击“下发并验证”后，配置会由 P4 验证并保存。")
        } catch { setError(title: "未找到本机配置", message: "未找到可用的本机用量监控配置，请手动填写站点和管理员 API Key。") }
    }
    func configureMonitor(forget: Bool = false) async {
        guard connected, !monitorBusy else { return }
        monitorConfigurationSucceeded = false
        monitorBusy = true
        defer { monitorBusy = false }
        do {
            if forget { _ = try await request("monitor_forget", [:]) }
            else {
                let c = try MonitorConfiguration(site: monitorSite, key: monitorKey)
                _ = try await request("monitor_configure", ["site": c.site, "key": c.key])
            }
            monitorKey = ""
            monitorMessage = forget ? "正在清除设备配置" : "设备正在通过 Wi-Fi 验证连接…"
            for _ in 0..<90 {
                try await Task.sleep(nanoseconds: 1_000_000_000)
                let status = try await request("monitor_get_status", [:], expected: "monitor_status")
                monitorConfigured = status["configured"] as? Bool == true
                if let ok = status["configuration_result"] as? Bool {
                    monitorConfigurationSucceeded = ok
                    resetMonitorStatus()
                    if ok {
                        setNotice(title: forget ? "监控配置已清除" : "监控配置已保存", message: forget ? "设备上的监控配置已清除。" : "验证和保存成功，P4 可以脱离 Mac 独立刷新。")
                    } else { setError(title: "监控配置失败", message: "设备未能验证或保存监控配置，请检查板上 Wi-Fi、时间与管理员密钥后重试。") }
                    return
                }
            }
            resetMonitorStatus()
            setError(title: "监控配置未完成", message: "设备未在等待时间内完成验证，请检查板上连接状态后重试。")
        } catch {
            // Do not interpolate configuration, payloads or database errors.
            resetMonitorStatus()
            if !(error is CancellationError) { setError(title: "监控配置失败", message: "配置未完成，请检查 USB、板上 Wi-Fi、时间和管理员密钥。") }
        }
    }
    private func resetMonitorStatus() {
        monitorMessage = connected ? (monitorConfigured ? "设备已保存独立 Wi-Fi 监控配置" : "设备尚未配置独立 Wi-Fi 监控") : "连接 USB 后可读取设备监控状态"
    }
    func refreshMonitorStatus(userInitiated: Bool = false) async {
        guard connected else { return }
        do {
            let status = try await request("monitor_get_status", [:], expected: "monitor_status")
            monitorConfigured = status["configured"] as? Bool == true
            resetMonitorStatus()
        } catch {
            resetMonitorStatus()
            if userInitiated, !(error is CancellationError) { setError(title: "读取状态失败", message: "无法读取监控状态，请检查连接和固件版本。") }
        }
    }
    private func request(_ op: String, _ fields: [String: Any], expected: String? = nil, timeout: Double = 3) async throws -> [String: Any] {
        guard usbOpen else { throw DeskError.usbDisconnected }
        let id = try nextRequest()
        var object = fields; object["op"] = op; object["request_id"] = id
        let packet = Packet(kind: .control, sequence: id, payload: try JSONControl.data(object))
        return try await awaitReply(packet, id: id, expected: expected ?? op, acknowledged: op, timeout: timeout)
    }
    private func awaitReply(_ packet: Packet, id: UInt16, expected: String, acknowledged: String? = nil, timeout: Double) async throws -> [String: Any] {
        let identity = UUID()
        return try await withTaskCancellationHandler {
            try Task.checkCancellation()
            return try await withCheckedThrowingContinuation { continuation in
                pending[id] = Pending(identity: identity, expected: expected, acknowledged: acknowledged ?? expected, continuation: continuation)
                if Task.isCancelled {
                    pending.removeValue(forKey: id)?.continuation.resume(throwing: CancellationError()); return
                }
                do { try transport.send(packet) }
                catch { pending.removeValue(forKey: id)?.continuation.resume(throwing: error); return }
                Task { [weak self] in
                    do { try await Task.sleep(nanoseconds: UInt64(timeout * 1_000_000_000)) } catch { return }
                    guard let self, let current = self.pending[id], current.identity == identity else { return }
                    self.pending.removeValue(forKey: id)?.continuation.resume(throwing: DeskError.timeout(expected))
                }
            }
        } onCancel: {
            Task { @MainActor [weak self] in
                guard let self, self.pending[id]?.identity == identity else { return }
                self.pending.removeValue(forKey: id)?.continuation.resume(throwing: CancellationError())
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
            } catch {
                // The request owner reports one failure. Unsolicited or old
                // state packets must not create another background alert.
                if let id, id == sequence, pending[id]?.expected == "state" {
                    pending.removeValue(forKey: id)?.continuation.resume(throwing: DeskError.invalidDevice)
                }
                return
            }
        }
        if let id, id == sequence, let waiting = pending[id] {
            if op == waiting.expected || (op == "ack" && fields["acknowledged"] as? String == waiting.acknowledged) {
                pending.removeValue(forKey: id)
                if op == "ack", fields["ok"] as? Bool != true {
                    if waiting.expected.hasPrefix("file_") {
                        // Never display untrusted payload text or interpolate a path.
                        waiting.continuation.resume(throwing: FileTransferError.rejected(FileAcknowledgement.safeErrorCode(fields)))
                    } else { waiting.continuation.resume(throwing: DeviceFailure(code: DeviceAcknowledgement.safeErrorCode(fields))) }
                }
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
            timing.deviceMetrics = DeviceFrameMetrics(fields: fields)
            latestPresentationMS = timing.queueToReceiptMS
            _ = performance.record(timing)
            presentedTotal += 1
            // Full reports sort up to 1800 samples. Compute on the 1s timer, plus the first ACK.
            if let waiting = presentedWait {
                refreshPerformance()
                presentedWait = nil; waiting.1.resume()
            }
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
        ControlNumber.unsigned(object[key])
    }
    private func failPending(_ error: Error) {
        let waiting = pending.values; pending.removeAll()
        for item in waiting { item.continuation.resume(throwing: error) }
    }

    func beginDisplay() async {
        guard connected, !changingMode, !displayActive, !sleeping, !stopping else { return }
        guard !fileBusy, !syncing else { setError(title: "暂时无法开启副屏", message: "请等待文件传输或配置同步完成后再开启副屏。"); return }
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
        encodedTotal = 0; enqueuedTotal = 0; usbCompletedTotal = 0
        encoderDiagnostics = nil; encoderQueueStatistics = nil; jpegDeliveryStatistics = nil
        let delivery = LatestFrameDelivery<EncodedFrame>(); delivery.start(); encodedDelivery = delivery
        let stream = DisplayCapture(); capture = stream
        stream.onJPEG = { [weak self] data in
            guard let token = delivery.submit(data) else { return }
            Task { @MainActor [weak self] in
                guard let self else { delivery.stop(); return }
                self.deliverEncoded(from: delivery, token: token, operation: operation)
            }
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
        stream.onDiagnostics = { [weak self] diagnostics in Task { @MainActor in
            guard let self, self.transition == operation else { return }; self.encoderDiagnostics = diagnostics
        } }
        do {
            try await stream.start(displayID: P4DisplayID(handle), jpegRotationDegrees: directJPEGRotationDegrees)
            guard transition == operation else { return }
            let first = try await awaitJPEG()
            guard transition == operation, connected else { throw CancellationError() }
            activeSession = UInt32.random(in: 1...UInt32.max)
            _ = try await request("set_mode", ["mode": "display", "session": activeSession,
                                               "jpeg_rotation_degrees": directJPEGRotationDegrees], timeout: 3)
            guard transition == operation else { return }
            videoEnabled = true; videoSequence = 0; presentedTotal = 0; presentedFrames = 0; latestPresentationMS = nil
            frameEnqueued.removeAll(); frameTokens.removeAll()
            performance.reset(nowNS: DispatchTime.now().uptimeNanoseconds); captureToReceiptP95MS = nil; presentationMS = nil
            effectivePresentedFPS = 0; performanceSampleCount = 0
            try await waitForPresentation(first)
            guard transition == operation else { return }
            displayActive = true; changingMode = false
            message = inputAllowed ? "USB 副屏已呈现，支持单指点击/拖动与双指滚动。" : "USB 副屏已呈现；开启辅助功能后可触摸操作。"
        } catch {
            if transition == operation { report(error); await endDisplay(sendPad: usbOpen) }
        }
    }
    private func deliverEncoded(from delivery: LatestFrameDelivery<EncodedFrame>, token: LatestDeliveryToken, operation: Int) {
        guard transition == operation, encodedDelivery === delivery else { delivery.stop(); return }
        guard let jpeg = delivery.take(token) else { return }
        encoded(jpeg, operation: operation)
        if let next = delivery.complete(token) {
            Task { @MainActor [weak self] in
                guard let self else { delivery.stop(); return }
                self.deliverEncoded(from: delivery, token: next, operation: operation)
            }
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
        enqueuedTotal &+= 1
        frameTokens[token] = sequence
        if frameTokens.count > 256 {
            for obsolete in frameTokens.keys.sorted().prefix(frameTokens.count - 256) { frameTokens.removeValue(forKey: obsolete) }
        }
        frameEnqueued[sequence] = FrameTiming(sequence: sequence, capturedNS: frame.capturedNS,
            encodeStartedNS: frame.encodeStartedNS, encodedNS: frame.encodedNS, enqueuedNS: time,
            captureUsesPresentationTimestamp: frame.captureUsesPresentationTimestamp, jpegBytes: UInt64(frame.data.count),
            captureTimestampSource: frame.captureTimestampSource, jpegQuality: frame.jpegQuality,
            jpegSampling: frame.jpegSampling, jpegEncodingAttempts: frame.jpegEncodingAttempts,
            jpegPayloadFallback: frame.jpegPayloadFallback)
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
        if let delivery = encodedDelivery {
            delivery.stop(); jpegDeliveryStatistics = delivery.statistics
            encodedTotal = delivery.statistics.submitted
        }
        encodedDelivery = nil
        performance.stop(); refreshPerformance()
        input.release(); transport.clearVideo()
        let waitingJPEG = jpegWait; jpegWait = nil; waitingJPEG?.1.resume(throwing: CancellationError())
        let waitingFrame = presentedWait; presentedWait = nil; waitingFrame?.1.resume(throwing: CancellationError())
        let stream = capture; capture = nil
        let handle = displayHandle; displayHandle = nil
        activeSession = 0; firstJPEG = nil; frameEnqueued.removeAll(); frameTokens.removeAll()
        if let stream { await stream.stop() }
        if transition == operation, let stream { encoderQueueStatistics = stream.encoderStatistics }
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
        guard snapshot.notes.count < 32 else { setError(title: "无法添加便签", message: "便签最多 32 条，请删除不需要的便签后重试。"); return }
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
        guard snapshot.buttons.count < 48 else { setError(title: "无法添加快捷按钮", message: "快捷按钮最多 48 个，请删除不需要的按钮后重试。"); return }
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
    @discardableResult
    private func save(notifyOnFailure: Bool = true) -> Bool {
        do {
            try snapshot.validated()
            try FileManager.default.createDirectory(at: storeURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try JSONEncoder().encode(snapshot).write(to: storeURL, options: .atomic)
            return true
        } catch {
            if notifyOnFailure { setError(title: "本机保存失败", message: "编辑数据未能保存到 Mac，请检查本机磁盘空间和权限。设备已保存的数据不受影响。") }
            return false
        }
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
        let data: Data
        do {
            let observedNS = DispatchTime.now().uptimeNanoseconds
            let encoder = JSONEncoder(); encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            encoder.keyEncodingStrategy = .convertToSnakeCase
            let metrics = try JSONSerialization.jsonObject(with: encoder.encode(performance.report(nowNS: observedNS)))
            var report: [String: Any] = ["schema": "p4desk.host-performance.v1", "os": ProcessInfo.processInfo.operatingSystemVersionString,
                "observed_ns": observedNS,
                "display_width": 1024, "display_height": 600, "encoder": codec, "display_active": displayActive,
                "usb_speed_mbps": usbSpeedMbps, "usb_speed": usbSpeedDescription,
                "capture_target_fps": VideoProfile.framesPerSecond, "jpeg_requested_quality": VideoProfile.jpegQuality,
                "presented_total": presentedTotal, "encoded_total": encodedDelivery?.statistics.submitted ?? encodedTotal,
                "enqueued_total": enqueuedTotal, "usb_completed_total": usbCompletedTotal,
                "measurement": "ScreenCaptureKit WindowServer display time, sample presentation timestamp, or capture callback to host receipt of device LCD presentation ACK; source is recorded per frame. Includes USB return transit. Physical LCD latency is not measured.",
                "rates": "Successful JPEG packets and presentation ACKs divided by elapsed observation time, at most 30 seconds, including idle time. JPEG wire throughput includes each 16-byte header, excludes control/HID traffic. Stopped stream rates are zero; a static desktop normally generates fewer frames.",
                "device_stages": "decode includes JPEG validation and hardware decode; copy includes crop/rotation, or is zero for negotiated direct decode; present includes LCD draw/cache writeback through the first full DMA source-read completion. DMA completion is not optical display completion. Separate host/device monotonic clocks are never subtracted.",
                "metrics": metrics]
            if let diagnostics = encoderDiagnostics {
                report["encoder_diagnostics"] = try JSONSerialization.jsonObject(with: encoder.encode(diagnostics))
            }
            if let statistics = capture?.encoderStatistics ?? encoderQueueStatistics {
                report["encoder_queue"] = try JSONSerialization.jsonObject(with: encoder.encode(statistics))
            }
            if let statistics = encodedDelivery?.statistics ?? jpegDeliveryStatistics {
                report["main_actor_delivery"] = try JSONSerialization.jsonObject(with: encoder.encode(statistics))
            }
            // Freeze the observation before showing the picker. A blocking
            // runModal stalls MainActor JPEG/USB delivery and biases the FPS.
            data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
        } catch { setError(title: "导出失败", message: "性能诊断数据未能生成，请重试。"); return }
        let panel = NSSavePanel(); panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "P4Desk-performance.json"
        let save: (NSApplication.ModalResponse) -> Void = { [weak self] response in
            guard response == .OK, let url = panel.url else { return }
            do {
                try data.write(to: url, options: .atomic)
                self?.setNotice(title: "诊断数据已导出", message: "文件已保存，其中不包含画面、便签或快捷按钮内容。")
            } catch { self?.setError(title: "导出失败", message: "性能诊断文件未能保存，请检查目标文件夹权限和磁盘空间。") }
        }
        if let window = NSApp.keyWindow { panel.beginSheetModal(for: window, completionHandler: save) }
        else { panel.begin(completionHandler: save) }
    }
    func sync() async {
        lastSyncResult = SyncResult()
        guard connected else { failSync(DeskError.usbDisconnected); return }
        guard !syncing, !fileBusy, !displayActive, !changingMode else { failSync(FileTransferError.busy); return }
        guard sdReady else { failSync(FileTransferError.storageUnavailable); return }
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
            lastSyncResult.generation = state.generation
            let revision = editRevision, tool = URL(fileURLWithPath: fontToolPath), font = URL(fileURLWithPath: fontPath)
            message = "正在生成本次配置的中文字库…"
            let bake = Task.detached(priority: .utility) { try FontPackage.bake(snapshot: state, toolURL: tool, fontURL: font) }
            fontTask = bake
            let data = try await bake.value
            lastSyncResult.fontBytes = data.count
            guard connected, currentConnection(epoch) else { throw DeskError.usbDisconnected }
            let stateObject = try JSONSerialization.jsonObject(with: JSONEncoder().encode(state))
            // The device may open staging before its reply reaches the host.
            started = true
            _ = try await request("sync_begin", ["generation": state.generation, "state": stateObject,
                                                "font_length": data.count, "font_sha256": FontPackage.hash(data)], timeout: 5)
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
            let localSaved = save(notifyOnFailure: false)
            lastSyncResult.committed = true; lastSyncResult.errorCode = "none"
            if localSaved {
                setNotice(title: "同步完成", message: dirty ? "本次同步已提交。同步期间产生的新编辑尚未同步。" : "设备已切换到新的便签、快捷按钮和字体。")
            } else {
                setNotice(title: "设备同步完成，本机保存未完成", message: "新的便签、快捷按钮和字体已在设备生效；Mac 本地副本未能保存。请检查本机磁盘空间和权限后再同步。")
            }
        } catch {
            if started, currentConnection(epoch) { _ = try? await request("sync_abort", [:], timeout: 1.5) }
            failSync(error)
        }
    }
    var fileConnectionEpoch: UInt64 { connectionEpoch }
    func fileControl(_ op: String, _ fields: [String: Any], expected: String? = nil, timeout: Double = 8) async throws -> [String: Any] {
        try await request(op, fields, expected: expected, timeout: timeout)
    }
    func fileChunk(offset: UInt32, data: Data) async throws {
        let id = try nextRequest()
        let payload = try FileResourceChunk.payload(offset: offset, data: data)
        _ = try await awaitReply(Packet(kind: .resource, sequence: id, payload: payload), id: id, expected: "file_chunk", timeout: 8)
    }

    func setError(title: String = "操作未完成", message text: String) {
        setNotice(title: title, message: text)
    }
    func setNotice(title: String, message text: String) {
        // Only connection and progress live in the footer. Success, warning and
        // failure results share one dismissible presentation and CLI suppression.
        message = connected ? (displayActive ? "USB 副屏运行中" : "Pad 已连接，可同步编辑或传输文件。") : "USB 未连接，连接后可继续操作。"
        let notice = OperationNotice(title: title, message: text)
        guard noticeQueue.enqueue(notice, suppressed: suppressNotices || stopping) else { return }
        if !DeskEditorWindow.shared.isVisible { DeskEditorWindow.shared.show() }
    }
    func dismissNotice() {
        noticeQueue.dismissCurrent()
        // Let SwiftUI finish dismissing the current alert before presenting a
        // queued result. A new success must not replace an unacknowledged error.
        DispatchQueue.main.async { [weak self] in self?.noticeQueue.presentNext() }
    }
    private func errorMessage(_ error: Error) -> String {
        if let error = error as? DeviceFailure { return error.errorDescription ?? "设备操作未完成。" }
        if let error = error as? FileTransferError { return error.errorDescription ?? "文件操作未完成。" }
        if let error = error as? DeskError { return error.errorDescription ?? "操作未完成。" }
        if error is ProtocolError { return "配置未通过验证，请检查便签、快捷按钮或减少内容后重试。" }
        return "操作未完成，请检查设备、权限与资源后重试。"
    }
    private func failSync(_ error: Error) {
        lastSyncResult.errorCode = Diagnostics.safeErrorCode(error)
        if error is CancellationError { return }
        setError(title: "同步失败", message: errorMessage(error) + "\n\n设备已有的便签、快捷按钮和字体会保留。")
    }
    func report(_ error: Error) {
        guard !(error is CancellationError) else { return }
        setError(message: errorMessage(error))
    }
    func shutdown() async {
        stopping = true; invalidateFileOperation(message: "应用正在退出，传输已停止"); heartbeatTask?.cancel(); performanceTask?.cancel(); fontTask?.cancel(); saveTask?.cancel()
        await endDisplay(sendPad: usbOpen)
        failPending(DeskError.usbDisconnected)
        transport.stop(); usbOpen = false; connected = false; save()
        for observer in observers { NSWorkspace.shared.notificationCenter.removeObserver(observer); NotificationCenter.default.removeObserver(observer) }
        observers.removeAll()
    }
}
