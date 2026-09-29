import XCTest
@testable import P4DeskCore

final class CaptureRateGateTests: XCTestCase {
    func testJitteredSixtyFPSAcceptsAllThirtySeconds() {
        var gate = CaptureRateGate()
        gate.start()
        let base: Int64 = 1_000_000_000_000
        let jitter: [Int64] = [-500_000, 500_000, -250_000, 250_000]
        for index in 0..<1800 {
            let time = base + Int64(index) * 1_000_000_000 / 60 + jitter[index % jitter.count]
            XCTAssertTrue(gate.accept(presentationNS: UInt64(time)), "60 Hz frame \(index)")
        }
    }

    func testOneHundredTwentyFPSIsBoundedAndDuplicatesNeverRefillCredit() {
        var gate = CaptureRateGate()
        gate.start()
        var accepted = 0
        for index in 0..<(120 * 33) {
            let stamp = 1_000_000_000 + UInt64(index) * 1_000_000_000 / 120
            if gate.accept(presentationNS: stamp) { accepted += 1 }
            XCTAssertFalse(gate.accept(presentationNS: stamp))
        }
        XCTAssertEqual(accepted, 1981) // 60/sec plus the initial bounded burst.
    }

    func testOldAndRateLimitedSamplesCannotInflateBudget() {
        var gate = CaptureRateGate()
        gate.start()
        XCTAssertTrue(gate.accept(presentationNS: 1000))
        XCTAssertTrue(gate.accept(presentationNS: 1001))
        XCTAssertFalse(gate.accept(presentationNS: 1000))
        XCTAssertFalse(gate.accept(presentationNS: 0))
        XCTAssertFalse(gate.accept(presentationNS: 1002))
        for _ in 0..<100 { XCTAssertFalse(gate.accept(presentationNS: 1002)) }
        XCTAssertFalse(gate.accept(presentationNS: 1003))
        XCTAssertFalse(gate.accept(presentationNS: 1000 + 16_666_666))
        XCTAssertTrue(gate.accept(presentationNS: 1000 + 16_666_667))
    }

    func testStopRestartAndLargeGapsRemainSafeAtUInt64Maximum() {
        var gate = CaptureRateGate()
        XCTAssertFalse(gate.accept(presentationNS: UInt64.max))
        gate.start()
        XCTAssertTrue(gate.accept(presentationNS: UInt64.max - 40_000_000))
        XCTAssertTrue(gate.accept(presentationNS: UInt64.max))
        XCTAssertFalse(gate.accept(presentationNS: UInt64.max))
        XCTAssertFalse(gate.accept(presentationNS: 0))
        gate.stop()
        XCTAssertFalse(gate.accept(presentationNS: 1))
        gate.start()
        XCTAssertTrue(gate.accept(presentationNS: 0))
        XCTAssertTrue(gate.accept(presentationNS: UInt64.max)) // Saturates without multiplication overflow.
        XCTAssertFalse(gate.accept(presentationNS: UInt64.max - 1))
        gate.stop(); gate.start()
        XCTAssertTrue(gate.accept(presentationNS: 1))
        XCTAssertTrue(gate.accept(presentationNS: 2))
        XCTAssertFalse(gate.accept(presentationNS: 3))
    }
}
