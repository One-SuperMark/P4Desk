import Foundation
import SQLite3

public struct MonitorConfiguration: CustomStringConvertible {
    public let site: String
    public let key: String
    public var description: String { "MonitorConfiguration([redacted])" }
    public enum Failure: Error { case invalidSite, invalidKey, unavailable }
    public init(site: String, key: String) throws {
        var input = site.trimmingCharacters(in: .whitespacesAndNewlines)
        if !input.contains("://") { input = "https://" + input }
        while input.hasSuffix("/") { input.removeLast() }
        guard input.utf8.count <= 240, input.utf8.allSatisfy({ $0 >= 33 && $0 <= 126 }),
              !input.contains("%"), !input.contains("\\"),
              var parts = URLComponents(string: input), parts.scheme == "https",
              let host = parts.host, !host.isEmpty,
              host.utf8.allSatisfy({ (45...57).contains($0) && $0 != 47 || (65...90).contains($0) || (97...122).contains($0) }),
              parts.user == nil, parts.password == nil, parts.query == nil, parts.fragment == nil,
              !parts.path.split(separator: "/").contains(where: { $0 == "." || $0 == ".." }),
              parts.port == nil || (1...65535).contains(parts.port!) else { throw Failure.invalidSite }
        if parts.path.hasSuffix("/api/v1") { parts.path.removeLast(7) }
        guard let normalized = parts.url?.absoluteString else { throw Failure.invalidSite }
        let secret = key.trimmingCharacters(in: .whitespacesAndNewlines)
        guard (1...512).contains(secret.utf8.count), secret.utf8.allSatisfy({ (33...126).contains($0) }) else { throw Failure.invalidKey }
        self.site = normalized; self.key = secret
    }
    /// User-triggered read of the current Rust data store; never creates/migrates it.
    public static func readLocal(database: URL? = nil) throws -> Self {
        let path = database ?? FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("Sub2APIMonitor/data-v2.sqlite3")
        var db: OpaquePointer?
        guard sqlite3_open_v2(path.path, &db, SQLITE_OPEN_READONLY, nil) == SQLITE_OK, let handle = db else {
            if db != nil { sqlite3_close(db) }; throw Failure.unavailable
        }
        defer { sqlite3_close(handle) }
        sqlite3_busy_timeout(handle, 500)
        func read(_ sql: String, bindings: [String] = []) throws -> String {
            var stmt: OpaquePointer?
            guard sqlite3_prepare_v2(handle, sql, -1, &stmt, nil) == SQLITE_OK, let stmt else { throw Failure.unavailable }
            defer { sqlite3_finalize(stmt) }
            let transient = unsafeBitCast(-1, to: sqlite3_destructor_type.self)
            for (i, value) in bindings.enumerated() { sqlite3_bind_text(stmt, Int32(i + 1), value, -1, transient) }
            guard sqlite3_step(stmt) == SQLITE_ROW, sqlite3_column_bytes(stmt, 0) <= 1024,
                  let value = sqlite3_column_text(stmt, 0) else { throw Failure.unavailable }
            return String(cString: value)
        }
        let site = try read("SELECT value FROM settings WHERE key='monitor.site'")
        let key = try read("SELECT value FROM credentials WHERE service=? AND account=?", bindings: ["cloud.supermark.Sub2APIMonitor", site])
        return try Self(site: site, key: key)
    }
}
