import Foundation

public struct Note: Codable, Identifiable, Equatable {
    public var id: String
    public var title: String
    public var body: String
    public var updatedMS: UInt64
    enum CodingKeys: String, CodingKey { case id, title, body; case updatedMS = "updated_ms" }
    public init(id: String = UUID().uuidString.lowercased(), title: String = "便签", body: String = "", updatedMS: UInt64 = UInt64(Date().timeIntervalSince1970 * 1000)) {
        self.id = id; self.title = title; self.body = body; self.updatedMS = updatedMS
    }
}
public struct DeskAction: Codable, Equatable {
    public var kind: String
    public var keyCode: UInt16?
    public var modifiers: UInt32?
    public var bundlePath: String?
    public var usage: UInt16?
    enum CodingKeys: String, CodingKey { case kind, modifiers, usage; case keyCode = "key_code"; case bundlePath = "bundle_path" }
    public init(kind: String, keyCode: UInt16? = nil, modifiers: UInt32? = nil, bundlePath: String? = nil, usage: UInt16? = nil) {
        self.kind = kind; self.keyCode = keyCode; self.modifiers = modifiers; self.bundlePath = bundlePath; self.usage = usage
    }
    public static func shortcut(_ key: UInt16, flags: UInt32) -> DeskAction { DeskAction(kind: "shortcut", keyCode: key, modifiers: flags) }
    public static func application(_ path: String) -> DeskAction { DeskAction(kind: "application", bundlePath: path) }
    public static func media(_ usage: UInt16) -> DeskAction { DeskAction(kind: "media", usage: usage) }
}
public struct DeskButton: Codable, Identifiable, Equatable {
    public var id: String
    public var label: String
    public var action: DeskAction
    public init(id: String = UUID().uuidString.lowercased(), label: String = "新按钮", action: DeskAction = .shortcut(8, flags: 1 << 20)) {
        self.id = id; self.label = label; self.action = action
    }
}
public struct Snapshot: Codable, Equatable {
    public var generation: UInt64
    public var notes: [Note]
    public var buttons: [DeskButton]
    public var deletedNoteIDs: [String]
    enum CodingKeys: String, CodingKey { case generation, notes, buttons; case deletedNoteIDs = "deleted_note_ids" }
    public init(generation: UInt64 = 0, notes: [Note] = [], buttons: [DeskButton] = [], deletedNoteIDs: [String] = []) {
        self.generation = generation; self.notes = notes; self.buttons = buttons; self.deletedNoteIDs = deletedNoteIDs
    }
    public func validated() throws {
        guard notes.count <= 32, buttons.count <= 48, deletedNoteIDs.count <= 4096 else { throw ProtocolError.invalidSnapshot("数量超过限制") }
        func idOK(_ id: String) -> Bool {
            !id.isEmpty && id.utf8.count <= 64 && id.utf8.allSatisfy {
                (0x30...0x39).contains($0) || (0x41...0x5a).contains($0) || (0x61...0x7a).contains($0) || $0 == 0x2d || $0 == 0x5f
            }
        }
        guard Set(notes.map(\.id)).count == notes.count, Set(buttons.map(\.id)).count == buttons.count else { throw ProtocolError.invalidSnapshot("ID 重复") }
        for note in notes {
            guard idOK(note.id), note.title.unicodeScalars.count <= 128, note.body.unicodeScalars.count <= 2000,
                  !deletedNoteIDs.contains(note.id) else { throw ProtocolError.invalidSnapshot("便签字段不符合限制") }
        }
        let media: Set<UInt16> = [0xb0, 0xcd, 0xb5, 0xb6, 0xe2, 0xe9, 0xea]
        for button in buttons {
            guard idOK(button.id), button.label.unicodeScalars.count <= 64 else { throw ProtocolError.invalidSnapshot("按钮字段不符合限制") }
            switch button.action.kind {
            case "shortcut":
                guard let key = button.action.keyCode, key <= 127, let flags = button.action.modifiers,
                      flags & ~UInt32(0x00ff0000) == 0 else { throw ProtocolError.invalidSnapshot("快捷键无效") }
            case "application":
                guard let path = button.action.bundlePath, path.hasPrefix("/"), path.hasSuffix(".app"),
                      path.utf8.count <= 4096, !path.contains("\0") else { throw ProtocolError.invalidSnapshot("应用路径无效") }
            case "media": guard let usage = button.action.usage, media.contains(usage) else { throw ProtocolError.invalidSnapshot("媒体操作无效") }
            default: throw ProtocolError.invalidSnapshot("动作类型无效")
            }
        }
        guard deletedNoteIDs.allSatisfy(idOK) else { throw ProtocolError.invalidSnapshot("删除记录无效") }
        let data = try JSONEncoder().encode(self)
        // Leave room for sync_begin metadata within CONTROL's 64 KiB limit.
        guard data.count <= 60 * 1024 else { throw ProtocolError.invalidSnapshot("配置超过 60 KiB，请减少便签文字") }
    }
    /// Deletions always win, including deletions performed while the Mac was disconnected.
    public mutating func mergeDevice(_ device: Snapshot) throws {
        try device.validated()
        let tombstones = Set(deletedNoteIDs).union(device.deletedNoteIDs)
        var merged = Dictionary(uniqueKeysWithValues: notes.map { ($0.id, $0) })
        for note in device.notes {
            if !tombstones.contains(note.id), merged[note.id].map({ note.updatedMS > $0.updatedMS }) ?? true { merged[note.id] = note }
        }
        notes = merged.values.filter { !tombstones.contains($0.id) }.sorted { $0.updatedMS > $1.updatedMS }
        deletedNoteIDs = tombstones.sorted()
        generation = max(generation, device.generation)
        try validated()
    }
}
