//! Fixed C/Rust radio ABI. No credentials or network identifiers are logged.
use std::fmt;
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WifiAp {
    pub id: u32,
    pub rssi: i32,
    pub security: u8,
    pub channel: u8,
    pub saved: u8,
    pub reserved: u8,
    pub name: [u8; 36],
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BleDevice {
    pub id: u32,
    pub rssi: i32,
    pub connectable: u8,
    pub connected: u8,
    pub bonded: u8,
    pub reserved: u8,
    pub name: [u8; 52],
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RadioSnapshot {
    pub revision: u32,
    pub backend: u32,
    pub wifi_on: u32,
    pub wifi_scan: u32,
    pub wifi_phase: u32,
    pub wifi_error: u32,
    pub bt_on: u32,
    pub bt_scan: u32,
    pub bt_ready: u32,
    pub bt_error: u32,
    pub bt_peer_id: u32,
    pub bt_secure: u32,
    pub phone_connected: u32,
    pub services_count: u32,
    pub ap_count: u32,
    pub ble_count: u32,
    pub ssid: [u8; 36],
    pub ip: [u8; 20],
    pub saved_ssid: [u8; 36],
    pub coprocessor: [u8; 20],
    pub aps: [WifiAp; 16],
    pub devices: [BleDevice; 16],
    pub services: [[u8; 40]; 12],
    pub wifi_rssi_dbm: i32,
    pub wifi_rssi_valid: u32,
}
impl Default for RadioSnapshot {
    fn default() -> Self {
        // All members are fixed width integers or arrays of them.
        unsafe { std::mem::zeroed() }
    }
}
const _: () = {
    assert!(std::mem::size_of::<WifiAp>() == 48);
    assert!(std::mem::size_of::<BleDevice>() == 64);
    assert!(std::mem::size_of::<RadioSnapshot>() == 2456);
    assert!(std::mem::offset_of!(RadioSnapshot, wifi_rssi_dbm) == 2448);
    assert!(std::mem::offset_of!(RadioSnapshot, aps) == 176);
    assert!(std::mem::offset_of!(RadioSnapshot, services) == 1968);
};
pub fn label(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end])
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
impl WifiAp {
    pub fn label(&self) -> String {
        label(&self.name)
    }
}
impl BleDevice {
    pub fn label(&self) -> String {
        let s = label(&self.name);
        if s.is_empty() {
            format!("未命名设备 {:02}", self.id)
        } else {
            s
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WifiIndicator {
    Disabled,
    Offline,
    Connecting,
    Connected,
    Unavailable,
    Error,
}
impl WifiIndicator {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "已关闭",
            Self::Offline => "未连接",
            Self::Connecting => "连接中",
            Self::Connected => "已连接",
            Self::Unavailable => "不可用",
            Self::Error => "连接异常",
        }
    }
}
impl RadioSnapshot {
    pub fn wifi_signal_level(&self) -> Option<u8> {
        if self.wifi_indicator() != WifiIndicator::Connected || self.wifi_rssi_valid == 0 {
            return None;
        }
        match self.wifi_rssi_dbm {
            -55..=-1 => Some(3),
            -67..=-56 => Some(2),
            -80..=-68 => Some(1),
            -127..=-81 => Some(0),
            _ => None,
        }
    }
    pub fn wifi_indicator(&self) -> WifiIndicator {
        if self.wifi_on == 0 {
            WifiIndicator::Disabled
        } else if self.backend == 3 {
            WifiIndicator::Unavailable
        } else if self.backend == 1 || self.wifi_phase == 4 {
            WifiIndicator::Connecting
        } else if self.wifi_phase == 5 {
            // A scan/save error does not invalidate an established IP connection.
            WifiIndicator::Connected
        } else if self.wifi_error != 0 {
            WifiIndicator::Error
        } else {
            WifiIndicator::Offline
        }
    }
    /// Refresh the desktop for Wi-Fi changes, without reacting to BLE scan traffic.
    pub fn wifi_status_changed(&self, previous: &Self) -> bool {
        self.wifi_indicator() != previous.wifi_indicator()
            || self.wifi_signal_level() != previous.wifi_signal_level()
            || self.wifi_status() != previous.wifi_status()
            || self.ssid != previous.ssid
            || self.ip != previous.ip
    }
    pub fn aps(&self) -> &[WifiAp] {
        &self.aps[..(self.ap_count as usize).min(16)]
    }
    pub fn devices(&self) -> &[BleDevice] {
        &self.devices[..(self.ble_count as usize).min(16)]
    }
    pub fn services(&self) -> &[[u8; 40]] {
        &self.services[..(self.services_count as usize).min(12)]
    }
    pub fn wifi_status(&self) -> &'static str {
        if self.backend == 3 {
            "无线模块不可用"
        } else if self.wifi_on == 0 {
            "已关闭"
        } else if self.backend == 1 {
            "正在启动无线模块"
        } else if self.wifi_error != 0 {
            error_label(self.wifi_error)
        } else if self.wifi_phase == 5 {
            "已连接"
        } else if self.wifi_phase == 4 {
            "正在连接"
        } else if self.wifi_scan != 0 {
            "正在搜索网络"
        } else {
            "未连接"
        }
    }
    pub fn ble_status(&self) -> &'static str {
        if self.bt_on == 0 {
            "已关闭"
        } else if self.backend == 3 {
            "无线模块不可用"
        } else if self.bt_error != 0 {
            error_label(self.bt_error)
        } else if self.bt_ready == 0 {
            "正在启动蓝牙"
        } else if self.bt_scan != 0 {
            "正在搜索附近设备"
        } else {
            "可被发现，名称为 P4Desk"
        }
    }
}
pub fn error_label(error: u32) -> &'static str {
    match error {
        1 => "无线初始化失败，请重启后重试",
        2 => "保存失败，请检查本机存储",
        3 => "搜索失败，请重试",
        4 => "密码或配对验证失败",
        5 => "操作超时，请重试",
        6 => "列表已更新，请重新选择",
        7 => "暂不支持此认证方式",
        8 => "连接已断开，请重试",
        9 => "蓝牙操作失败，请重试",
        10 => "正在处理，请稍后重试",
        _ => "操作未完成，请重试",
    }
}
#[derive(Clone)]
pub enum RadioCommand {
    WifiEnable(bool),
    WifiScan,
    WifiConnect { id: u32, password: String },
    WifiDisconnect,
    WifiForget,
    WifiSavedConnect,
    BleEnable(bool),
    BleScan,
    BleConnect(u32),
    BleDisconnect,
    BlePair,
    BleForget,
}
impl fmt::Debug for RadioCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // UiCommand derives Debug; never print passwords via that wrapper.
        write!(f, "RadioCommand({})", self.parts().0)
    }
}
impl RadioCommand {
    pub fn parts(&self) -> (u32, u32, &[u8]) {
        match self {
            Self::WifiEnable(v) => (1, *v as u32, &[]),
            Self::WifiScan => (2, 0, &[]),
            Self::WifiConnect { id, password } => (3, *id, password.as_bytes()),
            Self::WifiDisconnect => (4, 0, &[]),
            Self::WifiForget => (5, 0, &[]),
            Self::BleEnable(v) => (6, *v as u32, &[]),
            Self::BleScan => (7, 0, &[]),
            Self::BleConnect(id) => (8, *id, &[]),
            Self::BleDisconnect => (9, 0, &[]),
            Self::BlePair => (10, 0, &[]),
            Self::BleForget => (11, 0, &[]),
            Self::WifiSavedConnect => (12, 0, &[]),
        }
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsSection {
    #[default]
    Wifi,
    Bluetooth,
    Display,
    DateTime,
    Storage,
    About,
}
impl SettingsSection {
    pub const ALL: [Self; 6] = [
        Self::Wifi,
        Self::Bluetooth,
        Self::Display,
        Self::DateTime,
        Self::Storage,
        Self::About,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::Wifi => "Wi-Fi",
            Self::Bluetooth => "蓝牙",
            Self::Display => "显示器",
            Self::DateTime => "日期与时间",
            Self::Storage => "存储",
            Self::About => "关于",
        }
    }
}
#[derive(Default)]
pub struct SettingsView {
    pub section: SettingsSection,
    pub page: usize,
    pub selected_ble: Option<u32>,
    pub join: Option<WifiJoin>,
    pub confirm_forget: bool,
}
pub struct WifiJoin {
    pub id: u32,
    pub name: String,
    pub secured: bool,
    pub password: String,
    pub uppercase: bool,
    pub symbols: bool,
    pub reveal: bool,
}
impl WifiJoin {
    pub fn new(ap: &WifiAp) -> Self {
        Self {
            id: ap.id,
            name: ap.label(),
            secured: ap.security != 0,
            password: String::new(),
            uppercase: false,
            symbols: false,
            reveal: false,
        }
    }
    pub fn valid(&self) -> bool {
        if self.secured {
            (8..=63).contains(&self.password.len())
        } else {
            self.password.is_empty()
        }
    }
    pub fn push(&mut self, c: char) {
        if c.is_ascii() && !c.is_ascii_control() && self.password.len() < 63 {
            self.password.push(c);
        }
    }
}
