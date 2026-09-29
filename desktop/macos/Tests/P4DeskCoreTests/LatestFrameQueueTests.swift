import XCTest
@testable import P4DeskCore

final class LatestFrameQueueTests: XCTestCase {
    private final class Values: @unchecked Sendable {
        private let lock = NSLock()
        private var values: [Int] = []
        func append(_ value: Int) { lock.lock(); defer { lock.unlock() }; values.append(value) }
        var snapshot: [Int] { lock.lock(); defer { lock.unlock() }; return values }
    }
    func testSlowSynchronousConsumerSkipsQueuedOldFrames() throws {
        let frames = LatestFrameQueue<Int>()
        frames.start()
        let first = try XCTUnwrap(frames.submit(0))
        let entered = DispatchSemaphore(value: 0), resume = DispatchSemaphore(value: 0)
        let done = expectation(description: "consumer drains current and newest")
        let consumer = DispatchQueue(label: "latest-frame-regression")
        let consumed = Values()
        consumer.async {
            consumed.append(first.value)
            entered.signal()
            // A real blocked consumer models synchronous ImageIO. Producer calls still run.
            guard resume.wait(timeout: .now() + 3) == .success else { done.fulfill(); return }
            var next = frames.complete(first.token)
            while let work = next {
                consumed.append(work.value)
                next = frames.complete(work.token)
            }
            done.fulfill()
        }
        XCTAssertEqual(entered.wait(timeout: .now() + 2), .success)
        for value in 1...99 { XCTAssertNil(frames.submit(value)) }
        XCTAssertTrue(frames.statistics.inFlight)
        XCTAssertTrue(frames.statistics.pending)
        XCTAssertEqual(frames.statistics.submitted, 100)
        XCTAssertEqual(frames.statistics.replaced, 98)
        resume.signal()
        wait(for: [done], timeout: 3)
        XCTAssertEqual(consumed.snapshot, [0, 99])
        XCTAssertEqual(frames.statistics.completed, 2)
        XCTAssertFalse(frames.statistics.inFlight)
        XCTAssertFalse(frames.statistics.pending)
    }

    func testLateCompletionAfterStopAndRestartCannotReleaseNewWork() throws {
        let frames = LatestFrameQueue<Int>()
        frames.start()
        let old = try XCTUnwrap(frames.submit(1))
        XCTAssertNil(frames.submit(2))
        frames.stop()
        XCTAssertFalse(frames.isCurrent(old.token))
        XCTAssertNil(frames.submit(3))
        XCTAssertNil(frames.complete(old.token))
        XCTAssertFalse(frames.statistics.pending)
        frames.start()
        let current = try XCTUnwrap(frames.submit(4))
        XCTAssertNil(frames.submit(5))
        XCTAssertNil(frames.complete(old.token))
        XCTAssertTrue(frames.isCurrent(current.token))
        XCTAssertEqual(frames.statistics.completed, 0)
        let latest = try XCTUnwrap(frames.complete(current.token))
        XCTAssertEqual(latest.value, 5)
        XCTAssertFalse(frames.isCurrent(current.token))
        XCTAssertTrue(frames.isCurrent(latest.token))
        XCTAssertNil(frames.complete(latest.token))
        XCTAssertEqual(frames.statistics.completed, 2)
    }

    func testStopReleasesRetainedPendingValue() throws {
        final class Value {}
        let frames = LatestFrameQueue<Value>()
        frames.start()
        let current = try XCTUnwrap(frames.submit(Value()))
        weak var pending: Value?
        do {
            let value = Value(); pending = value
            XCTAssertNil(frames.submit(value))
        }
        XCTAssertNotNil(pending)
        frames.stop()
        XCTAssertNil(pending)
        XCTAssertNil(frames.complete(current.token))
        XCTAssertFalse(frames.statistics.inFlight)
    }

    func testDeliveryBlockedExecutorTakesNewestFrameInsteadOfScheduledOldFrame() throws {
        let delivery = LatestFrameDelivery<Int>()
        delivery.start()
        let executor = DispatchQueue(label: "latest-delivery-blocked-executor")
        let entered = DispatchSemaphore(value: 0), resume = DispatchSemaphore(value: 0)
        let done = expectation(description: "one delivery after executor resumes")
        let values = Values()
        executor.async {
            entered.signal()
            _ = resume.wait(timeout: .now() + 3)
        }
        XCTAssertEqual(entered.wait(timeout: .now() + 2), .success)
        let token = try XCTUnwrap(delivery.submit(0))
        executor.async {
            if let value = delivery.take(token) { values.append(value) }
            XCTAssertNil(delivery.complete(token))
            done.fulfill()
        }
        for value in 1...299 { XCTAssertNil(delivery.submit(value)) }
        XCTAssertEqual(delivery.statistics.submitted, 300)
        XCTAssertEqual(delivery.statistics.replaced, 299)
        XCTAssertFalse(delivery.statistics.inFlight)
        XCTAssertTrue(delivery.statistics.pending)
        resume.signal()
        wait(for: [done], timeout: 3)
        // This first valid delivery can still fulfil a first-JPEG waiter; no old FIFO is drained.
        XCTAssertEqual(values.snapshot, [299])
        XCTAssertEqual(delivery.statistics.completed, 1)
        XCTAssertFalse(delivery.statistics.pending)
    }

    func testDeliverySlowConsumerKeepsOnlyNewestFollowupAndOneScheduledToken() throws {
        let delivery = LatestFrameDelivery<Int>()
        delivery.start()
        let first = try XCTUnwrap(delivery.submit(0))
        let entered = DispatchSemaphore(value: 0), resume = DispatchSemaphore(value: 0)
        let done = expectation(description: "current and one latest followup delivered")
        let values = Values()
        DispatchQueue(label: "latest-delivery-slow-consumer").async {
            if let value = delivery.take(first) { values.append(value) }
            entered.signal()
            _ = resume.wait(timeout: .now() + 3)
            if let next = delivery.complete(first), let value = delivery.take(next) {
                values.append(value)
                XCTAssertNil(delivery.complete(next))
            }
            done.fulfill()
        }
        XCTAssertEqual(entered.wait(timeout: .now() + 2), .success)
        for value in 1...100 { XCTAssertNil(delivery.submit(value)) }
        XCTAssertTrue(delivery.statistics.inFlight)
        XCTAssertTrue(delivery.statistics.pending)
        XCTAssertEqual(delivery.statistics.replaced, 99)
        resume.signal()
        wait(for: [done], timeout: 3)
        XCTAssertEqual(values.snapshot, [0, 100])
        XCTAssertEqual(delivery.statistics.completed, 2)
        XCTAssertFalse(delivery.statistics.inFlight)
        XCTAssertFalse(delivery.statistics.pending)
    }

    func testDeliveryRestartRejectsOldTokenAndReleasesReplacedOrStoppedValue() throws {
        final class Value {}
        let delivery = LatestFrameDelivery<Value>()
        delivery.start()
        weak var replaced: Value?
        let old: LatestDeliveryToken
        do {
            let value = Value(); replaced = value
            old = try XCTUnwrap(delivery.submit(value))
        }
        XCTAssertNotNil(replaced)
        XCTAssertNil(delivery.submit(Value()))
        XCTAssertNil(replaced) // The scheduled token retains no payload.
        delivery.stop()
        XCTAssertNil(delivery.take(old))
        delivery.start()
        weak var pending: Value?
        let current: LatestDeliveryToken
        do {
            let value = Value(); pending = value
            current = try XCTUnwrap(delivery.submit(value))
        }
        XCTAssertNil(delivery.take(old))
        XCTAssertNil(delivery.complete(old))
        XCTAssertNotNil(pending)
        XCTAssertNotNil(delivery.take(current))
        XCTAssertNil(pending)
        XCTAssertNil(delivery.take(current)) // Never consume one scheduled task twice.
        XCTAssertNil(delivery.complete(current))
        XCTAssertEqual(delivery.statistics.completed, 1)
        do {
            let value = Value(); pending = value
            XCTAssertNotNil(delivery.submit(value))
        }
        delivery.stop()
        XCTAssertNil(pending)
        XCTAssertFalse(delivery.statistics.pending)
    }
}
