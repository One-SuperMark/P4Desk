use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

fn step_ui(app: &mut App, backend: &mut HeadlessBackend, state: &Arc<Mutex<LauncherState>>) {
    app.step_with_builder(backend, |size| build_launcher_ui(state.clone(), size));
}

fn tap(
    app: &mut App,
    backend: &mut HeadlessBackend,
    state: &Arc<Mutex<LauncherState>>,
    point: Point,
) {
    backend.event(TouchEvent::Down(point));
    step_ui(app, backend, state);
    backend.event(TouchEvent::Up(point));
    step_ui(app, backend, state);
    // Lifecycle assertions interact with the app after its real splash ends.
    let dirty = {
        let mut s = state.lock().unwrap();
        if s.app_launch.frame(s.monotonic_ms).is_some() {
            let now = s.monotonic_ms + app_launcher::app_launch::APP_LAUNCH_DURATION_MS;
            let unix = s.unix_ms;
            if s.tick(now, unix) {
                app.request_rebuild();
            }
            s.take_launch_animation_dirty(app.size())
        } else {
            None
        }
    };
    if let Some(rect) = dirty {
        app.mark_dirty(rect);
        step_ui(app, backend, state);
    }
}

#[test]
fn calculator_background_preserves_instance_and_kill_recreates() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    state.lock().unwrap().open_app("calculator");
    let first = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!(),
    };
    first.lock().unwrap().input_digit('7');
    let size = Size::new(1024.0, 600.0);
    let mut ui = build_launcher_ui(state.clone(), size).create_render_object();
    ui.layout(&BoxConstraints::tight(size));
    assert!(ui.dispatch_touch(&TouchEvent::HardwareBack));
    assert!(state
        .lock()
        .unwrap()
        .running_apps
        .contains_key("calculator"));
    state.lock().unwrap().open_app("calculator");
    let resumed = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!(),
    };
    assert!(Arc::ptr_eq(&first, &resumed));
    assert_eq!(resumed.lock().unwrap().current_input, "7");
    let mut ui = build_launcher_ui(state.clone(), size).create_render_object();
    ui.layout(&BoxConstraints::tight(size));
    assert!(ui.dispatch_touch(&TouchEvent::HardwareKill));
    state.lock().unwrap().open_app("calculator");
    let new = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!(),
    };
    assert!(!Arc::ptr_eq(&first, &new));
    assert_eq!(new.lock().unwrap().current_input, "0");
}
#[test]
fn fifth_hidden_app_closes_oldest_instance_and_reopening_starts_fresh() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    let original = match &s.active_app {
        ActiveApp::Calculator(c) => Arc::downgrade(c),
        _ => panic!(),
    };
    original.upgrade().unwrap().lock().unwrap().input_digit('7');
    s.background_active_app();
    for id in ["notes", "settings", "clock"] {
        s.open_app(id);
        s.background_active_app();
    }
    s.open_app("mac");
    assert_eq!(
        s.running_apps.len(),
        4,
        "foreground does not consume a background slot"
    );
    assert!(original.upgrade().is_some());
    s.background_active_app();
    assert_eq!(
        s.background_app_ids().collect::<Vec<_>>(),
        ["notes", "settings", "clock", "mac"]
    );
    assert_eq!(s.running_apps.len(), 4);
    assert!(
        original.upgrade().is_none(),
        "eviction must release the instance"
    );
    s.open_app("calculator");
    match &s.active_app {
        ActiveApp::Calculator(c) => assert_eq!(c.lock().unwrap().current_input, "0"),
        _ => panic!(),
    }
}

#[test]
fn resuming_oldest_with_full_background_preserves_state_and_rehide_moves_it_last() {
    let mut s = LauncherState::new();
    s.open_app("notes");
    let original = match &s.active_app {
        ActiveApp::Notes(n) => n.clone(),
        _ => panic!(),
    };
    original.lock().unwrap().selected = 3;
    s.background_active_app();
    for id in ["calculator", "settings", "clock"] {
        s.open_app(id);
        s.background_active_app();
    }
    s.open_app("mac");
    s.open_app("notes");
    match &s.active_app {
        ActiveApp::Notes(n) => {
            assert!(Arc::ptr_eq(n, &original));
            assert_eq!(n.lock().unwrap().selected, 3);
        }
        _ => panic!(),
    }
    assert_eq!(
        s.background_app_ids().collect::<Vec<_>>(),
        ["calculator", "settings", "clock", "mac"]
    );
    // Reopening the current app must not insert a duplicate or evict another app.
    s.open_app("notes");
    assert_eq!(s.running_apps.len(), 4);
    assert!(s.running_apps.contains_key("calculator"));
    s.background_active_app();
    assert_eq!(
        s.background_app_ids().collect::<Vec<_>>(),
        ["settings", "clock", "mac", "notes"]
    );
    assert!(!s.running_apps.contains_key("calculator"));
    s.open_app("notes");
    s.kill_active_app();
    assert_eq!(
        s.background_app_ids().collect::<Vec<_>>(),
        ["settings", "clock", "mac"]
    );
    assert_eq!(s.running_apps.len(), 3);
}

#[test]
fn closing_oldest_settings_resets_editor_and_timer_service_keeps_running() {
    let mut s = LauncherState::new();
    s.open_app("settings");
    s.manual_time_open = true;
    s.background_active_app();
    s.timer.toggle(0);
    for id in ["timer", "notes", "clock", "mac", "calculator"] {
        s.open_app(id);
        s.background_active_app();
    }
    assert_eq!(
        s.background_app_ids().collect::<Vec<_>>(),
        ["notes", "clock", "mac", "calculator"]
    );
    assert!(!s.manual_time_open);
    assert!(!s.running_apps.contains_key("timer"));
    s.tick(1000, 0);
    assert!(s.timer.is_running());
    assert_eq!(s.timer.remaining_ms, 25 * 60_000 - 1000);
    s.open_app("display");
    s.background_active_app();
    assert_eq!(
        s.running_apps.len(),
        4,
        "USB handoff is not a background app"
    );
}
#[test]
fn notes_scroll_and_selection_survive_background() {
    let mut s = LauncherState::new();
    s.open_app("notes");
    let notes = match &s.active_app {
        ActiveApp::Notes(n) => n.clone(),
        _ => panic!(),
    };
    {
        let mut n = notes.lock().unwrap();
        n.selected = 3;
        n.scroll.set_offset(150.0);
    }
    s.open_app("clock");
    s.open_app("notes");
    let restored = match &s.active_app {
        ActiveApp::Notes(n) => n.clone(),
        _ => panic!(),
    };
    assert!(Arc::ptr_eq(&notes, &restored));
    assert_eq!(restored.lock().unwrap().scroll.offset(), 150.0);
}
#[test]
fn settings_subview_back_keeps_settings_before_returning_to_desktop() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        s.open_app("settings");
        s.manual_time_open = true;
    }
    let size = Size::new(1024.0, 600.0);
    let mut ui = build_launcher_ui(state.clone(), size).create_render_object();
    ui.layout(&BoxConstraints::tight(size));
    assert!(ui.dispatch_touch(&TouchEvent::HardwareBack));
    {
        let s = state.lock().unwrap();
        assert!(!s.manual_time_open);
        assert!(matches!(s.active_app, ActiveApp::Settings));
    }
    let mut ui = build_launcher_ui(state.clone(), size).create_render_object();
    ui.layout(&BoxConstraints::tight(size));
    assert!(ui.dispatch_touch(&TouchEvent::HardwareBack));
    let s = state.lock().unwrap();
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert_eq!(s.page_controller.page(), 0);
    assert!(s.running_apps.contains_key("settings"));
}
#[test]
fn eight_desktop_icons_route_to_apps_and_system_commands() {
    use app_launcher::UiCommand;
    let size = Size::new(1024.0, 600.0);
    for (id, x, y) in [
        ("clock", 132.0, 278.0),
        ("timer", 348.0, 278.0),
        ("notes", 564.0, 278.0),
        ("calculator", 780.0, 278.0),
        ("mac", 132.0, 466.0),
        ("settings", 348.0, 466.0),
        ("display", 564.0, 466.0),
        ("screen", 780.0, 466.0),
    ] {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        if id == "display" {
            state.lock().unwrap().connected = true;
        }
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), size), size);
        app.step(&mut backend);
        tap(&mut app, &mut backend, &state, Point::new(x, y));
        let mut s = state.lock().unwrap();
        assert_eq!(s.page_controller.page(), 0);
        assert_eq!(s.total_pages, 2);
        match id {
            "display" => {
                assert_eq!(s.active_app.id(), Some("display"));
                assert!(
                    s.app_launch.frame(s.monotonic_ms).is_some(),
                    "USB holds the full-color bridge while Mac prepares"
                );
                assert!(matches!(
                    s.take_commands().as_slice(),
                    [UiCommand::StartDisplayTransition { duration_ms: 600 }]
                ));
            }
            "screen" => assert!(matches!(
                s.take_commands().as_slice(),
                [UiCommand::Screen(false)]
            )),
            _ => assert_eq!(s.active_app.id(), Some(id)),
        }
    }
}
#[test]
fn desktop_swipe_across_clock_ticks_never_launches_apps() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    backend.event(TouchEvent::Down(Point::new(132.0, 278.0)));
    step_ui(&mut app, &mut backend, &state);
    state.lock().unwrap().tick(1000, 1_800_000_000_000);
    app.request_rebuild();
    step_ui(&mut app, &mut backend, &state);
    backend.event(TouchEvent::Move(Point::new(103.0, 278.0)));
    step_ui(&mut app, &mut backend, &state);
    assert_eq!(state.lock().unwrap().page_controller.page(), 0);
    state.lock().unwrap().tick(2000, 1_800_000_001_000);
    app.request_rebuild();
    step_ui(&mut app, &mut backend, &state);
    backend.event(TouchEvent::Up(Point::new(103.0, 278.0)));
    step_ui(&mut app, &mut backend, &state);
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));

    // The last icon is a system action. An outward swipe must not turn the screen off.
    backend.event(TouchEvent::Down(Point::new(780.0, 466.0)));
    step_ui(&mut app, &mut backend, &state);
    backend.event(TouchEvent::Move(Point::new(917.0, 466.0)));
    step_ui(&mut app, &mut backend, &state);
    state.lock().unwrap().tick(3000, 1_800_000_002_000);
    app.request_rebuild();
    step_ui(&mut app, &mut backend, &state);
    backend.event(TouchEvent::Up(Point::new(917.0, 466.0)));
    step_ui(&mut app, &mut backend, &state);
    let mut s = state.lock().unwrap();
    assert_eq!(s.page_controller.page(), 0);
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert!(s.running_apps.is_empty());
    assert!(s.take_commands().is_empty());
}
#[test]
fn touch_back_side_dock_resume_and_kill_obey_app_lifecycle() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    tap(&mut app, &mut backend, &state, Point::new(780.0, 278.0));
    let first = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!("calculator icon did not open calculator"),
    };
    first.lock().unwrap().input_digit('7');
    tap(&mut app, &mut backend, &state, Point::new(48.0, 28.0));
    {
        let s = state.lock().unwrap();
        assert!(matches!(s.active_app, ActiveApp::Launcher));
        assert!(s.running_apps.contains_key("calculator"));
    }
    tap(&mut app, &mut backend, &state, Point::new(954.0, 310.0));
    let resumed = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!("side dock did not resume calculator"),
    };
    assert!(Arc::ptr_eq(&first, &resumed));
    assert_eq!(resumed.lock().unwrap().current_input, "7");
    tap(&mut app, &mut backend, &state, Point::new(976.0, 28.0));
    assert!(state.lock().unwrap().running_apps.is_empty());
    assert_eq!(
        state.lock().unwrap().recent_app_ids().collect::<Vec<_>>(),
        ["calculator"]
    );
    tap(&mut app, &mut backend, &state, Point::new(954.0, 310.0));
    let fresh = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!("calculator did not reopen"),
    };
    assert!(!Arc::ptr_eq(&first, &fresh));
    assert_eq!(fresh.lock().unwrap().current_input, "0");
}

#[test]
fn recent_shortcuts_do_not_keep_closed_instances_or_change_background_limits() {
    let mut s = LauncherState::new();
    assert_eq!(s.recent_app_ids().count(), 0);
    s.open_app("calculator");
    let weak = match &s.active_app {
        ActiveApp::Calculator(c) => Arc::downgrade(c),
        _ => panic!(),
    };
    s.kill_active_app();
    assert!(weak.upgrade().is_none());
    assert_eq!(s.recent_app_ids().collect::<Vec<_>>(), ["calculator"]);
    for id in ["clock", "notes", "settings", "timer", "mac"] {
        s.open_app(id);
        s.background_active_app();
    }
    assert_eq!(
        s.recent_app_ids().collect::<Vec<_>>(),
        ["notes", "settings", "timer", "mac"]
    );
    assert_eq!(s.running_apps.len(), 4);
    assert!(!s.running_apps.contains_key("clock"));
    s.open_app("settings");
    s.kill_active_app();
    assert_eq!(
        s.recent_app_ids().collect::<Vec<_>>(),
        ["notes", "timer", "mac", "settings"]
    );
    assert_eq!(s.running_apps.len(), 3);
    // USB is an openable recent entry, but its temporary page is never retained.
    s.open_app("display");
    s.kill_active_app();
    assert_eq!(
        s.recent_app_ids().collect::<Vec<_>>(),
        ["timer", "mac", "settings", "display"]
    );
    assert!(!s.running_apps.contains_key("display"));
    s.open_app("invalid");
    assert_eq!(s.recent_app_ids().count(), 4);
    assert_eq!(s.recent_app_ids().last(), Some("display"));
}
#[test]
fn desktop_notice_stays_between_cards_and_grid_and_keeps_apps_touchable() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    let before = backend.pixels.clone();
    state.lock().unwrap().notice = "便签与字体已同步".repeat(8);
    app.request_rebuild();
    step_ui(&mut app, &mut backend, &state);
    // A persistent sync result must not obscure either row of application icons.
    for y in 208..600 {
        assert_eq!(
            &before[y * 1024..(y + 1) * 1024],
            &backend.pixels[y * 1024..(y + 1) * 1024]
        );
    }
    tap(&mut app, &mut backend, &state, Point::new(564.0, 278.0));
    let s = state.lock().unwrap();
    assert!(matches!(s.active_app, ActiveApp::Notes(_)));
}
#[test]
fn shorter_button_sync_returns_to_a_usable_mac_page_immediately() {
    use p4desk_protocol::{Action, Button, Snapshot};
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        s.open_app("mac");
        s.mac_page = 3;
        s.apply_snapshot(Snapshot {
            buttons: vec![Button {
                id: "copy".into(),
                label: "复制".into(),
                action: Action::Shortcut {
                    key_code: 8,
                    modifiers: 0x100000,
                },
            }],
            ..Snapshot::default()
        });
    }
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    tap(&mut app, &mut backend, &state, Point::new(100.0, 155.0));
    let mut s = state.lock().unwrap();
    assert_eq!(s.mac_page, 0);
    assert!(matches!(
        s.take_commands().as_slice(),
        [app_launcher::UiCommand::Action(id)] if id == "copy"
    ));
}
#[test]
fn all_pad_apps_rasterize_1024_600_and_external_ticks_rebuild() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    for id in app_launcher::APP_IDS {
        state.lock().unwrap().open_app(id);
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), size), size);
        app.step(&mut backend);
        assert_eq!(backend.pixels.len(), 1024 * 600);
        assert!(backend.pixels.iter().any(|p| *p != 0));
        state.lock().unwrap().tick(1000, 1_800_000_000_000);
        app.request_rebuild();
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(backend.frames, 2);
    }
}
