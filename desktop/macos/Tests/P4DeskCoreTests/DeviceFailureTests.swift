import XCTest
@testable import P4DeskCore

final class DeviceFailureTests: XCTestCase {
    func testSyncAckPreservesKnownErrorWithoutProtocolOperationOrPayload() throws {
        let payload = Data(#"{"op":"ack","request_id":21,"acknowledged":"sync_commit","ok":false,"error":"font_read"}"#.utf8)
        let fields = try JSONControl.object(payload)
        let failure = DeviceFailure(code: DeviceAcknowledgement.safeErrorCode(fields))
        XCTAssertEqual(failure.code, "font_read")
        XCTAssertTrue(failure.errorDescription?.contains("字库") == true)
        XCTAssertFalse(failure.errorDescription?.contains("sync_commit") == true)
        XCTAssertFalse(failure.errorDescription?.contains("font_read") == true)
    }
    func testDeviceErrorsAreSanitizedAndErrorFieldWins() {
        XCTAssertEqual(DeviceAcknowledgement.safeErrorCode(["error": "font_hash", "code": "font_read"]), "font_hash")
        XCTAssertEqual(DeviceAcknowledgement.safeErrorCode(["code": "font_coverage"]), "font_coverage")
        XCTAssertEqual(DeviceAcknowledgement.safeErrorCode(["error": "secret path and note contents"]), "device_rejected")
        XCTAssertEqual(DeviceFailure(code: "secret path").code, "device_rejected")
        XCTAssertEqual(DeviceAcknowledgement.safeErrorCode(["error": true]), "device_rejected")
        XCTAssertEqual(DeviceAcknowledgement.safeErrorCode([:]), "device_rejected")
    }
    func testSyncFailureReasonsGiveActionableChineseMessages() {
        for code in ["font_read", "font_write", "font_fsync", "font_invalid", "font_coverage", "font_hash", "generation_rename", "sync_incomplete", "sync_not_started", "chunk_offset", "sd_unavailable", "sd_space", "files_busy", "sync_busy", "generation_exists"] {
            let failure = DeviceFailure(code: code)
            XCTAssertEqual(failure.code, code)
            XCTAssertNotNil(failure.errorDescription)
            XCTAssertFalse(failure.errorDescription?.contains(code) ?? true)
        }
        XCTAssertTrue(DeviceFailure(code: "sd_space").errorDescription?.contains("空间不足") == true)
        XCTAssertTrue(DeviceFailure(code: "font_hash").errorDescription?.contains("校验失败") == true)
    }
    func testSyncResultContainsOnlyNumericMetadataAndFixedState() {
        let result = SyncResult(committed: true, errorCode: "none", fontBytes: 4096, generation: 42)
        XCTAssertTrue(result.committed)
        XCTAssertEqual(result.fontBytes, 4096)
        XCTAssertEqual(result.generation, 42)
        XCTAssertEqual(SyncResult().errorCode, "not_started")
    }
}
