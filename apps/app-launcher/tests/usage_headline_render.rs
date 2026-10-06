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
