use app_launcher::{build_launcher_ui, headless::HeadlessBackend, session::Session, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Rect, Size};

const CLOCK: i64 = 1_791_216_000_000;
const SIZE: Size = Size::new(1024.0, 600.0);

fn monitor() -> LauncherState {
    let mut state = LauncherState::new();
    state.open_app("sub2api-monitor");
    state.usage.config =
        Some(app_launcher::usage::Config::new("monitor.example", "test-key").unwrap());
    state.tick(0, CLOCK);
    state
}

#[test]
fn monitor_preserves_wall_time_without_rebuilding_each_second() {
    let mut state = monitor();
    let revision = state.revision;
    for second in 1..60 {
        assert!(!state.tick(second * 1_000, CLOCK + second as i64 * 1_000));
        assert_eq!(state.revision, revision);
        assert_eq!(state.unix_ms, CLOCK + second as i64 * 1_000);
    }
    assert!(state.clock.ends_with(":59"));
    assert!(state.tick(60_000, CLOCK + 60_000));
    assert_eq!(state.revision, revision + 1);
    assert!(state.clock.ends_with(":00"));
    assert!(!state.tick(61_000, CLOCK + 61_000));
}

#[test]
fn hidden_timer_and_session_continue_without_monitor_repaints() {
    let mut state = monitor();
    state.timer.set_countdown_seconds(10);
    state.timer.toggle(0);
    let revision = state.revision;
    for second in 1..10 {
        assert!(!state.tick(second * 1_000, CLOCK + second as i64 * 1_000));
        assert_eq!(state.timer.remaining_ms, 10_000 - second * 1_000);
        let checkpoint = Session::capture(&state);
        assert_eq!(checkpoint.timer.remaining_ms, state.timer.remaining_ms);
        assert!(checkpoint.timer.was_running);
        assert_eq!(state.revision, revision);
    }
    assert!(
        state.tick(10_000, CLOCK + 10_000),
        "completion is a real state event"
    );
    assert!(state.timer.finished);
    assert!(!state.timer.is_running());
    assert_eq!(state.timer.finished_at_ms(), Some(10_000));
    assert!(state.timer_completion.progress(10_000).is_none());
    assert!(!state.tick(11_000, CLOCK + 11_000));
    let checkpoint = Session::capture(&state);
    assert!(checkpoint.timer.finished);
    assert_eq!(checkpoint.timer.remaining_ms, 0);
    assert!(!checkpoint.timer.was_running);
}

#[test]
fn clock_sync_validity_changes_and_explicit_events_update_immediately() {
    let mut state = monitor();
    assert!(!state.tick(1_000, CLOCK + 1_000));
    assert!(
        state.tick(1_010, CLOCK - 1_000),
        "clock corrections are immediate"
    );
    assert!(state.tick(1_020, 0), "losing valid wall time is immediate");
    assert!(!state.time_valid);
    assert!(
        state.tick(1_030, CLOCK + 2_000),
        "NTP restoration is immediate"
    );
    state.last_time_refresh();
    let revision = state.revision;
    assert!(state.tick(1_040, CLOCK + 2_000));
    assert!(state.revision > revision);
    let revision = state.revision;
    state.changed(); // Network results and touch events use this same path.
    assert_eq!(state.revision, revision + 1);
    assert!(!state.tick(1_050, CLOCK + 2_000));
    assert_eq!(
        state.revision,
        revision + 1,
        "tick retains the pending real event"
    );
}

#[test]
fn launches_and_other_apps_retain_their_existing_cadence() {
    let mut state = monitor();
    state.tick(900, CLOCK + 900);
    state.launch_app("clock", Rect::from_ltwh(30.0, 30.0, 60.0, 60.0));
    state.launch_app("sub2api-monitor", Rect::from_ltwh(110.0, 30.0, 60.0, 60.0));
    assert!(state.app_launch.frame(1_000).is_some());
    assert!(state.tick(1_000, CLOCK + 1_000));
    assert!(state.take_launch_animation_dirty(SIZE).is_some());
    assert!(!state.tick(1_016, CLOCK + 1_016));
    assert!(state.take_launch_animation_dirty(SIZE).is_some());
    state.tick(1_900, CLOCK + 1_900);
    assert!(
        state.take_launch_animation_dirty(SIZE).is_some(),
        "final launch frame remains dirty"
    );
    assert!(!state.tick(2_000, CLOCK + 2_000));
    for app in ["clock", "timer", "calculator", "settings"] {
        state.open_app(app);
        assert!(state.tick(3_000, CLOCK + 3_000));
        assert!(state.tick(4_000, CLOCK + 4_000));
        state.tick(2_000, CLOCK + 2_000);
    }
}

#[test]
fn idle_monitor_does_not_submit_lcd_frames_between_minute_boundaries() {
    let state = Arc::new(Mutex::new(monitor()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    let initial_frames = backend.frames;
    for second in 1..60 {
        if state
            .lock()
            .unwrap()
            .tick(second * 1_000, CLOCK + second as i64 * 1_000)
        {
            app.request_rebuild();
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert_eq!(backend.frames, initial_frames);
    assert!(state.lock().unwrap().tick(60_000, CLOCK + 60_000));
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    assert_eq!(backend.frames, initial_frames + 1);
}
