import Foundation
import P4DeskNative
import P4DeskCore

final class USBSession {
    enum Event {
        case connected(speedMbps: UInt64), disconnected, bytes(Data)
        case sent(UInt32, atNanoseconds: UInt64, transferMicroseconds: UInt64?)
        case error(String)
    }
    var onEvent: ((Event) -> Void)?
    private var handle: UnsafeMutableRawPointer?
    func start() {
        guard handle == nil else { return }
        handle = P4USBCreate({ context, event, token, bytes, length, reason in
            guard let context else { return }
            let session = Unmanaged<USBSession>.fromOpaque(context).takeUnretainedValue()
            let value: Event
            switch event {
            case UInt32(P4USB_CONNECTED): value = .connected(speedMbps: UInt64(token))
            case UInt32(P4USB_DISCONNECTED): value = .disconnected
            case UInt32(P4USB_BYTES):
                guard let bytes, length <= 65_536 else { return }
                value = .bytes(Data(bytes: bytes, count: length))
            case UInt32(P4USB_SENT):
                let completedNS = DispatchTime.now().uptimeNanoseconds
                var transferUS: UInt64?
                if let bytes, length == 8 {
                    var decoded: UInt64 = 0
                    for index in 0..<8 { decoded |= UInt64(bytes[index]) << (index * 8) }
                    transferUS = decoded
                }
                value = .sent(token, atNanoseconds: completedNS, transferMicroseconds: transferUS)
            default: value = .error(reason.map { String(cString: $0) } ?? "USB 操作失败")
            }
            DispatchQueue.main.async { [weak session] in session?.onEvent?(value) }
        }, Unmanaged.passUnretained(self).toOpaque(), 0x303a, 0x4044)
    }
    func send(_ packet: Packet, token: UInt32? = nil) throws {
        guard let handle else { throw DeskError.usbDisconnected }
        let data = try packet.encoded()
        let accepted = data.withUnsafeBytes { bytes in
            P4USBEnqueue(handle, bytes.bindMemory(to: UInt8.self).baseAddress, data.count,
                         packet.kind == .jpeg, token ?? UInt32(packet.sequence))
        }
        if !accepted { throw DeskError.usbQueueUnavailable }
    }
    func clearVideo() { if let handle { P4USBClearVideo(handle) } }
    func reconnect() { if let handle { P4USBReconnect(handle) } }
    func stop() {
        if let handle { P4USBStop(handle); self.handle = nil }
    }
    deinit { stop() }
}

enum DeskError: LocalizedError {
    case usbDisconnected, usbQueueUnavailable, timeout(String), deviceRejected(String), invalidDevice,
         permission(String), displayUnavailable, captureUnavailable, encodingFailed, fontUnavailable, fontBakeFailed, invalidFont
    var errorDescription: String? {
        switch self {
        case .usbDisconnected: return "USB 未连接。请连接板上 Type-A USB-OTG 大接口；Type-C 小接口仅用于供电／烧录调试。"
        case .usbQueueUnavailable: return "USB 发送队列不可用，请重新连接。"
        case .timeout: return "设备响应超时，请保持 USB 连接并重试。"
        case .deviceRejected: return "设备未能完成操作，请检查连接、配置和存储状态。"
        case .invalidDevice: return "设备协议或屏幕尺寸不匹配。"
        case .permission(let name): return "需要在系统设置中允许\(name)。"
        case .displayUnavailable: return "当前系统无法创建 P4 Desk 虚拟显示器。"
        case .captureUnavailable: return "无法捕获 P4 Desk 虚拟显示器。"
        case .encodingFailed: return "无法生成设备支持的 JPEG 画面。"
        case .fontUnavailable: return "缺少字体生成工具或 TTF/OTF 字库，请在设置中选择。"
        case .fontBakeFailed: return "字体包生成失败，请检查 TTF/OTF 字库与文字覆盖。"
        case .invalidFont: return "字体包校验失败或超过 8 MiB。"
        }
    }
}
