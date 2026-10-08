import Foundation
import CryptoKit
import Darwin

public enum FileTransferError: Error, LocalizedError, Equatable {
    case unsupported, busy, storageUnavailable, invalidPath, readOnly, notRegularFile, tooLarge,
         sourceChanged, localRead, invalidListing, rejected(String)
    public var errorDescription: String? {
        switch self {
        case .unsupported: return "设备固件不支持文件传输，请升级固件。"
        case .busy: return "请先结束副屏或等待配置同步完成。"
        case .storageUnavailable: return "TF 卡未就绪，无法传输文件。"
        case .invalidPath: return "文件名或目标路径不符合 TF 卡规则。"
        case .readOnly: return "设备系统目录只读，请选择其他文件夹。"
        case .notRegularFile: return "请直接选择普通文件；文件夹和符号链接暂不支持上传。"
        case .tooLarge: return "单个文件最大支持 256 MiB。"
        case .sourceChanged: return "传输期间源文件发生变化，请重新选择并上传。"
        case .localRead: return "无法读取所选文件，请检查文件权限。"
        case .invalidListing: return "设备返回的目录数据无效，请重新连接。"
        case .rejected(let code):
            switch code {
            case "exists", "already_exists": return "目标已有同名文件，本次未覆盖。请先重命名或选择其他目录。"
            case "no_space": return "TF 卡空间不足，文件未提交。"
            case "not_found": return "目标文件夹不存在，请刷新目录。"
            case "read_only", "protected": return "目标只读，请选择其他文件夹。"
            case "busy", "files_busy", "sync_busy": return "设备正在处理其他存储操作，请稍后再试。"
            case "display_busy": return "请先将设备切回 Pad，再传输文件。"
            case "files_unavailable": return "设备文件管理服务未启动，请重新启动设备。"
            case "too_many_entries": return "文件夹内容过多，请选择其他文件夹或整理后重试。"
            case "invalid_name": return "文件名不符合 TF 卡规则，请重命名后上传。"
            case "offset", "resource_length": return "文件传输顺序或分块无效，请重新上传。"
            case "cancelled": return "设备已取消本次文件传输。"
            case "permission_denied", "io": return "TF 卡读写失败，请检查存储状态。"
            case "hash_mismatch", "integrity": return "文件校验失败，未提交到 TF 卡。"
            case "too_large": return FileTransferError.tooLarge.errorDescription
            case "invalid_path": return FileTransferError.invalidPath.errorDescription
            case "sd_unavailable", "storage_unavailable": return FileTransferError.storageUnavailable.errorDescription
            default: return "设备未完成文件操作，请检查 TF 卡与连接。"
            }
        }
    }
}

public extension FileTransferError {
    var diagnosticCode: String {
        switch self {
        case .unsupported: return "unsupported"
        case .busy: return "host_busy"
        case .storageUnavailable: return "storage_unavailable"
        case .invalidPath: return "invalid_path"
        case .readOnly: return "protected"
        case .notRegularFile: return "not_regular_file"
        case .tooLarge: return "too_large"
        case .sourceChanged: return "source_changed"
        case .localRead: return "local_read"
        case .invalidListing: return "invalid_listing"
        case .rejected(let code): return FileAcknowledgement.safeErrorCode(["error": code])
        }
    }
}

public enum FileAcknowledgement {
    // Rust v1 sends error. Support the earlier code spelling while keeping all
    // untrusted messages out of the UI and diagnostic logs.
    public static func safeErrorCode(_ fields: [String: Any]) -> String {
        let value = fields["error"] as? String ?? fields["code"] as? String ?? "failed"
        let allowed: Set<String> = ["exists", "already_exists", "no_space", "not_found", "read_only", "protected",
            "busy", "files_busy", "sync_busy", "display_busy", "files_unavailable", "hash_mismatch", "integrity",
            "offset", "resource_length", "cancelled", "too_large", "too_many_entries", "invalid_path", "invalid_name",
            "sd_unavailable", "storage_unavailable", "invalid_font", "invalid_text", "invalid_image", "not_directory",
            "unsupported", "permission_denied", "io", "hello_required"]
        return allowed.contains(value) ? value : "failed"
    }
}

public enum RemoteFilePath {
    public static let maximumUTF8Bytes = 512
    public static func validate(_ path: String, allowRoot: Bool = true) throws {
        if path.isEmpty, allowRoot { return }
        guard !path.isEmpty, path.utf8.count <= maximumUTF8Bytes else { throw FileTransferError.invalidPath }
        let parts = path.split(separator: "/", omittingEmptySubsequences: false)
        for part in parts {
            guard !part.isEmpty, part != ".", part != "..", part.utf8.count <= 255,
                  !part.hasSuffix("."), !part.hasSuffix(" "),
                  !part.unicodeScalars.contains(where: { $0.value < 32 || $0.value == 127 || "\\:*?\"<>|".unicodeScalars.contains($0) })
            else { throw FileTransferError.invalidPath }
        }
    }
    public static func isSystem(_ path: String) -> Bool {
        path.split(separator: "/").first?.lowercased() == "p4desk"
    }
    public static func joined(_ directory: String, name: String) throws -> String {
        // A single selected file always stays in the selected directory.
        guard !name.contains("/") else { throw FileTransferError.invalidPath }
        let result = directory.isEmpty ? name : directory + "/" + name
        try validate(result, allowRoot: false)
        guard !isSystem(result) else { throw FileTransferError.readOnly }
        return result
    }
    public static func parent(_ path: String) -> String {
        path.split(separator: "/").dropLast().joined(separator: "/")
    }
}

public struct RemoteFileEntry: Codable, Identifiable, Equatable, Sendable {
    public var name: String
    public var path: String
    public var directory: Bool
    public var size: UInt64
    public var modifiedSeconds: Int64?
    public var readOnly: Bool
    public var id: String { path }
    enum CodingKeys: String, CodingKey {
        case name, path, directory, size
        case modifiedSeconds = "modified_seconds", readOnly = "read_only"
    }
    public init(name: String, path: String, directory: Bool, size: UInt64, modifiedSeconds: Int64? = nil, readOnly: Bool = false) {
        self.name = name; self.path = path; self.directory = directory; self.size = size
        self.modifiedSeconds = modifiedSeconds; self.readOnly = readOnly
    }
}

public struct RemoteFileListing: Codable, Equatable, Sendable {
    public var path: String
    public var entries: [RemoteFileEntry]
    public var total: UInt32
    public var truncated: Bool
    public var readOnly: Bool
    enum CodingKeys: String, CodingKey { case path, entries, total, truncated; case readOnly = "read_only" }
    public static func decode(_ object: [String: Any], expectedPath: String) throws -> Self {
        guard JSONSerialization.isValidJSONObject(object) else { throw FileTransferError.invalidListing }
        let listing: Self
        do { listing = try JSONDecoder().decode(Self.self, from: JSONSerialization.data(withJSONObject: object)) }
        catch { throw FileTransferError.invalidListing }
        try RemoteFilePath.validate(listing.path)
        guard listing.path == expectedPath, listing.entries.count <= 32,
              listing.entries.count <= Int(listing.total), Set(listing.entries.map(\.path)).count == listing.entries.count else {
            throw FileTransferError.invalidListing
        }
        for entry in listing.entries {
            try RemoteFilePath.validate(entry.path, allowRoot: false)
            guard !entry.name.contains("/"), entry.path == (listing.path.isEmpty ? entry.name : listing.path + "/" + entry.name),
                  !entry.name.isEmpty else { throw FileTransferError.invalidListing }
        }
        return listing
    }
}

public struct FileUploadManifest: Equatable, Sendable {
    public var length: UInt64
    public var sha256: String
}
public enum FileUploadCommand: Sendable {
    case begin(path: String, manifest: FileUploadManifest)
    case chunk(offset: UInt32, data: Data)
    case commit
    case abort
}
public struct FileUploadProgress: Sendable {
    public enum Phase: Sendable { case hashing, sending, verifying }
    public var phase: Phase
    public var completed: UInt64
    public var total: UInt64
}
public enum FileResourceChunk {
    public static let maximumDataBytes = 32_768
    public static func payload(offset: UInt32, data: Data) throws -> Data {
        guard !data.isEmpty, data.count <= maximumDataBytes else { throw ProtocolError.invalidLength }
        var result = Data((0..<4).map { UInt8(truncatingIfNeeded: offset >> ($0 * 8)) })
        result.append(data)
        return result
    }
}

/// Owns a single descriptor and reads bounded blocks off the UI actor. Identity,
/// size and timestamps are checked through the descriptor, never a second path.
private actor UploadSource {
    let handle: FileHandle
    let initial: stat
    let length: UInt64
    var hasher = SHA256()
    init(url: URL) throws {
        guard url.isFileURL else { throw FileTransferError.notRegularFile }
        let descriptor = url.withUnsafeFileSystemRepresentation { value in
            value.map { Darwin.open($0, O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_NONBLOCK) } ?? -1
        }
        guard descriptor >= 0 else { throw errno == ELOOP ? FileTransferError.notRegularFile : FileTransferError.localRead }
        var value = stat()
        guard fstat(descriptor, &value) == 0, (value.st_mode & S_IFMT) == S_IFREG, value.st_size >= 0 else {
            Darwin.close(descriptor); throw FileTransferError.notRegularFile
        }
        guard UInt64(value.st_size) <= FileTransferEngine.maximumFileBytes else {
            Darwin.close(descriptor); throw FileTransferError.tooLarge
        }
        handle = FileHandle(fileDescriptor: descriptor, closeOnDealloc: true)
        initial = value; length = UInt64(value.st_size)
    }
    func check() throws {
        var current = stat()
        guard fstat(handle.fileDescriptor, &current) == 0,
              current.st_dev == initial.st_dev, current.st_ino == initial.st_ino, current.st_size == initial.st_size,
              current.st_mtimespec.tv_sec == initial.st_mtimespec.tv_sec, current.st_mtimespec.tv_nsec == initial.st_mtimespec.tv_nsec,
              current.st_ctimespec.tv_sec == initial.st_ctimespec.tv_sec, current.st_ctimespec.tv_nsec == initial.st_ctimespec.tv_nsec
        else { throw FileTransferError.sourceChanged }
    }
    func prepare(progress: @Sendable (FileUploadProgress) async -> Void) async throws -> FileUploadManifest {
        var hash = SHA256(), read: UInt64 = 0
        do {
            while read < length {
                try Task.checkCancellation()
                guard let data = try handle.read(upToCount: FileResourceChunk.maximumDataBytes), !data.isEmpty else { throw FileTransferError.sourceChanged }
                hash.update(data: data); read += UInt64(data.count)
                await progress(FileUploadProgress(phase: .hashing, completed: read, total: length))
            }
            try check()
            try handle.seek(toOffset: 0)
            return FileUploadManifest(length: length, sha256: hash.finalize().map { String(format: "%02x", $0) }.joined())
        } catch let error as FileTransferError { throw error }
        catch is CancellationError { throw CancellationError() }
        catch { throw FileTransferError.localRead }
    }
    func read() throws -> Data {
        try Task.checkCancellation(); try check()
        do {
            let data = try handle.read(upToCount: FileResourceChunk.maximumDataBytes) ?? Data()
            hasher.update(data: data)
            return data
        } catch { throw FileTransferError.localRead }
    }
    func verify(_ manifest: FileUploadManifest) throws {
        try check()
        guard hasher.finalize().map({ String(format: "%02x", $0) }).joined() == manifest.sha256 else { throw FileTransferError.sourceChanged }
    }
    func close() { try? handle.close() }
}

public enum FileTransferEngine {
    public static let maximumFileBytes: UInt64 = 256 * 1_024 * 1_024
    /// One chunk in flight; the caller returns from send only after its ACK.
    /// Commit makes the validated temporary file visible on the device.
    public static func upload(url: URL, destination: String,
        send: @Sendable (FileUploadCommand) async throws -> Void,
        progress: @escaping @Sendable (FileUploadProgress) async -> Void = { _ in }) async throws -> FileUploadManifest {
        try RemoteFilePath.validate(destination, allowRoot: false)
        guard !RemoteFilePath.isSystem(destination) else { throw FileTransferError.readOnly }
        let source = try await Task.detached(priority: .utility) { try UploadSource(url: url) }.value
        var begun = false
        do {
            let manifest = try await source.prepare(progress: progress)
            try Task.checkCancellation()
            // Abort even if the begin reply is lost; the device may have opened
            // a temporary file before the host timeout/cancellation.
            begun = true
            try await send(.begin(path: destination, manifest: manifest))
            var offset: UInt64 = 0
            while offset < manifest.length {
                try Task.checkCancellation()
                let data = try await source.read()
                guard !data.isEmpty, offset + UInt64(data.count) <= manifest.length else { throw FileTransferError.sourceChanged }
                try await send(.chunk(offset: UInt32(offset), data: data))
                offset += UInt64(data.count)
                await progress(FileUploadProgress(phase: .sending, completed: offset, total: manifest.length))
            }
            try Task.checkCancellation()
            try await source.verify(manifest)
            await progress(FileUploadProgress(phase: .verifying, completed: manifest.length, total: manifest.length))
            try await send(.commit)
            await source.close()
            return manifest
        } catch {
            if begun { try? await send(.abort) }
            await source.close()
            throw error
        }
    }
}
