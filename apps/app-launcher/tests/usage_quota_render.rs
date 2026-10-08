//! A static quota card must use the same minute clock in live and rebuilt
//! dashboards. Five-second headline sampling must not leave a different bar.
use app_launcher::{build_launcher_ui, headless::HeadlessBackend, usage::*, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};

const CLOCK: i64 = 1_791_287_400_000; // Exactly a wall-clock minute.
const MONOTONIC: u64 = 10_000;
const SIZE: Size = Size::new(1024., 600.);
const WIDTH: usize = 1024;

// Production overview: quota card x=616, inset=18, Session y=136+76.
// The two-pixel blue time strip starts another 34 pixels below that label.
const SESSION_X: usize = 634;
const SESSION_Y: usize = 246;
const SESSION_WIDTH: usize = 348;

fn fixture(light: bool) -> Arc<Mutex<LauncherState>> {
    let mut state = LauncherState::new();
    state.settings.light_appearance = light;
    state.open_app("sub2api-monitor");
    state.tick(MONOTONIC, CLOCK);
    state.usage.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    state.usage.scope = Some(api::scope(Period::Day, Page::Overview, None, CLOCK, 480).unwrap());
    state.usage.data = Some(Arc::new(Data {
        totals: Some(Totals {
            tokens: 270_879_573,
            cost: 142.87,
            requests: Some(428),
        }),
        accounts: vec![Account {
            id: 1,
            label: "synthetic-account".into(),
            platform: "openai".into(),
            kind: "oauth".into(),
            plan_label: "Synthetic Plan".into(),
            usage_updated_ms: Some(CLOCK),
            five: Some(Window {
                used: 24.,
                reset_ms: Some(CLOCK + 3_600_000),
                window_minutes: Some(300),
            }),
            seven: Some(Window {
                used: 61.,
                reset_ms: Some(CLOCK + 86_400_000),
                window_minutes: Some(10_080),
            }),
        }],
        ..Data::default()
    }));
    state.usage.status = "更新于 14:30:00 · 实时采样 5 秒 / 看板 60 秒".into();
    Arc::new(Mutex::new(state))
}

fn rebuilt_pixels(state: &Arc<Mutex<LauncherState>>) -> Vec<u16> {
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    backend.pixels
}

fn assert_matches_rebuilt(state: &Arc<Mutex<LauncherState>>, live: &[u16]) {
    let rebuilt = rebuilt_pixels(state);
    let mismatches = rebuilt.iter().zip(live).filter(|(a, b)| a != b).count();
    assert_eq!(
        mismatches, 0,
        "live quota chrome differs from a new production tree"
    );
}

fn blue_strip_end(pixels: &[u16]) -> usize {
    // Sample the unfilled interior, rather than assuming any palette or AA
    // formula. Ignore each rounded track cap and inspect its actual pixels.
    let row = &pixels[SESSION_Y * WIDTH..(SESSION_Y + 1) * WIDTH];
    let track = row[SESSION_X + SESSION_WIDTH - 8];
    (SESSION_X + 4..SESSION_X + SESSION_WIDTH - 8)
        .rfind(|&x| row[x] != track)
        .expect("the synthetic one-hour remainder must paint a visible blue strip")
}

fn assert_main_number_unchanged(before: &[u16], after: &[u16]) {
    for y in 192..289 {
        assert_eq!(
            &before[y * WIDTH + 44..y * WIDTH + 598],
            &after[y * WIDTH + 44..y * WIDTH + 598],
            "wall time changed the unchanged TOTAL TOKENS number at row {y}"
        );
    }
}

#[test]
fn quota_time_strip_is_stable_within_a_minute_and_shrinks_on_the_minute_rebuild() {
    for light in [false, true] {
        let state = fixture(light);
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        let before = backend.pixels.clone();
        let initial_frames = backend.frames;
        let initial_revision = state.lock().unwrap().revision;
        let initial_end = blue_strip_end(&before);

        for second in 1..60 {
            let mut state_guard = state.lock().unwrap();
            assert!(!state_guard.tick(MONOTONIC + second * 1_000, CLOCK + second as i64 * 1_000,));
            assert_eq!(state_guard.revision, initial_revision);
            drop(state_guard);
            let mut builders = 0;
            app.step_with_builder(&mut backend, |size| {
                builders += 1;
                build_launcher_ui(state.clone(), size)
            });
            assert_eq!(
                builders, 0,
                "idle wall time rebuilt the monitor each second"
            );
            assert_eq!(
                backend.frames, initial_frames,
                "idle wall time submitted an LCD frame"
            );
            if second % 5 == 0 || second == 59 {
                assert_matches_rebuilt(&state, &backend.pixels);
            }
        }

        assert!(state
            .lock()
            .unwrap()
            .tick(MONOTONIC + 60_000, CLOCK + 60_000));
        assert_eq!(state.lock().unwrap().revision, initial_revision + 1);
        app.request_rebuild();
        let mut builders = 0;
        app.step_with_builder(&mut backend, |size| {
            builders += 1;
            build_launcher_ui(state.clone(), size)
        });
        assert_eq!(
            builders, 1,
            "minute boundary must refresh the quota card once"
        );
        assert_eq!(backend.frames, initial_frames + 1);
        let next_end = blue_strip_end(&backend.pixels);
        assert!(
            next_end < initial_end,
            "the blue time strip did not shrink: {initial_end} -> {next_end}"
        );
        assert_main_number_unchanged(&before, &backend.pixels);
        assert_matches_rebuilt(&state, &backend.pixels);

        assert!(!state
            .lock()
            .unwrap()
            .tick(MONOTONIC + 61_000, CLOCK + 61_000));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(backend.frames, initial_frames + 1);
    }
}
