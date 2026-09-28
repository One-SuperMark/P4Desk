use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

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
fn navigation_hits_actual_landscape_coordinates() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    // The settings navigation button is below the 480px boundary of the old board layout.
    backend.event(TouchEvent::Down(Point::new(78.0, 457.0)));
    backend.event(TouchEvent::Up(Point::new(78.0, 457.0)));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Settings
    ));
    assert_eq!(backend.frames, 2);
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
