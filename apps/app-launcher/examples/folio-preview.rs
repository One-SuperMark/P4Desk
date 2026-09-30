//! Actual Rust renderer with synthetic demo state. Never loaded by firmware.
use app_launcher::battery::{BatteryReading, ChargeState};
use app_launcher::headless::HeadlessBackend;
use app_launcher::status_bar::StatusPanelKind;
use app_launcher::{build_launcher_ui, LauncherState};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tiny_flutter::{App, Point, Size, TouchEvent};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or("artifacts/folio".into()));
    std::fs::create_dir_all(&out)?;
    for scene in [
        "desktop",
        "desktop-slide",
        "desktop-page-2",
        "file-manager",
        "office-viewer",
        "sub2api-monitor",
        "desktop-controls-pressed",
        "desktop-controls-canceled",
        "desktop-background",
        "desktop-background-full",
        "desktop-recent-closed",
        "desktop-battery-low",
        "desktop-battery-full",
        "desktop-battery-unknown",
        "control-center",
        "clock",
        "timer",
        "notes",
        "notes-empty",
        "notes-delete",
        "notes-long",
        "mac",
        "mac-empty",
        "mac-long",
        "display",
        "display-offline",
        "settings",
    ] {
        let mut s = LauncherState::new();
        s.settings.icon_theme = match std::env::args().nth(3).as_deref() {
            Some("folio") => app_launcher::icon_theme::IconTheme::Folio,
            Some("whitesur") => app_launcher::icon_theme::IconTheme::WhiteSur,
            Some("numix") => app_launcher::icon_theme::IconTheme::Numix,
            _ => app_launcher::icon_theme::IconTheme::Colloid,
        };
        s.settings.light_appearance = std::env::args().nth(2).as_deref() == Some("light");
        s.settings.icon_light_override = std::env::args().nth(4).map(|s| s == "light");
        if scene == "desktop-page-2" {
            s.page_controller.set_page(1);
        }
        s.tick(123_000, 1_790_712_418_000);
        s.connected = !matches!(scene, "display-offline" | "mac-empty");
        s.usb_connected = s.connected;
        s.sd_ready = true;
        s.radio.backend = 2;
        s.radio.wifi_on = 1;
        s.radio.wifi_phase = 5;
        s.radio.wifi_rssi_valid = 1;
        s.radio.wifi_rssi_dbm = -63;
        s.radio.bt_on = 1;
        s.radio.bt_ready = 1;
        s.radio.ssid[..8].copy_from_slice(b"Home LAN");
        s.radio.ip[..12].copy_from_slice(b"192.168.1.23");
        s.battery.update(BatteryReading {
            voltage_mv: match scene {
                "desktop-battery-full" => Some(4200),
                "desktop-battery-low" => Some(3500),
                "desktop-battery-unknown" => None,
                _ => Some(3990),
            },
            charge: ChargeState::PluggedInAssumed,
        });
        if scene == "desktop-background" || scene == "desktop-background-full" {
            let count = if scene == "desktop-background" { 2 } else { 4 };
            for id in ["clock", "timer", "calculator", "notes", "mac", "settings"]
                .into_iter()
                .take(count)
            {
                s.open_app(id);
                s.background_active_app();
            }
        }
        if scene == "desktop-recent-closed" {
            for id in ["clock", "timer", "calculator", "settings"] {
                s.open_app(id);
                s.kill_active_app();
            }
        }
        if scene == "control-center" {
            s.status_panel_open = true;
            s.status_panel_kind = StatusPanelKind::Control;
        } else if !scene.starts_with("desktop") {
            s.open_app(
                if matches!(scene, "file-manager" | "office-viewer" | "sub2api-monitor") {
                    scene
                } else {
                    scene.split('-').next().unwrap()
                },
            );
        }
        if matches!(
            scene,
            "notes" | "notes-delete" | "notes-long" | "mac" | "mac-long"
        ) {
            use p4desk_protocol::{Action, Button, Note, Snapshot};
            s.apply_snapshot(Snapshot {
                notes: vec![Note { id:"preview-note".into(),title:"Folio on P4Desk".into(),
                    body:"One task at a time.\n\nA calm workspace for your day.\nNotes are edited on Mac and kept on the TF card.".into(), updated_ms:0 }],
                buttons: ["Copy", "Paste", "Finder", "Terminal", "Play / Pause", "Mute"].iter().enumerate().map(|(i,label)|
                    Button {id:format!("preview-{i}"),label:label.to_string(),action:match i {
                        0 | 1 => Action::Shortcut { key_code: if i == 0 { 8 } else { 9 }, modifiers: 0x100000 },
                        2 | 3 => Action::Application { bundle_path: if i == 2 { "/System/Library/CoreServices/Finder.app" } else { "/System/Applications/Utilities/Terminal.app" }.into() },
                        _ => Action::Media { usage: if i == 4 { 0xcd } else { 0xe2 } },
                    }}).collect(),
                ..Snapshot::default()
            });
            if scene == "notes-delete" {
                if let app_launcher::ActiveApp::Notes(view) = &s.active_app {
                    view.lock().unwrap().confirm_delete = true;
                }
            }
            if scene == "notes-long" {
                s.snapshot.notes[0].title = "Folio layout and reading space ".repeat(4);
                s.snapshot.notes[0].body =
                    "One task at a time. A calm workspace for your day.\n".repeat(30);
            }
            if scene == "mac-long" {
                s.snapshot.buttons[0].label =
                    "A shortcut label with a long name that wraps across the card".into();
            }
        }
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        let state = Arc::new(Mutex::new(s));
        let mut app = App::new(build_launcher_ui(state.clone(), size), size);
        app.step(&mut backend);
        if scene == "desktop-slide" {
            for event in [
                TouchEvent::Down(Point::new(760.0, 380.0)),
                TouchEvent::Move(Point::new(328.0, 380.0)),
            ] {
                backend.event(event);
                app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
            }
        }
        if scene.starts_with("desktop-controls-") {
            backend.event(TouchEvent::Down(Point::new(954.0, 216.0)));
            app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
            if scene.ends_with("canceled") {
                backend.event(TouchEvent::Cancel);
                app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
            }
        }
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{scene}.png")))?;
    }
    Ok(())
}
