import Foundation
import AppKit
import ScreenCaptureKit
import VideoToolbox
import CoreImage
import ImageIO
import UniformTypeIdentifiers

struct EncodedFrame {
    let data: Data
    let capturedNS: UInt64
    let encodeStartedNS: UInt64
    let encodedNS: UInt64
    let captureUsesPresentationTimestamp: Bool
}

final class JPEGEncoder {
    private let queue = DispatchQueue(label: "com.p4desk.jpeg", qos: .userInitiated)
    private let context = CIContext(options: [.cacheIntermediates: false])
    private var session: VTCompressionSession?
    private var encoding = false
    private var stopped = false
    private struct InputFrame {
        let buffer: CVPixelBuffer
        let capturedNS: UInt64
        let presentationTimestamp: Bool
    }
    private var latest: InputFrame?
    private var current: InputFrame?
    private var encodeStartedNS: UInt64 = 0
    var onJPEG: ((EncodedFrame) -> Void)?
    var onError: (() -> Void)?
    var onBackend: ((String) -> Void)?
    init() {}

    func start() {
        queue.sync {
            stopped = false
            let specification: [CFString: Any] = [kVTVideoEncoderSpecification_EnableHardwareAcceleratedVideoEncoder: true]
            let result = VTCompressionSessionCreate(allocator: kCFAllocatorDefault, width: 1024, height: 600,
                codecType: kCMVideoCodecType_JPEG, encoderSpecification: specification as CFDictionary,
                imageBufferAttributes: nil, compressedDataAllocator: nil, outputCallback: { ref, _, status, _, sample in
                    guard let ref else { return }
                    let encoder = Unmanaged<JPEGEncoder>.fromOpaque(ref).takeUnretainedValue()
                    var data: Data?
                    if status == noErr, let sample, let block = CMSampleBufferGetDataBuffer(sample) {
                        let length = CMBlockBufferGetDataLength(block)
                        if length > 0, length <= 1_048_576 {
                            var copy = Data(count: length)
                            let result = copy.withUnsafeMutableBytes { raw in
                                CMBlockBufferCopyDataBytes(block, atOffset: 0, dataLength: length, destination: raw.baseAddress!)
                            }
                            if result == noErr { data = copy }
                        }
                    }
                    let encoded = data
                    encoder.queue.async { encoder.finishVT(encoded) }
                }, refcon: Unmanaged.passUnretained(self).toOpaque(), compressionSessionOut: &session)
            if result == noErr, let session {
                VTSessionSetProperty(session, key: kVTCompressionPropertyKey_RealTime, value: kCFBooleanTrue)
                VTSessionSetProperty(session, key: kVTCompressionPropertyKey_Quality, value: NSNumber(value: 0.78))
                VTSessionSetProperty(session, key: kVTCompressionPropertyKey_ExpectedFrameRate, value: NSNumber(value: 30))
                VTSessionSetProperty(session, key: kVTCompressionPropertyKey_AllowFrameReordering, value: kCFBooleanFalse)
                VTSessionSetProperty(session, key: kVTCompressionPropertyKey_MaxFrameDelayCount, value: NSNumber(value: 0))
                VTCompressionSessionPrepareToEncodeFrames(session)
                onBackend?("VideoToolbox JPEG")
            } else { session = nil; onBackend?("ImageIO JPEG") }
        }
    }
    func submit(_ buffer: CVPixelBuffer, capturedNS: UInt64 = DispatchTime.now().uptimeNanoseconds, presentationTimestamp: Bool = false) {
        let frame = InputFrame(buffer: buffer, capturedNS: capturedNS, presentationTimestamp: presentationTimestamp)
        queue.async { [self] in
            guard !stopped else { return }
            if encoding { latest = frame; return }
            encode(frame)
        }
    }
    private func encode(_ frame: InputFrame) {
        let buffer = frame.buffer
        current = frame; encoding = true; encodeStartedNS = DispatchTime.now().uptimeNanoseconds
        if let session {
            let result = VTCompressionSessionEncodeFrame(session, imageBuffer: buffer,
                presentationTimeStamp: CMTime(value: Int64(DispatchTime.now().uptimeNanoseconds), timescale: 1_000_000_000),
                duration: CMTime(value: 1, timescale: 30), frameProperties: nil, sourceFrameRefcon: nil, infoFlagsOut: nil)
            if result == noErr { return }
            VTCompressionSessionInvalidate(session); self.session = nil; onBackend?("ImageIO JPEG")
        }
        finish(imageIO(buffer))
    }
    private func finishVT(_ data: Data?) {
        guard !stopped, encoding else { return }
        if let data, Self.isBaselineJPEG(data) { finish(data); return }
        // JPEG hardware availability is runtime-dependent. Unsupported output switches once to ImageIO.
        if let session { VTCompressionSessionInvalidate(session); self.session = nil }
        onBackend?("ImageIO JPEG")
        finish(current.flatMap { imageIO($0.buffer) })
    }
    private func finish(_ data: Data?) {
        guard !stopped else { return }
        if let data, data.count <= 1_048_576, Self.isBaselineJPEG(data), let current {
            onJPEG?(EncodedFrame(data: data, capturedNS: current.capturedNS, encodeStartedNS: encodeStartedNS,
                                encodedNS: DispatchTime.now().uptimeNanoseconds,
                                captureUsesPresentationTimestamp: current.presentationTimestamp))
        }
        else { onError?() }
        encoding = false; current = nil
        if let next = latest { latest = nil; encode(next) }
    }
    private func imageIO(_ buffer: CVPixelBuffer) -> Data? {
        let image = CIImage(cvPixelBuffer: buffer)
        guard let cgImage = context.createCGImage(image, from: CGRect(x: 0, y: 0, width: 1024, height: 600)) else { return nil }
        let data = NSMutableData()
        guard let destination = CGImageDestinationCreateWithData(data, UTType.jpeg.identifier as CFString, 1, nil) else { return nil }
        let properties: [CFString: Any] = [kCGImageDestinationLossyCompressionQuality: 0.78,
                                           kCGImagePropertyJFIFDictionary: [kCGImagePropertyJFIFIsProgressive: false]]
        CGImageDestinationAddImage(destination, cgImage, properties as CFDictionary)
        return CGImageDestinationFinalize(destination) ? data as Data : nil
    }
    func stop() {
        queue.sync {
            stopped = true; latest = nil; current = nil; encoding = false
            if let session { VTCompressionSessionInvalidate(session); self.session = nil }
        }
    }
    static func isBaselineJPEG(_ data: Data) -> Bool {
        let bytes = [UInt8](data)
        guard bytes.count >= 4, bytes[0] == 0xff, bytes[1] == 0xd8,
              bytes[bytes.count - 2] == 0xff, bytes.last == 0xd9 else { return false }
        var offset = 2
        while offset + 3 < bytes.count {
            guard bytes[offset] == 0xff else { return false }
            while offset < bytes.count, bytes[offset] == 0xff { offset += 1 }
            guard offset + 2 < bytes.count else { return false }
            let marker = bytes[offset]; offset += 1
            if marker == 0xda { return false } // SOF0 must precede scan data.
            if marker == 0xd9 { return false }
            let length = Int(bytes[offset]) << 8 | Int(bytes[offset + 1])
            guard length >= 2, offset + length <= bytes.count else { return false }
            if marker == 0xc0 {
                guard length >= 8, bytes[offset + 2] == 8 else { return false }
                let height = Int(bytes[offset + 3]) << 8 | Int(bytes[offset + 4])
                let width = Int(bytes[offset + 5]) << 8 | Int(bytes[offset + 6])
                return width == 1024 && height == 600
            }
            if (0xc1...0xcf).contains(marker), ![0xc4, 0xc8, 0xcc].contains(marker) { return false }
            offset += length
        }
        return false
    }
}

final class DisplayCapture: NSObject, SCStreamOutput, SCStreamDelegate {
    private var stream: SCStream?
    private let queue = DispatchQueue(label: "com.p4desk.capture", qos: .userInitiated)
    private let encoder = JPEGEncoder()
    var onJPEG: ((EncodedFrame) -> Void)?
    var onFailure: (() -> Void)?
    var onBackend: ((String) -> Void)?
    private var active = false
    private var lifecycle = UUID()
    @MainActor
    func start(displayID: CGDirectDisplayID) async throws {
        let operation = UUID(); lifecycle = operation
        var ids = [CGDirectDisplayID](repeating: 0, count: 32), count: UInt32 = 0
        var registered = false
        for _ in 0..<40 {
            guard lifecycle == operation else { throw CancellationError() }
            if CGGetActiveDisplayList(32, &ids, &count) == .success,
               ids.prefix(Int(count)).contains(displayID), CGDisplayPixelsWide(displayID) == 1024,
               CGDisplayPixelsHigh(displayID) == 600 { registered = true; break }
            try await Task.sleep(nanoseconds: 125_000_000)
        }
        guard registered else { throw DeskError.displayUnavailable }
        var display: SCDisplay?
        for _ in 0..<10 {
            guard lifecycle == operation else { throw CancellationError() }
            let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: false)
            guard lifecycle == operation else { throw CancellationError() }
            display = content.displays.first { $0.displayID == displayID }
            if display != nil { break }
            try await Task.sleep(nanoseconds: 150_000_000)
        }
        guard let display else { throw DeskError.captureUnavailable }
        let filter = SCContentFilter(display: display, excludingWindows: [])
        let configuration = SCStreamConfiguration()
        configuration.width = 1024; configuration.height = 600
        configuration.pixelFormat = kCVPixelFormatType_32BGRA
        configuration.minimumFrameInterval = CMTime(value: 1, timescale: 30)
        configuration.queueDepth = 3
        configuration.showsCursor = true
        configuration.capturesAudio = false
        configuration.colorSpaceName = CGColorSpace.sRGB
        encoder.onJPEG = { [weak self] data in self?.onJPEG?(data) }
        encoder.onError = { [weak self] in self?.onFailure?() }
        encoder.onBackend = { [weak self] name in self?.onBackend?(name) }
        encoder.start()
        let newStream = SCStream(filter: filter, configuration: configuration, delegate: self)
        try newStream.addStreamOutput(self, type: .screen, sampleHandlerQueue: queue)
        stream = newStream
        queue.sync { active = true }
        do {
            try await newStream.startCapture()
            guard lifecycle == operation else {
                try? await newStream.stopCapture(); encoder.stop(); throw CancellationError()
            }
        }
        catch { queue.sync { active = false }; stream = nil; encoder.stop(); throw error }
    }
    @MainActor
    func stop() async {
        lifecycle = UUID()
        queue.sync { active = false }
        if let stream { try? await stream.stopCapture() }
        stream = nil; encoder.stop()
    }
    func stream(_ stream: SCStream, didOutputSampleBuffer sample: CMSampleBuffer, of type: SCStreamOutputType) {
        guard active, type == .screen, sample.isValid,
              let attachments = CMSampleBufferGetSampleAttachmentsArray(sample, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
              let status = attachments.first?[.status] as? Int, status == SCFrameStatus.complete.rawValue,
              let buffer = CMSampleBufferGetImageBuffer(sample) else { return }
        let received = DispatchTime.now().uptimeNanoseconds
        let time = CMTimeConvertScale(CMSampleBufferGetPresentationTimeStamp(sample), timescale: 1_000_000_000, method: .default)
        let stamp = time.isNumeric && time.value > 0 ? UInt64(time.value) : received
        let hasTimestamp = stamp <= received && received - stamp < 5_000_000_000
        encoder.submit(buffer, capturedNS: hasTimestamp ? stamp : received, presentationTimestamp: hasTimestamp)
    }
    func stream(_ stream: SCStream, didStopWithError error: Error) { onFailure?() }
}
