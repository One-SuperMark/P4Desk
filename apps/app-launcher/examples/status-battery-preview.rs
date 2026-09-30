//! Synthetic status states through the actual Rust UI; no simulated value enters firmware.
use app_launcher::battery::{BatteryReading, ChargeState};
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, LauncherState};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tiny_flutter::{App, Size};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/status-battery".into()),
    );
    std::fs::create_dir_all(&out)?;
    for (name, mv, usb, host, sd, panel) in [
        ("desktop", Some(3990), true, true, true, false),
        ("details", Some(3990), true, true, true, true),
        ("low", Some(3500), false, false, true, false),
        ("unknown", None, false, false, false, false),
        ("usb-wait", Some(3900), true, false, true, true),
        ("typec-charge", Some(4140), false, false, true, true),
        ("wifi-connected", Some(4140), true, true, true, false),
        ("wifi-connecting", Some(4140), true, true, true, false),
        ("wifi-offline", Some(4140), true, true, true, false),
        ("wifi-error", Some(4140), true, true, true, false),
        ("wifi-details", Some(4140), true, true, true, true),
        ("wifi-long-name", Some(4140), true, true, true, true),
        ("wifi-medium", Some(4140), true, true, true, false),
        ("wifi-weak", Some(4140), true, true, true, false),
        ("wifi-very-weak", Some(4140), true, true, true, false),
    ] {
        let mut state = LauncherState::new();
        state.tick(123_000, 1_790_712_418_000);
        state.reset_reason = 9;
        state.usb_connected = usb;
        state.connected = host;
        state.sd_ready = sd;
        state.battery.update(BatteryReading {
            voltage_mv: mv,
            charge: if name == "typec-charge" {
                ChargeState::PluggedInAssumed
            } else {
                ChargeState::Unknown
            },
        });
        if name.starts_with("wifi-") {
            state.radio.backend = 2;
            state.radio.wifi_rssi_valid = 1;
            state.radio.wifi_rssi_dbm = match name {
                "wifi-medium" => -63,
                "wifi-weak" => -74,
                "wifi-very-weak" => -88,
                _ => -42,
            };
            state.radio.wifi_on = 1;
            state.radio.wifi_phase = match name {
                "wifi-connecting" => 4,
                "wifi-offline" | "wifi-error" => 2,
                _ => 5,
            };
            if name == "wifi-error" {
                state.radio.wifi_error = 4;
            }
            state.radio.ssid[..8].copy_from_slice(b"Home LAN");
            state.radio.ip[..12].copy_from_slice(b"192.168.1.23");
            if name == "wifi-long-name" {
                state.radio.ssid[..32].fill(b'W');
            }
            state.status_panel_kind = app_launcher::status_bar::StatusPanelKind::Wifi;
        }
        state.status_panel_open = panel;
        let state = Arc::new(Mutex::new(state));
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(state, size), size).step(&mut backend);
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{name}.png")))?;
    }
    Ok(())
}
