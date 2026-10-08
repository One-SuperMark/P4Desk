import Foundation

/// A completed operation is acknowledged once instead of remaining in a status banner.
public struct OperationNotice: Identifiable, Equatable, Sendable {
    public let id: UUID
    public let title: String
    public let message: String

    public init(title: String, message: String) {
        self.id = UUID(); self.title = title; self.message = message
    }
}

/// Preserve an existing failure while another operation finishes. Presentation
/// advances only after dismissal; identical reports are coalesced. At most eight
/// unread notices wait behind the current one; overflow preserves existing items.
public struct OperationNoticeQueue: Sendable {
    private static let pendingLimit = 8
    public private(set) var current: OperationNotice?
    private var pending: [OperationNotice] = []

    public init() {}

    @discardableResult
    public mutating func enqueue(_ notice: OperationNotice, suppressed: Bool = false) -> Bool {
        guard !suppressed else { return false }
        let duplicate = { (existing: OperationNotice) in
            existing.title == notice.title && existing.message == notice.message
        }
        guard !(current.map(duplicate) ?? false), !pending.contains(where: duplicate) else { return false }
        guard pending.count < Self.pendingLimit else { return false }
        pending.append(notice)
        presentNext()
        return true
    }

    public mutating func dismissCurrent() { current = nil }

    public mutating func presentNext() {
        guard current == nil, !pending.isEmpty else { return }
        current = pending.removeFirst()
    }
}
