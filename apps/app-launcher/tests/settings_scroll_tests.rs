use app_launcher::headless::HeadlessBackend;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tiny_flutter::prelude::*;

struct StridedBackend {
    inner: HeadlessBackend,
    cropped_flushes: usize,
}
impl PlatformBackend for StridedBackend {
    fn screen_size(&self) -> Size {
        self.inner.screen_size()
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        self.inner.poll_touch()
    }
    fn begin_frame(&mut self) {
        self.inner.begin_frame();
    }
    fn end_frame(&mut self) {
        self.inner.end_frame();
    }
    fn flush(&mut self, _: Rect, _: &[u16]) {
        panic!("strided backend must not pack pixels");
    }
    fn flush_strided(&mut self, r: Rect, data: &[u16], stride: usize) -> bool {
        let width = r.width as usize;
        let height = r.height as usize;
        assert_eq!(data.len(), (height - 1) * stride + width);
        if width != stride {
            self.cropped_flushes += 1;
        }
        for y in 0..height {
            self.inner.flush(
                Rect::from_ltwh(r.x, r.y + y as f32, r.width, 1.0),
                &data[y * stride..y * stride + width],
            );
        }
        true
    }
}

#[test]
fn strided_flush_matches_packed_output_for_cropped_and_bottom_edge_regions() {
    let size = Size::new(160.0, 120.0);
    let make = || {
        CustomPaint::new(Stripes(
            Arc::new(AtomicUsize::new(0)),
            Color::from_hex(0x205080),
        ))
        .size(size)
    };
    let mut packed = App::new(make(), size);
    let mut strided = App::new(make(), size);
    let mut a = HeadlessBackend::new(160, 120);
    let mut b = StridedBackend {
        inner: HeadlessBackend::new(160, 120),
        cropped_flushes: 0,
    };
    packed.step(&mut a);
    strided.step(&mut b);
    for r in [
        Rect::from_ltwh(10.0, 12.0, 70.0, 55.0),
        Rect::from_ltwh(150.0, 100.0, 10.0, 20.0),
        Rect::from_ltwh(0.0, 119.0, 160.0, 1.0),
    ] {
        packed.mark_dirty(r);
        strided.mark_dirty(r);
        packed.step(&mut a);
        strided.step(&mut b);
        assert!(a.pixels == b.inner.pixels);
    }
    assert_eq!(b.cropped_flushes, 2);
}

struct Stripes(Arc<AtomicUsize>, Color);
impl CustomPainter for Stripes {
    fn paint(&self, c: &mut Canvas, size: Size) {
        self.0.fetch_add(1, Ordering::Relaxed);
        for row in 0..(size.height as usize / 20) {
            c.draw_rect(
                Rect::from_ltwh(0.0, row as f32 * 20.0, size.width, 14.0),
                if row % 2 == 0 { self.1 } else { Color::WHITE },
            );
        }
    }
}
fn view(
    controller: ScrollController,
    count: Arc<AtomicUsize>,
    key: Option<u64>,
    color: Color,
    height: f32,
) -> impl Widget {
    let mut scroll = SingleChildScrollView::new(
        CustomPaint::new(Stripes(count, color)).size(Size::new(100.0, height)),
    )
    .controller(controller);
    if let Some(key) = key {
        scroll = scroll.raster_cache(key, Color::BLACK);
    }
    Stack::new().push(
        Positioned::new(Container::new().width(100.0).height(80.0).child(scroll))
            .left(12.0)
            .top(8.0),
    )
}

#[test]
fn cached_scroll_matches_live_pixels_and_reuses_content_after_root_rebuild() {
    let size = Size::new(160.0, 120.0);
    let count = Arc::new(AtomicUsize::new(0));
    let controller = ScrollController::new();
    let mut backend = HeadlessBackend::new(160, 120);
    let make = || {
        view(
            controller.clone(),
            count.clone(),
            Some(1),
            Color::from_hex(0x306090),
            300.0,
        )
    };
    let mut app = App::new(make(), size);
    app.step(&mut backend);
    assert_eq!(count.load(Ordering::Relaxed), 1);
    backend.event(TouchEvent::Down(Point::new(60.0, 70.0)));
    app.step_with_builder(&mut backend, |_| make());
    let pressed_paints = count.load(Ordering::Relaxed);
    for y in [58.0, 46.0, 38.0] {
        backend.event(TouchEvent::Move(Point::new(60.0, y)));
        app.step_with_builder(&mut backend, |_| make());
    }
    assert_eq!(controller.offset(), 32.0);
    assert_eq!(
        app.take_frame_metrics().opaque_frames,
        3,
        "owned drag should replace the opaque viewport directly"
    );
    assert_eq!(
        count.load(Ordering::Relaxed),
        pressed_paints,
        "drag must reuse pixels"
    );
    let mut live = HeadlessBackend::new(160, 120);
    App::new(
        view(
            ScrollController::with_offset(32.0),
            Arc::new(AtomicUsize::new(0)),
            None,
            Color::from_hex(0x306090),
            300.0,
        ),
        size,
    )
    .step(&mut live);
    assert!(
        backend.pixels == live.pixels,
        "cached viewport clipping differs from live rendering"
    );
    backend.event(TouchEvent::Up(Point::new(60.0, 38.0)));
    app.step_with_builder(&mut backend, |_| make());
    app.request_rebuild();
    app.step_with_builder(&mut backend, |_| make());
    assert_eq!(
        count.load(Ordering::Relaxed),
        pressed_paints,
        "controller cache must survive rebuilds"
    );
}

#[test]
fn opaque_scroll_skips_background_but_preserves_overlaid_content() {
    let size = Size::new(160.0, 120.0);
    let controller = ScrollController::new();
    let background_paints = Arc::new(AtomicUsize::new(0));
    let make = || {
        Stack::new()
            .push(
                CustomPaint::new(Stripes(
                    background_paints.clone(),
                    Color::from_hex(0x208030),
                ))
                .size(size),
            )
            .push(view(
                controller.clone(),
                Arc::new(AtomicUsize::new(0)),
                Some(1),
                Color::from_hex(0x205080),
                300.0,
            ))
            .push(
                Positioned::new(
                    Container::new()
                        .width(40.0)
                        .height(22.0)
                        .color(Color::from_rgba(220, 30, 80, 128)),
                )
                .left(30.0)
                .top(42.0),
            )
    };
    let mut backend = HeadlessBackend::new(160, 120);
    let mut app = App::new(make(), size);
    app.step(&mut backend);
    backend.event(TouchEvent::Down(Point::new(95.0, 70.0)));
    app.step_with_builder(&mut backend, |_| make());
    let before = background_paints.load(Ordering::Relaxed);
    backend.event(TouchEvent::Move(Point::new(95.0, 38.0)));
    app.step_with_builder(&mut backend, |_| make());
    assert_eq!(background_paints.load(Ordering::Relaxed), before);
    let fast = backend.pixels.clone();
    app.mark_dirty(Rect::from_ltwh(0.0, 0.0, 160.0, 120.0));
    app.step_with_builder(&mut backend, |_| make());
    assert!(background_paints.load(Ordering::Relaxed) > before);
    assert!(
        fast == backend.pixels,
        "overlay order or background pixels changed"
    );
}

#[test]
fn cache_revision_replaces_old_pixels_and_over_budget_content_falls_back() {
    let size = Size::new(160.0, 120.0);
    let controller = ScrollController::new();
    let count = Arc::new(AtomicUsize::new(0));
    let mut backend = HeadlessBackend::new(160, 120);
    let mut app = App::new(
        view(
            controller.clone(),
            count.clone(),
            Some(1),
            Color::WHITE,
            300.0,
        ),
        size,
    );
    app.step(&mut backend);
    let before = backend.pixels.clone();
    app.set_root(view(
        controller.clone(),
        count.clone(),
        Some(2),
        Color::from_hex(0xff0000),
        300.0,
    ));
    app.step(&mut backend);
    assert_eq!(count.load(Ordering::Relaxed), 2);
    assert!(backend.pixels != before);
    app.set_root(view(
        controller,
        count.clone(),
        Some(3),
        Color::WHITE,
        12000.0,
    ));
    app.step(&mut backend);
    app.mark_dirty(Rect::from_ltwh(0.0, 0.0, 160.0, 120.0));
    app.step(&mut backend);
    assert_eq!(
        count.load(Ordering::Relaxed),
        4,
        "oversize cache must fall back to live paint"
    );
}

#[test]
fn scrolling_keeps_pointer_outside_viewport_and_applies_final_release_sample() {
    let controller = ScrollController::new();
    let mut root = view(
        controller.clone(),
        Arc::new(AtomicUsize::new(0)),
        Some(1),
        Color::WHITE,
        300.0,
    )
    .create_render_object();
    root.layout(&BoxConstraints::tight(Size::new(160.0, 120.0)));
    assert!(root.dispatch_touch(&TouchEvent::Down(Point::new(60.0, 70.0))));
    assert!(root.captures_touch());
    root.dispatch_touch(&TouchEvent::Move(Point::new(140.0, -10.0)));
    assert_eq!(controller.offset(), 80.0);
    root.dispatch_touch(&TouchEvent::Up(Point::new(140.0, -30.0)));
    assert_eq!(controller.offset(), 100.0);
    assert!(!root.captures_touch());
}

#[test]
fn dragging_at_top_boundary_clears_button_press_without_activating_it() {
    let controller = ScrollController::new();
    let taps = Arc::new(AtomicUsize::new(0));
    let make = || {
        let taps = taps.clone();
        Container::new().width(100.0).height(80.0).child(
            SingleChildScrollView::new(
                Container::new().width(100.0).height(300.0).child(
                    ElevatedButton::new(Text::new("Tap"))
                        .style(
                            ButtonStyle::new()
                                .size(100.0, 60.0)
                                .color(Color::WHITE)
                                .pressed_color(Color::from_hex(0xff0000)),
                        )
                        .on_pressed(move || {
                            taps.fetch_add(1, Ordering::Relaxed);
                        }),
                ),
            )
            .controller(controller.clone())
            .raster_cache(1, Color::BLACK),
        )
    };
    let size = Size::new(100.0, 80.0);
    let mut backend = HeadlessBackend::new(100, 80);
    let mut app = App::new(make(), size);
    app.step(&mut backend);
    let resting = backend.pixels.clone();
    backend.event(TouchEvent::Down(Point::new(50.0, 20.0)));
    app.step_with_builder(&mut backend, |_| make());
    assert!(backend.pixels != resting);
    backend.event(TouchEvent::Move(Point::new(50.0, 40.0)));
    app.step_with_builder(&mut backend, |_| make());
    assert_eq!(controller.offset(), 0.0);
    assert!(
        backend.pixels == resting,
        "canceled press must redraw even when already at the scroll boundary"
    );
    backend.event(TouchEvent::Up(Point::new(50.0, 40.0)));
    app.step_with_builder(&mut backend, |_| make());
    assert_eq!(taps.load(Ordering::Relaxed), 0);
}
