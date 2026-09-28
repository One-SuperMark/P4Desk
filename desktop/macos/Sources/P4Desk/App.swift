import SwiftUI
import AppKit
import P4DeskCore

@MainActor
final class DeskAppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        DeskModel.shared.start()
        DispatchQueue.main.async { DeskEditorWindow.shared.show() }
    }
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        DeskEditorWindow.shared.show()
        return false
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        Task { await DeskModel.shared.shutdown(); sender.reply(toApplicationShouldTerminate: true) }
        return .terminateLater
    }
}

/// Own the editor window directly so launch, reopen and the menu use one instance.
/// This does not depend on SwiftUI's internal window identifiers or scene restoration.
@MainActor
final class DeskEditorWindow {
    static let shared = DeskEditorWindow()
    private var controller: NSWindowController?
    private init() {}

    func show() {
        if controller == nil {
            let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 960, height: 650),
                                  styleMask: [.titled, .closable, .miniaturizable, .resizable],
                                  backing: .buffered, defer: false)
            window.title = "P4 Desk"
            window.contentViewController = NSHostingController(rootView: DeskEditor(model: .shared))
            window.contentMinSize = NSSize(width: 800, height: 560)
            window.setContentSize(NSSize(width: 960, height: 650))
            window.isReleasedWhenClosed = false
            let frameName = NSWindow.FrameAutosaveName("P4Desk.EditorWindow")
            if !window.setFrameUsingName(frameName) { window.center() }
            window.setFrameAutosaveName(frameName)
            controller = NSWindowController(window: window)
        }
        guard let controller, let window = controller.window else { return }
        if window.isMiniaturized { window.deminiaturize(nil) }
        NSApp.unhide(nil)
        controller.showWindow(nil)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}

struct P4DeskApp: App {
    @NSApplicationDelegateAdaptor(DeskAppDelegate.self) private var delegate
    @StateObject private var model = DeskModel.shared
    var body: some Scene {
        MenuBarExtra("P4 Desk", systemImage: model.displayActive ? "display.2" : "rectangle.and.hand.point.up.left") {
            MenuContent(model: model)
        }
        Settings { DeskSettings(model: model).frame(width: 560) }
    }
}

struct MenuContent: View {
    @ObservedObject var model: DeskModel
    var body: some View {
        Text(model.connectionStatus)
        Text(model.displayActive ? "当前模式：USB 副屏" : "当前模式：Pad")
        Button(model.displayActive ? "切回 Pad" : "开启 USB 副屏") {
            Task { if model.displayActive { await model.endDisplay(sendPad: true) } else { await model.beginDisplay() } }
        }.disabled(!model.connected || model.changingMode)
        Button("打开便签与按钮配置") { DeskEditorWindow.shared.show() }
        Button(model.syncing ? "正在同步…" : "同步到设备") { Task { await model.sync() } }
            .disabled(!model.connected || model.syncing || !model.sdReady)
        Divider()
        SettingsLink { Text("设置与权限…") }
        Button("重新连接") { model.reconnect() }
        Divider()
        Button("退出 P4 Desk") { NSApp.terminate(nil) }.keyboardShortcut("q")
    }
}

struct DeskEditor: View {
    @ObservedObject var model: DeskModel
    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Image(systemName: model.connected ? "checkmark.circle.fill" : "cable.connector")
                    .foregroundStyle(model.connected ? Color.green : Color.secondary)
                VStack(alignment: .leading) {
                    Text(model.connectionStatus).font(.headline)
                    Text(model.displayActive ? "USB 副屏 · 1024 × 600" : "Pad · 离线便签与桌面快捷面板").font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                Button(model.displayActive ? "切回 Pad" : "开启副屏") {
                    Task { if model.displayActive { await model.endDisplay(sendPad: true) } else { await model.beginDisplay() } }
                }.disabled(!model.connected || model.changingMode)
                Button(model.syncing ? "同步中" : model.dirty ? "同步更改" : "重新同步") { Task { await model.sync() } }
                    .buttonStyle(.borderedProminent).disabled(!model.connected || model.syncing || !model.sdReady)
                SettingsLink { Image(systemName: "gearshape") }
            }.padding()
            Divider()
            TabView {
                NotesEditor(model: model).tabItem { Label("便签", systemImage: "note.text") }
                ButtonsEditor(model: model).tabItem { Label("快捷按钮", systemImage: "square.grid.3x3") }
                DisplayStatus(model: model).tabItem { Label("副屏", systemImage: "display") }
            }.padding(.top, 8)
            Divider()
            HStack {
                Text(model.message).font(.caption).lineLimit(3).frame(maxWidth: .infinity, alignment: .leading)
                if model.syncing { ProgressView(value: model.syncProgress).frame(width: 160) }
            }.padding(12)
        }.frame(minWidth: 800, minHeight: 560)
    }
}

struct NotesEditor: View {
    @ObservedObject var model: DeskModel
    @State private var selection: String?
    var body: some View {
        HSplitView {
            VStack(spacing: 0) {
                List(selection: $selection) {
                    ForEach(model.snapshot.notes) { note in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(note.title.isEmpty ? "未命名便签" : note.title).font(.headline).lineLimit(1)
                            Text(note.body.isEmpty ? "暂无内容" : note.body).font(.caption).foregroundStyle(.secondary).lineLimit(2)
                        }.padding(.vertical, 4).tag(note.id)
                    }
                }
                HStack {
                    Button { model.addNote(); selection = model.snapshot.notes.first?.id } label: { Image(systemName: "plus") }
                    Button { if let selection { model.deleteNote(selection); self.selection = model.snapshot.notes.first?.id } } label: { Image(systemName: "minus") }
                        .disabled(selection == nil)
                    Spacer(); Text("\(model.snapshot.notes.count)/32").font(.caption).foregroundStyle(.secondary)
                }.padding(10)
            }.frame(minWidth: 220, idealWidth: 260)
            if let id = selection, let note = model.snapshot.notes.first(where: { $0.id == id }) {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("标题", text: Binding(get: { model.snapshot.notes.first(where: { $0.id == id })?.title ?? "" },
                                                       set: { model.editNote(id, title: $0) })).textFieldStyle(.roundedBorder)
                    TextEditor(text: Binding(get: { model.snapshot.notes.first(where: { $0.id == id })?.body ?? "" },
                                             set: { model.editNote(id, body: $0) }))
                        .font(.system(size: 16)).padding(6).background(.quaternary.opacity(0.15)).clipShape(RoundedRectangle(cornerRadius: 8))
                    HStack { Text("本地自动保存 · USB 同步后可在 Pad 离线查看"); Spacer(); Text("\(note.body.unicodeScalars.count)/2000") }
                        .font(.caption).foregroundStyle(.secondary)
                }.padding().frame(minWidth: 430)
            } else {
                ContentUnavailableView("选择或新建便签", systemImage: "note.text", description: Text("中文内容会随配置生成字库并同步到设备。"))
                    .frame(minWidth: 430)
            }
        }.onAppear { if selection == nil { selection = model.snapshot.notes.first?.id } }
    }
}

struct ButtonsEditor: View {
    @ObservedObject var model: DeskModel
    @State private var selection: String?
    var body: some View {
        HSplitView {
            VStack(spacing: 0) {
                List(selection: $selection) {
                    ForEach(model.snapshot.buttons) { button in
                        Label(button.label.isEmpty ? "未命名按钮" : button.label, systemImage: icon(button.action.kind)).tag(button.id)
                    }
                }
                HStack {
                    Button { model.addButton(); selection = model.snapshot.buttons.last?.id } label: { Image(systemName: "plus") }
                    Button { if let selection { model.deleteButton(selection); self.selection = model.snapshot.buttons.first?.id } } label: { Image(systemName: "minus") }
                        .disabled(selection == nil)
                    Spacer(); Text("\(model.snapshot.buttons.count)/48").font(.caption).foregroundStyle(.secondary)
                }.padding(10)
            }.frame(minWidth: 220, idealWidth: 260)
            if let id = selection, let button = model.snapshot.buttons.first(where: { $0.id == id }) {
                Form {
                    TextField("按钮名称", text: Binding(get: { model.snapshot.buttons.first(where: { $0.id == id })?.label ?? "" },
                                                          set: { model.editButton(id, label: $0) }))
                    Picker("操作", selection: Binding(get: { button.action.kind }, set: { kind in
                        let action: DeskAction = kind == "application" ? .application("/System/Library/CoreServices/Finder.app")
                            : kind == "media" ? .media(0xcd) : .shortcut(8, flags: 1 << 20)
                        model.editButton(id, action: action)
                    })) {
                        Text("快捷键").tag("shortcut"); Text("启动应用").tag("application"); Text("媒体控制").tag("media")
                    }
                    switch button.action.kind {
                    case "shortcut":
                        LabeledContent("组合键") {
                            ShortcutRecorder(action: button.action) { key, flags in model.editButton(id, action: .shortcut(key, flags: flags)) }
                                .frame(width: 230, height: 36)
                        }
                        Text("点击组合键框后按下所需快捷键。按钮作用于 Mac 当前应用。").font(.caption).foregroundStyle(.secondary)
                    case "application":
                        LabeledContent("应用") {
                            Button(button.action.bundlePath.map { URL(fileURLWithPath: $0).deletingPathExtension().lastPathComponent } ?? "选择应用…") {
                                let panel = NSOpenPanel(); panel.canChooseDirectories = false; panel.allowsMultipleSelection = false
                                panel.allowedContentTypes = [.applicationBundle]
                                if panel.runModal() == .OK, let url = panel.url { model.editButton(id, action: .application(url.path)) }
                            }
                        }
                        Text("从本机选择应用；无需屏幕录制权限。").font(.caption).foregroundStyle(.secondary)
                    default:
                        Picker("媒体操作", selection: Binding(get: { button.action.usage ?? 0xcd }, set: { model.editButton(id, action: .media($0)) })) {
                            Text("播放").tag(UInt16(0xb0))
                            Text("播放 / 暂停").tag(UInt16(0xcd)); Text("下一首").tag(UInt16(0xb5)); Text("上一首").tag(UInt16(0xb6))
                            Text("静音").tag(UInt16(0xe2)); Text("增大音量").tag(UInt16(0xe9)); Text("减小音量").tag(UInt16(0xea))
                        }
                        Text("媒体按钮通过设备的标准 USB 媒体键执行。").font(.caption).foregroundStyle(.secondary)
                    }
                }.formStyle(.grouped).frame(minWidth: 430)
            } else { ContentUnavailableView("选择或新建按钮", systemImage: "square.grid.3x3").frame(minWidth: 430) }
        }.onAppear { if selection == nil { selection = model.snapshot.buttons.first?.id } }
    }
    private func icon(_ kind: String) -> String { kind == "application" ? "app" : kind == "media" ? "play.circle" : "keyboard" }
}

struct DisplayStatus: View {
    @ObservedObject var model: DeskModel
    var body: some View {
        Form {
            LabeledContent("当前模式", value: model.displayActive ? "USB 副屏" : "Pad")
            LabeledContent("画面", value: "1024 × 600，1 倍桌面比例")
            LabeledContent("编码", value: model.codec)
            LabeledContent("LCD 呈现回执", value: "\(model.presentedFrames) 帧")
            if let ms = model.presentationMS { LabeledContent("最近发送至呈现回执", value: String(format: "%.1f ms", ms)) }
            if let ms = model.captureToReceiptP95MS { LabeledContent("采集至呈现回执 P95（近 30 秒）", value: String(format: "%.1f ms · %d 样本", ms, model.performanceSampleCount)) }
            LabeledContent("有效呈现帧率（近 30 秒）", value: String(format: "%.1f FPS", model.effectivePresentedFPS))
            Text("回执耗时含采集、编码、USB 发送及设备回传；不代表 LCD 光学实测延迟。").font(.caption).foregroundStyle(.secondary)
            Button("导出性能诊断 JSON…") { model.exportPerformance() }
            LabeledContent("屏幕录制", value: model.screenAllowed ? "已允许" : "未允许")
            LabeledContent("辅助功能", value: model.inputAllowed ? "已允许" : "未允许")
            Text("开启后在系统显示设置中排列 P4 Desk，可将任意普通窗口拖入。单指点击或拖动、双指滚动；板上三指长按退出。断线或睡眠后回到 Pad。").foregroundStyle(.secondary)
            Text("仅连接 USB HS 数据接口。应用在收到设备 LCD 呈现回执后显示副屏已开启。").font(.caption).foregroundStyle(.secondary)
        }.formStyle(.grouped)
    }
}

struct DeskSettings: View {
    @ObservedObject var model: DeskModel
    var body: some View {
        Form {
            Section("系统权限") {
                HStack {
                    Text("屏幕录制：\(model.screenAllowed ? "已允许" : "未允许")"); Spacer()
                    Button("请求权限") { model.requestScreenPermission() }
                    Button("系统设置") { model.openPrivacy("Privacy_ScreenCapture") }
                }
                HStack {
                    Text("辅助功能：\(model.inputAllowed ? "已允许" : "未允许")"); Spacer()
                    Button("请求权限") { model.requestInputPermission() }
                    Button("系统设置") { model.openPrivacy("Privacy_Accessibility") }
                }
                Button("刷新权限状态") { model.refreshPermissions() }
            }
            Section("中文字库") {
                HStack { Text(model.fontPath.isEmpty ? "尚未选择 TTF/OTF 字库" : URL(fileURLWithPath: model.fontPath).lastPathComponent); Spacer(); Button("选择字体…") { model.selectFont() } }
                HStack { Text(model.fontToolPath.isEmpty ? "缺少 p4desk-fontpack" : "字体生成工具已配置"); Spacer(); Button("选择工具…") { model.selectFontTool() } }
                Text("只打包本次便签与按钮所需字符。同步按完整一代提交，失败保留设备上一有效版本。").font(.caption).foregroundStyle(.secondary)
            }
            Section("连接") {
                LabeledContent("USB", value: model.connectionStatus)
                LabeledContent("TF 卡", value: model.sdReady ? "就绪" : "未就绪")
                Button("重新连接并读取设备状态") { model.reconnect() }
            }
        }.formStyle(.grouped).padding().onAppear { model.refreshPermissions() }
    }
}

struct ShortcutRecorder: NSViewRepresentable {
    var action: DeskAction
    var recorded: (UInt16, UInt32) -> Void
    func makeNSView(context: Context) -> RecorderView { let view = RecorderView(); view.recorded = recorded; return view }
    func updateNSView(_ view: RecorderView, context: Context) { view.action = action; view.recorded = recorded; view.needsDisplay = true }
}
final class RecorderView: NSView {
    var action = DeskAction.shortcut(8, flags: 1 << 20)
    var recorded: ((UInt16, UInt32) -> Void)?
    override var acceptsFirstResponder: Bool { true }
    override func mouseDown(with event: NSEvent) { window?.makeFirstResponder(self); needsDisplay = true }
    override func becomeFirstResponder() -> Bool { needsDisplay = true; return true }
    override func resignFirstResponder() -> Bool { needsDisplay = true; return true }
    override func keyDown(with event: NSEvent) {
        guard !event.isARepeat else { return }
        let mask: NSEvent.ModifierFlags = [.command, .control, .option, .shift, .function]
        recorded?(event.keyCode, UInt32(truncatingIfNeeded: event.modifierFlags.intersection(mask).rawValue))
        window?.makeFirstResponder(nil); needsDisplay = true
    }
    override func draw(_ dirtyRect: NSRect) {
        let focused = window?.firstResponder === self
        let path = NSBezierPath(roundedRect: bounds.insetBy(dx: 1, dy: 1), xRadius: 6, yRadius: 6)
        NSColor.controlBackgroundColor.setFill(); path.fill()
        (focused ? NSColor.controlAccentColor : NSColor.separatorColor).setStroke(); path.lineWidth = focused ? 2 : 1; path.stroke()
        let title = focused ? "请按下组合键…" : summary(action)
        let attributes: [NSAttributedString.Key: Any] = [.font: NSFont.systemFont(ofSize: 14), .foregroundColor: NSColor.labelColor]
        let size = (title as NSString).size(withAttributes: attributes)
        (title as NSString).draw(at: NSPoint(x: (bounds.width - size.width) / 2, y: (bounds.height - size.height) / 2), withAttributes: attributes)
    }
    private func summary(_ action: DeskAction) -> String {
        let flags = action.modifiers ?? 0
        let modifiers = [(18, "⌃"), (19, "⌥"), (17, "⇧"), (20, "⌘"), (23, "fn ")].filter { flags & (1 << $0.0) != 0 }.map(\.1).joined()
        let names: [UInt16: String] = [0:"A",1:"S",2:"D",3:"F",4:"H",5:"G",6:"Z",7:"X",8:"C",9:"V",11:"B",12:"Q",13:"W",14:"E",15:"R",16:"Y",17:"T",18:"1",19:"2",20:"3",21:"4",22:"6",23:"5",24:"=",25:"9",26:"7",27:"−",28:"8",29:"0",31:"O",32:"U",34:"I",35:"P",37:"L",38:"J",40:"K",45:"N",46:"M",36:"↩",48:"Tab",49:"空格",51:"⌫",53:"Esc",123:"←",124:"→",125:"↓",126:"↑"]
        let key = action.keyCode ?? 8
        return modifiers + (names[key] ?? "键 \(key)")
    }
}
