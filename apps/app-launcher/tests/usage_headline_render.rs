//! Exercise the real widget tree: an animated headline must stay local, then
//! match an independently rendered settled screen without old digit trails.
use app_launcher::{build_launcher_ui, headless::HeadlessBackend, usage::*, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, PlatformBackend, Rect, Size, TouchEvent};

const NOW: i64 = 1_791_287_400_000;
struct Recorded {
    inner: HeadlessBackend,
    flushes: Vec<Rect>,
    strided: bool,
    strided_flushes: usize,
    packed_flushes: usize,
    frame_open: bool,
}
impl Recorded {
    fn new() -> Self {
        Self {
            inner: HeadlessBackend::new(1024, 600),
            flushes: Vec::new(),
            strided: false,
            strided_flushes: 0,
            packed_flushes: 0,
            frame_open: false,
        }
    }
    fn new_strided() -> Self {
        Self {
            strided: true,
            ..Self::new()
        }
    }
}
impl PlatformBackend for Recorded {
    fn begin_frame(&mut self) {
        assert!(!self.frame_open);
        self.frame_open = true;
        self.inner.begin_frame();
    }
    fn end_frame(&mut self) {
        assert!(self.frame_open);
        self.inner.end_frame();
        self.frame_open = false;
    }
    fn screen_size(&self) -> Size {
        self.inner.screen_size()
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        self.inner.poll_touch()
    }
    fn flush(&mut self, rect: Rect, pixels: &[u16]) {
        self.packed_flushes += 1;
        self.flushes.push(rect);
        self.inner.flush(rect, pixels);
    }
    fn flush_strided(&mut self, rect: Rect, pixels: &[u16], stride: usize) -> bool {
        if !self.strided {
            return false;
        }
        assert!(self.frame_open, "strided copy escaped display ownership");
        assert_eq!(
            stride, 1024,
            "source pitch must remain the full framebuffer"
        );
        let (x, y, width, height) = (
            rect.x as usize,
            rect.y as usize,
            rect.width as usize,
            rect.height as usize,
        );
        assert_eq!(
            rect,
            Rect::from_ltwh(x as f32, y as f32, width as f32, height as f32)
        );
        assert!(width > 0 && height > 0 && x + width <= 1024 && y + height <= 600);
        assert_eq!(pixels.len(), (height - 1) * stride + width);
        // Match the hardware contract: this borrowed slice already starts at
        // (x, y). Its row pitch is the source framebuffer's width, while each
        // destination row keeps its independent absolute display coordinates.
        for row in 0..height {
            let source = row * stride;
            let destination = (y + row) * 1024 + x;
            self.inner.pixels[destination..destination + width]
                .copy_from_slice(&pixels[source..source + width]);
        }
        self.strided_flushes += 1;
        self.flushes.push(rect);
        true
    }
}
fn fixture(tokens: u64) -> Arc<Mutex<LauncherState>> {
    let mut state = LauncherState::new();
    state.open_app("sub2api-monitor");
    state.tick(10_000, NOW);
    state.usage.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    state.usage.scope = Some(api::scope(Period::Day, Page::Overview, None, NOW, 480).unwrap());
    state.usage.data = Some(Arc::new(Data {
        totals: Some(Totals {
            tokens,
            cost: 142.87,
            requests: Some(428),
        }),
        total_label: "当前账号汇总".into(),
        ..Default::default()
    }));
    Arc::new(Mutex::new(state))
}
#[test]
fn real_counter_animation_only_flushes_number_strip_and_finishes_without_trails() {
    verify_transition(270_879_573, 280_000_000, None);
}
#[test]
fn small_update_does_not_upload_unchanged_leading_digits_or_abbreviation() {
    verify_transition(270_879_573, 270_880_573, Some(250.));
}
#[test]
fn changing_digit_count_and_font_size_clears_old_glyphs() {
    verify_transition(9_999_999, 10_001_000, None);
    verify_transition(u64::MAX, 270_879_573, None);
}
#[test]
fn hardware_strided_counter_matches_packed_reference_without_horizontal_wrap() {
    for (from, target, max_damage_width) in [
        (270_879_573, 280_000_000, None),
        (270_879_573, 270_880_573, Some(250.)),
        (9_999_999, 10_001_000, None),
        (u64::MAX, 270_879_573, None),
    ] {
        verify_transition_with_backend(from, target, max_damage_width, Recorded::new_strided());
    }
}
fn verify_transition(from: u64, target: u64, max_damage_width: Option<f32>) {
    verify_transition_with_backend(from, target, max_damage_width, Recorded::new());
}
fn verify_transition_with_backend(
    from: u64,
    target: u64,
    max_damage_width: Option<f32>,
    mut backend: Recorded,
) {
    let state = fixture(from);
    let size = Size::new(1024.0, 600.0);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    {
        let mut state = state.lock().unwrap();
        let mut next = (**state.usage.data.as_ref().unwrap()).clone();
        next.totals.as_mut().unwrap().tokens = target;
        state.usage.data = Some(Arc::new(next));
        state.tick(10_100, NOW + 100);
    }
    // Emulate the worker's one result rebuild; later counter frames must not.
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    backend.flushes.clear();
    let before = backend.inner.pixels.clone();
    for elapsed in (33..1_023).step_by(33) {
        state
            .lock()
            .unwrap()
            .tick(10_100 + elapsed, NOW + 100 + elapsed as i64);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(
        backend.flushes.len() >= 20,
        "animation did not produce visible intermediate frames"
    );
    if backend.strided {
        assert_eq!(
            backend.packed_flushes, 0,
            "strided output silently fell back"
        );
        assert!(backend.strided_flushes >= backend.flushes.len() + 2);
        assert!(!backend.frame_open, "animation left the display frame open");
    }
    for rect in &backend.flushes {
        if let Some(max) = max_damage_width {
            assert!(rect.width < max, "unchanged prefix repainted: {rect:?}");
        }
        assert!(
            rect.width * rect.height <= 48_000.0,
            "unexpected whole-page damage: {rect:?}"
        );
        assert!(
            rect.y >= 192.0 && rect.bottom() <= 289.0,
            "damage reached chart or metadata: {rect:?}"
        );
    }
    // Everything below the headline is unchanged during its animation.
    assert_eq!(&backend.inner.pixels[300 * 1024..], &before[300 * 1024..]);
    assert_eq!(
        state.lock().unwrap().usage.headline.sample(11_200).value,
        Some(target)
    );
    let mut fresh = HeadlessBackend::new(1024, 600);
    let mut settled = App::new(build_launcher_ui(state.clone(), size), size);
    settled.step_with_builder(&mut fresh, |size| build_launcher_ui(state.clone(), size));
    let differences = backend
        .inner
        .pixels
        .iter()
        .zip(&fresh.pixels)
        .enumerate()
        .filter_map(|(i, (a, b))| (a != b).then_some((i % 1024, i / 1024)))
        .collect::<Vec<_>>();
    assert!(
        differences.is_empty(),
        "animation left {} stale pixels, first coordinates: {:?}",
        differences.len(),
        &differences[..differences.len().min(12)]
    );
}

#[test]
fn live_samples_update_tokens_cost_timestamp_and_error_without_replacing_the_tree() {
    let state = fixture(270_879_573);
    let size = Size::new(1024., 600.);
    let mut backend = Recorded::new_strided();
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    backend.flushes.clear();
    let revision = state.lock().unwrap().revision;
    {
        let mut s = state.lock().unwrap();
        let mut data = (**s.usage.data.as_ref().unwrap()).clone();
        data.totals.as_mut().unwrap().tokens = 270_880_573;
        data.totals.as_mut().unwrap().cost = 146.42;
        s.usage.data = Some(Arc::new(data));
        s.usage.status = "更新于 14:30:05 · 实时采样 5 秒 / 看板 60 秒".into();
        s.tick(10_100, NOW + 100);
    }
    // A fast network completion neither increments global revision nor asks
    // App to rebuild. The existing render objects must detect all live fields.
    assert_eq!(state.lock().unwrap().revision, revision);
    for elapsed in (0..1_089).step_by(33) {
        state
            .lock()
            .unwrap()
            .tick(10_100 + elapsed, NOW + 100 + elapsed as i64);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(!backend.flushes.is_empty());
    assert!(
        backend
            .flushes
            .iter()
            .all(|r| r.width * r.height < 450_000.),
        "fast sample produced whole-page damage: {:?}",
        backend.flushes
    );
    assert_matches_settled(&state, &backend.inner.pixels, size);
    for (status, partial) in [("网络连接失败", false), ("趋势未更新", true)] {
        backend.flushes.clear();
        {
            let mut s = state.lock().unwrap();
            s.usage.stale = true;
            s.usage.status = status.into();
            let mut data = (**s.usage.data.as_ref().unwrap()).clone();
            data.warning = partial.then(|| status.to_owned());
            s.usage.data = Some(Arc::new(data));
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(backend.flushes.len(), 1);
        assert!(
            backend.flushes[0].height <= 24.,
            "error update redrew unrelated cards"
        );
        assert_matches_settled(&state, &backend.inner.pixels, size);
    }
    // Navigation deliberately retains the normal new-tree path; no previous
    // scope's live render object may leak DAY content into MONTH.
    {
        let mut s = state.lock().unwrap();
        s.usage.navigate(Page::Overview, Period::Month, None);
        s.changed();
    }
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    assert_matches_settled(&state, &backend.inner.pixels, size);
}
fn assert_matches_settled(state: &Arc<Mutex<LauncherState>>, pixels: &[u16], size: Size) {
    let mut fresh = HeadlessBackend::new(1024, 600);
    let mut settled = App::new(build_launcher_ui(state.clone(), size), size);
    settled.step_with_builder(&mut fresh, |size| build_launcher_ui(state.clone(), size));
    let mismatches = pixels
        .iter()
        .zip(&fresh.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(mismatches, 0, "live metadata left stale pixels");
}

#[test]
fn trend_only_live_samples_keep_damage_in_the_chart_card_and_match_a_fresh_tree() {
    let state = fixture(270_879_573);
    {
        let mut s = state.lock().unwrap();
        let mut data = (**s.usage.data.as_ref().unwrap()).clone();
        data.trend = (0..3)
            .map(|i| app_launcher::usage::Point {
                date: format!("2026-10-06 {i:02}:00"),
                totals: Totals {
                    tokens: 1_000 + i * 300,
                    cost: 1.2,
                    requests: Some(10),
                },
            })
            .collect();
        data.yesterday_trend = data.trend.clone();
        s.usage.data = Some(Arc::new(data));
    }
    let size = Size::new(1024., 600.);
    let mut backend = Recorded::new_strided();
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    for change in 0..7 {
        backend.flushes.clear();
        {
            let mut s = state.lock().unwrap();
            let mut data = (**s.usage.data.as_ref().unwrap()).clone();
            match change {
                0 => data.trend.last_mut().unwrap().totals.tokens += 500,
                1 => data.trend.last_mut().unwrap().date = "2026-10-06 03:00".into(),
                2 => data.trend.last_mut().unwrap().totals.cost += 0.25,
                3 => data.yesterday_trend[0].totals.tokens += 2_500,
                4 => s.usage.chart_selected = Some(0),
                5 => data.trend.clear(),
                _ => data.trend = data.yesterday_trend.clone(),
            }
            s.usage.data = Some(Arc::new(data));
        }
        let mut builders = 0;
        app.step_with_builder(&mut backend, |size| {
            builders += 1;
            build_launcher_ui(state.clone(), size)
        });
        assert_eq!(builders, 0, "trend-only update replaced the dashboard tree");
        assert_eq!(
            backend.flushes,
            vec![Rect::from_ltwh(24., 358., 576., 184.)]
        );
        assert_matches_settled(&state, &backend.inner.pixels, size);
    }
}
