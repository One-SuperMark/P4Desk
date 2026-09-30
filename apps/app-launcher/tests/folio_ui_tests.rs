use app_launcher::headless::HeadlessBackend;
use app_launcher::radio::RadioCommand;
use app_launcher::status_bar::StatusPanelKind;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState, UiCommand};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};
struct Harness {
    state: Arc<Mutex<LauncherState>>,
    app: App,
    backend: HeadlessBackend,
}
impl Harness {
    fn new() -> Self {
        Self::with_background(&[])
    }
    fn with_background(ids: &[&str]) -> Self {
        Self::with_history(ids, false)
    }
    fn with_history(ids: &[&str], close: bool) -> Self {
        let mut initial = LauncherState::new();
        for id in ids {
            initial.open_app(id);
            if close {
                initial.kill_active_app();
            } else {
                initial.background_active_app();
            }
        }
        let state = Arc::new(Mutex::new(initial));
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(
            build_launcher_ui(state.clone(), Size::new(1024.0, 600.0)),
            Size::new(1024.0, 600.0),
        );
        app.step(&mut backend);
        Self {
            state,
            app,
            backend,
        }
    }
    fn tap(&mut self, x: f32, y: f32) {
        for event in [
            TouchEvent::Down(Point::new(x, y)),
            TouchEvent::Up(Point::new(x, y)),
        ] {
            self.backend.event(event);
            self.app.step_with_builder(&mut self.backend, |size| {
                build_launcher_ui(self.state.clone(), size)
            });
        }
    }
}
#[test]
fn control_center_queues_real_actions_without_fabricating_radio_state() {
    let mut h = Harness::new();
    h.tap(954.0, 216.0);
    assert!(h.state.lock().unwrap().status_panel_kind == StatusPanelKind::Control);
    h.tap(425.0, 150.0);
    h.tap(600.0, 150.0);
    h.tap(400.0, 308.0);
    let mut s = h.state.lock().unwrap();
    assert!(matches!(
        s.take_commands().as_slice(),
        [
            UiCommand::Radio(RadioCommand::WifiEnable(true)),
            UiCommand::Radio(RadioCommand::BleEnable(true)),
            UiCommand::Brightness(25)
        ]
    ));
    assert_eq!(s.radio.wifi_on, 0);
    assert_eq!(s.radio.bt_on, 0);
}
#[test]
fn control_center_dismiss_does_not_launch_underlying_icon_and_navigation_closes_it() {
    let mut h = Harness::new();
    h.tap(954.0, 216.0);
    h.tap(132.0, 278.0);
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    assert!(!h.state.lock().unwrap().status_panel_open);
    h.tap(954.0, 216.0);
    h.tap(420.0, 475.0);
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Settings
    ));
    assert!(!h.state.lock().unwrap().status_panel_open);
}
#[test]
fn status_plate_is_passive_and_details_remain_available_in_control_center() {
    let mut h = Harness::new();
    let desktop = h.backend.pixels.clone();
    // Fixed two-column grid: battery/Wi-Fi, USB/TF, BLE/empty.
    for (x, y) in [
        (930.0, 58.0),
        (976.0, 58.0),
        (930.0, 103.0),
        (976.0, 103.0),
        (930.0, 148.0),
        (976.0, 148.0),
        (954.0, 35.0),
        (954.0, 170.0),
    ] {
        h.tap(x, y);
        let mut s = h.state.lock().unwrap();
        assert!(matches!(s.active_app, ActiveApp::Launcher));
        assert!(!s.status_panel_open);
        assert!(s.take_commands().is_empty());
        assert!(s.app_launch.frame(0).is_none());
        assert_eq!(h.backend.pixels, desktop);
    }
    h.tap(954.0, 216.0);
    h.tap(580.0, 390.0);
    assert!(h.state.lock().unwrap().status_panel_kind == StatusPanelKind::Device);
    assert!(h.state.lock().unwrap().status_panel_open);
}

#[test]
fn widgets_and_side_dock_launch_with_their_visible_icon_origins() {
    for (tap, id, cx, cy, width) in [
        ((140.0, 150.0), "clock", 395.0, 77.0, 58.0),
        ((660.0, 150.0), "timer", 835.0, 77.0, 58.0),
        ((954.0, 310.0), "settings", 954.0, 310.0, 68.0 * 0.94),
    ] {
        let mut h = if id == "settings" {
            Harness::with_background(&["settings"])
        } else {
            Harness::new()
        };
        h.tap(tap.0, tap.1);
        let s = h.state.lock().unwrap();
        let f = s.app_launch.frame(0).unwrap();
        assert_eq!(f.id, id);
        assert!((f.source.x + f.source.width * 0.5 - cx).abs() < 0.1);
        assert!((f.source.y + f.source.height * 0.5 - cy).abs() < 0.1);
        assert!((f.source.width - width).abs() < 0.1);
    }
}

#[test]
fn empty_dock_has_no_placeholder_icons_or_glass_and_cannot_launch_apps() {
    use app_launcher::widgets::WallpaperPainter;
    use tiny_flutter::tiny_gfx::Pixmap565;
    use tiny_flutter::{Canvas, CustomPainter};
    let mut h = Harness::new();
    let mut wallpaper = Pixmap565::new(1024, 600).unwrap();
    WallpaperPainter.paint(
        &mut Canvas::new(wallpaper.as_mut()),
        Size::new(1024.0, 600.0),
    );
    for y in 264..576 {
        assert_eq!(
            &h.backend.pixels[y * 1024 + 908..y * 1024 + 1000],
            &wallpaper.data()[y * 1024 + 908..y * 1024 + 1000]
        );
    }
    for y in [310.0, 382.0, 454.0, 526.0] {
        h.tap(954.0, y);
    }
    let mut s = h.state.lock().unwrap();
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert!(s.running_apps.is_empty());
    assert!(s.take_commands().is_empty());
}

#[test]
fn recent_background_apps_follow_open_order_from_top_to_bottom() {
    let ids = ["settings", "clock", "timer", "calculator"];
    for (i, id) in ids.iter().enumerate() {
        let mut h = Harness::with_background(&ids);
        let cy = 310.0 + i as f32 * 72.0;
        h.tap(954.0, cy);
        let s = h.state.lock().unwrap();
        assert_eq!(s.active_app.id(), Some(*id));
        assert_eq!(s.running_apps.len(), 3);
        assert!(!s.running_apps.contains_key(*id));
        let frame = s.app_launch.frame(0).unwrap();
        assert!((frame.source.y + frame.source.height * 0.5 - cy).abs() < 0.1);
        assert!((frame.source.width - 68.0 * 0.94).abs() < 0.1);
    }
}

#[test]
fn closed_recent_apps_stay_visible_and_reopen_from_top_to_bottom() {
    let opened = ["clock", "settings", "calculator", "timer", "notes"];
    let visible = ["settings", "calculator", "timer", "notes"];
    for (i, id) in visible.iter().enumerate() {
        let mut h = Harness::with_history(&opened, true);
        {
            let s = h.state.lock().unwrap();
            assert!(s.running_apps.is_empty());
            assert_eq!(s.recent_app_ids().collect::<Vec<_>>(), visible);
        }
        let cy = 310.0 + i as f32 * 72.0;
        h.tap(954.0, cy);
        let s = h.state.lock().unwrap();
        assert_eq!(s.active_app.id(), Some(*id));
        assert!(s.running_apps.is_empty());
        assert_eq!(s.recent_app_ids().count(), 4);
        assert_eq!(s.recent_app_ids().last(), Some(*id));
        let frame = s.app_launch.frame(0).unwrap();
        assert!((frame.source.y + frame.source.height * 0.5 - cy).abs() < 0.1);
    }
}

#[test]
fn desktop_card_grid_and_dock_clock_update_at_midnight_and_launch_keeps_its_time() {
    use app_launcher::live_clock_icon::ClockTime;
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        s.settings.timezone_minutes = 0;
        s.open_app("clock");
        s.background_active_app();
        s.tick(0, 946771199000); // 2000-01-01 23:59:59 UTC.
        assert_eq!(s.clock, "23:59:59");
    }
    let render = || {
        let mut backend = HeadlessBackend::new(1024, 600);
        let size = Size::new(1024.0, 600.0);
        App::new(build_launcher_ui(state.clone(), size), size).step(&mut backend);
        backend.pixels
    };
    let before = render();
    assert!(state.lock().unwrap().tick(1000, 946771200000));
    assert_eq!(state.lock().unwrap().clock, "00:00:00");
    let after = render();
    let crop = |pixels: &[u16], x: usize, y: usize, w: usize, h: usize| {
        (y..y + h)
            .flat_map(|row| pixels[row * 1024 + x..row * 1024 + x + w].iter().copied())
            .collect::<Vec<_>>()
    };
    // Numeric seconds, card icon, desktop icon and recent-app icon all use local time.
    for (x, y, w, h) in [
        (46, 66, 270, 70),
        (379, 61, 32, 32),
        (102, 248, 60, 60),
        (935, 291, 38, 38),
    ] {
        assert_ne!(crop(&before, x, y, w, h), crop(&after, x, y, w, h));
    }
    // Idle timer card should not repaint different pixels just because a second elapsed.
    assert_eq!(
        crop(&before, 484, 65, 280, 72),
        crop(&after, 484, 65, 280, 72)
    );
    let mut s = state.lock().unwrap();
    s.launch_app(
        "clock",
        tiny_flutter::Rect::from_ltwh(62.0, 208.0, 140.0, 140.0),
    );
    s.tick(1100, 946771201000); // Clock correction while launch is in progress.
    assert_eq!(
        s.app_launch.frame(1100).unwrap().clock,
        ClockTime::parse("00:00:00")
    );
    assert_eq!(s.clock, "00:00:01");
}
