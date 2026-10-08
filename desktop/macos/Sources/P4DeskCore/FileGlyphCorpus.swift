import Foundation
import Darwin

/// A bounded set of glyphs. Stores characters, not file bodies or file history.
public struct FileGlyphCorpus: Sendable, Equatable {
    public static let maximumScalars = 4_000
    public static let previewBytes = 32_768
    public var text: String
    public var truncated: Bool
    public static func collect(urls: [URL], directoryNames: [String], previous: String) -> Self {
        var scalars = Set<Unicode.Scalar>()
        var truncated = false
        func add(_ text: String) {
            for scalar in text.unicodeScalars where scalar.value >= 32 && scalar.value != 127 && scalar.value != 0xfeff {
                if scalars.contains(scalar) { continue }
                if scalars.count >= maximumScalars { truncated = true; continue }
                scalars.insert(scalar)
            }
        }
        // Names are the highest priority if a pathological text exceeds budget.
        for url in urls { add(url.lastPathComponent) }
        for name in directoryNames { add(name) }
        add(previous)
        for url in urls {
            guard let prefix = regularFilePrefix(url), let text = decodeUTF8Prefix(prefix) else { continue }
            add(text)
        }
        return Self(text: String(String.UnicodeScalarView(scalars.sorted { $0.value < $1.value })), truncated: truncated)
    }
    public static func decodeUTF8Prefix(_ data: Data) -> String? {
        guard !data.contains(0) else { return nil }
        for removed in 0...min(3, data.count) {
            if let value = String(data: data.prefix(data.count - removed), encoding: .utf8) { return value }
        }
        return nil
    }
    private static func regularFilePrefix(_ url: URL) -> Data? {
        guard url.isFileURL else { return nil }
        let descriptor = url.withUnsafeFileSystemRepresentation { value in
            value.map { Darwin.open($0, O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC) } ?? -1
        }
        guard descriptor >= 0 else { return nil }
        let handle = FileHandle(fileDescriptor: descriptor, closeOnDealloc: true)
        defer { try? handle.close() }
        var value = stat()
        guard fstat(descriptor, &value) == 0, (value.st_mode & S_IFMT) == S_IFREG else { return nil }
        return try? handle.read(upToCount: previewBytes)
    }
}
