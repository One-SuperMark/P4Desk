use app_launcher::battery::{estimate_percent, BatteryReading, BatteryState, ChargeState};
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};

#[test]
fn voltage_curve_is_bounded_monotonic_and_does_not_call_voltage_full_charging() {
    assert_eq!(estimate_percent(3200), 0);
    assert_eq!(estimate_percent(3800), 50);
    assert_eq!(estimate_percent(4200), 100);
    for mv in 2500..4500 {
        assert!(estimate_percent(mv) <= estimate_percent(mv + 1));
        assert!(estimate_percent(mv) <= 100);
    }
    let mut b = BatteryState::default();
    b.update(BatteryReading {
        voltage_mv: Some(4200),
        charge: ChargeState::Unknown,
    });
    assert_eq!(b.percent, Some(100));
    assert_eq!(b.charge, ChargeState::Unknown);
    assert_eq!(b.charge_label(), "无法读取");
}
#[test]
fn missing_invalid_samples_clear_readings_and_smoothing_avoids_voltage_spikes() {
    let mut b = BatteryState::default();
    assert_eq!(b.percent_label(), "--%");
    b.update(BatteryReading {
        voltage_mv: Some(3800),
        charge: ChargeState::Unknown,
    });
    assert_eq!(b.percent, Some(50));
    b.update(BatteryReading {
        voltage_mv: Some(4200),
        charge: ChargeState::Unknown,
    });
    assert_eq!(b.voltage_mv, Some(3900));
    assert_ne!(b.percent, Some(100));
    for mv in [None, Some(0), Some(2499), Some(4501), Some(u16::MAX)] {
        b.update(BatteryReading {
            voltage_mv: mv,
            charge: ChargeState::Unknown,
        });
        assert_eq!(b.percent, None);
        assert_eq!(b.voltage_label(), "无法读取");
    }
    b.update(BatteryReading {
        voltage_mv: Some(3500),
        charge: ChargeState::Unknown,
    });
    assert_eq!(
        b.percent,
        Some(10),
        "recovery starts from a new valid measurement"
    );
}
#[test]
fn icon_bolt_requires_explicit_charge_telemetry() {
    use tiny_flutter::graphics::svg_icons_generated::{UI_BATTERY_50, UI_BATTERY_BOLT};
    let mut b = BatteryState::default();
    b.update(BatteryReading {
        voltage_mv: Some(3800),
        charge: ChargeState::Unknown,
    });
    assert!(std::ptr::eq(
        app_launcher::status_bar::battery_icon(&b),
        &UI_BATTERY_50
    ));
    b.update(BatteryReading {
        voltage_mv: Some(3800),
        charge: ChargeState::Charging,
    });
    assert!(std::ptr::eq(
        app_launcher::status_bar::battery_icon(&b),
        &UI_BATTERY_BOLT
    ));
}
#[test]
fn requested_plugged_in_indicator_is_distinct_from_measured_charge_and_can_clear() {
    use tiny_flutter::graphics::svg_icons_generated::{UI_BATTERY_100, UI_BATTERY_BOLT};
    let mut b = BatteryState::default();
    b.update(BatteryReading {
        voltage_mv: Some(4200),
        charge: ChargeState::PluggedInAssumed,
    });
    assert_eq!(b.percent, Some(100));
    assert_eq!(b.charge_label(), "插电充电");
    assert_ne!(b.charge, ChargeState::Full);
    assert!(std::ptr::eq(
        app_launcher::status_bar::battery_icon(&b),
        &UI_BATTERY_BOLT
    ));
    b.update(BatteryReading {
        voltage_mv: Some(4200),
        charge: ChargeState::Unknown,
    });
    assert_eq!(b.charge_label(), "无法读取");
    assert!(std::ptr::eq(
        app_launcher::status_bar::battery_icon(&b),
        &UI_BATTERY_100
    ));
}
#[test]
fn reset_labels_do_not_misdiagnose_poweron_as_brownout_and_uptime_survives_app_changes() {
    use app_launcher::boot_diagnostics::{reset_reason_label, uptime_label};
    assert_eq!(reset_reason_label(1), "上电／硬件复位");
    assert_eq!(reset_reason_label(9), "欠压复位");
    assert_eq!(reset_reason_label(11), "USB 复位");
    assert_eq!(reset_reason_label(14), "电源毛刺复位");
    assert_eq!(reset_reason_label(u32::MAX), "原因未知");
    assert_eq!(uptime_label(3_661_999), "1:01:01");
    assert_eq!(uptime_label(360_000_000), "100:00:00");
    let mut s = LauncherState::new();
    s.reset_reason = 9;
    s.tick(123_000, 0);
    s.open_app("timer");
    s.back_active_app();
    assert_eq!(s.reset_reason, 9);
    assert_eq!(uptime_label(s.monotonic_ms), "0:02:03");
}
fn tap(
    s: &Arc<Mutex<LauncherState>>,
    app: &mut App,
    backend: &mut HeadlessBackend,
    x: f32,
    y: f32,
) {
    for e in [
        TouchEvent::Down(Point::new(x, y)),
        TouchEvent::Up(Point::new(x, y)),
    ] {
        backend.event(e);
        app.step_with_builder(backend, |size| build_launcher_ui(s.clone(), size));
    }
}
#[test]
fn device_details_outside_tap_only_dismisses_and_cache_stays_clean() {
    let size = Size::new(1024.0, 600.0);
    let s = Arc::new(Mutex::new(LauncherState::new()));
    let mut app = App::new(build_launcher_ui(s.clone(), size), size);
    let mut backend = HeadlessBackend::new(1024, 600);
    app.step(&mut backend);
    let desktop = backend.pixels.clone();
    tap(&s, &mut app, &mut backend, 954.0, 216.0);
    tap(&s, &mut app, &mut backend, 580.0, 390.0);
    assert!(s.lock().unwrap().status_panel_open);
    tap(&s, &mut app, &mut backend, 780.0, 260.0);
    assert!(
        s.lock().unwrap().status_panel_open,
        "card tap must not dismiss or launch behind it"
    );
    tap(&s, &mut app, &mut backend, 348.0, 278.0);
    assert!(!s.lock().unwrap().status_panel_open);
    assert!(matches!(s.lock().unwrap().active_app, ActiveApp::Launcher));
    assert_eq!(backend.pixels, desktop);
    // After closing, a fresh gesture is required to launch an app.
    tap(&s, &mut app, &mut backend, 348.0, 278.0);
    assert!(matches!(s.lock().unwrap().active_app, ActiveApp::Timer));
}

#[test]
fn back_closes_status_card_and_mode_switch_cannot_leave_hidden_popup() {
    let mut s = LauncherState::new();
    s.status_panel_open = true;
    s.back_active_app();
    assert!(!s.status_panel_open);
    s.status_panel_open = true;
    s.open_app("clock");
    assert!(!s.status_panel_open);
    s.background_active_app();
    s.status_panel_open = true;
    s.mode = p4desk_protocol::Mode::Display;
    s.tick(100, 0);
    assert!(!s.status_panel_open);
}

#[test]
fn wifi_status_card_opens_wifi_settings_and_closes_overlay() {
    let size = Size::new(1024.0, 600.0);
    let s = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut state = s.lock().unwrap();
        state.settings_view.section = app_launcher::radio::SettingsSection::Bluetooth;
        state.status_panel_open = true;
        state.status_panel_kind = app_launcher::status_bar::StatusPanelKind::Wifi;
    }
    let mut app = App::new(build_launcher_ui(s.clone(), size), size);
    let mut backend = HeadlessBackend::new(1024, 600);
    app.step(&mut backend);
    assert!(s.lock().unwrap().status_panel_open);
    tap(&s, &mut app, &mut backend, 810.0, 318.0);
    let state = s.lock().unwrap();
    assert!(!state.status_panel_open);
    assert!(matches!(state.active_app, ActiveApp::Settings));
    assert!(state.settings_view.section == app_launcher::radio::SettingsSection::Wifi);
}
