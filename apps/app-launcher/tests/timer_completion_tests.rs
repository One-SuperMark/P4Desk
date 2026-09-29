use app_launcher::headless::HeadlessBackend;
use app_launcher::timer::{Phase, TimerKind};
use app_launcher::timer_completion::COMPLETION_DURATION_MS;
use app_launcher::{build_launcher_ui, LauncherState};
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const SIZE: Size = Size::new(1024.0, 600.0);
const DEADLINE: u64 = 60_000;
fn countdown() -> LauncherState {
    let mut s = LauncherState::new();
    s.open_app("timer");
    s.timer.set_countdown(1);
    s.timer.toggle(0);
    s.tick(DEADLINE - 10, 0);
    s
}
fn render(state: Arc<Mutex<LauncherState>>) -> Vec<u16> {
    let mut backend = HeadlessBackend::new(1024, 600);
    App::new(build_launcher_ui(state, SIZE), SIZE).step(&mut backend);
    backend.pixels
}
#[test]
fn completion_is_finite_throttled_and_never_rebuilds_each_animation_frame() {
    let mut s = countdown();
    s.tick(DEADLINE, 0);
    assert!(s.timer.finished);
    assert_eq!(s.timer.finished_at_ms(), Some(DEADLINE));
    assert_eq!(s.timer_completion.progress(DEADLINE), Some(0.0));
    let rect = s.take_timer_animation_dirty(SIZE).unwrap();
    assert_eq!(rect, Rect::from_ltwh(0.0, 0.0, 1024.0, 600.0));
    assert!(s.take_timer_animation_dirty(SIZE).is_none());
    let revision = s.revision;
    assert!(!s.tick(DEADLINE + 10, 0));
    assert!(s.take_timer_animation_dirty(SIZE).is_none());
    assert!(!s.tick(DEADLINE + 33, 0));
    assert!(s.take_timer_animation_dirty(SIZE).is_some());
    assert_eq!(s.revision, revision);
    s.tick(DEADLINE + COMPLETION_DURATION_MS + 200, 0);
    assert!(
        s.take_timer_animation_dirty(SIZE).is_some(),
        "settled frame must erase the halo even after a delayed loop"
    );
    assert!(s.take_timer_animation_dirty(SIZE).is_none());
    assert!(s
        .timer_completion
        .progress(DEADLINE + COMPLETION_DURATION_MS + 200)
        .is_none());
    s.tick(DEADLINE + 10_000, 0);
    assert!(s.take_timer_animation_dirty(SIZE).is_none());
    assert_eq!(s.timer.kind, TimerKind::Countdown);
    assert_eq!(s.timer.remaining_ms, 0);
    assert!(!s.timer.is_running());
}
#[test]
fn every_pomodoro_phase_uses_the_same_completion_trigger_without_extra_cycles() {
    for phase in Phase::ALL {
        let mut s = LauncherState::new();
        s.open_app("timer");
        s.timer.set_phase(phase);
        s.timer.toggle(0);
        let deadline = s.timer.duration_ms();
        s.tick(deadline, 0);
        assert!(s.timer_completion.progress(deadline + 300).is_some());
        assert_eq!(s.timer.completed_cycles, u32::from(phase == Phase::Focus));
        s.tick(deadline + 100, 2_000_000_000_000);
        assert_eq!(s.timer.finished_at_ms(), Some(deadline));
        assert_eq!(
            s.timer_completion.progress(deadline + 300),
            Some(300.0 / COMPLETION_DURATION_MS as f32)
        );
    }
}
#[test]
fn hidden_completed_timers_do_not_replay_and_reset_or_restart_cancels() {
    for hidden in 0..3 {
        let mut s = countdown();
        match hidden {
            0 => s.background_active_app(),
            1 => s.mode = Mode::Display,
            _ => s.settings.screen_on = false,
        }
        s.tick(DEADLINE, 0);
        assert!(s.timer.finished);
        assert!(s.timer_completion.progress(DEADLINE).is_none());
        s.mode = Mode::Pad;
        s.settings.screen_on = true;
        s.open_app("timer");
        s.tick(DEADLINE + 100, 0);
        assert!(s.take_timer_animation_dirty(SIZE).is_none());
    }
    for action in 0..4 {
        let mut s = countdown();
        s.tick(DEADLINE, 0);
        match action {
            0 => s.timer.reset(),
            1 => s.timer.toggle(DEADLINE),
            2 => s.timer.set_phase(Phase::Break),
            _ => s.background_active_app(),
        }
        s.tick(DEADLINE + 100, 0);
        assert!(s.timer_completion.progress(DEADLINE + 100).is_none());
        assert!(s.take_timer_animation_dirty(SIZE).is_none());
    }
    let mut s = countdown();
    s.tick(DEADLINE + COMPLETION_DURATION_MS + 200, 0);
    assert!(s.timer.finished);
    assert!(
        s.take_timer_animation_dirty(SIZE).is_none(),
        "late detection must not play an outdated finish"
    );
}
#[test]
fn live_effect_matches_fresh_renders_and_cleans_up_the_entire_screen_exactly() {
    let state = Arc::new(Mutex::new(countdown()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    state.lock().unwrap().tick(DEADLINE, 0);
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    let settled = {
        let mut s = countdown();
        s.tick(DEADLINE + COMPLETION_DURATION_MS + 200, 0);
        render(Arc::new(Mutex::new(s)))
    };
    let mut changed = 0;
    for elapsed in [
        33,
        130,
        260,
        500,
        560,
        850,
        1200,
        1520,
        1580,
        1800,
        2100,
        2400,
        2700,
        3000,
        COMPLETION_DURATION_MS - 1,
        COMPLETION_DURATION_MS,
        COMPLETION_DURATION_MS + 400,
    ] {
        let dirty = {
            let mut s = state.lock().unwrap();
            s.tick(DEADLINE + elapsed, 0);
            s.take_timer_animation_dirty(SIZE)
        };
        let frames = backend.frames;
        if let Some(r) = dirty {
            app.mark_dirty(r);
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(backend.frames, frames + usize::from(dirty.is_some()));
        assert!(
            backend.pixels == render(state.clone()),
            "live effect differs from fresh render at {elapsed} ms"
        );
        if elapsed < COMPLETION_DURATION_MS && backend.pixels != settled {
            changed += 1;
        }
        if elapsed >= COMPLETION_DURATION_MS {
            assert!(
                backend.pixels == settled,
                "final effect must leave no stale glow pixels"
            );
        }
    }
    assert!(changed >= 6, "effect must have visible intermediate frames");
}

fn completed_at(elapsed: u64) -> Arc<Mutex<LauncherState>> {
    let mut state = countdown();
    state.tick(DEADLINE, 0);
    state.tick(DEADLINE + elapsed, 0);
    Arc::new(Mutex::new(state))
}

#[test]
fn completion_badge_pops_from_bottom_with_a_small_overshoot_then_settles_at_center() {
    let base = render(completed_at(0));
    fn extent(pixels: &[u16], base: &[u16]) -> (usize, usize) {
        let theme = Color::from_hex(0xf18b77).to_rgb565();
        let rows: Vec<_> = (0..600)
            .filter(|y| {
                (477..547)
                    .filter(|x| pixels[y * 1024 + x] == theme && base[y * 1024 + x] != theme)
                    .count()
                    > 20
            })
            .collect();
        (
            *rows.first().expect("themed badge visible"),
            *rows.last().unwrap(),
        )
    }
    let lower = extent(&render(completed_at(90)), &base);
    let over = extent(&render(completed_at(400)), &base);
    let center = extent(&render(completed_at(560)), &base);
    assert!(
        (lower.0 + lower.1) / 2 > 400,
        "badge must enter from bottom: {lower:?}"
    );
    assert!(
        (over.0 + over.1) / 2 < 295,
        "small spring overshoot required: {over:?}"
    );
    assert!(
        ((center.0 + center.1) as i32 / 2 - 300).abs() <= 3,
        "badge must settle at center: {center:?}"
    );
}

#[test]
fn colored_circle_grows_before_covering_navigation_and_uses_current_button_theme() {
    let base = render(completed_at(0));
    let partial = render(completed_at(900));
    for (x, y) in [
        (0, 0),
        (1023, 0),
        (0, 599),
        (1023, 599),
        (48, 28),
        (976, 28),
    ] {
        assert_eq!(
            partial[y * 1024 + x],
            base[y * 1024 + x],
            "expand must grow locally first"
        );
    }
    assert_eq!(
        partial[360 * 1024 + 512],
        Color::from_hex(0xf18b77).to_rgb565()
    );
    for (phase, theme) in [
        (Phase::Focus, 0xf18b77),
        (Phase::Break, 0x8ed4b5),
        (Phase::LongBreak, 0x9dbded),
    ] {
        let mut s = LauncherState::new();
        s.open_app("timer");
        s.timer.set_phase(phase);
        s.timer.toggle(0);
        let deadline = s.timer.duration_ms();
        s.tick(deadline, 0);
        s.tick(deadline + 1580, 0);
        let pixels = render(Arc::new(Mutex::new(s)));
        for (x, y) in [
            (0, 0),
            (1023, 0),
            (0, 599),
            (1023, 599),
            (48, 28),
            (976, 28),
            (512, 478),
            (512, 274),
        ] {
            assert_eq!(pixels[y * 1024 + x], Color::from_hex(theme).to_rgb565());
        }
        assert_eq!(
            pixels[315 * 1024 + 506],
            Color::WHITE.to_rgb565(),
            "white completion check in themed circle"
        );
    }
}

#[test]
fn transparent_circle_reveals_center_then_edges_and_leaves_no_fullscreen_overlay() {
    let base = render(completed_at(COMPLETION_DURATION_MS + 10));
    let reveal = render(completed_at(2200));
    for y in 250..350 {
        for x in 412..612 {
            assert_eq!(
                reveal[y * 1024 + x],
                base[y * 1024 + x],
                "center must be a transparent window, not a faded theme"
            );
        }
    }
    let theme = Color::from_hex(0xf18b77).to_rgb565();
    for (x, y) in [
        (0, 0),
        (1023, 0),
        (0, 599),
        (1023, 599),
        (48, 28),
        (976, 28),
    ] {
        assert_eq!(
            reveal[y * 1024 + x],
            theme,
            "outside remains solid until the clear circle arrives"
        );
    }
    assert_eq!(
        render(completed_at(3055)),
        base,
        "the whole screen must be restored by the end of the reveal"
    );
}

#[test]
fn tap_dismisses_overlay_without_starting_hidden_timer_and_next_tap_starts_normally() {
    let state = completed_at(500);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    let button = Point::new(512.0, 478.0);
    for event in [TouchEvent::Down(button), TouchEvent::Up(button)] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    {
        let state = state.lock().unwrap();
        assert!(state.timer.finished);
        assert!(!state.timer.is_running());
        assert!(state
            .timer_completion
            .progress(state.monotonic_ms)
            .is_none());
    }
    for event in [TouchEvent::Down(button), TouchEvent::Up(button)] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(state.lock().unwrap().timer.is_running());
}

#[test]
fn held_or_canceled_gestures_cannot_fall_through_after_the_animation_expires() {
    for cancel in [false, true] {
        let state = completed_at(500);
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
        app.step(&mut backend);
        let button = Point::new(512.0, 478.0);
        backend.event(TouchEvent::Down(button));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        if cancel {
            backend.event(TouchEvent::Cancel);
            app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        }
        {
            let mut s = state.lock().unwrap();
            s.tick(DEADLINE + COMPLETION_DURATION_MS + 100, 0);
            app.mark_dirty(s.take_timer_animation_dirty(SIZE).unwrap());
        }
        app.request_rebuild();
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        backend.event(TouchEvent::Up(button));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert!(!state.lock().unwrap().timer.is_running());
        assert!(state.lock().unwrap().timer.finished);
    }
    let state = completed_at(500);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    for event in [
        TouchEvent::Down(Point::new(400.0, 300.0)),
        TouchEvent::Move(Point::new(500.0, 300.0)),
        TouchEvent::Up(Point::new(400.0, 300.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(
        state
            .lock()
            .unwrap()
            .timer_completion
            .progress(DEADLINE + 500)
            .is_some(),
        "a drag is not a dismiss tap"
    );
}

#[test]
fn completion_consumes_release_of_a_button_pressed_before_the_deadline() {
    let state = Arc::new(Mutex::new(countdown()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    let button = Point::new(512.0, 478.0);
    backend.event(TouchEvent::Down(button));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    {
        let mut state = state.lock().unwrap();
        state.tick(DEADLINE + 100, 0);
        app.mark_dirty(state.take_timer_animation_dirty(SIZE).unwrap());
    }
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    backend.event(TouchEvent::Up(button));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    let state = state.lock().unwrap();
    assert!(state.timer.finished);
    assert!(!state.timer.is_running());
    assert!(state
        .timer_completion
        .progress(state.monotonic_ms)
        .is_some());
}
