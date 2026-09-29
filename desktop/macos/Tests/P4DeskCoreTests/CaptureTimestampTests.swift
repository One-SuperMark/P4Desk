import XCTest
import CoreMedia
import Darwin
@testable import P4DeskCore

final class CaptureTimestampTests: XCTestCase {
    func testDisplayTimeUsesSDKTimebaseAndHasPriorityOverPTS() {
        let referenceNS: Int64 = 123_456_789_123
        let rawTicks = CMClockConvertHostTimeToSystemUnits(CMTime(value: referenceNS, timescale: 1_000_000_000))
        let result = CaptureTimestampResolver.resolve(displayTime: rawTicks,
            samplePTS: CMTime(value: referenceNS - 100_000_000, timescale: 1_000_000_000),
            receivedNS: UInt64(referenceNS) + 1_000_000)
        var timebase = mach_timebase_info_data_t()
        XCTAssertEqual(mach_timebase_info(&timebase), KERN_SUCCESS)
        let denominator = max(UInt64(timebase.denom), 1)
        let tickNS = (UInt64(timebase.numer) + denominator - 1) / denominator
        let expected = UInt64(referenceNS)
        let difference = result.capturedNS >= expected ? result.capturedNS - expected : expected - result.capturedNS
        XCTAssertLessThanOrEqual(difference, max(2, tickNS + 2))
        XCTAssertEqual(result.source, .displayTime)
        XCTAssertTrue(result.usesTimestamp)
    }

    func testRejectedDisplayTimeFallsBackToValidPTS() {
        let received: UInt64 = 10_000_000_000
        let pts = CMTime(value: 9_900_000_000, timescale: 1_000_000_000)
        let future = CMClockConvertHostTimeToSystemUnits(CMTime(value: 11_000_000_000, timescale: 1_000_000_000))
        let old = CMClockConvertHostTimeToSystemUnits(CMTime(value: 4_000_000_000, timescale: 1_000_000_000))
        for raw in [nil, 0, future, old, UInt64.max] as [UInt64?] {
            let result = CaptureTimestampResolver.resolve(displayTime: raw, samplePTS: pts, receivedNS: received)
            XCTAssertEqual(result.capturedNS, 9_900_000_000)
            XCTAssertEqual(result.source, .samplePTS)
            XCTAssertTrue(result.usesTimestamp)
        }
    }

    func testInvalidPTSIsExplicitCallbackWithNoTimestampFlag() {
        let received: UInt64 = 10_000_000_000
        let invalid: [CMTime] = [.invalid, .indefinite, .positiveInfinity, .negativeInfinity, .zero,
            CMTime(value: -1, timescale: 1_000_000_000),
            CMTime(value: 1, timescale: 0, flags: .valid, epoch: 0),
            CMTime(value: 1, timescale: -1, flags: .valid, epoch: 0)]
        for pts in invalid {
            let result = CaptureTimestampResolver.resolve(displayTime: nil, samplePTS: pts, receivedNS: received)
            XCTAssertEqual(result.capturedNS, received)
            XCTAssertEqual(result.source, .callback)
            XCTAssertFalse(result.usesTimestamp)
        }
    }

    func testFutureAndAgeBoundaryAreCheckedAfterNanosecondConversion() {
        let received: UInt64 = 10_000_000_000
        for value in [10_000_000_001, 5_000_000_000, 4_999_999_999] as [Int64] {
            let result = CaptureTimestampResolver.resolve(displayTime: nil,
                samplePTS: CMTime(value: value, timescale: 1_000_000_000), receivedNS: received)
            XCTAssertEqual(result.source, .callback)
            XCTAssertFalse(result.usesTimestamp)
            XCTAssertEqual(result.capturedNS, received)
        }
        for value in [10_000_000_000, 5_000_000_001] as [Int64] {
            let result = CaptureTimestampResolver.resolve(displayTime: nil,
                samplePTS: CMTime(value: value, timescale: 1_000_000_000), receivedNS: received)
            XCTAssertEqual(result.source, .samplePTS)
            XCTAssertTrue(result.usesTimestamp)
            XCTAssertEqual(result.capturedNS, UInt64(value))
        }
        let scaled = CaptureTimestampResolver.resolve(displayTime: nil,
            samplePTS: CMTime(value: 9900, timescale: 1000), receivedNS: received)
        XCTAssertEqual(scaled.capturedNS, 9_900_000_000)
        XCTAssertEqual(scaled.source, .samplePTS)
    }

    func testForeignEpochAndConversionOverflowCannotBecomeHostTimestamp() {
        let received: UInt64 = 10_000_000_000
        let foreign = CaptureTimestampResolver.resolve(displayTime: nil,
            samplePTS: CMTime(value: 9_900_000_000, timescale: 1_000_000_000, flags: .valid, epoch: 1),
            receivedNS: received)
        XCTAssertEqual(foreign.source, .callback)
        XCTAssertFalse(foreign.usesTimestamp)
        let overflow = CaptureTimestampResolver.resolve(displayTime: UInt64.max,
            samplePTS: CMTime(value: Int64.max, timescale: 1), receivedNS: UInt64.max)
        XCTAssertEqual(overflow.source, .callback)
        XCTAssertEqual(overflow.capturedNS, UInt64.max)
        XCTAssertFalse(overflow.usesTimestamp)
    }
}
