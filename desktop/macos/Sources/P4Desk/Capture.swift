import Foundation
import AppKit
import ScreenCaptureKit
import VideoToolbox
import CoreImage
import Metal
import ImageIO
import UniformTypeIdentifiers
import P4DeskCore

enum VideoProfile {
    static let framesPerSecond: Int32 = 60
    // Zero asks ScreenCaptureKit for the display's native cadence.
    static let capturePollFPS: Int32 = 0
    static let jpegQuality = 1.0
}

struct EncodedFrame {
    let data: Data
    let capturedNS: UInt64
    let encodeStartedNS: UInt64
    let encodedNS: UInt64
    let captureTimestampSource: CaptureTimestampSource
    let jpegQuality: Double
    let jpegSampling: JPEGSampling
    let jpegEncodingAttempts: Int
    let jpegPayloadFallback: Bool
    var captureUsesPresentationTimestamp: Bool { captureTimestampSource != .callback }
}

enum JPEGRotationError: Error {
    case unsupportedDegrees, invalidBuffer, poolCreation(CVReturn), poolAllocation(CVReturn), renderFailed

    var diagnosticReason: String {
        switch self {
        case .unsupportedDegrees: return "unsupported_rotation"
        case .invalidBuffer: return "rotation_input_format"
        case .poolCreation(let status): return "rotation_pool_create_\(status)"
        case .poolAllocation(let status): return "rotation_pool_allocate_\(status)"
        case .renderFailed: return "rotation_render_failed"
        }
    }
}

// Only the encoder queue uses this renderer. Its pool is a hard allocation cap,
// rather than just a minimum buffer count, including buffers still retained by VT.
final class JPEGRotationRenderer {
    static let maximumBuffers = 3
    private static let bounds = CGRect(x: 0, y: 0, width: 1024, height: 600)
    private let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
    private let context: CIContext
    private var pool: CVPixelBufferPool?
    private(set) var rotationDegrees = 0
    let backend: String

    init() {
        let options: [CIContextOption: Any] = [.cacheIntermediates: false]
        if let device = MTLCreateSystemDefaultDevice() {
            context = CIContext(mtlDevice: device, options: options)
            backend = "Core Image Metal"
        } else {
            context = CIContext(options: options)
            backend = "Core Image"
        }
    }
    func start(jpegRotationDegrees: Int) throws {
        stop()
        guard jpegRotationDegrees == 0 || jpegRotationDegrees == 180 else { throw JPEGRotationError.unsupportedDegrees }
        if jpegRotationDegrees == 180 {
            let attributes: [CFString: Any] = [
                kCVPixelBufferWidthKey: 1024, kCVPixelBufferHeightKey: 600,
                kCVPixelBufferPixelFormatTypeKey: kCVPixelFormatType_32BGRA,
                kCVPixelBufferIOSurfacePropertiesKey: [:], kCVPixelBufferMetalCompatibilityKey: true
            ]
            var newPool: CVPixelBufferPool?
            let status = CVPixelBufferPoolCreate(kCFAllocatorDefault,
                [kCVPixelBufferPoolMinimumBufferCountKey: 1] as CFDictionary,
                attributes as CFDictionary, &newPool)
            guard status == kCVReturnSuccess, let newPool else { throw JPEGRotationError.poolCreation(status) }
            pool = newPool
        }
        rotationDegrees = jpegRotationDegrees
    }
    func prepare(_ buffer: CVPixelBuffer) throws -> CVPixelBuffer {
        guard rotationDegrees == 180 else { return buffer }
        guard CVPixelBufferGetWidth(buffer) == 1024, CVPixelBufferGetHeight(buffer) == 600,
              CVPixelBufferGetPixelFormatType(buffer) == kCVPixelFormatType_32BGRA,
              let pool else { throw JPEGRotationError.invalidBuffer }
        return try autoreleasepool {
            var output: CVPixelBuffer?
            let status = CVPixelBufferPoolCreatePixelBufferWithAuxAttributes(kCFAllocatorDefault, pool,
                [kCVPixelBufferPoolAllocationThresholdKey: Self.maximumBuffers] as CFDictionary, &output)
            guard status == kCVReturnSuccess, let output else { throw JPEGRotationError.poolAllocation(status) }
            CVBufferSetAttachment(output, kCVImageBufferCGColorSpaceKey, colorSpace, .shouldPropagate)
            let image = CIImage(cvPixelBuffer: buffer, options: [.colorSpace: colorSpace]).oriented(.down)
            let destination = CIRenderDestination(pixelBuffer: output)
            destination.colorSpace = colorSpace
            destination.isDithered = false
            do {
                let task = try context.startTask(toRender: image, from: Self.bounds, to: destination, at: .zero)
                // A scheduled Metal render is not yet safe for VT or CPU access. Wait
                // on this bounded worker before handing over its completed pixels.
                _ = try task.waitUntilCompleted()
            } catch { throw JPEGRotationError.renderFailed }
            return output
        }
    }
    func imageIOJPEG(_ preparedBuffer: CVPixelBuffer, shouldContinue: () -> Bool = { true }) -> JPEGEncodingResult? {
        // All size retries receive the same completed, already rotated pixels.
        // Explicit sRGB avoids inheriting a device-dependent CGImage color space.
        let image = CIImage(cvPixelBuffer: preparedBuffer, options: [.colorSpace: colorSpace])
        guard let cgImage = context.createCGImage(image, from: Self.bounds, format: .RGBA8, colorSpace: colorSpace) else { return nil }
        return BoundedJPEGEncoder.encode { quality in
            guard shouldContinue() else { return nil }
            return autoreleasepool {
                let data = NSMutableData()
                guard let destination = CGImageDestinationCreateWithData(data, UTType.jpeg.identifier as CFString, 1, nil) else { return nil }
                let properties: [CFString: Any] = [kCGImageDestinationLossyCompressionQuality: quality,
                    kCGImagePropertyJFIFDictionary: [kCGImagePropertyJFIFIsProgressive: false]]
                CGImageDestinationAddImage(destination, cgImage, properties as CFDictionary)
                return CGImageDestinationFinalize(destination) ? data as Data : nil
            }
        }
    }
    func stop() {
        if let pool { CVPixelBufferPoolFlush(pool, .excessBuffers) }
        pool = nil; rotationDegrees = 0
    }
}

final class JPEGEncoder {
    private let queue = DispatchQueue(label: "com.p4desk.jpeg", qos: .userInitiated)
    private let queueKey = DispatchSpecificKey<Bool>()
    private let rotationRenderer = JPEGRotationRenderer()
    private struct InputFrame {
        let buffer: CVPixelBuffer
        let capturedNS: UInt64
        let timestampSource: CaptureTimestampSource
    }
    private let frames = LatestFrameQueue<InputFrame>()
    private var current: (work: LatestFrameWork<InputFrame>, startedNS: UInt64, preparedBuffer: CVPixelBuffer?)?
    private var diagnostics = JPEGEncoderDiagnostics()
    var onJPEG: ((EncodedFrame) -> Void)?
    var onError: (() -> Void)?
    var onBackend: ((String) -> Void)?
    var onDiagnostics: ((JPEGEncoderDiagnostics) -> Void)?
    var statistics: LatestFrameQueueStatistics { frames.statistics }
    init() { queue.setSpecific(key: queueKey, value: true) }

    @discardableResult
    func start(jpegRotationDegrees: Int = 0) -> Bool {
        frames.stop()
        return synchronized {
            current = nil; diagnostics = JPEGEncoderDiagnostics()
            do { try rotationRenderer.start(jpegRotationDegrees: jpegRotationDegrees) }
            catch {
                diagnostics.backend = "JPEG 无法启动"
                diagnostics.fallbackReason = (error as? JPEGRotationError)?.diagnosticReason ?? "rotation_initialization_failed"
                publishDiagnostics(); return false
            }
            diagnostics.actualRotationDegrees = rotationRenderer.rotationDegrees
            if rotationRenderer.rotationDegrees != 0 {
                diagnostics.rotationBackend = rotationRenderer.backend
                diagnostics.rotationPoolCapacity = JPEGRotationRenderer.maximumBuffers
            }
            // ImageIO's tested quality-1 output is baseline JFIF with full-range
            // BT.601 and 4:4:4. VT JPEG on this machine remains 4:2:0 and uses a
            // different RGB-to-YCbCr matrix even when its quality is set to 1.
            diagnostics.backend = "ImageIO JPEG"
            publishDiagnostics()
            frames.start()
            return true
        }
    }
    func submit(_ buffer: CVPixelBuffer, capturedNS: UInt64 = DispatchTime.now().uptimeNanoseconds,
                timestampSource: CaptureTimestampSource = .callback) {
        let frame = InputFrame(buffer: buffer, capturedNS: capturedNS, timestampSource: timestampSource)
        // Coalesce on the producer thread, including while ImageIO blocks the encoder queue.
        guard let work = frames.submit(frame) else { return }
        schedule(work)
    }
    private func schedule(_ work: LatestFrameWork<InputFrame>) {
        queue.async { [weak self] in self?.encode(work) }
    }
    private func encode(_ work: LatestFrameWork<InputFrame>) {
        guard frames.isCurrent(work.token) else { return }
        let frame = work.value
        current = (work, DispatchTime.now().uptimeNanoseconds, nil)
        let buffer: CVPixelBuffer
        do { buffer = try rotationRenderer.prepare(frame.buffer) }
        catch {
            diagnostics.lastPreparationError = (error as? JPEGRotationError)?.diagnosticReason ?? "rotation_preparation_failed"
            publishDiagnostics(); finish(nil, token: work.token); return
        }
        guard frames.isCurrent(work.token) else { return }
        current?.preparedBuffer = buffer
        let result = rotationRenderer.imageIOJPEG(buffer) { self.frames.isCurrent(work.token) }
        finish(result, token: work.token)
    }
    private func finish(_ result: JPEGEncodingResult?, token: LatestFrameToken) {
        guard frames.isCurrent(token), current?.work.token == token else { return }
        if let result, let current {
            let changed = diagnostics.actualQuality != result.quality || diagnostics.actualSampling != result.info.sampling
                || diagnostics.encodingAttempts != result.attempts || diagnostics.payloadFallback != result.payloadFallback
            diagnostics.actualQuality = result.quality
            diagnostics.actualSampling = result.info.sampling
            diagnostics.encodingAttempts = result.attempts
            diagnostics.payloadFallback = result.payloadFallback
            diagnostics.jpegHasJFIF = result.info.hasJFIF
            // Session diagnostics describe the latest selection. Payload bytes and
            // quality/sampling travel on each frame for the transmission report.
            if changed { publishDiagnostics() }
            onJPEG?(EncodedFrame(data: result.data, capturedNS: current.work.value.capturedNS, encodeStartedNS: current.startedNS,
                                encodedNS: DispatchTime.now().uptimeNanoseconds,
                                captureTimestampSource: current.work.value.timestampSource,
                                jpegQuality: result.quality, jpegSampling: result.info.sampling,
                                jpegEncodingAttempts: result.attempts, jpegPayloadFallback: result.payloadFallback))
        }
        else { onError?() }
        current = nil
        if let next = frames.complete(token) { schedule(next) }
    }
    private func publishDiagnostics() {
        onBackend?(diagnostics.displayName); onDiagnostics?(diagnostics)
    }
    private func synchronized<T>(_ action: () -> T) -> T {
        if DispatchQueue.getSpecific(key: queueKey) == true { return action() }
        else { return queue.sync(execute: action) }
    }
    func stop() {
        frames.stop()
        synchronized { frames.stop(); current = nil; rotationRenderer.stop() }
    }
    deinit {
        frames.stop()
        // Release renderer resources on their owner queue; do not block whichever
        // thread releases the last reference, including an encoder callback.
        let finalBuffer = current?.preparedBuffer, finalRenderer = rotationRenderer
        queue.async {
            finalRenderer.stop()
            withExtendedLifetime(finalBuffer) {}
        }
    }
    static func isBaselineJPEG(_ data: Data) -> Bool { JPEGHeader.inspect(data) != nil }

}

struct JPEGEncoderDiagnostics: Codable {
    var backend = "尚未启动"
    var requestedQuality = VideoProfile.jpegQuality
    var colorSpace = "sRGB"
    var maximumPayloadBytes = BoundedJPEGEncoder.maximumBytes
    var qualityFallbackOrder = BoundedJPEGEncoder.qualities
    var actualQuality: Double?
    var actualSampling: JPEGSampling?
    var encodingAttempts: Int?
    var payloadFallback = false
    var jpegHasJFIF: Bool?
    var targetFrameRate = VideoProfile.framesPerSecond
    var capturePollFrameRate = VideoProfile.capturePollFPS
    var actualRotationDegrees = 0
    var rotationBackend: String?
    var rotationPoolCapacity = 0
    var lastPreparationError: String?
    var reportedQuality: Double?
    var videoToolboxHardwareAccelerated: Bool?
    var createStatus: Int32?
    var prepareStatus: Int32?
    var propertyStatus: [String: Int32] = [:]
    var hardwareQueryStatus: Int32?
    var qualityQueryStatus: Int32?
    var lastEncodeStatus: Int32?
    var fallbackReason: String?
    var displayName: String {
        guard backend == "VideoToolbox JPEG" else { return backend }
        return "\(backend) · \(videoToolboxHardwareAccelerated.map { $0 ? "硬件" : "软件" } ?? "硬件状态未知")"
    }
}

final class DisplayCapture: NSObject, SCStreamOutput, SCStreamDelegate {
    private var stream: SCStream?
    private var activity: NSObjectProtocol?
    private let queue = DispatchQueue(label: "com.p4desk.capture", qos: .userInitiated)
    private let encoder = JPEGEncoder()
    var onJPEG: ((EncodedFrame) -> Void)?
    var onFailure: (() -> Void)?
    var onBackend: ((String) -> Void)?
    var onDiagnostics: ((JPEGEncoderDiagnostics) -> Void)?
    var encoderStatistics: LatestFrameQueueStatistics { encoder.statistics }
    private var active = false
    private var frameGate = CaptureRateGate(framesPerSecond: UInt64(VideoProfile.framesPerSecond))
    private var lifecycle = UUID()
    @MainActor
    func start(displayID: CGDirectDisplayID, jpegRotationDegrees: Int = 0) async throws {
        guard jpegRotationDegrees == 0 || jpegRotationDegrees == 180 else { throw DeskError.encodingFailed }
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
        // The virtual display uses a 60 Hz mode. Native cadence avoids a
        // second interval scheduler; the bounded gate still caps encoding at 60.
        configuration.minimumFrameInterval = VideoProfile.capturePollFPS == 0
            ? .zero : CMTime(value: 1, timescale: VideoProfile.capturePollFPS)
        configuration.queueDepth = 3
        configuration.showsCursor = true
        configuration.capturesAudio = false
        configuration.colorSpaceName = CGColorSpace.sRGB
        encoder.onJPEG = { [weak self] data in self?.onJPEG?(data) }
        encoder.onError = { [weak self] in self?.onFailure?() }
        encoder.onBackend = { [weak self] name in self?.onBackend?(name) }
        encoder.onDiagnostics = { [weak self] diagnostics in self?.onDiagnostics?(diagnostics) }
        guard encoder.start(jpegRotationDegrees: jpegRotationDegrees) else { throw DeskError.encodingFailed }
        let newStream = SCStream(filter: filter, configuration: configuration, delegate: self)
        try newStream.addStreamOutput(self, type: .screen, sampleHandlerQueue: queue)
        stream = newStream
        queue.sync { frameGate.start(); active = true }
        // Streaming remains user work after the editor closes. Keep App Nap
        // from throttling it while allowing the normal system sleep lifecycle.
        activity = ProcessInfo.processInfo.beginActivity(
            options: [.userInitiatedAllowingIdleSystemSleep, .latencyCritical],
            reason: "P4 Desk USB display capture and transmission")
        do {
            try await newStream.startCapture()
            guard lifecycle == operation else {
                try? await newStream.stopCapture(); encoder.stop(); throw CancellationError()
            }
        }
        catch {
            queue.sync { active = false; frameGate.stop() }
            stream = nil; encoder.stop(); finishActivity(); throw error
        }
    }
    @MainActor
    func stop() async {
        lifecycle = UUID()
        queue.sync { active = false; frameGate.stop() }
        if let stream { try? await stream.stopCapture() }
        stream = nil; encoder.stop(); finishActivity()
    }
    private func finishActivity() {
        if let activity { ProcessInfo.processInfo.endActivity(activity); self.activity = nil }
    }
    deinit { if let activity { ProcessInfo.processInfo.endActivity(activity) } }
    func stream(_ stream: SCStream, didOutputSampleBuffer sample: CMSampleBuffer, of type: SCStreamOutputType) {
        guard active, type == .screen, sample.isValid,
              let attachments = CMSampleBufferGetSampleAttachmentsArray(sample, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
              let status = attachments.first?[.status] as? Int, status == SCFrameStatus.complete.rawValue,
              let buffer = CMSampleBufferGetImageBuffer(sample) else { return }
        let received = DispatchTime.now().uptimeNanoseconds
        let timestamp = CaptureTimestampResolver.resolve(
            displayTime: ControlNumber.unsigned(attachments.first?[.displayTime]),
            samplePTS: CMSampleBufferGetPresentationTimeStamp(sample), receivedNS: received)
        guard frameGate.accept(presentationNS: timestamp.capturedNS) else { return }
        encoder.submit(buffer, capturedNS: timestamp.capturedNS, timestampSource: timestamp.source)
    }
    func stream(_ stream: SCStream, didStopWithError error: Error) { onFailure?() }
}
