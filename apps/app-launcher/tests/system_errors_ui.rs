//! Typed asynchronous failures use a modal, without interrupting the current app.
use app_launcher::{
    build_launcher_ui,
    headless::HeadlessBackend,
    launcher_state::{
        ActiveApp, DialogContent, LauncherState, SystemError, SystemNotice, UiCommand,
    },
    launcher_ui::system_error_dialog_bounds,
};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, BoxConstraints, Point, Rect, Size, TouchEvent};

fn fixture(id: &str) -> Arc<Mutex<LauncherState>> {
    let mut state = LauncherState::new();
    state.open_app(id);
    state.take_commands();
    Arc::new(Mutex::new(state))
}
fn frame(state: &Arc<Mutex<LauncherState>>) -> (App, HeadlessBackend) {
    let mut backend = HeadlessBackend::new(1024, 600);
    let size = Size::new(1024.0, 600.0);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    (app, backend)
}
fn tap(
    app: &mut App,
    backend: &mut HeadlessBackend,
    state: &Arc<Mutex<LauncherState>>,
    x: f32,
    y: f32,
) {
    for event in [
        TouchEvent::Down(Point::new(x, y)),
        TouchEvent::Up(Point::new(x, y)),
    ] {
        backend.event(event);
        app.step_with_builder(backend, |size| build_launcher_ui(state.clone(), size));
    }
}
#[test]
fn typed_sync_failure_is_visible_over_each_page_that_hides_ordinary_notices() {
    let bounds = system_error_dialog_bounds(Size::new(1024.0, 600.0));
    assert_eq!(bounds.x + bounds.width * 0.5, 512.0);
    assert_eq!(bounds.y + bounds.height * 0.5, 300.0);
    for name in ["clock", "timer", "calculator", "sub2api-monitor"] {
        let state = fixture(name);
        let (_, before) = frame(&state);
        state.lock().unwrap().show_error(SystemError::SyncFailed);
        let (_, after) = frame(&state);
        let different = (160..440)
            .flat_map(|y| (232..792).map(move |x| y * 1024 + x))
            .filter(|&i| before.pixels[i] != after.pixels[i])
            .count();
        assert!(different > 10_000, "typed failure was hidden on {name}");
    }
}
#[test]
fn the_close_button_dismisses_the_error_and_preserves_the_running_app() {
    let state = fixture("clock");
    let id = state.lock().unwrap().show_error(SystemError::SyncFailed);
    let (mut app, mut backend) = frame(&state);
    tap(&mut app, &mut backend, &state, 512.0, 391.0);
    let mut state = state.lock().unwrap();
    assert!(state.error_dialog.is_none());
    assert!(matches!(state.active_app, ActiveApp::Clock));
    assert!(
        matches!(state.take_commands().as_slice(), [UiCommand::DismissError { id: actual }] if *actual == id)
    );
}
#[test]
fn the_x_button_remains_clickable_over_an_active_launch_animation() {
    let state = fixture("clock");
    {
        let mut state = state.lock().unwrap();
        state.launch_app("timer", Rect::from_ltwh(320.0, 210.0, 120.0, 120.0));
        state.show_error(SystemError::OperationFailed);
        state.take_commands();
        assert!(state.app_launch.frame(state.monotonic_ms).is_some());
    }
    let (mut app, mut backend) = frame(&state);
    tap(&mut app, &mut backend, &state, 752.0, 200.0);
    let mut state = state.lock().unwrap();
    assert!(state.error_dialog.is_none());
    assert!(matches!(state.active_app, ActiveApp::Timer));
    assert!(matches!(
        state.take_commands().as_slice(),
        [UiCommand::DismissError { .. }]
    ));
}
#[test]
fn modal_blocks_the_background_and_hardware_back_only_closes_the_error() {
    let state = fixture("file-manager");
    state.lock().unwrap().show_error(SystemError::SyncFailed);
    let (mut app, mut backend) = frame(&state);
    // Underlying Home is covered by the modal's input barrier.
    tap(&mut app, &mut backend, &state, 48.0, 30.0);
    assert!(matches!(state.lock().unwrap().active_app, ActiveApp::Files));
    assert!(state.lock().unwrap().error_dialog.is_some());
    assert!(state.lock().unwrap().take_commands().is_empty());
    backend.event(TouchEvent::HardwareBack);
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    let mut state = state.lock().unwrap();
    assert!(state.error_dialog.is_none());
    assert!(matches!(state.active_app, ActiveApp::Files));
    assert!(matches!(
        state.take_commands().as_slice(),
        [UiCommand::DismissError { .. }]
    ));
}
#[test]
fn a_stale_dialog_click_cannot_dismiss_a_newer_error() {
    let state = fixture("clock");
    let previous = state.lock().unwrap().show_error(SystemError::SyncFailed);
    let size = Size::new(1024.0, 600.0);
    let mut stale = build_launcher_ui(state.clone(), size).create_render_object();
    stale.layout(&BoxConstraints::tight(size));
    let next = state.lock().unwrap().show_error(SystemError::SaveFailed);
    assert_ne!(previous, next);
    for event in [
        TouchEvent::Down(Point::new(512.0, 391.0)),
        TouchEvent::Up(Point::new(512.0, 391.0)),
    ] {
        stale.dispatch_touch(&event);
    }
    let mut state = state.lock().unwrap();
    assert_eq!(state.error_dialog.unwrap().id, next);
    assert!(state.take_commands().is_empty());
}
#[test]
fn repeated_visible_failures_share_an_id_but_a_new_occurrence_can_be_dismissed() {
    let mut state = LauncherState::new();
    let first = state.show_error(SystemError::SyncFailed);
    assert_eq!(state.show_error(SystemError::SyncFailed), first);
    assert!(state.dismiss_error(first));
    let second = state.show_error(SystemError::SyncFailed);
    assert_ne!(first, second);
    assert!(!state.dismiss_error(first));
    assert_eq!(state.error_dialog.unwrap().id, second);
}
#[test]
fn notice_text_does_not_implicitly_create_an_error_dialog() {
    let state = fixture("clock");
    let (_, ordinary) = frame(&state);
    state.lock().unwrap().notice = "同步失败，保留已有便签和字体".into();
    let (_, with_notice) = frame(&state);
    assert!(state.lock().unwrap().error_dialog.is_none());
    assert!(ordinary.pixels == with_notice.pixels);
}
#[test]
fn sync_success_is_closable_and_leaves_the_original_page_without_a_banner() {
    let state = fixture("clock");
    let (_, original) = frame(&state);
    {
        let mut state = state.lock().unwrap();
        state.notice = "便签与字体已同步".into();
        state.show_notice(SystemNotice::NotesSynced);
        assert!(state.notice.is_empty());
        assert!(state.error_dialog.is_none());
        assert_eq!(
            state.active_dialog().unwrap().content,
            DialogContent::Notice(SystemNotice::NotesSynced)
        );
    }
    let (mut app, mut backend) = frame(&state);
    assert!(original.pixels != backend.pixels);
    tap(&mut app, &mut backend, &state, 512.0, 391.0);
    assert!(state.lock().unwrap().active_dialog().is_none());
    let (_, closed) = frame(&state);
    assert!(original.pixels == closed.pixels);
}
#[test]
fn success_waits_for_an_unread_error_and_dismiss_ids_do_not_cross_messages() {
    let mut state = LauncherState::new();
    let error = state.show_error(SystemError::SaveFailed);
    let success = state.show_notice(SystemNotice::NotesSynced);
    assert_eq!(state.active_dialog().unwrap().id, error);
    assert_eq!(state.error_dialog.unwrap().error, SystemError::SaveFailed);
    assert!(state.dismiss_dialog(error));
    assert_eq!(state.active_dialog().unwrap().id, success);
    assert!(state.error_dialog.is_none());
    assert!(!state.dismiss_error(error));
    assert_eq!(
        state.active_dialog().unwrap().content,
        DialogContent::Notice(SystemNotice::NotesSynced)
    );
    assert!(state.dismiss_dialog(success));
    assert!(state.active_dialog().is_none());
}
#[test]
fn an_interrupted_notice_is_preserved_with_a_new_id_after_the_error() {
    let mut state = LauncherState::new();
    let notice = state.show_notice(SystemNotice::UnsupportedWifiSecurity);
    let error = state.show_error(SystemError::OperationFailed);
    assert!(!state.dismiss_dialog(notice));
    assert!(state.dismiss_dialog(error));
    let restored = state.active_dialog().unwrap();
    assert_ne!(restored.id, notice);
    assert_eq!(
        restored.content,
        DialogContent::Notice(SystemNotice::UnsupportedWifiSecurity)
    );
    assert!(!state.dismiss_dialog(notice));
    assert!(state.dismiss_dialog(restored.id));
}
#[test]
fn an_unrelated_result_does_not_clear_the_usb_handoff_page_state() {
    let mut state = LauncherState::new();
    state.connected = true;
    state.open_app("display");
    state.notice = "等待 Mac 应用".into();
    let id = state.show_notice(SystemNotice::NotesSynced);
    assert_eq!(state.notice, "等待 Mac 应用");
    assert!(matches!(state.active_app, ActiveApp::DisplaySetup));
    assert!(state.dismiss_dialog(id));
    assert_eq!(state.notice, "等待 Mac 应用");
}
