//! Synthetic radio fixtures; uses the actual firmware UI, never claims a live connection.
use app_launcher::headless::HeadlessBackend;
use app_launcher::radio::{SettingsSection, WifiJoin};
use app_launcher::{build_launcher_ui, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or("artifacts/settings".into()),
    );
    std::fs::create_dir_all(&out)?;
    for scene in [
        "theme",
        "appearance",
        "appearance-clear",
        "appearance-off",
        "wifi",
        "wifi-password",
        "bluetooth",
        "ble-details",
        "display",
        "date",
        "date-syncing",
        "date-synced",
        "date-timeout",
        "date-offline",
        "date-restored",
        "storage",
        "about",
    ] {
        let mut s = LauncherState::new();
        s.settings.light_appearance = std::env::args().nth(2).as_deref() == Some("light");
        if std::env::args().nth(3).as_deref() == Some("whitesur") {
            s.settings.icon_theme = app_launcher::icon_theme::IconTheme::WhiteSur;
        }
        s.settings.icon_light_override = std::env::args().nth(4).map(|s| s == "light");
        s.open_app("settings");
        s.sd_ready = true;
        s.radio.backend = 2;
        s.radio.wifi_on = 1;
        s.radio.wifi_phase = 5;
        s.radio.ap_count = 4;
        s.radio.ssid[..8].copy_from_slice(b"Home LAN");
        s.radio.ip[..12].copy_from_slice(b"192.168.1.23");
        for (i, name) in ["Home LAN", "Studio", "Guest Wi-Fi", "P4Desk Lab"]
            .into_iter()
            .enumerate()
        {
            let ap = &mut s.radio.aps[i];
            ap.id = i as u32 + 1;
            ap.rssi = -42 - i as i32 * 10;
            ap.security = 1;
            ap.name[..name.len()].copy_from_slice(name.as_bytes());
        }
        s.radio.bt_on = 1;
        s.radio.bt_ready = 1;
        s.radio.ble_count = 3;
        for (i, name) in ["BLE Sensor", "Desk Light", "Phone"]
            .into_iter()
            .enumerate()
        {
            let d = &mut s.radio.devices[i];
            d.id = i as u32 + 10;
            d.rssi = -48;
            d.connectable = 1;
            d.name[..name.len()].copy_from_slice(name.as_bytes());
        }
        s.settings_view.section = match scene {
            "theme" => SettingsSection::Appearance,
            "bluetooth" | "ble-details" => SettingsSection::Bluetooth,
            "appearance" | "appearance-clear" | "appearance-off" => SettingsSection::Appearance,
            "display" => SettingsSection::Display,
            "date" | "date-syncing" | "date-synced" | "date-timeout" | "date-offline"
            | "date-restored" => SettingsSection::DateTime,
            "storage" => SettingsSection::Storage,
            "about" => SettingsSection::About,
            _ => SettingsSection::Wifi,
        };
        if scene == "theme" {
            s.settings_view.appearance_scroll.set_offset(420.0);
        }
        if scene.starts_with("date-") {
            let epoch = 1790748000;
            s.tick(1000, i64::from(epoch) * 1000);
            s.radio.time_sync.last_sync_unix_s = epoch;
            s.radio.time_sync.success_count = 1;
            s.radio.time_sync.phase = match scene {
                "date-syncing" => 1,
                "date-timeout" => 3,
                "date-offline" => 0,
                _ => 2,
            };
            if scene == "date-timeout" {
                s.radio.time_sync.error = 1;
            }
            if scene == "date-offline" {
                s.radio.wifi_phase = 2;
            }
            if scene == "date-restored" {
                s.time_estimated = true;
                s.radio.wifi_phase = 2;
                s.radio.time_sync = Default::default();
            }
        }
        if scene == "storage" {
            s.persistence_status = app_launcher::session::PersistenceStatus::Saved;
        }
        if scene == "appearance-clear" {
            s.settings.glass_amount = 25;
        }
        if scene == "appearance-off" {
            s.settings.glass_amount = 0;
        }
        if scene == "wifi-password" {
            s.settings_view.join = Some(WifiJoin::new(&s.radio.aps[1]));
        }
        if scene == "ble-details" {
            s.settings_view.selected_ble = Some(10);
            s.radio.bt_peer_id = 10;
            s.radio.devices[0].connected = 1;
            s.radio.services_count = 1;
            s.radio.services[0][..6].copy_from_slice(b"0x180a");
        }
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(Arc::new(Mutex::new(s)), size), size).step(&mut backend);
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("settings-{scene}.png")))?;
    }
    Ok(())
}
