import Foundation
import CryptoKit
import P4DeskCore

enum FontPackage {
    static func bake(snapshot: Snapshot, toolURL: URL, fontURL: URL) throws -> Data {
        guard FileManager.default.isExecutableFile(atPath: toolURL.path), FileManager.default.fileExists(atPath: fontURL.path) else { throw DeskError.fontUnavailable }
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("p4desk-font-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let snapshotURL = directory.appendingPathComponent("snapshot.json")
        let outputURL = directory.appendingPathComponent("ui.font")
        try JSONEncoder().encode(snapshot).write(to: snapshotURL, options: .atomic)
        try run(toolURL, ["bake", "--font", fontURL.path, "--snapshot", snapshotURL.path,
                          "--output", outputURL.path, "--sizes", "18,22,28,36"])
        try run(toolURL, ["validate", "--input", outputURL.path, "--snapshot", snapshotURL.path])
        let data = try Data(contentsOf: outputURL)
        guard !data.isEmpty, data.count <= 8 * 1024 * 1024 else { throw DeskError.invalidFont }
        return data
    }
    static func run(_ tool: URL, _ arguments: [String]) throws {
        let process = Process()
        process.executableURL = tool; process.arguments = arguments
        // Tool output may describe user text. Do not print, persist or display it.
        process.standardOutput = FileHandle.nullDevice; process.standardError = FileHandle.nullDevice
        try process.run()
        let deadline = Date().addingTimeInterval(120)
        while process.isRunning {
            if Task<Never, Never>.isCancelled || Date() > deadline {
                process.terminate(); process.waitUntilExit()
                throw DeskError.fontBakeFailed
            }
            Thread.sleep(forTimeInterval: 0.05)
        }
        guard process.terminationStatus == 0 else { throw DeskError.fontBakeFailed }
    }
    static func hash(_ data: Data) -> String { SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined() }
}
