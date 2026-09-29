import XCTest
@testable import P4DeskCore

final class JPEGFrameTests: XCTestCase {
    private func jpeg(yFactor: UInt8 = 0x11, width: UInt16 = 1024, height: UInt16 = 600,
                      sof: UInt8 = 0xc0, precision: UInt8 = 8, jfif: Bool = true) -> Data {
        var bytes: [UInt8] = [0xff, 0xd8]
        if jfif { bytes += [0xff, 0xe0, 0, 7, 0x4a, 0x46, 0x49, 0x46, 0] }
        bytes += [0xff, sof, 0, 17, precision, UInt8(height >> 8), UInt8(height & 255),
                  UInt8(width >> 8), UInt8(width & 255), 3,
                  1, yFactor, 0, 2, 0x11, 1, 3, 0x11, 1,
                  0xff, 0xda, 0, 12, 3, 1, 0, 2, 0x11, 3, 0x11, 0, 63, 0,
                  0x00, 0xff, 0xd9]
        return Data(bytes)
    }

    func testReadsActual444422420SamplingAndJFIF() {
        for (factor, sampling) in [(UInt8(0x11), JPEGSampling.yuv444), (0x21, .yuv422), (0x22, .yuv420)] {
            let info = JPEGHeader.inspect(jpeg(yFactor: factor))
            XCTAssertEqual(info?.width, 1024); XCTAssertEqual(info?.height, 600)
            XCTAssertEqual(info?.sampling, sampling); XCTAssertEqual(info?.hasJFIF, true)
        }
        XCTAssertEqual(JPEGHeader.inspect(jpeg(jfif: false))?.hasJFIF, false)
    }

    func testRejectsWrongSizeProgressivePrecisionAndUnsupportedSampling() {
        XCTAssertNil(JPEGHeader.inspect(jpeg(width: 1023)))
        XCTAssertNil(JPEGHeader.inspect(jpeg(height: 608)))
        XCTAssertNil(JPEGHeader.inspect(jpeg(sof: 0xc2)))
        XCTAssertNil(JPEGHeader.inspect(jpeg(precision: 12)))
        XCTAssertNil(JPEGHeader.inspect(jpeg(yFactor: 0x12)))
        XCTAssertNil(JPEGHeader.inspect(jpeg(yFactor: 0)))
        var scaledFactors = jpeg(yFactor: 0x22)
        scaledFactors[25] = 0x22; scaledFactors[28] = 0x22
        XCTAssertNil(JPEGHeader.inspect(scaledFactors))
        var rgbComponentIDs = jpeg()
        rgbComponentIDs[21] = 0x52; rgbComponentIDs[24] = 0x47; rgbComponentIDs[27] = 0x42
        XCTAssertNil(JPEGHeader.inspect(rgbComponentIDs))
    }

    func testMalformedLengthsAndTruncationAreRejected() {
        let original = jpeg()
        for count in 0..<original.count { XCTAssertNil(JPEGHeader.inspect(original.prefix(count))) }
        var invalid = original
        invalid[4] = 0xff; invalid[5] = 0xff
        XCTAssertNil(JPEGHeader.inspect(invalid))
        var missingScan = original
        missingScan.removeSubrange(30..<44)
        XCTAssertNil(JPEGHeader.inspect(missingScan))
    }

    func testFirstLegalFramePreservesFullQualityAndStopsEncoding() {
        let original = jpeg(); var requested: [Double] = []
        let result = BoundedJPEGEncoder.encode { requested.append($0); return original }
        XCTAssertEqual(requested, [1])
        XCTAssertEqual(result?.quality, 1); XCTAssertEqual(result?.attempts, 1)
        XCTAssertEqual(result?.info.sampling, .yuv444); XCTAssertEqual(result?.payloadFallback, false)
        XCTAssertEqual(result?.data, original)
    }

    func testOversizeReencodesAtMeasuredNextQualityAndRecordsSampling() {
        let oversized = Data(repeating: 0, count: BoundedJPEGEncoder.maximumBytes + 1)
        let legal = jpeg(yFactor: 0x22); var requested: [Double] = []
        let result = BoundedJPEGEncoder.encode { quality in
            requested.append(quality); return quality == 1 ? oversized : legal
        }
        XCTAssertEqual(requested, [1, 0.99]); XCTAssertEqual(result?.quality, 0.99)
        XCTAssertEqual(result?.attempts, 2); XCTAssertEqual(result?.payloadFallback, true)
        XCTAssertEqual(result?.info.sampling, .yuv420); XCTAssertEqual(result?.data, legal)
        XCTAssertLessThanOrEqual(result!.data.count, BoundedJPEGEncoder.maximumBytes)
    }

    func testOversizeRetriesAreBoundedAndInvalidOrFailedOutputIsNotEmitted() {
        var requested: [Double] = []
        let oversized = Data(repeating: 0, count: BoundedJPEGEncoder.maximumBytes + 1)
        XCTAssertNil(BoundedJPEGEncoder.encode { requested.append($0); return oversized })
        XCTAssertEqual(requested, BoundedJPEGEncoder.qualities)
        XCTAssertEqual(requested.count, 6)
        XCTAssertNil(BoundedJPEGEncoder.encode { _ in nil })
        XCTAssertNil(BoundedJPEGEncoder.encode { _ in self.jpeg(sof: 0xc2) })
    }

    func testUnmarkedMatrixIsRejectedEvenWhenBaselineAndSizeAreValid() {
        let unmarked = jpeg(jfif: false)
        XCTAssertNotNil(JPEGHeader.inspect(unmarked))
        XCTAssertNil(BoundedJPEGEncoder.encode { _ in unmarked })
    }

    func testFrameDiagnosticsPreserveActualFallbackAndDecodeOlderMetadata() throws {
        let timing = FrameTiming(sequence: 4, capturedNS: 1, encodeStartedNS: 2, encodedNS: 3,
            enqueuedNS: 4, captureUsesPresentationTimestamp: true, jpegBytes: 816_546,
            jpegQuality: 0.99, jpegSampling: .yuv420, jpegEncodingAttempts: 2, jpegPayloadFallback: true)
        let encoded = try JSONEncoder().encode(timing)
        XCTAssertEqual(try JSONDecoder().decode(FrameTiming.self, from: encoded), timing)
        var legacy = try XCTUnwrap(JSONSerialization.jsonObject(with: encoded) as? [String: Any])
        for key in ["jpegQuality", "jpegSampling", "jpegEncodingAttempts", "jpegPayloadFallback"] { legacy.removeValue(forKey: key) }
        let old = try JSONDecoder().decode(FrameTiming.self, from: JSONSerialization.data(withJSONObject: legacy))
        XCTAssertNil(old.jpegQuality); XCTAssertNil(old.jpegSampling)
        XCTAssertNil(old.jpegEncodingAttempts); XCTAssertNil(old.jpegPayloadFallback)
    }
}
