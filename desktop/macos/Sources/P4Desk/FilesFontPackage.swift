import Foundation
import P4DeskCore

enum FilesFontError: Error { case corpusLimit, oversized, corpusPersistence }

struct FilesFontPackage {
    let directory: URL
    let url: URL
    let sha256: String
    let unsupportedCount: Int
    let corpus: String
    private static var corpusURL: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("P4Desk", isDirectory: true).appendingPathComponent("file-glyphs-v1.txt")
    }
    static func prepare(urls: [URL], directoryNames: [String], tool: URL, font: URL) throws -> Self {
        guard FileManager.default.isExecutableFile(atPath: tool.path), FileManager.default.fileExists(atPath: font.path) else { throw DeskError.fontUnavailable }
        var previous = ""
        if FileManager.default.fileExists(atPath: corpusURL.path) {
            do {
                guard let size = try corpusURL.resourceValues(forKeys: [.fileSizeKey]).fileSize, size <= 256 * 1_024 else { throw FilesFontError.corpusPersistence }
                previous = try String(contentsOf: corpusURL, encoding: .utf8)
            } catch { throw FilesFontError.corpusPersistence }
        }
        let corpus = FileGlyphCorpus.collect(urls: urls, directoryNames: directoryNames, previous: previous)
        // Preserve the installed corpus if the union cannot fit; never replace
        // it with a subset that would lose earlier filenames.
        guard !corpus.truncated else { throw FilesFontError.corpusLimit }
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("p4desk-file-font-" + UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        do {
            let text = directory.appendingPathComponent("glyphs.txt")
            let output = directory.appendingPathComponent("files.p4f")
            let report = directory.appendingPathComponent("report.json")
            try corpus.text.write(to: text, atomically: true, encoding: .utf8)
            try FontPackage.run(tool, ["bake", "--font", font.path, "--text-file", text.path, "--output", output.path,
                                       "--sizes", "14,18,22", "--missing-glyphs", "skip", "--report", report.path])
            try FontPackage.run(tool, ["validate", "--input", output.path])
            guard let size = try output.resourceValues(forKeys: [.fileSizeKey]).fileSize, size > 0, size <= 8 * 1_024 * 1_024 else { throw FilesFontError.oversized }
            guard let reportSize = try report.resourceValues(forKeys: [.fileSizeKey]).fileSize, reportSize <= 4_096,
                  let fields = try JSONSerialization.jsonObject(with: Data(contentsOf: report)) as? [String: Any],
                  fields["valid"] as? Bool == true, let unsupported = fields["unsupported_count"] as? Int, unsupported >= 0 else { throw DeskError.invalidFont }
            // Only the generated, capped glyph package is read here; ordinary
            // user files always use the streaming transfer engine.
            let hash = FontPackage.hash(try Data(contentsOf: output, options: [.mappedIfSafe]))
            return Self(directory: directory, url: output, sha256: hash, unsupportedCount: unsupported,
                        corpus: corpus.text)
        } catch { try? FileManager.default.removeItem(at: directory); throw error }
    }
    func saveCorpus() throws {
        do {
            try FileManager.default.createDirectory(at: Self.corpusURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try corpus.write(to: Self.corpusURL, atomically: true, encoding: .utf8)
        } catch { throw FilesFontError.corpusPersistence }
    }
    func cleanUp() { try? FileManager.default.removeItem(at: directory) }
}
