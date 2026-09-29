use app_launcher::headless::HeadlessBackend;
use app_launcher::pomodoro_ui::{
    timer_layout, timer_mode_bounds, timer_mode_keys, TimerAction as A,
};
use app_launcher::timer::{Phase, TimerKind};
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

struct Harness {
    state: Arc<Mutex<LauncherState>>,
    app: App,
    backend: HeadlessBackend,
}
impl Harness {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        state.lock().unwrap().open_app("timer");
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
    fn touch(&mut self, event: TouchEvent) {
        self.backend.event(event);
        self.app.step_with_builder(&mut self.backend, |size| {
            build_launcher_ui(self.state.clone(), size)
        });
    }
    fn tap_point(&mut self, point: Point) {
        self.touch(TouchEvent::Down(point));
        self.touch(TouchEvent::Up(point));
    }
    fn point(&self, action: A) -> Point {
        let r = if matches!(action, A::Kind(_)) {
            let bounds = timer_mode_bounds(1024.0);
            timer_mode_keys(Size::new(bounds.width, bounds.height))
                .into_iter()
                .find(|key| key.action == action)
                .unwrap()
                .rect
                .shift(Offset::new(bounds.x, bounds.y))
        } else {
            timer_layout(&self.state.lock().unwrap().timer, Size::new(976.0, 506.0))
                .key(action)
                .unwrap()
                .rect
                .shift(Offset::new(24.0, 78.0))
        };
        Point::new(r.x + r.width * 0.5, r.y + r.height * 0.5)
    }
    fn tap(&mut self, action: A) {
        self.tap_point(self.point(action));
    }
    fn tick(&mut self, now: u64, unix: i64) {
        self.state.lock().unwrap().tick(now, unix);
        self.app.request_rebuild();
        self.app.step_with_builder(&mut self.backend, |size| {
            build_launcher_ui(self.state.clone(), size)
        });
    }
}
#[test]
fn touch_start_pause_resume_reset_and_phase_selection() {
    let mut h = Harness::new();
    h.tap(A::Toggle);
    h.tick(35_000, 0);
    h.tap(A::Phase(Phase::Focus));
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 1_465_000);
    assert!(h.state.lock().unwrap().timer.is_running());
    h.tap(A::Toggle);
    h.tick(65_000, 0);
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 1_465_000);
    h.tap(A::Toggle);
    h.tick(75_000, 0);
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 1_455_000);
    h.tap(A::Reset);
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 1_500_000);
    assert!(!h.state.lock().unwrap().timer.is_running());
    h.tap(A::Phase(Phase::LongBreak));
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 900_000);
    h.tap(A::Toggle);
    h.tap(A::Next);
    let state = h.state.lock().unwrap();
    assert_eq!(state.timer.phase, Phase::Focus);
    assert!(!state.timer.is_running());
    assert_eq!(state.timer.completed_cycles, 0);
}
#[test]
fn countdown_presets_and_mode_switches_do_not_reset_on_selected_tab() {
    let mut h = Harness::new();
    h.tap(A::Kind(TimerKind::Countdown));
    h.tap(A::Preset(10));
    h.tap(A::Toggle);
    h.tick(9_000, 0);
    h.tap(A::Kind(TimerKind::Countdown));
    h.tap(A::Preset(10));
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 591_000);
    assert!(h.state.lock().unwrap().timer.is_running());
    h.tap(A::Preset(1));
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 60_000);
    assert!(!h.state.lock().unwrap().timer.is_running());
    h.tap(A::Kind(TimerKind::Pomodoro));
    assert_eq!(h.state.lock().unwrap().timer.phase, Phase::Focus);
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 1_500_000);
}
#[test]
fn ten_second_preset_survives_mode_changes_and_finishes_at_the_real_deadline() {
    let mut h = Harness::new();
    h.tap(A::Kind(TimerKind::Countdown));
    h.tap(A::SecondsPreset(10));
    assert_eq!(h.state.lock().unwrap().timer.display(), "00:10");
    h.tap(A::Toggle);
    h.tick(2_400, 0);
    h.tap(A::SecondsPreset(10));
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 7_600);
    assert!(h.state.lock().unwrap().timer.is_running());
    h.tick(9_999, 0);
    assert_eq!(h.state.lock().unwrap().timer.display(), "00:01");
    assert!(!h.state.lock().unwrap().timer.finished);
    h.tick(10_000, 0);
    {
        let s = h.state.lock().unwrap();
        assert_eq!(s.timer.display(), "00:00");
        assert_eq!(s.timer.finished_at_ms(), Some(10_000));
        assert!(s.timer_completion.progress(10_000).is_some());
    }
    h.tick(
        10_000 + app_launcher::timer_completion::COMPLETION_DURATION_MS,
        0,
    );
    h.tap(A::Reset);
    assert_eq!(h.state.lock().unwrap().timer.display(), "00:10");
    h.tap(A::Kind(TimerKind::Pomodoro));
    h.tap(A::Kind(TimerKind::Countdown));
    assert_eq!(h.state.lock().unwrap().timer.countdown_ms, 10_000);
    assert_eq!(h.state.lock().unwrap().timer.display(), "00:10");
}
#[test]
fn home_and_display_mode_keep_timer_running_through_time_sync_and_return() {
    let mut h = Harness::new();
    h.tap(A::Toggle);
    h.tap_point(Point::new(48.0, 28.0));
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    h.tick(60_000, 1_800_000_000_000);
    h.state.lock().unwrap().mode = p4desk_protocol::Mode::Display;
    h.tick(90_000, 1_900_000_000_000);
    h.state.lock().unwrap().mode = p4desk_protocol::Mode::Pad;
    h.state.lock().unwrap().open_app("timer");
    h.tick(120_000, 0);
    assert_eq!(h.state.lock().unwrap().timer.display(), "23:00");
    assert!(h.state.lock().unwrap().timer.is_running());
    h.tap_point(Point::new(976.0, 28.0));
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    assert!(!h.state.lock().unwrap().running_apps.contains_key("timer"));
    // The timer is a shared service, preserving the project's existing close semantics.
    h.tick(150_000, 0);
    assert_eq!(h.state.lock().unwrap().timer.display(), "22:30");
}
#[test]
fn canceled_touch_cannot_start_or_skip_and_controls_do_not_overlap() {
    let mut h = Harness::new();
    let point = h.point(A::Toggle);
    h.touch(TouchEvent::Down(point));
    h.touch(TouchEvent::Cancel);
    h.touch(TouchEvent::Up(point));
    assert!(!h.state.lock().unwrap().timer.is_running());
    h.touch(TouchEvent::Down(point));
    h.touch(TouchEvent::Move(Point::new(1.0, 1.0)));
    h.touch(TouchEvent::Up(Point::new(1.0, 1.0)));
    assert!(!h.state.lock().unwrap().timer.is_running());
    for kind in [TimerKind::Pomodoro, TimerKind::Countdown] {
        h.state.lock().unwrap().timer.kind = kind;
        let layout = timer_layout(&h.state.lock().unwrap().timer, Size::new(976.0, 506.0));
        for (i, key) in layout.keys.iter().enumerate() {
            assert!(
                key.rect.x >= 0.0
                    && key.rect.y >= 0.0
                    && key.rect.right() <= 976.0
                    && key.rect.bottom() <= 506.0
            );
            assert!(key.rect.height >= 44.0);
            for other in &layout.keys[i + 1..] {
                assert!(key.rect.intersect(&other.rect).is_none());
            }
        }
    }
}
#[test]
fn completed_fourth_session_has_a_working_long_break_start_button() {
    let mut h = Harness::new();
    h.state.lock().unwrap().timer.completed_cycles = 3;
    h.tap(A::Toggle);
    h.tick(1_500_000, 0);
    assert_eq!(h.state.lock().unwrap().timer.completed_cycles, 4);
    assert_eq!(
        app_launcher::pomodoro_ui::primary_label(&h.state.lock().unwrap().timer),
        "开始长休息"
    );
    h.tick(
        1_500_000 + app_launcher::timer_completion::COMPLETION_DURATION_MS,
        0,
    );
    h.tap(A::Toggle);
    assert_eq!(h.state.lock().unwrap().timer.phase, Phase::LongBreak);
    assert_eq!(h.state.lock().unwrap().timer.remaining_ms, 900_000);
    assert!(h.state.lock().unwrap().timer.is_running());
}
