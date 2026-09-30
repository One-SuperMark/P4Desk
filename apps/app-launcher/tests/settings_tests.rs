use app_launcher::headless::HeadlessBackend;
use app_launcher::radio::{label, RadioCommand, RadioSnapshot, SettingsSection, WifiJoin};
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
        let mut s = LauncherState::new();
        s.open_app("settings");
        s.radio.wifi_on = 1;
        s.radio.backend = 2;
        s.radio.ap_count = 1;
        s.radio.aps[0].id = 17;
        s.radio.aps[0].name[..4].copy_from_slice(b"Test");
        s.radio.aps[0].security = 1;
        let state = Arc::new(Mutex::new(s));
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), size), size);
        app.step(&mut backend);
        Self {
            state,
            app,
            backend,
        }
    }
    fn tap(&mut self, x: f32, y: f32) {
        for e in [
            TouchEvent::Down(Point::new(x, y)),
            TouchEvent::Up(Point::new(x, y)),
        ] {
            self.backend.event(e);
            self.app.step_with_builder(&mut self.backend, |size| {
                build_launcher_ui(self.state.clone(), size)
            });
        }
    }
    fn commands(&self) -> Vec<UiCommand> {
        self.state.lock().unwrap().take_commands()
    }
}
#[test]
fn wifi_and_ble_toggles_queue_real_radio_actions() {
    let mut h = Harness::new();
    h.tap(950.0, 121.0);
    assert!(matches!(
        h.commands().as_slice(),
        [UiCommand::Radio(RadioCommand::WifiEnable(false))]
    ));
    h.tap(100.0, 177.0);
    assert!(h.state.lock().unwrap().settings_view.section == SettingsSection::Bluetooth);
    h.tap(950.0, 121.0);
    assert!(matches!(
        h.commands().as_slice(),
        [UiCommand::Radio(RadioCommand::BleEnable(true))]
    ));
}
#[test]
fn password_dialog_is_modal_and_joins_selected_network() {
    let mut h = Harness::new();
    h.tap(460.0, 382.0);
    assert!(h.state.lock().unwrap().settings_view.join.is_some());
    h.tap(60.0, 300.0);
    assert!(h.state.lock().unwrap().settings_view.section == SettingsSection::Wifi);
    h.tap(860.0, 537.0);
    assert!(h.commands().is_empty());
    for _ in 0..8 {
        h.tap(147.0, 230.0);
    }
    h.tap(860.0, 537.0);
    let c = h.commands();
    match &c[0] {
        UiCommand::Radio(RadioCommand::WifiConnect { id, password }) => {
            assert_eq!(*id, 17);
            assert!(password == "11111111");
        }
        _ => panic!("Expected Wi-Fi connect"),
    };
    assert!(h.state.lock().unwrap().settings_view.join.is_none());
}
#[test]
fn home_and_close_remain_reachable() {
    let mut h = Harness::new();
    h.tap(34.0, 35.0);
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    assert!(h
        .state
        .lock()
        .unwrap()
        .running_apps
        .contains_key("settings"));
    let mut h = Harness::new();
    h.tap(986.0, 35.0);
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    assert!(!h
        .state
        .lock()
        .unwrap()
        .running_apps
        .contains_key("settings"));
}
#[test]
fn password_is_bounded_and_debug_is_redacted() {
    let r = RadioSnapshot::default();
    let mut j = WifiJoin::new(&r.aps[0]);
    j.secured = true;
    for _ in 0..100 {
        j.push('x');
    }
    assert_eq!(j.password.len(), 63);
    assert!(j.valid());
    j.push('\n');
    assert_eq!(j.password.len(), 63);
    let c = UiCommand::Radio(RadioCommand::WifiConnect {
        id: 1,
        password: "never-log-me".into(),
    });
    assert!(!format!("{c:?}").contains("never-log-me"));
}
#[test]
fn snapshot_counts_and_untrusted_names_are_bounded() {
    let mut r = RadioSnapshot::default();
    r.ap_count = u32::MAX;
    r.ble_count = u32::MAX;
    r.services_count = u32::MAX;
    assert_eq!(r.aps().len(), 16);
    assert_eq!(r.devices().len(), 16);
    assert_eq!(r.services().len(), 12);
    assert_eq!(label(b"abc\n\xff\0hidden"), "abc �");
}

#[test]
fn network_time_button_requires_wifi_and_is_disabled_while_syncing() {
    for (phase, wifi_phase, allowed) in [
        (0, 2, false),
        (0, 5, true),
        (1, 5, false),
        (2, 5, true),
        (3, 5, true),
    ] {
        let mut h = Harness::new();
        {
            let mut s = h.state.lock().unwrap();
            s.radio.wifi_phase = wifi_phase;
            s.radio.time_sync.phase = phase;
        }
        h.tap(100.0, 366.0); // Date & Time after Appearance.
        assert!(h.state.lock().unwrap().settings_view.section == SettingsSection::DateTime);
        h.tap(898.0, 285.0);
        let commands = h.commands();
        if allowed {
            assert!(matches!(
                commands.as_slice(),
                [UiCommand::Radio(RadioCommand::WifiTimeSync)]
            ));
            assert_eq!(RadioCommand::WifiTimeSync.parts(), (13, 0, &[][..]));
        } else {
            assert!(commands.is_empty());
        }
    }
}

#[test]
fn network_time_status_distinguishes_wifi_from_internet_and_formats_local_last_success() {
    use app_launcher::radio::TimeSyncSnapshot;
    let mut status = TimeSyncSnapshot::default();
    assert_eq!(status.status(false), "等待 Wi-Fi 连接");
    assert_eq!(status.status(true), "等待自动对时");
    assert_eq!(status.last_sync_label(480), "尚未通过 Wi-Fi 对时");
    status.phase = 3;
    status.error = 1;
    assert!(status.status(true).contains("超时"));
    assert!(!status.status(false).contains("超时"));
    status.last_sync_unix_s = 946684800;
    assert_eq!(
        status.last_sync_label(480),
        "上次：2000 年 01 月 01 日  08:00:00"
    );
    assert_eq!(
        status.last_sync_label(-60),
        "上次：1999 年 12 月 31 日  23:00:00"
    );
}
