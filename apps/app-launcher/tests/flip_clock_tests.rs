use app_launcher::flip_clock::FLIP_DURATION_MS;
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const SIZE: Size = Size::new(1024.0, 600.0);
const DAY: i64 = 1_800_000_000_000 / 86_400_000 * 86_400_000;
const REFERENCE_TIME: i64 = DAY + (20 * 3600 + 6 * 60 + 42) * 1000;

fn clock_state() -> LauncherState {
    let mut state = LauncherState::new();
    state.settings.timezone_minutes = 0;
    state.tick(10, REFERENCE_TIME + 990);
    state.open_app("clock");
    state
}

#[test]
fn wall_second_changes_immediately_and_animation_never_rebuilds_every_frame() {
    let mut state = clock_state();
    let before = state.flip_clock.sample(10);
    assert_eq!(before.to.period, Some("PM"));
    assert_eq!(before.to.cards[1..], [[0, 6], [4, 2]]);
    // The wall second changes while monotonic time is still in the same second.
    assert!(state.tick(20, REFERENCE_TIME + 1000));
    assert_eq!(state.clock, "20:06:43");
    let revision = state.revision;
    assert!(state.take_clock_animation_dirty(SIZE).is_none());
    assert!(!state.tick(53, REFERENCE_TIME + 1033));
    let dirty = state.take_clock_animation_dirty(SIZE).unwrap();
    assert_eq!(state.revision, revision);
    assert!(dirty.x > 650.0 && dirty.right() <= 1024.0);
    assert!(dirty.y > 100.0 && dirty.bottom() < 480.0);
    assert!(state.take_clock_animation_dirty(SIZE).is_none());
    let sample = state.flip_clock.sample(53);
    assert_eq!(sample.from, before.to);
    assert_eq!(sample.to.cards[2], [4, 3]);
    assert!(sample.progress > 0.0 && sample.progress < 1.0);
}

#[test]
fn delayed_loop_always_submits_one_settled_frame() {
    let mut state = clock_state();
    state.tick(20, REFERENCE_TIME + 1000);
    let after = FLIP_DURATION_MS + 170;
    state.tick(20 + after, REFERENCE_TIME + 1000 + after as i64);
    assert!(state.take_clock_animation_dirty(SIZE).is_some());
    assert_eq!(state.flip_clock.sample(20 + after).progress, 1.0);
    assert!(state.take_clock_animation_dirty(SIZE).is_none());
}

#[test]
fn noon_and_midnight_flip_all_cards_and_update_period() {
    for (before, after, period) in [
        (DAY + 43_199_000, DAY + 43_200_000, "PM"),
        (DAY + 86_399_000, DAY + 86_400_000, "AM"),
    ] {
        let mut state = LauncherState::new();
        state.settings.timezone_minutes = 0;
        state.tick(0, before);
        state.open_app("clock");
        state.tick(1000, after);
        let sample = state.flip_clock.sample(1000);
        assert_eq!(sample.to.cards, [[1, 2], [0, 0], [0, 0]]);
        assert_eq!(sample.to.period, Some(period));
        assert_eq!(sample.progress, 0.0);
        state.tick(1033, after + 33);
        let dirty = state.take_clock_animation_dirty(SIZE).unwrap();
        assert!(dirty.x < 30.0 && dirty.right() > 990.0);
    }
}

#[test]
fn time_sync_invalid_time_and_hidden_clock_snap_without_replaying_old_turns() {
    let mut state = clock_state();
    state.tick(20, REFERENCE_TIME + 1000);
    state.last_time_refresh();
    state.tick(30, REFERENCE_TIME - 3_600_000);
    assert_eq!(state.flip_clock.sample(30).progress, 1.0);
    assert!(state.take_clock_animation_dirty(SIZE).is_none());
    state.tick(40, 0);
    assert_eq!(state.flip_clock.sample(40).to.period, None);
    assert_eq!(state.clock, "--:--:--");
    state.tick(50, REFERENCE_TIME + 2000);
    assert_eq!(state.flip_clock.sample(50).progress, 1.0);
    state.tick(60, REFERENCE_TIME + 3000);
    state.mode = Mode::Display;
    state.tick(70, REFERENCE_TIME + 3010);
    state.mode = Mode::Pad;
    state.tick(80, REFERENCE_TIME + 3020);
    // The production loop never consumes Pad dirty while in Display mode.
    assert_eq!(state.flip_clock.sample(80).progress, 1.0);
    assert!(state.take_clock_animation_dirty(SIZE).is_none());
    for hidden in 0..3 {
        state.tick(
            100 + hidden * 100,
            REFERENCE_TIME + (3 + hidden as i64) * 1000,
        );
        match hidden {
            0 => state.mode = Mode::Display,
            1 => state.settings.screen_on = false,
            _ => state.background_active_app(),
        }
        assert!(state.take_clock_animation_dirty(SIZE).is_none());
        state.mode = Mode::Pad;
        state.settings.screen_on = true;
        state.open_app("clock");
        assert_eq!(state.flip_clock.sample(state.monotonic_ms).progress, 1.0);
    }
}

#[test]
fn local_animation_frames_equal_fresh_render_and_preserve_other_cards_and_controls() {
    let state = Arc::new(Mutex::new(clock_state()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    state.lock().unwrap().tick(20, REFERENCE_TIME + 1000);
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    for elapsed in [20, 85, 180, 240, 280, 360, 460, 540, 580, 640, 790] {
        let dirty = {
            let mut s = state.lock().unwrap();
            s.tick(20 + elapsed, REFERENCE_TIME + 1000 + elapsed as i64);
            s.take_clock_animation_dirty(SIZE)
        };
        let before = backend.pixels.clone();
        let frames = backend.frames;
        if let Some(rect) = dirty {
            app.mark_dirty(rect);
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(backend.frames, frames + usize::from(dirty.is_some()));
        if let Some(rect) = dirty {
            for y in 0..600 {
                for x in 0..1024 {
                    if !rect.contains(Point::new(x as f32, y as f32)) {
                        assert_eq!(backend.pixels[y * 1024 + x], before[y * 1024 + x]);
                    }
                }
            }
        }
        let mut fresh_backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(state.clone(), SIZE), SIZE).step(&mut fresh_backend);
        assert_eq!(
            backend.pixels, fresh_backend.pixels,
            "partial repaint differed at elapsed {elapsed}ms"
        );
    }
}

#[test]
fn home_icon_backgrounds_clock_and_removed_controls_do_not_trigger_actions() {
    let state = Arc::new(Mutex::new(clock_state()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    for point in [
        Point::new(908.0, 546.0),
        Point::new(916.0, 28.0),
        Point::new(48.0, 28.0),
    ] {
        backend.event(TouchEvent::Down(point));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        backend.event(TouchEvent::Up(point));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        if point.x > 100.0 {
            let mut s = state.lock().unwrap();
            assert!(matches!(s.active_app, ActiveApp::Clock));
            assert!(s.take_commands().is_empty());
        }
    }
    let s = state.lock().unwrap();
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert!(s.running_apps.contains_key("clock"));
}

#[test]
fn close_icon_kills_clock() {
    let state = Arc::new(Mutex::new(clock_state()));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    backend.event(TouchEvent::Down(Point::new(976.0, 28.0)));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    backend.event(TouchEvent::Up(Point::new(976.0, 28.0)));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    let s = state.lock().unwrap();
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert!(!s.running_apps.contains_key("clock"));
}

#[test]
fn numeral_ink_is_split_equally_by_the_card_hinge() {
    let state = Arc::new(Mutex::new(clock_state()));
    let mut backend = HeadlessBackend::new(1024, 600);
    App::new(build_launcher_ui(state, SIZE), SIZE).step(&mut backend);
    // Isolate the large hour numeral from the lower-left PM label. The first
    // plate spans y=145..454 with its mechanical hinge at y≈299.5.
    let rows: Vec<usize> = (160..400)
        .filter(|&y| {
            (100..270).any(|x| {
                let pixel = backend.pixels[y * 1024 + x];
                pixel >> 11 >= 27 && (pixel >> 5) & 63 >= 54 && pixel & 31 >= 27
            })
        })
        .collect();
    let top = *rows.first().expect("hour digit has visible ink") as f32;
    let bottom = (*rows.last().unwrap() + 1) as f32;
    assert!(
        ((top + bottom) * 0.5 - 299.5).abs() <= 1.5,
        "digit ink {top}..{bottom} must straddle the hinge evenly"
    );
}

fn render(state: LauncherState) -> Vec<u16> {
    let mut backend = HeadlessBackend::new(1024, 600);
    App::new(build_launcher_ui(Arc::new(Mutex::new(state)), SIZE), SIZE).step(&mut backend);
    backend.pixels
}

fn render_turn(elapsed: u64) -> Vec<u16> {
    let mut state = clock_state();
    state.tick(20, REFERENCE_TIME + 1000);
    state.tick(20 + elapsed, REFERENCE_TIME + 1000 + elapsed as i64);
    render(state)
}

#[test]
fn release_contact_and_completion_keep_the_exact_accepted_clock_face() {
    let old = render(clock_state());
    let final_face = render_turn(FLIP_DURATION_MS);
    assert_eq!(
        render_turn(0),
        old,
        "the first frame must not move the digits"
    );
    assert_eq!(
        render_turn(540),
        final_face,
        "contact must land on the native, crisp glyphs"
    );
    assert_eq!(render_turn(FLIP_DURATION_MS + 100), final_face);
}

#[test]
fn edge_on_leaf_exposes_new_top_and_keeps_old_bottom_until_the_back_unfolds() {
    let old = render(clock_state());
    let new = render_turn(FLIP_DURATION_MS);
    let edge_on = render_turn(280);
    let unfolding = render_turn(360);
    // Away from the moving-edge shadow, the two stationary faces prove layer
    // order at the 90-degree handoff. This catches whole-number crossfades.
    for y in 160..240 {
        for x in 710..980 {
            assert_eq!(edge_on[y * 1024 + x], new[y * 1024 + x]);
            assert_eq!(unfolding[y * 1024 + x], new[y * 1024 + x]);
        }
    }
    for y in 350..420 {
        for x in 710..980 {
            assert_eq!(edge_on[y * 1024 + x], old[y * 1024 + x]);
        }
    }
    let lower_leaf = |pixels: &[u16]| {
        (310..390)
            .flat_map(|y| (710..980).map(move |x| pixels[y * 1024 + x]))
            .collect::<Vec<_>>()
    };
    assert_ne!(lower_leaf(&unfolding), lower_leaf(&old));
    assert_ne!(lower_leaf(&unfolding), lower_leaf(&new));
}

#[test]
fn forward_leaf_has_perspective_and_all_moving_pixels_fit_the_damage_rect() {
    let old = render(clock_state());
    let projected = render_turn(200);
    // Perspective widens the free edge past the stationary card's x=1000
    // right side. Pure vertical compression cannot produce these pixels.
    assert!((180..290).any(|y| (1001..1013).any(|x| projected[y * 1024 + x] != 0)));
    for elapsed in (20..=FLIP_DURATION_MS).step_by(20) {
        let mut state = clock_state();
        state.tick(20, REFERENCE_TIME + 1000);
        state.tick(20 + elapsed, REFERENCE_TIME + 1000 + elapsed as i64);
        let damage = state.take_clock_animation_dirty(SIZE).unwrap();
        let pixels = render(state);
        for (i, (&a, &b)) in pixels.iter().zip(&old).enumerate() {
            if a != b {
                assert!(
                    damage.contains(Point::new((i % 1024) as f32, (i / 1024) as f32)),
                    "moving pixel at {}:{} escaped the damage rect at {elapsed} ms",
                    i % 1024,
                    i / 1024
                );
            }
        }
    }
}
