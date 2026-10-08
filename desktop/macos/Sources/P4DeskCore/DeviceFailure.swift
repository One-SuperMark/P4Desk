import Foundation

/// Only fixed protocol error labels cross the host UI/diagnostic boundary.
public struct DeviceFailure: Error, LocalizedError, Equatable, Sendable {
    public let code: String
    public init(code: String) { self.code = DeviceAcknowledgement.safeErrorCode(["error": code]) }
    public var errorDescription: String? {
        switch code {
        case "sd_unavailable", "storage_unavailable": return "TF 卡未就绪，请在设备上检查存储状态。"
        case "sd_space", "no_space": return "TF 卡空间不足，请腾出空间后重试。"
        case "files_busy", "sync_busy", "busy": return "设备正在传输文件或同步配置，请等待完成后重试。"
        case "display_busy": return "请先将设备切回 Pad，再执行此操作。"
        case "font_read", "font_open": return "设备无法读取同步字库，请检查 TF 卡并重试。"
        case "font_create", "font_write", "font_fsync", "storage_create", "state_write", "state_fsync", "state_rename", "generation_rename", "directory_fsync", "staging_cleanup":
            return "设备未能完成存储写入，请检查 TF 卡后重试。"
        case "font_hash", "integrity": return "传输内容校验失败，请重新同步。"
        case "font_invalid", "font_length", "font_coverage": return "同步字库不完整或不兼容，请检查所选字体并重新同步。"
        case "sync_incomplete", "chunk_offset", "resource_length", "transfer_closed", "sync_not_started":
            return "同步传输未完整完成，请保持 USB 连接并重新同步。"
        case "generation_exists", "generation_invalid": return "同步版本不匹配，请重新连接设备后再同步。"
        case "state_size": return "便签与快捷按钮配置过大，请减少内容后重试。"
        case "state_invalid", "state_encode", "manifest_invalid", "settings_invalid": return "设备未能验证配置，请检查内容后重试。"
        case "hello_required", "version", "json_invalid", "request_id": return "设备连接或协议不匹配，请重新连接或升级固件。"
        case "mode_rejected", "jpeg_rotation": return "设备未能切换显示模式，请重新连接后重试。"
        case "monitor_busy": return "监控服务正在验证配置，请稍后重试。"
        case "monitor_unavailable", "monitor_config_invalid": return "设备未能接受监控配置，请检查固件与配置。"
        case "time_invalid": return "时间设置超出设备支持范围，请检查后重试。"
        case "cancelled": return "本次操作已取消。"
        default: return "设备未完成本次操作，请检查连接与存储状态后重试。"
        }
    }
}

public enum DeviceAcknowledgement {
    private static let codes: Set<String> = [
        "sd_unavailable", "storage_unavailable", "sd_space", "no_space", "files_busy", "sync_busy", "busy", "display_busy",
        "font_read", "font_open", "font_create", "font_write", "font_fsync", "font_hash", "font_invalid", "font_length", "font_coverage",
        "storage_create", "storage_read", "state_write", "state_fsync", "state_rename", "state_encode", "state_read", "state_invalid", "state_size",
        "generation_rename", "generation_exists", "generation_invalid", "directory_fsync", "staging_cleanup", "manifest_invalid",
        "sync_incomplete", "sync_not_started", "chunk_offset", "resource_length", "transfer_closed", "hello_required", "version", "json_invalid", "request_id",
        "mode_rejected", "jpeg_rotation", "settings_invalid", "monitor_busy", "monitor_unavailable", "monitor_config_invalid", "time_invalid", "integrity", "cancelled"
    ]
    public static func safeErrorCode(_ fields: [String: Any]) -> String {
        let code = fields["error"] as? String ?? fields["code"] as? String ?? "device_rejected"
        return codes.contains(code) ? code : "device_rejected"
    }
}

public struct SyncResult: Equatable, Sendable {
    public var committed: Bool
    public var errorCode: String
    public var fontBytes: Int
    public var generation: UInt64
    public init(committed: Bool = false, errorCode: String = "not_started", fontBytes: Int = 0, generation: UInt64 = 0) {
        self.committed = committed; self.errorCode = errorCode; self.fontBytes = fontBytes; self.generation = generation
    }
}
