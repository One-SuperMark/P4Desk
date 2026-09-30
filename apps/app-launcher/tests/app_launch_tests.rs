use app_launcher::app_launch::{
    APP_LAUNCH_DURATION_MS, APP_LAUNCH_FRAME_INTERVAL_MS, EXPAND_END_MS, REVEAL_START_MS,
};
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const SIZE: Size = Size::new(1024.0, 600.0);
struct Harness {
    state: Arc<Mutex<LauncherState>>,
    app: App,
    backend: HeadlessBackend,
}
impl Harness {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
        app.step(&mut backend);
        Self {
            state,
            app,
            backend,
        }
    }
    fn step(&mut self) {
        self.app.step_with_builder(&mut self.backend, |size| {
            build_launcher_ui(self.state.clone(), size)
        });
    }
    fn touch(&mut self, e: TouchEvent) {
        self.backend.event(e);
        self.step();
    }
    fn tap(&mut self, p: Point) {
        self.touch(TouchEvent::Down(p));
        self.touch(TouchEvent::Up(p));
    }
    fn tick(&mut self, ms: u64) {
        let mut s = self.state.lock().unwrap();
        if s.tick(ms, 0) {
            self.app.request_rebuild();
        }
        if let Some(rect) = s.take_launch_animation_dirty(SIZE) {
            self.app.mark_dirty(rect);
        }
        drop(s);
        self.step();
    }
    fn fresh(&self) -> Vec<u16> {
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(self.state.clone(), SIZE), SIZE).step(&mut backend);
        backend.pixels
    }
}

#[test]
fn six_app_splashes_use_the_clicked_icon_bounds_and_their_own_svg() {
    let mut splashes = Vec::new();
    for (id, x, y) in [
        ("clock", 132.0, 278.0),
        ("timer", 348.0, 278.0),
        ("notes", 564.0, 278.0),
        ("calculator", 780.0, 278.0),
        ("mac", 132.0, 466.0),
        ("settings", 348.0, 466.0),
    ] {
        let mut h = Harness::new();
        h.tap(Point::new(x, y));
        let frame = h.state.lock().unwrap().app_launch.frame(0).unwrap();
        assert_eq!(frame.id, id);
        assert!((frame.source.x + frame.source.width * 0.5 - x).abs() < 1.0);
        assert!((frame.source.y + frame.source.height * 0.5 - y).abs() < 1.0);
        assert!(
            (frame.source.width - 140.0 * 0.94).abs() < 0.1,
            "source must be the visible pressed SVG, not its label/hit slot"
        );
        h.tick(420);
        assert_eq!(h.backend.pixels, h.fresh());
        splashes.push(h.backend.pixels);
    }
    for (i, a) in splashes.iter().enumerate() {
        for b in &splashes[i + 1..] {
            assert_ne!(a, b, "splash must show the selected app's SVG");
        }
    }
}

#[test]
fn launch_repaints_progress_without_restarting_and_leaves_a_clean_live_app() {
    let mut h = Harness::new();
    h.tap(Point::new(348.0, 278.0));
    let initial = h.backend.pixels.clone();
    for ms in [
        33,
        130,
        EXPAND_END_MS - 1,
        EXPAND_END_MS,
        300,
        REVEAL_START_MS - 1,
        REVEAL_START_MS,
        500,
        640,
        850,
        APP_LAUNCH_DURATION_MS - 1,
        APP_LAUNCH_DURATION_MS,
        1_400,
    ] {
        h.tick(ms);
        assert_eq!(
            h.backend.pixels,
            h.fresh(),
            "stale launch pixels at {ms} ms"
        );
    }
    assert_ne!(initial, h.backend.pixels);
    assert!(h.state.lock().unwrap().app_launch.frame(1_400).is_none());
    let direct = Arc::new(Mutex::new(LauncherState::new()));
    direct.lock().unwrap().open_app("timer");
    let mut backend = HeadlessBackend::new(1024, 600);
    App::new(build_launcher_ui(direct, SIZE), SIZE).step(&mut backend);
    assert_eq!(
        h.backend.pixels, backend.pixels,
        "last frame must exactly match the normal app"
    );
}

#[test]
fn launch_frame_gate_and_external_revision_do_not_reset_the_timeline() {
    let mut s = LauncherState::new();
    s.launch_app("timer", Rect::from_ltwh(323.0, 108.0, 137.0, 137.0));
    s.tick(0, 0);
    let revision = s.revision;
    assert!(s.take_launch_animation_dirty(SIZE).is_some());
    assert!(s.take_launch_animation_dirty(SIZE).is_none());
    s.tick(APP_LAUNCH_FRAME_INTERVAL_MS - 1, 0);
    assert!(s.take_launch_animation_dirty(SIZE).is_none());
    s.tick(APP_LAUNCH_FRAME_INTERVAL_MS, 0);
    assert!(s.take_launch_animation_dirty(SIZE).is_some());
    assert_eq!(
        s.revision, revision,
        "animation must not rebuild the tree each frame"
    );
    s.changed();
    s.tick(700, 1_800_000_000_000);
    assert_eq!(s.app_launch.frame(700).unwrap().elapsed_ms, 700);
    s.tick(APP_LAUNCH_DURATION_MS, 0);
    assert!(
        s.take_launch_animation_dirty(SIZE).is_some(),
        "submit final cleanup even after delayed loop"
    );
    assert!(s.take_launch_animation_dirty(SIZE).is_none());
}

#[test]
fn circle_cover_icon_fade_and_reveal_keep_animating_across_phase_boundaries() {
    let mut s = LauncherState::new();
    s.launch_app("timer", Rect::from_ltwh(323.0, 108.0, 137.0, 137.0));
    for now in [0, 144, 300, 420, 460, 500, 690, APP_LAUNCH_DURATION_MS] {
        s.tick(now, 0);
        assert!(
            s.take_launch_animation_dirty(SIZE).is_some(),
            "phase stalled at {now}"
        );
        assert!(s.take_launch_animation_dirty(SIZE).is_none());
    }
}

#[test]
fn idle_desktop_keeps_one_backdrop_and_twenty_launches_release_it_at_expiry() {
    let mut h = Harness::new();
    let cache = h.state.lock().unwrap().desktop_backdrop.clone();
    assert!(cache.lock().unwrap().is_some());
    for cycle in 0..20 {
        let now = cycle * 2_000;
        // Firmware polls animation dirty on every idle desktop iteration.
        h.tick(now);
        h.tick(now + 16);
        assert!(
            cache.lock().unwrap().is_some(),
            "idle must keep prepared backdrop"
        );
        h.tap(Point::new(348.0, 278.0));
        assert!(
            cache.lock().unwrap().is_some(),
            "launch must transfer the backdrop"
        );
        h.tick(now + 100);
        h.tick(now + 16 + APP_LAUNCH_DURATION_MS);
        assert!(
            cache.lock().unwrap().is_none(),
            "finished app must release it"
        );
        h.state.lock().unwrap().background_active_app();
        assert!(cache.lock().unwrap().is_none());
        h.app.request_rebuild();
        h.step();
        assert!(
            cache.lock().unwrap().is_some(),
            "normal desktop prepares next launch"
        );
    }
}

#[test]
fn backdrop_releases_on_direct_open_cancel_and_hidden_desktop() {
    for action in 0..5 {
        let mut h = Harness::new();
        let cache = h.state.lock().unwrap().desktop_backdrop.clone();
        if action < 3 {
            h.tap(Point::new(348.0, 278.0));
        }
        {
            let mut s = h.state.lock().unwrap();
            match action {
                0 => s.background_active_app(),
                1 => s.kill_active_app(),
                2 => s.open_app("clock"),
                3 => s.settings.screen_on = false,
                _ => s.mode = Mode::Display,
            }
            s.tick(100, 0);
            s.take_launch_animation_dirty(SIZE);
        }
        assert!(cache.lock().unwrap().is_none());
        if action >= 3 {
            // Even a newly painted hidden desktop must not reallocate it.
            h.app.request_rebuild();
            h.step();
            assert!(cache.lock().unwrap().is_none());
        }
    }
}

#[test]
fn taps_and_a_held_release_on_splash_never_start_the_timer() {
    let mut h = Harness::new();
    h.tap(Point::new(348.0, 278.0));
    let start = Point::new(512.0, 478.0);
    h.tick(200);
    h.tap(start);
    assert!(!h.state.lock().unwrap().timer.is_running());
    h.touch(TouchEvent::Down(start));
    h.tick(1_100);
    h.touch(TouchEvent::Move(start));
    h.touch(TouchEvent::Up(start));
    assert!(
        !h.state.lock().unwrap().timer.is_running(),
        "held release must still belong to splash"
    );
    h.tap(start);
    assert!(
        h.state.lock().unwrap().timer.is_running(),
        "a new gesture after expiry must work"
    );
}

#[test]
fn background_timer_and_resumed_calculator_state_survive_app_launch() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    let first = match &s.active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!(),
    };
    first.lock().unwrap().input_digit('7');
    s.background_active_app();
    s.timer.set_countdown_seconds(10);
    s.timer.toggle(0);
    s.launch_app("calculator", Rect::from_ltwh(800.0, 100.0, 146.0, 146.0));
    s.tick(700, 0);
    let resumed = match &s.active_app {
        ActiveApp::Calculator(c) => c.clone(),
        _ => panic!(),
    };
    assert!(Arc::ptr_eq(&first, &resumed));
    assert_eq!(resumed.lock().unwrap().current_input, "7");
    assert_eq!(s.timer.remaining_ms, 9_300);
    s.tick(10_000, 0);
    assert!(s.timer.finished);
    assert!(
        s.timer_completion.progress(10_000).is_none(),
        "hidden completion must not cover calculator"
    );
}

#[test]
fn back_kill_screen_off_and_display_cancel_launch_without_replay() {
    for action in 0..4 {
        let mut h = Harness::new();
        h.tap(Point::new(348.0, 278.0));
        h.tick(100);
        match action {
            0 => h.touch(TouchEvent::HardwareBack),
            1 => h.touch(TouchEvent::HardwareKill),
            2 => h.state.lock().unwrap().settings.screen_on = false,
            _ => h.state.lock().unwrap().mode = Mode::Display,
        }
        h.tick(200);
        {
            let mut s = h.state.lock().unwrap();
            assert!(s.app_launch.frame(200).is_none());
            s.settings.screen_on = true;
            s.mode = Mode::Pad;
        }
        h.tick(300);
        assert!(h.state.lock().unwrap().app_launch.frame(300).is_none());
    }
    for p in [Point::new(780.0, 466.0)] {
        let mut h = Harness::new();
        h.tap(p);
        let s = h.state.lock().unwrap();
        assert!(matches!(s.active_app, ActiveApp::Launcher));
        assert!(s.app_launch.frame(0).is_none(), "screen-off is immediate");
    }
}

#[test]
fn usb_holds_full_color_until_valid_first_frame_and_never_becomes_a_background_app() {
    let mut h = Harness::new();
    h.state.lock().unwrap().connected = true;
    h.tap(Point::new(564.0, 466.0));
    for ms in [0, 130, 260, 339] {
        h.tick(ms);
        assert!(h.state.lock().unwrap().take_commands().is_empty());
        assert_eq!(h.backend.pixels, h.fresh());
    }
    h.tick(340);
    assert!(matches!(
        h.state.lock().unwrap().take_commands().as_slice(),
        [app_launcher::UiCommand::StartDisplayTransition { duration_ms: 600 }]
    ));
    let covered = h.backend.pixels.clone();
    assert!(covered
        .iter()
        .all(|p| *p == app_launcher::app_icons::launch_color_for("display", false).to_rgb565()));
    for ms in [640, 940, 3000, 7000] {
        h.tick(ms);
        let mut s = h.state.lock().unwrap();
        assert_eq!(s.app_launch.frame(ms).unwrap().elapsed_ms, 340);
        assert!(s.take_commands().is_empty());
        assert!(s.desktop_backdrop.lock().unwrap().is_none());
        assert_eq!(h.backend.pixels, covered);
    }
    let mut s = h.state.lock().unwrap();
    s.mode = Mode::Display;
    s.tick(7100, 0);
    assert_eq!(
        s.active_app.id(),
        Some("display"),
        "keep fallback route until LCD reveal completes"
    );
    s.complete_display_launch();
    assert!(matches!(s.active_app, ActiveApp::Launcher));
    assert!(!s.running_apps.contains_key("display"));
    s.mode = Mode::Pad;
    s.tick(8000, 0);
    assert!(s.take_commands().is_empty());
}

#[test]
fn cancellation_drops_queued_display_requests() {
    for action in 0..5 {
        let mut h = Harness::new();
        h.state.lock().unwrap().connected = true;
        h.tap(Point::new(564.0, 466.0));
        h.tick(340);
        match action {
            0 => h.touch(TouchEvent::HardwareBack),
            1 => h.touch(TouchEvent::HardwareKill),
            2 => h.state.lock().unwrap().settings.screen_on = false,
            3 => h.state.lock().unwrap().open_app("timer"),
            _ => h.state.lock().unwrap().background_active_app(),
        }
        h.tick(2000);
        let mut s = h.state.lock().unwrap();
        assert!(s
            .take_commands()
            .iter()
            .all(|c| matches!(c, app_launcher::UiCommand::CancelDisplayTransition)));
        assert!(s.app_launch.frame(2000).is_none());
        assert!(!s.running_apps.contains_key("display"));
    }
}

#[test]
fn offline_usb_keeps_full_animation_and_stays_on_intermediate_page() {
    let mut h = Harness::new();
    h.tap(Point::new(564.0, 466.0));
    for ms in [130, 340, 640, 939] {
        h.tick(ms);
        assert!(h.state.lock().unwrap().app_launch.frame(ms).is_some());
    }
    h.tick(APP_LAUNCH_DURATION_MS);
    let mut s = h.state.lock().unwrap();
    assert!(s.take_commands().is_empty());
    assert!(s.notice.contains("Mac 未连接"));
    assert!(matches!(s.active_app, ActiveApp::DisplaySetup));
    s.connected = true;
    s.tick(2000, 0);
    assert!(s.take_commands().is_empty());
    s.request_display_mode();
    assert!(matches!(
        s.take_commands().as_slice(),
        [app_launcher::UiCommand::RequestMode(Mode::Display)]
    ));
}

#[test]
fn timeout_and_disconnect_reveal_intermediate_page_instead_of_returning_home() {
    for disconnect in [false, true] {
        let mut h = Harness::new();
        h.state.lock().unwrap().connected = true;
        h.tap(Point::new(564.0, 466.0));
        h.tick(340);
        h.state.lock().unwrap().take_commands();
        if disconnect {
            h.state.lock().unwrap().connected = false;
        }
        let failed = if disconnect { 2000 } else { 10340 };
        h.tick(failed);
        {
            let mut s = h.state.lock().unwrap();
            assert_eq!(s.app_launch.frame(failed).unwrap().elapsed_ms, 340);
            assert!(matches!(
                s.take_commands().as_slice(),
                [app_launcher::UiCommand::CancelDisplayTransition]
            ));
        }
        h.tick(failed + 300);
        assert_eq!(
            h.state
                .lock()
                .unwrap()
                .app_launch
                .frame(failed + 300)
                .unwrap()
                .elapsed_ms,
            640
        );
        h.tick(failed + 600);
        let mut s = h.state.lock().unwrap();
        assert!(s.app_launch.frame(failed + 600).is_none());
        assert!(matches!(s.active_app, ActiveApp::DisplaySetup));
        assert!(!s.notice.is_empty());
        assert!(s.take_commands().is_empty());
    }
}

#[test]
fn jpeg_reveal_is_opaque_at_start_and_leaves_exact_pixels_and_tail_at_end() {
    use app_launcher::app_launch::paint_usb_display_reveal;
    let count = 1024 * 600;
    let mut source: Vec<u16> = (0..1024 * 608).map(|n| (n % 65536) as u16).collect();
    let original = source.clone();
    for elapsed in [0, 300, 600, 900] {
        source.copy_from_slice(&original);
        let mut canvas = Canvas::new(tiny_gfx::Pixmap565Mut::new(&mut source[..count], 1024, 600));
        paint_usb_display_reveal(
            &mut canvas,
            SIZE,
            elapsed,
            600,
            false,
            app_launcher::icon_theme::IconTheme::Colloid,
        );
        assert_eq!(
            &source[count..],
            &original[count..],
            "MCU tail must remain untouched"
        );
        if elapsed == 0 {
            assert!(source[..count]
                .iter()
                .all(|p| *p
                    == app_launcher::app_icons::launch_color_for("display", false).to_rgb565()));
        } else if elapsed >= 600 {
            assert_eq!(source, original);
        } else {
            assert_eq!(source[0], original[0], "outside is revealed first");
            assert_ne!(source[300 * 1024 + 512], original[300 * 1024 + 512]);
        }
    }
}
