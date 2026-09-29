//! ESP-IDF 6.0.2 esp_reset_reason_t values, passed over the internal u32 HAL.
//! POWERON can also follow an EN-pin reset; it is not proof of power loss.
pub fn reset_reason_label(reason: u32) -> &'static str {
    match reason {
        1 => "上电／硬件复位",
        2 => "外部引脚复位",
        3 => "软件复位",
        4 => "程序异常",
        5..=7 => "看门狗复位",
        8 => "休眠唤醒",
        9 => "欠压复位",
        10 => "SDIO 复位",
        11 => "USB 复位",
        12 => "JTAG 复位",
        13 => "eFuse 异常",
        14 => "电源毛刺复位",
        15 => "CPU 锁死",
        _ => "原因未知",
    }
}
pub fn uptime_label(ms: u64) -> String {
    let seconds = ms / 1000;
    format!(
        "{}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
