import SwiftUI
import AppKit
import P4DeskCore

struct FileTransferResult: Identifiable {
    let id = UUID()
    let name: String
    let success: Bool
    let message: String
}

@MainActor
extension DeskModel {
    var fileActionsAvailable: Bool {
        connected && sdReady && fileTransferSupported && !syncing && !displayActive && !changingMode && !fileBusy
    }
    private func claimFiles() throws -> (UUID, UInt64) {
        guard connected else { throw DeskError.usbDisconnected }
        guard fileTransferSupported else { throw FileTransferError.unsupported }
        guard sdReady else { throw FileTransferError.storageUnavailable }
        guard !fileBusy, !syncing, !displayActive, !changingMode else { throw FileTransferError.busy }
        let identity = UUID(); fileOperationIdentity = identity
        fileBusy = true; fileProgress = 0; fileTransferredBytes = 0; fileTotalBytes = 0
        return (identity, fileConnectionEpoch)
    }
    private func checkFiles(_ identity: UUID, epoch: UInt64, cancellation: Bool = true) throws {
        if cancellation { try Task.checkCancellation() }
        guard connected, fileConnectionEpoch == epoch, fileOperationIdentity == identity else { throw DeskError.usbDisconnected }
    }
    private func releaseFiles(_ identity: UUID) {
        guard fileOperationIdentity == identity else { return }
        fileOperationIdentity = nil; fileOperationTask = nil; fileBusy = false; fileUploading = false
    }
    func invalidateFileOperation(message: String) {
        let interruptedUpload = fileUploading
        fileOperationTask?.cancel(); fileOperationTask = nil; fileOperationIdentity = nil
        fileBusy = false; fileUploading = false; fileProgress = 0
        fileTransferredBytes = 0; fileTotalBytes = 0; fileEntries = []; fileTotal = 0; fileHasMore = false; fileNextOffset = 0
        fileMessage = message
        if interruptedUpload { setError(title: "文件传输中断", message: "本次传输已中断，已完成的文件会保留。请重连检查目标文件夹后重试。") }
    }
    func cancelFileOperation() {
        guard fileBusy else { return }
        fileMessage = "正在取消并清理临时文件…"
        fileOperationTask?.cancel()
    }
    private func startFileTask(notifyOnError: Bool = true, _ body: @escaping @MainActor (UUID, UInt64) async throws -> Void) {
        do {
            let (identity, epoch) = try claimFiles()
            fileOperationTask = Task { [weak self] in
                guard let self else { return }
                defer { self.releaseFiles(identity) }
                do { try await body(identity, epoch) }
                catch {
                    guard self.fileOperationIdentity == identity else { return }
                    self.resetFileStatus()
                    if notifyOnError {
                        if error is CancellationError { self.setNotice(title: "文件操作已取消", message: "未提交的临时文件会被清理；已完成的文件会保留。") }
                        else { self.setError(title: "文件操作失败", message: self.fileErrorMessage(error)) }
                    }
                }
            }
        } catch {
            resetFileStatus()
            if notifyOnError, !(error is CancellationError) { setError(title: "文件操作失败", message: fileErrorMessage(error)) }
        }
    }
    private func resetFileStatus() {
        if !connected { fileMessage = "连接 USB 后可浏览 TF 卡与传输文件" }
        else if !fileTransferSupported { fileMessage = "设备固件不支持文件传输，请升级固件" }
        else if !sdReady { fileMessage = "TF 卡未就绪，请在设备上检查存储状态" }
        else { fileMessage = fileReadOnly ? "系统目录只读" : "当前目录共 \(fileTotal) 项" }
    }
    func fileErrorMessage(_ error: Error) -> String {
        if error is CancellationError { return "已取消；未提交的临时文件会被清理。" }
        return (error as? FileTransferError)?.errorDescription ?? (error as? DeskError)?.errorDescription ?? "文件操作未完成，请检查 TF 卡与 USB 连接。"
    }
    private func readFilePage(_ path: String, offset: UInt32, identity: UUID, epoch: UInt64) async throws -> RemoteFileListing {
        try RemoteFilePath.validate(path)
        try checkFiles(identity, epoch: epoch)
        let reply = try await fileControl("file_list", ["path": path, "offset": offset, "limit": UInt16(32)], expected: "file_listing")
        try checkFiles(identity, epoch: epoch)
        return try RemoteFileListing.decode(reply, expectedPath: path)
    }
    private func applyFileListing(_ listing: RemoteFileListing, append: Bool) {
        fileDirectory = listing.path; fileReadOnly = listing.readOnly || RemoteFilePath.isSystem(listing.path)
        if append {
            let seen = Set(fileEntries.map(\.path))
            fileEntries.append(contentsOf: listing.entries.filter { !seen.contains($0.path) })
        } else { fileEntries = listing.entries }
        fileNextOffset = append ? fileNextOffset + UInt32(listing.entries.count) : UInt32(listing.entries.count)
        fileTotal = listing.total
        fileHasMore = listing.truncated && fileNextOffset < listing.total && !listing.entries.isEmpty
    }
    func browseFiles(_ path: String? = nil, loadMore: Bool = false, userInitiated: Bool = true) {
        let target = path ?? fileDirectory
        let offset = loadMore ? fileNextOffset : 0
        startFileTask(notifyOnError: userInitiated) { [weak self] identity, epoch in
            guard let self else { return }
            self.fileMessage = "正在读取 TF 卡目录…"
            let listing = try await self.readFilePage(target, offset: offset, identity: identity, epoch: epoch)
            self.applyFileListing(listing, append: loadMore)
            self.resetFileStatus()
        }
    }
    func createFileFolder(_ name: String) {
        let directory = fileDirectory
        startFileTask { [weak self] identity, epoch in
            guard let self else { return }
            let path = try RemoteFilePath.joined(directory, name: name)
            try self.checkFiles(identity, epoch: epoch)
            _ = try await self.fileControl("file_mkdir", ["path": path])
            // The mutation already committed. A later listing failure must not
            // imply creation failed or invite retrying the same name.
            self.resetFileStatus()
            self.setNotice(title: "文件夹已创建", message: "新文件夹已保存到 TF 卡。")
            if let listing = try? await self.readFilePage(directory, offset: 0, identity: identity, epoch: epoch) {
                self.applyFileListing(listing, append: false); self.resetFileStatus()
            }
        }
    }
    private func sendFileCommand(_ command: FileUploadCommand, identity: UUID, epoch: UInt64) async throws {
        // Abort is a separate non-cancelled cleanup task so cancellation does
        // not skip the device cleanup. Never send it after an epoch change.
        if case .abort = command {
            try checkFiles(identity, epoch: epoch, cancellation: false)
            let cleanup = Task { @MainActor [weak self] in
                guard let self else { return }
                try self.checkFiles(identity, epoch: epoch, cancellation: false)
                _ = try await self.fileControl("file_upload_abort", [:], timeout: 3)
            }
            try await cleanup.value
            return
        }
        try checkFiles(identity, epoch: epoch)
        switch command {
        case .begin(let path, let manifest):
            fileTotalBytes = manifest.length
            _ = try await fileControl("file_upload_begin", ["path": path, "length": manifest.length, "sha256": manifest.sha256], timeout: 12)
        case .chunk(let offset, let data): try await fileChunk(offset: offset, data: data)
        case .commit:
            let timeout = max(15, 30 + ceil(Double(fileTotalBytes) / Double(512 * 1_024)))
            _ = try await fileControl("file_upload_commit", [:], timeout: timeout)
        case .abort: break
        }
        try checkFiles(identity, epoch: epoch)
    }
    private func uploadOne(_ url: URL, destination: String, identity: UUID, epoch: UInt64,
                           index: Int = 0, count: Int = 1) async throws -> FileUploadManifest {
        try checkFiles(identity, epoch: epoch)
        return try await FileTransferEngine.upload(url: url, destination: destination, send: { [weak self] command in
            guard let self else { throw DeskError.usbDisconnected }
            try await self.sendFileCommand(command, identity: identity, epoch: epoch)
        }, progress: { [weak self] progress in
            guard let self else { return }
            await self.updateFileProgress(progress, identity: identity, index: index, count: count)
        })
    }
    private func updateFileProgress(_ progress: FileUploadProgress, identity: UUID, index: Int, count: Int) {
        guard fileOperationIdentity == identity else { return }
        fileTotalBytes = progress.total
        switch progress.phase {
        case .hashing:
            fileTransferredBytes = 0
            fileMessage = "正在校验第 \(index + 1)/\(count) 个文件…"
        case .sending:
            fileTransferredBytes = progress.completed
            let fraction = progress.total == 0 ? 1 : Double(progress.completed) / Double(progress.total)
            fileProgress = (Double(index) + fraction) / Double(count)
            fileMessage = "正在上传第 \(index + 1)/\(count) 个文件：\(Int(fraction * 100))%"
        case .verifying:
            fileMessage = "设备正在校验并提交第 \(index + 1)/\(count) 个文件…"
        }
    }
    private func installFilesFont(for urls: [URL], directory: String, identity: UUID, epoch: UInt64) async throws {
        fileMessage = "正在准备文件名字形和文本预览…"
        var names: [String] = []
        var offset: UInt32 = 0
        for _ in 0..<64 {
            let page = try await readFilePage(directory, offset: offset, identity: identity, epoch: epoch)
            names.append(contentsOf: page.entries.map(\.name))
            offset += UInt32(page.entries.count)
            if !page.truncated || page.entries.isEmpty || offset >= page.total { break }
        }
        let tool = URL(fileURLWithPath: fontToolPath), font = URL(fileURLWithPath: fontPath)
        let directoryNames = names
        let bake = Task.detached(priority: .utility) { try FilesFontPackage.prepare(urls: urls, directoryNames: directoryNames, tool: tool, font: font) }
        let package = try await withTaskCancellationHandler { try await bake.value } onCancel: { bake.cancel() }
        defer { package.cleanUp() }
        try checkFiles(identity, epoch: epoch)
        fileFontMissingCount = package.unsupportedCount
        fileGlyphWarning = package.unsupportedCount > 0 ? "部分字符（例如 Emoji）不在当前字体中，将以占位字形显示；文件内容会完整保存。" : ""
        do { _ = try await readFilePage("Fonts", offset: 0, identity: identity, epoch: epoch) }
        catch let error as FileTransferError {
            guard case .rejected("not_found") = error else { throw error }
            _ = try await fileControl("file_mkdir", ["path": "Fonts"])
        }
        let destination = "Fonts/P4Desk-" + package.sha256 + ".p4f"
        var found = false
        offset = 0
        for _ in 0..<64 {
            let page = try await readFilePage("Fonts", offset: offset, identity: identity, epoch: epoch)
            if page.entries.contains(where: { !$0.directory && $0.path == destination }) { found = true; break }
            offset += UInt32(page.entries.count)
            if !page.truncated || page.entries.isEmpty || offset >= page.total { break }
        }
        if !found { _ = try await uploadOne(package.url, destination: destination, identity: identity, epoch: epoch) }
        try checkFiles(identity, epoch: epoch)
        fileMessage = "正在启用独立文件预览字库…"
        // Persist the complete union first. An unsuccessful local write must
        // not install a pack whose earlier text cannot be retained next time.
        try package.saveCorpus()
        _ = try await fileControl("file_font_install", ["path": destination, "sha256": package.sha256], timeout: 15)
        try checkFiles(identity, epoch: epoch)
        fileFontInstalled = true
        fileProgress = 0; fileTransferredBytes = 0; fileTotalBytes = 0
    }
    private func prepareFilesFont(for urls: [URL], directory: String, identity: UUID, epoch: UInt64) async throws {
        fileGlyphWarning = ""; fileFontInstalled = false; fileFontMissingCount = 0; fileFontSkipped = false
        do { try await installFilesFont(for: urls, directory: directory, identity: identity, epoch: epoch) }
        catch {
            try checkFiles(identity, epoch: epoch)
            fileFontSkipped = true
            if case FilesFontError.corpusLimit = error {
                fileGlyphWarning = "文件预览所需字形超过 4,000 个，已保留原有字库；新增字符可能显示为占位字形，文件仍会完整上传。"
            } else if case FilesFontError.oversized = error {
                fileGlyphWarning = "文件预览字库超过 8 MiB，已保留原有字库；新增字符可能显示为占位字形，文件仍会完整上传。"
            } else if case FilesFontError.corpusPersistence = error {
                fileGlyphWarning = "Mac 字形记录无法读取或保存，已保留原有字库。请检查本机磁盘和权限；文件仍会完整上传。"
            } else {
                fileGlyphWarning = "文件预览字库未更新，部分文字可能显示为占位字形。请检查字体工具、TF 空间和固件；文件仍会完整上传。"
            }
        }
    }
    func chooseUploadFiles() {
        guard fileActionsAvailable, !fileReadOnly else { return }
        let panel = NSOpenPanel()
        panel.canChooseFiles = true; panel.canChooseDirectories = false
        panel.allowsMultipleSelection = true; panel.resolvesAliases = false
        panel.message = "选择要传到 TF 卡的文件；同名文件不会被覆盖，单个文件最大 256 MiB。"
        let selected: (NSApplication.ModalResponse) -> Void = { [weak self] result in
            guard result == .OK else { return }
            self?.uploadFiles(panel.urls)
        }
        if let window = NSApp.keyWindow { panel.beginSheetModal(for: window, completionHandler: selected) }
        else { panel.begin(completionHandler: selected) }
    }
    func uploadFiles(_ urls: [URL]) {
        guard !urls.isEmpty else { return }
        let directory = fileDirectory
        startFileTask { [weak self] identity, epoch in
            guard let self else { return }
            self.fileUploading = true; self.fileResults = []
            try await self.prepareFilesFont(for: urls, directory: directory, identity: identity, epoch: epoch)
            for (index, url) in urls.enumerated() {
                do {
                    let destination = try RemoteFilePath.joined(directory, name: url.lastPathComponent)
                    _ = try await self.uploadOne(url, destination: destination, identity: identity, epoch: epoch, index: index, count: urls.count)
                    self.fileResults.append(FileTransferResult(name: url.lastPathComponent, success: true, message: "已校验并保存到 TF 卡"))
                } catch {
                    guard self.fileOperationIdentity == identity else { return }
                    self.fileResults.append(FileTransferResult(name: url.lastPathComponent, success: false, message: self.fileErrorMessage(error)))
                    if error is CancellationError || !self.connected { throw error }
                }
            }
            self.fileProgress = 1
            let successes = self.fileResults.filter(\.success).count
            let failures = self.fileResults.count - successes
            self.resetFileStatus()
            let warning = self.fileGlyphWarning.isEmpty ? "" : "\n\n" + self.fileGlyphWarning
            if failures > 0 {
                let reason = self.fileResults.first(where: { !$0.success })?.message ?? "请检查连接与 TF 卡。"
                self.setError(title: successes == 0 ? "文件传输失败" : "部分文件未完成", message: "\(successes) 个文件已完成，\(failures) 个文件未完成。\n\n" + reason + "\n可在文件页查看每个文件的结果。" + warning)
            } else { self.setNotice(title: "文件传输完成", message: "\(successes) 个文件已校验并保存到 TF 卡。" + warning) }
            if let listing = try? await self.readFilePage(directory, offset: 0, identity: identity, epoch: epoch) {
                self.applyFileListing(listing, append: false)
            }
            self.resetFileStatus()
        }
    }
    /// Explicit CLI only: no automatic selection or scan of user directories.
    func uploadFileForDiagnostics(_ url: URL, destination: String) async throws -> FileUploadManifest {
        let (identity, epoch) = try claimFiles()
        fileUploading = true
        defer { releaseFiles(identity) }
        try RemoteFilePath.validate(destination, allowRoot: false)
        guard !RemoteFilePath.isSystem(destination) else { throw FileTransferError.readOnly }
        try await prepareFilesFont(for: [url], directory: RemoteFilePath.parent(destination), identity: identity, epoch: epoch)
        return try await uploadOne(url, destination: destination, identity: identity, epoch: epoch)
    }
}

struct FilesEditor: View {
    @ObservedObject var model: DeskModel
    @State private var showNewFolder = false
    @State private var folderName = ""
    private var status: String? {
        if !model.connected { return "请连接板上 Type-A USB-OTG 数据接口" }
        if !model.fileTransferSupported { return "设备固件不支持文件传输，请升级固件" }
        if !model.sdReady { return "TF 卡未就绪，请在设备上检查存储状态" }
        if model.displayActive || model.changingMode { return "切回 Pad 后可浏览并传输文件" }
        if model.syncing { return "配置同步完成后可传输文件" }
        return nil
    }
    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Button { model.browseFiles("") } label: { Image(systemName: "house") }.help("TF 卡根目录")
                Button { model.browseFiles(RemoteFilePath.parent(model.fileDirectory)) } label: { Image(systemName: "chevron.up") }
                    .disabled(model.fileDirectory.isEmpty)
                Text(model.fileDirectory.isEmpty ? "TF 卡" : "TF 卡 / " + model.fileDirectory)
                    .lineLimit(1).truncationMode(.middle).font(.headline)
                Spacer()
                Button { model.browseFiles() } label: { Image(systemName: "arrow.clockwise") }
                Button("新建文件夹") { folderName = ""; showNewFolder = true }.disabled(model.fileReadOnly)
                Button("传文件到 TF 卡…") { model.chooseUploadFiles() }.buttonStyle(.borderedProminent).disabled(model.fileReadOnly)
            }.disabled(!model.fileActionsAvailable).padding()
            Divider()
            if let status {
                ContentUnavailableView("文件传输", systemImage: "folder", description: Text(status)).frame(maxHeight: .infinity)
            } else {
                List {
                    ForEach(model.fileEntries) { entry in
                        HStack(spacing: 12) {
                            Image(systemName: entry.directory ? "folder.fill" : "doc")
                                .foregroundStyle(entry.directory ? Color.accentColor : Color.secondary)
                                .frame(width: 24)
                            VStack(alignment: .leading, spacing: 3) {
                                Text(entry.name).lineLimit(1)
                                if !entry.directory {
                                    Text(ByteCountFormatter.string(fromByteCount: Int64(clamping: entry.size), countStyle: .file))
                                        .font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            Spacer()
                            if entry.readOnly { Image(systemName: "lock").foregroundStyle(.secondary).help("只读") }
                            if entry.directory {
                                Button("打开") { model.browseFiles(entry.path) }.disabled(!model.fileActionsAvailable)
                            }
                        }.padding(.vertical, 3)
                    }
                    if model.fileHasMore {
                        Button("显示更多（\(model.fileEntries.count) / \(model.fileTotal)）") { model.browseFiles(loadMore: true) }
                            .disabled(!model.fileActionsAvailable)
                    } else if model.fileEntries.isEmpty && !model.fileBusy {
                        Text("此文件夹暂无文件").foregroundStyle(.secondary)
                    }
                }
                if !model.fileResults.isEmpty {
                    Divider()
                    HStack {
                        Text("传输结果").font(.caption).foregroundStyle(.secondary)
                        Spacer()
                        Button("关闭") { model.fileResults = [] }
                    }.padding(.horizontal, 12).padding(.top, 8)
                    ScrollView {
                        VStack(alignment: .leading, spacing: 8) {
                            ForEach(model.fileResults) { result in
                                HStack(alignment: .top) {
                                    Image(systemName: result.success ? "checkmark.circle.fill" : "exclamationmark.circle")
                                        .foregroundStyle(result.success ? Color.green : Color.orange)
                                    Text(result.name).lineLimit(1).frame(width: 160, alignment: .leading)
                                    Text(result.message).foregroundStyle(.secondary).frame(maxWidth: .infinity, alignment: .leading)
                                }.font(.caption)
                            }
                        }.padding(12)
                    }.frame(maxHeight: 110)
                }
            }
            Divider()
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text(model.fileMessage).font(.caption).foregroundStyle(.secondary).frame(maxWidth: .infinity, alignment: .leading)
                    if model.fileBusy { Button("取消") { model.cancelFileOperation() } }
                }
                if model.fileUploading {
                    ProgressView(value: model.fileProgress)
                    Text("\(ByteCountFormatter.string(fromByteCount: Int64(clamping: model.fileTransferredBytes), countStyle: .file)) / \(ByteCountFormatter.string(fromByteCount: Int64(clamping: model.fileTotalBytes), countStyle: .file))")
                        .font(.caption).monospacedDigit().foregroundStyle(.secondary)
                }
                Text("文件校验后保存，同名文件不会覆盖。p4desk 系统目录仅可浏览，单个文件最大 256 MiB。")
                    .font(.caption).foregroundStyle(.secondary)
            }.padding(12)
        }
        .alert("新建文件夹", isPresented: $showNewFolder) {
            TextField("文件夹名称", text: $folderName)
            Button("取消", role: .cancel) {}
            Button("创建") { model.createFileFolder(folderName) }.disabled(folderName.isEmpty)
        }
        .task { if model.fileActionsAvailable { model.browseFiles(userInitiated: false) } }
        .onChange(of: model.connected) { _, connected in if connected && model.fileActionsAvailable { model.browseFiles(userInitiated: false) } }
        .onChange(of: model.sdReady) { _, ready in if ready && model.fileActionsAvailable { model.browseFiles(userInitiated: false) } }
    }
}
