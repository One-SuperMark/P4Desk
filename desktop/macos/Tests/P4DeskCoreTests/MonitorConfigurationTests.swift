import XCTest
import SQLite3
@testable import P4DeskCore
final class MonitorConfigurationTests: XCTestCase {
    func testNormalizeAndRejectUnsafeAuthority() throws {
        XCTAssertEqual(try MonitorConfiguration(site: "monitor.example/api/v1/", key: "demo").site, "https://monitor.example")
        for site in ["http://example.com", "https://a:b@example.com", "https://example.com?x=y", "https://example.com/a/../b", "https://example.com/%2e"] {
            XCTAssertThrowsError(try MonitorConfiguration(site: site, key: "demo"))
        }
        XCTAssertThrowsError(try MonitorConfiguration(site: "example.com", key: "a\r\nb"))
        XCTAssertFalse(String(describing: try MonitorConfiguration(site: "example.com", key: "test-private-key")).contains("test-private-key"))
    }
    func testReadOnlyImportUsesSiteSpecificCredential() throws {
        let path = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".sqlite3")
        defer { try? FileManager.default.removeItem(at: path) }
        var db: OpaquePointer?
        XCTAssertEqual(sqlite3_open(path.path, &db), SQLITE_OK)
        XCTAssertEqual(sqlite3_exec(db, "CREATE TABLE settings(key TEXT, value TEXT); CREATE TABLE credentials(service TEXT, account TEXT, value TEXT); INSERT INTO settings VALUES('monitor.site','https://example.com'); INSERT INTO credentials VALUES('cloud.supermark.Sub2APIMonitor','https://example.com','demo-key'); INSERT INTO credentials VALUES('cloud.supermark.Sub2APIMonitor.translation','https://example.com','wrong-key');", nil, nil, nil), SQLITE_OK)
        sqlite3_close(db)
        let before = try Data(contentsOf: path)
        let config = try MonitorConfiguration.readLocal(database: path)
        XCTAssertEqual(config.key, "demo-key")
        XCTAssertEqual(before, try Data(contentsOf: path))
    }
    func testMissingDatabaseDoesNotCreateFile() {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        XCTAssertThrowsError(try MonitorConfiguration.readLocal(database: url))
        XCTAssertFalse(FileManager.default.fileExists(atPath: url.path))
    }
}
