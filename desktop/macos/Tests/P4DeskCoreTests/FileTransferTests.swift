import XCTest
import CryptoKit
@testable import P4DeskCore

private actor FileEndpoint {
    var manifest: FileUploadManifest?
    var destination: String?
    var bytes = Data()
    var offsets: [UInt32] = []
    var chunkSizes: [Int] = []
    var committed = false
    var aborted = false
    var rejectBegin = false
    var rejectChunk = false
    var cancelChunk = false
    var mutateURL: URL?
    func setRejectBegin() { rejectBegin = true }
    func setRejectChunk() { rejectChunk = true }
    func setCancelChunk() { cancelChunk = true }
    func setMutateURL(_ url: URL) { mutateURL = url }
    func send(_ command: FileUploadCommand) async throws {
        switch command {
        case .begin(let path, let value):
            destination = path; manifest = value
            if rejectBegin { throw FileTransferError.rejected("exists") }
        case .chunk(let offset, let data):
            // Sequential sender cannot send a second chunk before this ACK.
            try await Task.sleep(nanoseconds: 1_000_000)
            if rejectChunk { throw FileTransferError.rejected("no_space") }
            if cancelChunk { throw CancellationError() }
            offsets.append(offset); chunkSizes.append(data.count); bytes.append(data)
            if let url = mutateURL {
                let handle = try FileHandle(forWritingTo: url)
                try handle.truncate(atOffset: 0); try handle.close(); mutateURL = nil
            }
        case .commit: committed = true
        case .abort: aborted = true
        }
    }
    func result() -> (FileUploadManifest?, String?, Data, [UInt32], [Int], Bool, Bool) {
        (manifest, destination, bytes, offsets, chunkSizes, committed, aborted)
    }
}

final class FileTransferTests: XCTestCase {
    private var directory: URL!
    override func setUpWithError() throws {
        directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }
    override func tearDownWithError() throws { try? FileManager.default.removeItem(at: directory) }
    private func source(_ data: Data, name: String = "测试.txt") throws -> URL {
        let url = directory.appendingPathComponent(name); try data.write(to: url); return url
    }
    func testAcknowledgementMatchesRustErrorAndLegacyCodeWithoutExposingText() {
        let rust: [String: Any] = ["op": "ack", "request_id": 7, "acknowledged": "file_upload_begin", "ok": false, "error": "already_exists"]
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(rust), "already_exists")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["error": "not_found"]), "not_found")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["code": "no_space"]), "no_space")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["error": "protected", "code": "no_space"]), "protected")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["error": "private/path and content"]), "failed")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["error": "unknown", "code": "exists"]), "failed")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode(["error": true]), "failed")
        XCTAssertEqual(FileAcknowledgement.safeErrorCode([:]), "failed")
        for code in ["already_exists", "storage_unavailable", "protected", "files_busy", "sync_busy", "display_busy", "offset"] {
            XCTAssertFalse(FileTransferError.rejected(code).errorDescription?.contains(code) ?? true)
        }
    }
    func testDiagnosticLabelsOnlyEmitFixedSafeCodes() {
        XCTAssertEqual(FileTransferError.rejected("already_exists").diagnosticCode, "already_exists")
        XCTAssertEqual(FileTransferError.rejected("secret/path body").diagnosticCode, "failed")
        XCTAssertEqual(FileTransferError.sourceChanged.diagnosticCode, "source_changed")
        XCTAssertEqual(FileTransferError.localRead.diagnosticCode, "local_read")
        XCTAssertEqual(FileTransferError.notRegularFile.diagnosticCode, "not_regular_file")
        XCTAssertEqual(FileTransferError.invalidListing.diagnosticCode, "invalid_listing")
        XCTAssertEqual(FileTransferError.busy.diagnosticCode, "host_busy")
    }
    func testChunkLittleEndianAndBounds() throws {
        let payload = try FileResourceChunk.payload(offset: 0x12345678, data: Data([0xab, 0xcd]))
        XCTAssertEqual(payload, Data([0x78, 0x56, 0x34, 0x12, 0xab, 0xcd]))
        XCTAssertThrowsError(try FileResourceChunk.payload(offset: 0, data: Data()))
        XCTAssertThrowsError(try FileResourceChunk.payload(offset: 0, data: Data(count: 32769)))
        let packet = Packet(kind: .resource, sequence: 23, payload: try FileResourceChunk.payload(offset: 32_768, data: Data(count: 32_768)))
        var parser = PacketParser()
        XCTAssertEqual(parser.feed(try packet.encoded()), [packet])
    }
    func testPathValidationAndProtection() throws {
        try RemoteFilePath.validate("")
        try RemoteFilePath.validate("Downloads/中文照片 🌻.jpg")
        try RemoteFilePath.validate(String(repeating: "a", count: 255))
        for path in ["/root", "../file", "a/../file", "a//b", "./a", "a\\b", "a:b", "a*", "a?", "a\"", "a<", "a>", "a|", "file.", "file ", "a\u{0000}", "a\n", String(repeating: "中", count: 86)] {
            XCTAssertThrowsError(try RemoteFilePath.validate(path), path)
        }
        XCTAssertThrowsError(try RemoteFilePath.joined("", name: "a/b"))
        XCTAssertThrowsError(try RemoteFilePath.joined("P4Desk", name: "state.json"))
        XCTAssertEqual(try RemoteFilePath.joined("Downloads", name: "新文件.txt"), "Downloads/新文件.txt")
        XCTAssertTrue(RemoteFilePath.isSystem("p4DESK/data"))
        XCTAssertFalse(RemoteFilePath.isSystem("p4desk-other/data"))
        XCTAssertEqual(RemoteFilePath.parent("Downloads/a"), "Downloads")
    }
    func testListingValidationAndReadOnly() throws {
        var object: [String: Any] = ["op": "file_listing", "path": "p4desk", "entries": [["name": "资源", "path": "p4desk/资源", "directory": true, "size": 0, "modified_seconds": NSNull(), "read_only": true]], "total": 1, "truncated": false, "read_only": true]
        let listing = try RemoteFileListing.decode(object, expectedPath: "p4desk")
        XCTAssertTrue(listing.readOnly); XCTAssertEqual(listing.entries[0].name, "资源")
        XCTAssertNil(listing.entries[0].modifiedSeconds)
        XCTAssertThrowsError(try RemoteFileListing.decode(object, expectedPath: "Downloads"))
        object["entries"] = [["name": "资源", "path": "foreign/资源", "directory": true, "size": 0, "read_only": false]]
        XCTAssertThrowsError(try RemoteFileListing.decode(object, expectedPath: "p4desk"))
        object["entries"] = Array(repeating: ["name": "a", "path": "p4desk/a", "directory": false, "size": 1, "read_only": true], count: 2)
        object["total"] = 2
        XCTAssertThrowsError(try RemoteFileListing.decode(object, expectedPath: "p4desk"))
    }
    func testStreamingUploadExactContentAndDigest() async throws {
        let input = Data((0..<(3 * 32_768 + 7)).map { UInt8(truncatingIfNeeded: $0) })
        let url = try source(input), endpoint = FileEndpoint()
        let manifest = try await FileTransferEngine.upload(url: url, destination: "Downloads/测试.txt", send: { try await endpoint.send($0) })
        let result = await endpoint.result()
        XCTAssertEqual(manifest.length, UInt64(input.count))
        XCTAssertEqual(manifest.sha256, SHA256.hash(data: input).map { String(format: "%02x", $0) }.joined())
        XCTAssertEqual(result.2, input)
        XCTAssertEqual(result.3, [0, 32_768, 65_536, 98_304])
        XCTAssertEqual(result.4, [32_768, 32_768, 32_768, 7])
        XCTAssertEqual(result.1, "Downloads/测试.txt")
        XCTAssertTrue(result.5); XCTAssertFalse(result.6)
    }
    func testEmptyFileCommitsWithoutResourcePacket() async throws {
        let endpoint = FileEndpoint()
        let manifest = try await FileTransferEngine.upload(url: try source(Data()), destination: "Downloads/empty.txt", send: { try await endpoint.send($0) })
        let result = await endpoint.result()
        XCTAssertEqual(manifest.length, 0)
        XCTAssertEqual(manifest.sha256, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        XCTAssertTrue(result.3.isEmpty); XCTAssertTrue(result.5); XCTAssertFalse(result.6)
    }
    func testBeginReplyFailureStillAttemptsAbort() async throws {
        let endpoint = FileEndpoint(); await endpoint.setRejectBegin()
        do {
            _ = try await FileTransferEngine.upload(url: try source(Data([1])), destination: "Downloads/a.txt", send: { try await endpoint.send($0) })
            XCTFail("expected rejection")
        } catch { XCTAssertEqual(error as? FileTransferError, .rejected("exists")) }
        let result = await endpoint.result()
        XCTAssertTrue(result.6); XCTAssertFalse(result.5); XCTAssertTrue(result.2.isEmpty)
    }
    func testNoSpaceAbortsWithoutCommit() async throws {
        let endpoint = FileEndpoint(); await endpoint.setRejectChunk()
        do {
            _ = try await FileTransferEngine.upload(url: try source(Data([1, 2])), destination: "Downloads/a.txt", send: { try await endpoint.send($0) })
            XCTFail("expected rejection")
        } catch { XCTAssertEqual(error as? FileTransferError, .rejected("no_space")) }
        let result = await endpoint.result(); XCTAssertTrue(result.6); XCTAssertFalse(result.5)
    }
    func testCancellationAbortsWithoutCommit() async throws {
        let endpoint = FileEndpoint(); await endpoint.setCancelChunk()
        do {
            _ = try await FileTransferEngine.upload(url: try source(Data(count: 70_000)), destination: "Downloads/a.bin", send: { try await endpoint.send($0) })
            XCTFail("expected cancellation")
        } catch { XCTAssertTrue(error is CancellationError) }
        let result = await endpoint.result(); XCTAssertTrue(result.6); XCTAssertFalse(result.5)
    }
    func testCancelledTaskStopsBeforeFirstChunkAndStillAborts() async throws {
        let endpoint = FileEndpoint()
        let url = try source(Data(count: 70_000))
        let task = Task {
            try await FileTransferEngine.upload(url: url, destination: "Downloads/a.bin", send: { command in
                if case .chunk = command { try await Task.sleep(nanoseconds: 10_000_000_000) }
                try await endpoint.send(command)
            })
        }
        // Wait for the begin ACK rather than depending on a timing guess.
        for _ in 0..<10_000 {
            if await endpoint.result().0 != nil { break }
            await Task.yield()
        }
        let begun = await endpoint.result().0 != nil
        XCTAssertTrue(begun)
        task.cancel()
        do { _ = try await task.value; XCTFail("expected cancellation") }
        catch { XCTAssertTrue(error is CancellationError) }
        let result = await endpoint.result()
        XCTAssertTrue(result.6); XCTAssertFalse(result.5); XCTAssertTrue(result.3.isEmpty)
    }
    func testSymlinkRejectedBeforeBegin() async throws {
        let target = try source(Data([1, 2, 3]), name: "target.bin")
        let link = directory.appendingPathComponent("alias.bin")
        try FileManager.default.createSymbolicLink(at: link, withDestinationURL: target)
        let endpoint = FileEndpoint()
        do {
            _ = try await FileTransferEngine.upload(url: link, destination: "Downloads/alias.bin", send: { try await endpoint.send($0) })
            XCTFail("symbolic link accepted")
        } catch { XCTAssertEqual(error as? FileTransferError, .notRegularFile) }
        let result = await endpoint.result(); XCTAssertNil(result.0)
    }
    func testChangedSourceAbortsWithoutCommit() async throws {
        let url = try source(Data(count: 70_000)), endpoint = FileEndpoint()
        await endpoint.setMutateURL(url)
        do {
            _ = try await FileTransferEngine.upload(url: url, destination: "Downloads/a.bin", send: { try await endpoint.send($0) })
            XCTFail("expected source change")
        } catch { XCTAssertEqual(error as? FileTransferError, .sourceChanged) }
        let result = await endpoint.result(); XCTAssertEqual(result.3, [0]); XCTAssertTrue(result.6); XCTAssertFalse(result.5)
    }
    func testFolderAndOversizeRejectedBeforeBegin() async throws {
        let endpoint = FileEndpoint()
        do {
            _ = try await FileTransferEngine.upload(url: directory, destination: "Downloads/a", send: { try await endpoint.send($0) })
            XCTFail("folder accepted")
        } catch { XCTAssertEqual(error as? FileTransferError, .notRegularFile) }
        let big = try source(Data(), name: "large.bin")
        let handle = try FileHandle(forWritingTo: big)
        try handle.truncate(atOffset: FileTransferEngine.maximumFileBytes + 1); try handle.close()
        do {
            _ = try await FileTransferEngine.upload(url: big, destination: "Downloads/large.bin", send: { try await endpoint.send($0) })
            XCTFail("oversize accepted")
        } catch { XCTAssertEqual(error as? FileTransferError, .tooLarge) }
        let result = await endpoint.result(); XCTAssertNil(result.0); XCTAssertFalse(result.6)
    }
    func testProtectedPathRejectedBeforeReadingSource() async throws {
        let endpoint = FileEndpoint()
        do {
            _ = try await FileTransferEngine.upload(url: directory.appendingPathComponent("missing"), destination: "P4DESK/config.json", send: { try await endpoint.send($0) })
            XCTFail("system path accepted")
        } catch { XCTAssertEqual(error as? FileTransferError, .readOnly) }
        let result = await endpoint.result(); XCTAssertNil(result.0)
    }
}
