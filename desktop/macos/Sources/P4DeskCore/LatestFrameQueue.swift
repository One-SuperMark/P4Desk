import Foundation

public struct LatestFrameToken: Equatable, Sendable {
    fileprivate let generation: UInt64
    fileprivate let ordinal: UInt64
}

public struct LatestFrameWork<Value> {
    public let token: LatestFrameToken
    public let value: Value
}

public struct LatestFrameQueueStatistics: Codable, Equatable, Sendable {
    public let submitted: UInt64
    public let completed: UInt64
    public let replaced: UInt64
    public let inFlight: Bool
    public let pending: Bool
}

/// The producer replaces pending work before scheduling anything on a slow consumer queue.
/// A consumer owns one work item; this object retains at most one newer value.
public final class LatestFrameQueue<Value>: @unchecked Sendable {
    private let lock = NSLock()
    private var generation: UInt64 = 0
    private var ordinal: UInt64 = 0
    private var running = false
    private var busy: LatestFrameToken?
    private var pending: Value?
    private var submitted: UInt64 = 0
    private var completed: UInt64 = 0
    private var replaced: UInt64 = 0

    public init() {}

    public func start() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1; ordinal = 0; running = true
        busy = nil; pending = nil; submitted = 0; completed = 0; replaced = 0
    }

    public func stop() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1; running = false; busy = nil; pending = nil
    }

    /// Only a returned work item should enqueue a consumer block.
    public func submit(_ value: Value) -> LatestFrameWork<Value>? {
        lock.lock(); defer { lock.unlock() }
        guard running else { return nil }
        submitted &+= 1
        if busy != nil {
            if pending != nil { replaced &+= 1 }
            pending = value
            return nil
        }
        return reserve(value)
    }

    public func isCurrent(_ token: LatestFrameToken) -> Bool {
        lock.lock(); defer { lock.unlock() }
        return running && busy == token && token.generation == generation
    }

    /// A completion from an earlier start/stop generation cannot release current work.
    public func complete(_ token: LatestFrameToken) -> LatestFrameWork<Value>? {
        lock.lock(); defer { lock.unlock() }
        guard running, busy == token, token.generation == generation else { return nil }
        completed &+= 1
        busy = nil
        guard let next = pending else { return nil }
        pending = nil
        return reserve(next)
    }

    public var statistics: LatestFrameQueueStatistics {
        lock.lock(); defer { lock.unlock() }
        return LatestFrameQueueStatistics(submitted: submitted, completed: completed, replaced: replaced,
                                          inFlight: busy != nil, pending: pending != nil)
    }

    private func reserve(_ value: Value) -> LatestFrameWork<Value> {
        ordinal &+= 1
        let token = LatestFrameToken(generation: generation, ordinal: ordinal)
        busy = token
        return LatestFrameWork(token: token, value: value)
    }
}

public struct LatestDeliveryToken: Equatable, Sendable {
    fileprivate let generation: UInt64
    fileprivate let ordinal: UInt64
}

/// One scheduled consumer task and one newest pending value. The task captures a token,
/// not the value: an executor stalled before take() delivers the newest value when it resumes.
public final class LatestFrameDelivery<Value>: @unchecked Sendable {
    private let lock = NSLock()
    private var generation: UInt64 = 0
    private var ordinal: UInt64 = 0
    private var running = false
    private var scheduled: LatestDeliveryToken?
    private var taken = false
    private var latest: Value?
    private var submitted: UInt64 = 0
    private var completed: UInt64 = 0
    private var replaced: UInt64 = 0

    public init() {}
    public func start() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1; ordinal = 0; running = true
        scheduled = nil; taken = false; latest = nil
        submitted = 0; completed = 0; replaced = 0
    }
    public func stop() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1; running = false; scheduled = nil; taken = false; latest = nil
    }
    public func submit(_ value: Value) -> LatestDeliveryToken? {
        lock.lock(); defer { lock.unlock() }
        guard running else { return nil }
        submitted &+= 1
        if latest != nil { replaced &+= 1 }
        latest = value
        guard scheduled == nil else { return nil }
        return reserve()
    }
    public func take(_ token: LatestDeliveryToken) -> Value? {
        lock.lock(); defer { lock.unlock() }
        guard running, scheduled == token, token.generation == generation, !taken,
              let value = latest else { return nil }
        latest = nil; taken = true
        return value
    }
    public func complete(_ token: LatestDeliveryToken) -> LatestDeliveryToken? {
        lock.lock(); defer { lock.unlock() }
        guard running, scheduled == token, token.generation == generation, taken else { return nil }
        completed &+= 1; scheduled = nil; taken = false
        return latest == nil ? nil : reserve()
    }
    public var statistics: LatestFrameQueueStatistics {
        lock.lock(); defer { lock.unlock() }
        return LatestFrameQueueStatistics(submitted: submitted, completed: completed, replaced: replaced,
                                          inFlight: taken, pending: latest != nil)
    }
    private func reserve() -> LatestDeliveryToken {
        ordinal &+= 1
        let token = LatestDeliveryToken(generation: generation, ordinal: ordinal)
        scheduled = token
        return token
    }
}
