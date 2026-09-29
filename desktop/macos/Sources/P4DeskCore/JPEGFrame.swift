import Foundation

public enum JPEGSampling: String, Codable, Equatable {
    case yuv444 = "4:4:4"
    case yuv422 = "4:2:2"
    case yuv420 = "4:2:0"
    case grayscale = "4:0:0"
}

public struct JPEGFrameInfo: Equatable {
    public let width: Int
    public let height: Int
    public let sampling: JPEGSampling
    public let hasJFIF: Bool
}

/// Inspect the actual baseline JPEG bytes, rather than inferring sampling from
/// a quality setting or pixel-buffer format. The device validates entropy data.
public enum JPEGHeader {
    public static func inspect(_ data: Data, width: Int = 1024, height: Int = 600) -> JPEGFrameInfo? {
        data.withUnsafeBytes { raw -> JPEGFrameInfo? in
            let bytes = raw.bindMemory(to: UInt8.self)
            guard bytes.count >= 4, bytes[0] == 0xff, bytes[1] == 0xd8,
                  bytes[bytes.count - 2] == 0xff, bytes[bytes.count - 1] == 0xd9 else { return nil }
            var offset = 2, jfif = false
            var frame: JPEGFrameInfo?
            while offset + 3 < bytes.count {
                guard bytes[offset] == 0xff else { return nil }
                while offset < bytes.count, bytes[offset] == 0xff { offset += 1 }
                guard offset + 2 < bytes.count else { return nil }
                let marker = bytes[offset]; offset += 1
                guard marker != 0x00, marker != 0xd8, marker != 0xd9 else { return nil }
                let length = Int(bytes[offset]) << 8 | Int(bytes[offset + 1])
                guard length >= 2, length <= bytes.count - offset else { return nil }
                if marker == 0xda {
                    guard let frame else { return nil }
                    return JPEGFrameInfo(width: frame.width, height: frame.height, sampling: frame.sampling, hasJFIF: jfif)
                }
                if marker == 0xe0, length >= 7 {
                    jfif = jfif || [UInt8](bytes[(offset + 2)..<(offset + 7)]) == [0x4a, 0x46, 0x49, 0x46, 0]
                }
                if marker == 0xc0 {
                    guard frame == nil, length >= 11, bytes[offset + 2] == 8 else { return nil }
                    let actualHeight = Int(bytes[offset + 3]) << 8 | Int(bytes[offset + 4])
                    let actualWidth = Int(bytes[offset + 5]) << 8 | Int(bytes[offset + 6])
                    let components = Int(bytes[offset + 7])
                    guard actualWidth == width, actualHeight == height,
                          length == 8 + 3 * components, components == 1 || components == 3 else { return nil }
                    var factors: [(Int, Int)] = []
                    var ids = Set<UInt8>()
                    for index in 0..<components {
                        let base = offset + 8 + index * 3
                        guard ids.insert(bytes[base]).inserted else { return nil }
                        let factor = bytes[base + 1], h = Int(factor >> 4), v = Int(factor & 15)
                        guard (1...4).contains(h), (1...4).contains(v), bytes[base + 2] <= 3 else { return nil }
                        factors.append((h, v))
                    }
                    let sampling: JPEGSampling
                    if components == 1 {
                        guard bytes[offset + 8] == 1, factors[0].0 == 1, factors[0].1 == 1 else { return nil }
                        sampling = .grayscale
                    }
                    else {
                        let y = factors[0], cb = factors[1], cr = factors[2]
                        // P4's hardware parser expects canonical JFIF component
                        // order and 1x1 chroma factors, not equivalent scaled MCUs.
                        guard bytes[offset + 8] == 1, bytes[offset + 11] == 2, bytes[offset + 14] == 3,
                              cb.0 == 1, cb.1 == 1, cr.0 == 1, cr.1 == 1 else { return nil }
                        if y.0 == 1, y.1 == 1 { sampling = .yuv444 }
                        else if y.0 == 2, y.1 == 1 { sampling = .yuv422 }
                        else if y.0 == 2, y.1 == 2 { sampling = .yuv420 }
                        else { return nil }
                    }
                    frame = JPEGFrameInfo(width: actualWidth, height: actualHeight, sampling: sampling, hasJFIF: jfif)
                } else if (0xc1...0xcf).contains(marker), ![0xc4, 0xc8, 0xcc].contains(marker) {
                    return nil
                }
                offset += length
            }
            return nil
        }
    }
}

public struct JPEGEncodingResult {
    public let data: Data
    public let info: JPEGFrameInfo
    /// The ImageIO quality used for this accepted frame, not a session-wide value.
    public let quality: Double
    public let attempts: Int
    public let payloadFallback: Bool
}

/// All retries use the same prepared pixels. An oversized JPEG never reaches USB.
public enum BoundedJPEGEncoder {
    public static let maximumBytes = 1_048_576
    // On the tested macOS 27 encoder, 0.995 is identical to 1.0, including size.
    // 0.99 reduces high-entropy payloads; every attempt's SOF is still inspected.
    public static let qualities: [Double] = [1.0, 0.99, 0.95, 0.78, 0.60, 0.40]

    public static func encode(_ makeJPEG: (Double) -> Data?) -> JPEGEncodingResult? {
        var oversized = false
        for (index, quality) in qualities.enumerated() {
            guard let data = makeJPEG(quality) else { return nil }
            if data.count > maximumBytes { oversized = true; continue }
            // JFIF declares the full-range JPEG/BT.601 convention expected by
            // the device. An unmarked encoder-specific matrix is not accepted.
            guard let info = JPEGHeader.inspect(data), info.hasJFIF else { return nil }
            return JPEGEncodingResult(data: data, info: info, quality: quality,
                                      attempts: index + 1, payloadFallback: oversized)
        }
        return nil
    }
}
