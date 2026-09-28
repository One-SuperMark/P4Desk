use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tiny_flutter::prelude::*;

struct MockBackend {
    size: Size,
    flush_count: usize,
    last_rect: Option<Rect>,
    last_pixels: Vec<u16>,
    pending_touch: Option<TouchEvent>,
}

impl MockBackend {
    fn new(width: f32, height: f32) -> Self {
        Self {
            size: Size::new(width, height),
            flush_count: 0,
            last_rect: None,
            last_pixels: Vec::new(),
            pending_touch: None,
        }
    }
}

impl PlatformBackend for MockBackend {
    fn screen_size(&self) -> Size {
        self.size
    }

    fn flush(&mut self, rect: Rect, rgb565_data: &[u16]) {
        self.flush_count += 1;
        self.last_rect = Some(rect);
        self.last_pixels = rgb565_data.to_vec();
    }

    fn poll_touch(&mut self) -> Option<TouchEvent> {
        self.pending_touch.take()
    }
}

#[test]
fn test_box_constraints() {
    let loose = BoxConstraints::loose(Size::new(100.0, 200.0));
    assert_eq!(loose.min_width, 0.0);
    assert_eq!(loose.max_width, 100.0);

    let constrained = loose.constrain(Size::new(150.0, 50.0));
    assert_eq!(constrained.width, 100.0);
    assert_eq!(constrained.height, 50.0);

    let deflated = loose.deflate(EdgeInsets::all(10.0));
    assert_eq!(deflated.max_width, 80.0);
    assert_eq!(deflated.max_height, 180.0);
}

#[test]
fn test_color_rgb565_conversion() {
    let red = Color::RED;
    assert_eq!(red.r, 244);
    let rgb565 = red.to_rgb565();
    assert!(rgb565 > 0);

    let white = Color::WHITE;
    assert_eq!(white.to_rgb565(), 0xFFFF);

    let black = Color::BLACK;
    assert_eq!(black.to_rgb565(), 0x0000);
}

#[test]
fn test_font_measuring() {
    let font = Font::default_font();
    let size = font.measure_text("Hello Flutter", 16.0);
    assert!(size.width > 0.0);
    assert!(size.height > 0.0);

    // Verify narrowed hair space advance (1/3 narrower than standard fontdue hair space)
    let s_with_space = font.measure_text("9\u{200A}×", 34.0);
    let s_without_space = font.measure_text("9×", 34.0);
    let space_diff = s_with_space.width - s_without_space.width;
    assert!(space_diff > 1.9 && space_diff < 2.6);
}

#[test]
fn test_app_lifecycle_and_touch() {
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let ui = Container::new()
        .width(200.0)
        .height(100.0)
        .color(Color::SURFACE)
        .child(
            ElevatedButton::new(Text::new("Click Me").font_size(14.0)).on_pressed(move || {
                clicked_clone.store(true, Ordering::SeqCst);
            }),
        );

    let mut backend = MockBackend::new(480.0, 480.0);
    let mut app = App::new(ui, Size::new(480.0, 480.0));

    // First frame should flush entire screen
    app.step(&mut backend);
    assert_eq!(backend.flush_count, 1);
    assert!(backend.last_pixels.len() > 0);

    // No touch -> no repaint
    app.step(&mut backend);
    assert_eq!(backend.flush_count, 1);

    // Simulate touch Down on button
    backend.pending_touch = Some(TouchEvent::Down(Point::new(50.0, 50.0)));
    app.step(&mut backend);
    assert_eq!(backend.flush_count, 2);

    // Simulate touch Up on button -> trigger on_pressed
    backend.pending_touch = Some(TouchEvent::Up(Point::new(50.0, 50.0)));
    app.step(&mut backend);
    assert_eq!(backend.flush_count, 3);
    assert!(clicked.load(Ordering::SeqCst));
}

#[test]
fn test_expanded_layout() {
    // 300px row with 2x gap (10px each) + 2x flex:1 + 1x flex:2
    // total non-flex = 20px -> free = 280px -> total flex = 4 -> unit = 70px
    // child 1 (flex 1) = 70px
    // child 2 (flex 1) = 70px
    // child 3 (flex 2) = 140px
    let row = Row::new()
        .push(Expanded::new(Container::new().color(Color::RED)))
        .push(SizedBox::square(10.0))
        .push(Expanded::new(Container::new().color(Color::GREEN)))
        .push(SizedBox::square(10.0))
        .push(Expanded::new(Container::new().color(Color::BLUE)).flex(2));

    let mut render_row = row.create_render_object();
    let constraints = BoxConstraints::tight(Size::new(300.0, 50.0));
    let size = render_row.layout(&constraints);

    assert_eq!(size.width, 300.0);
    assert_eq!(size.height, 50.0);
}

#[test]
fn test_dsl_macros_and_widget_ext() {
    let tree = col![
        Text::new("Header").centered(),
        SizedBox::square(12.0),
        row![
            Text::new("A").padding_all(4.0).expanded(1),
            Text::new("B").padding_symmetric(2.0, 8.0).expanded(2),
        ],
    ];

    let mut render_tree = tree.create_render_object();
    let size = render_tree.layout(&BoxConstraints::tight(Size::new(200.0, 100.0)));
    assert_eq!(size, Size::new(200.0, 100.0));
}

#[test]
fn test_button_style_and_touch_cancel() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let style = ButtonStyle::new()
        .color(Color::from_rgb(10, 20, 30))
        .pressed_color(Color::from_rgb(50, 60, 70))
        .size(100.0, 50.0)
        .border_radius(12.0);

    let btn = ElevatedButton::text("Test")
        .style(style)
        .on_pressed(move || clicked_clone.store(true, Ordering::SeqCst));

    let mut render_btn = btn.create_render_object();
    render_btn.layout(&BoxConstraints::tight(Size::new(100.0, 50.0)));

    // 1. Touch down inside (50, 25)
    assert!(render_btn.dispatch_touch(&TouchEvent::Down(Point::new(50.0, 25.0))));
    assert!(!clicked.load(Ordering::SeqCst));

    // 2. Slip out to (150, 25) -> button resets pressed state
    assert!(render_btn.dispatch_touch(&TouchEvent::Move(Point::new(150.0, 25.0))));

    // 3. Touch Up outside -> click handler NOT executed
    render_btn.dispatch_touch(&TouchEvent::Up(Point::new(150.0, 25.0)));
    assert!(!clicked.load(Ordering::SeqCst));

    // 4. Touch Down inside and Cancel -> click handler NOT executed
    render_btn.dispatch_touch(&TouchEvent::Down(Point::new(50.0, 25.0)));
    render_btn.dispatch_touch(&TouchEvent::Cancel);
    render_btn.dispatch_touch(&TouchEvent::Up(Point::new(50.0, 25.0)));
    assert!(!clicked.load(Ordering::SeqCst));

    // 5. Normal click inside Down + Up -> clicked!
    render_btn.dispatch_touch(&TouchEvent::Down(Point::new(50.0, 25.0)));
    render_btn.dispatch_touch(&TouchEvent::Up(Point::new(50.0, 25.0)));
    assert!(clicked.load(Ordering::SeqCst));
}

#[test]
fn test_theme_data_presets_and_button_styles() {
    let amoled = ThemeData::amoled();
    assert_eq!(amoled.background(), Color::from_rgb(0, 0, 0));
    assert_eq!(
        amoled.color_scheme.btn_equals,
        amoled.color_scheme.btn_operator
    );
    assert_eq!(amoled.color_scheme.btn_function_text, Color::BLACK);
    assert_eq!(amoled.color_scheme.btn_number, Color::from_rgb(51, 51, 51));
    assert_eq!(
        amoled.color_scheme.btn_operator,
        Color::from_rgb(255, 159, 10)
    );

    let num_btn = amoled.number_button_style(80.0, 40.0, 10.0);
    assert_eq!(num_btn.width, Some(80.0));
    assert_eq!(num_btn.height, Some(40.0));
    assert_eq!(num_btn.color, amoled.color_scheme.btn_number);

    let light = ThemeData::light();
    assert_eq!(light.background(), Color::from_rgb(242, 244, 248));
    assert_ne!(amoled.background(), light.background());

    let cyber = ThemeData::cyber_blue();
    assert_eq!(cyber.background(), Color::from_rgb(10, 14, 26));
}

struct TestVectorPainter;
impl CustomPainter for TestVectorPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(10.0, 10.0);
        pb.line_to(size.width - 10.0, size.height - 10.0);
        if let Some(path) = pb.finish() {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(255, 0, 0, 255);
            let stroke = tiny_skia::Stroke {
                width: 2.0,
                ..Default::default()
            };
            canvas.stroke_path(&path, &paint, &stroke);
        }
    }
}

#[test]
fn test_custom_paint_vector_drawing() {
    let custom = CustomPaint::new(TestVectorPainter).size(Size::new(200.0, 100.0));
    let mut render = custom.create_render_object();
    let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));
    let measured = render.layout(&constraints);
    assert_eq!(measured, Size::new(200.0, 100.0));

    let mut pixmap = tiny_skia::Pixmap::new(200, 100).unwrap();
    let mut canvas = Canvas::new(pixmap.as_mut());
    render.paint(&mut canvas, Offset::ZERO);
}

#[derive(Default)]
struct MockPowerBackend {
    pub events: Vec<TouchEvent>,
    pub flush_count: usize,
    pub screen_power_state: bool,
}

impl PlatformBackend for MockPowerBackend {
    fn screen_size(&self) -> Size {
        Size::new(480.0, 480.0)
    }

    fn flush(&mut self, _rect: Rect, _rgb565_data: &[u16]) {
        self.flush_count += 1;
    }

    fn poll_touch(&mut self) -> Option<TouchEvent> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    fn set_screen_power(&mut self, on: bool) {
        self.screen_power_state = on;
    }
}

#[test]
fn test_power_button_and_touch_wake() {
    let mut backend = MockPowerBackend {
        events: Vec::new(),
        flush_count: 0,
        screen_power_state: true,
    };

    let size = Size::new(480.0, 480.0);
    let mut app = App::new(Container::new().width(480.0).height(480.0), size);
    assert!(app.is_screen_on());

    // 1. Initial frame renders
    app.step(&mut backend);
    assert_eq!(backend.flush_count, 1);

    // 2. Press Button 2 (HardwarePower) -> Screen turns OFF
    backend.events.push(TouchEvent::HardwarePower);
    app.step(&mut backend);
    assert!(
        !app.is_screen_on(),
        "Screen must be off after HardwarePower"
    );
    assert!(
        !backend.screen_power_state,
        "Backend power state must be false"
    );

    // 3. While screen is off, program state runs, but UI rendering halts (no flushes!)
    let flushes_before = backend.flush_count;
    app.step(&mut backend);
    assert_eq!(
        backend.flush_count, flushes_before,
        "No flush when screen is off"
    );

    // 4. Touch screen (TouchEvent::Down) -> Screen wakes up!
    backend
        .events
        .push(TouchEvent::Down(Point::new(240.0, 240.0)));
    app.step(&mut backend);
    assert!(app.is_screen_on(), "Touch Down must wake up screen");
    assert!(
        backend.screen_power_state,
        "Backend power state must be true"
    );
    assert!(
        backend.flush_count > flushes_before,
        "Screen must repaint upon waking"
    );

    // 5. HardwarePower toggle again
    backend.events.push(TouchEvent::HardwarePower);
    app.step(&mut backend);
    assert!(!app.is_screen_on());
    backend.events.push(TouchEvent::HardwarePower);
    app.step(&mut backend);
    assert!(app.is_screen_on());
}

#[test]
fn test_step_animated_gesture_tap_across_frames() {
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let mut backend = MockPowerBackend::default();
    let size = Size::new(200.0, 200.0);

    let initial = GestureDetector::new(
        Container::new()
            .width(100.0)
            .height(100.0)
            .color(Color::WHITE),
    )
    .on_tap(move || {
        clicked_clone.store(true, Ordering::SeqCst);
    });

    let mut app = App::new(initial, size);

    // Frame 1: Mouse down at (50, 50)
    backend
        .events
        .push(TouchEvent::Down(Point::new(50.0, 50.0)));
    let c1 = clicked.clone();
    app.step_animated(&mut backend, move |_| {
        let c = c1.clone();
        GestureDetector::new(
            Container::new()
                .width(100.0)
                .height(100.0)
                .color(Color::WHITE),
        )
        .on_tap(move || {
            c.store(true, Ordering::SeqCst);
        })
    });
    assert!(
        !clicked.load(Ordering::SeqCst),
        "Should not fire tap on Down"
    );

    // Frame 2: An intermediate animation frame with no touch events (holding down)
    let c2 = clicked.clone();
    app.step_animated(&mut backend, move |_| {
        let c = c2.clone();
        GestureDetector::new(
            Container::new()
                .width(100.0)
                .height(100.0)
                .color(Color::WHITE),
        )
        .on_tap(move || {
            c.store(true, Ordering::SeqCst);
        })
    });
    assert!(
        !clicked.load(Ordering::SeqCst),
        "Should not fire tap while holding"
    );

    // Frame 3: Mouse up at (50, 50)
    backend.events.push(TouchEvent::Up(Point::new(50.0, 50.0)));
    let c3 = clicked.clone();
    app.step_animated(&mut backend, move |_| {
        let c = c3.clone();
        GestureDetector::new(
            Container::new()
                .width(100.0)
                .height(100.0)
                .color(Color::WHITE),
        )
        .on_tap(move || {
            c.store(true, Ordering::SeqCst);
        })
    });
    assert!(
        clicked.load(Ordering::SeqCst),
        "Must trigger on_tap even when rebuilt across multiple frames!"
    );
}

#[test]
fn immediate_page_swipe_changes_once_and_never_taps_neighbor() {
    use std::sync::atomic::AtomicUsize;
    let taps = Arc::new(AtomicUsize::new(0));
    let controller = PageController::new();
    let children: Vec<Box<dyn Widget>> = (0..3)
        .map(|_| {
            let t = taps.clone();
            Box::new(
                GestureDetector::new(Container::new().width(200.0).height(150.0)).on_tap(
                    move || {
                        t.fetch_add(1, Ordering::SeqCst);
                    },
                ),
            ) as Box<dyn Widget>
        })
        .collect();
    let mut render = PageView::new(children)
        .controller(controller.clone())
        .transition(PageTransition::None)
        .create_render_object();
    render.layout(&BoxConstraints::tight(Size::new(200.0, 150.0)));
    render.dispatch_touch(&TouchEvent::Down(Point::new(180.0, 75.0)));
    render.dispatch_touch(&TouchEvent::Move(Point::new(100.0, 75.0)));
    assert_eq!(controller.page(), 1);
    render.dispatch_touch(&TouchEvent::Up(Point::new(50.0, 75.0)));
    assert_eq!(controller.page(), 1);
    assert_eq!(taps.load(Ordering::SeqCst), 0);
}

struct TouchCancelProbe {
    cancels: Arc<AtomicUsize>,
}

impl Widget for TouchCancelProbe {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderTouchCancelProbe {
            cancels: self.cancels.clone(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

struct RenderTouchCancelProbe {
    cancels: Arc<AtomicUsize>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderTouchCancelProbe {
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = constraints.constrain(Size::new(200.0, 150.0));
        self.size
    }

    fn paint(&self, _canvas: &mut Canvas, _offset: Offset) {}

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if matches!(event, TouchEvent::Cancel) {
            self.cancels.fetch_add(1, Ordering::SeqCst);
        }
        false
    }
}

struct PageTouchFixture {
    render: Box<dyn RenderBox>,
    controller: PageController,
    taps: Arc<AtomicUsize>,
    cancels: Vec<Arc<AtomicUsize>>,
    page_changes: Arc<AtomicUsize>,
}

fn page_touch_fixture(initial_page: usize) -> PageTouchFixture {
    let taps = Arc::new(AtomicUsize::new(0));
    let cancels: Vec<_> = (0..3).map(|_| Arc::new(AtomicUsize::new(0))).collect();
    let page_changes = Arc::new(AtomicUsize::new(0));
    let controller = PageController::with_initial_page(initial_page);
    let children: Vec<Box<dyn Widget>> = cancels
        .iter()
        .map(|c| {
            let t = taps.clone();
            Box::new(
                GestureDetector::new(TouchCancelProbe { cancels: c.clone() }).on_tap(move || {
                    t.fetch_add(1, Ordering::SeqCst);
                }),
            ) as Box<dyn Widget>
        })
        .collect();
    let changes = page_changes.clone();
    let mut render = PageView::new(children)
        .controller(controller.clone())
        .transition(PageTransition::None)
        .on_page_changed(move |_| {
            changes.fetch_add(1, Ordering::SeqCst);
        })
        .create_render_object();
    render.layout(&BoxConstraints::tight(Size::new(200.0, 150.0)));
    PageTouchFixture {
        render,
        controller,
        taps,
        cancels,
        page_changes,
    }
}

#[test]
fn page_view_requires_live_down_and_rearms_after_cancel() {
    let mut f = page_touch_fixture(1);
    for event in [
        TouchEvent::Move(Point::new(150.0, 75.0)),
        TouchEvent::Up(Point::new(150.0, 75.0)),
        TouchEvent::Down(Point::new(250.0, 75.0)),
        TouchEvent::Move(Point::new(150.0, 75.0)),
        TouchEvent::Up(Point::new(150.0, 75.0)),
    ] {
        assert!(!f.render.dispatch_touch(&event));
        assert_eq!(f.controller.page(), 1);
    }

    f.controller.set_page(0);
    f.render
        .layout(&BoxConstraints::tight(Size::new(200.0, 150.0)));
    f.render
        .dispatch_touch(&TouchEvent::Down(Point::new(180.0, 75.0)));
    f.render
        .dispatch_touch(&TouchEvent::Move(Point::new(100.0, 75.0)));
    assert_eq!(f.controller.page(), 1);
    f.render.dispatch_touch(&TouchEvent::Cancel);
    for event in [
        TouchEvent::Move(Point::new(190.0, 75.0)),
        TouchEvent::Up(Point::new(100.0, 75.0)),
    ] {
        assert!(!f.render.dispatch_touch(&event));
    }
    assert_eq!(f.controller.page(), 1);
    assert_eq!(f.page_changes.load(Ordering::SeqCst), 1);
    assert_eq!(f.taps.load(Ordering::SeqCst), 0);

    f.render
        .dispatch_touch(&TouchEvent::Down(Point::new(100.0, 75.0)));
    f.render
        .dispatch_touch(&TouchEvent::Up(Point::new(100.0, 75.0)));
    assert_eq!(f.taps.load(Ordering::SeqCst), 1);
}

#[test]
fn page_view_completed_touch_ignores_late_move_and_up() {
    let mut f = page_touch_fixture(1);
    f.render
        .dispatch_touch(&TouchEvent::Down(Point::new(100.0, 75.0)));
    f.render
        .dispatch_touch(&TouchEvent::Up(Point::new(100.0, 75.0)));
    assert_eq!(f.taps.load(Ordering::SeqCst), 1);
    for event in [
        TouchEvent::Move(Point::new(20.0, 75.0)),
        TouchEvent::Up(Point::new(20.0, 75.0)),
    ] {
        assert!(!f.render.dispatch_touch(&event));
    }
    assert_eq!(f.controller.page(), 1);
    assert_eq!(f.page_changes.load(Ordering::SeqCst), 0);
    assert_eq!(f.taps.load(Ordering::SeqCst), 1);
}

#[test]
fn page_view_up_only_swipe_cancels_origin_child() {
    // The second sequence reaches the Up threshold after a Move below the
    // higher Move threshold. Both paths must cancel the original child.
    for movement in [None, Some(Point::new(150.0, 75.0))] {
        let mut f = page_touch_fixture(0);
        f.render
            .dispatch_touch(&TouchEvent::Down(Point::new(180.0, 75.0)));
        if let Some(point) = movement {
            f.render.dispatch_touch(&TouchEvent::Move(point));
            assert_eq!(f.controller.page(), 0);
        }
        let end = movement.unwrap_or(Point::new(100.0, 75.0));
        assert!(f.render.dispatch_touch(&TouchEvent::Up(end)));
        assert_eq!(f.controller.page(), 1);
        assert_eq!(f.page_changes.load(Ordering::SeqCst), 1);
        assert_eq!(f.cancels[0].load(Ordering::SeqCst), 1);
        assert_eq!(f.cancels[1].load(Ordering::SeqCst), 0);
        assert_eq!(f.taps.load(Ordering::SeqCst), 0);
        assert!(!f.render.dispatch_touch(&TouchEvent::Up(end)));
        assert_eq!(f.controller.page(), 1);
    }
}

#[test]
fn page_view_boundary_swipes_never_launch_apps() {
    for (initial, start, end) in [
        (0, 40.0, 120.0),
        (2, 180.0, 100.0),
        (0, 40.0, 250.0),
        (2, 180.0, -40.0),
    ] {
        for with_move in [false, true] {
            let mut f = page_touch_fixture(initial);
            f.render
                .dispatch_touch(&TouchEvent::Down(Point::new(start, 75.0)));
            if with_move {
                f.render
                    .dispatch_touch(&TouchEvent::Move(Point::new(end, 75.0)));
            }
            assert!(f
                .render
                .dispatch_touch(&TouchEvent::Up(Point::new(end, 75.0))));
            assert_eq!(f.controller.page(), initial);
            assert_eq!(f.page_changes.load(Ordering::SeqCst), 0);
            assert!(f.cancels[initial].load(Ordering::SeqCst) > 0);
            assert_eq!(f.taps.load(Ordering::SeqCst), 0);
        }
    }
}

#[test]
fn moved_finger_inside_large_tile_does_not_become_tap() {
    let tapped = Arc::new(AtomicBool::new(false));
    let t = tapped.clone();
    let mut render = GestureDetector::new(Container::new().width(300.0).height(120.0))
        .on_tap(move || t.store(true, Ordering::SeqCst))
        .create_render_object();
    render.layout(&BoxConstraints::tight(Size::new(300.0, 120.0)));
    render.dispatch_touch(&TouchEvent::Down(Point::new(30.0, 50.0)));
    render.dispatch_touch(&TouchEvent::Move(Point::new(150.0, 50.0)));
    render.dispatch_touch(&TouchEvent::Up(Point::new(150.0, 50.0)));
    assert!(!tapped.load(Ordering::SeqCst));
}
#[test]
fn dirty_rect_on_odd_sized_dsi_surface_keeps_stride_and_bounds() {
    let mut pix = tiny_gfx::Pixmap565::new(1023, 599).unwrap();
    pix.data_mut().fill(0x1234);
    let mut out = vec![0; 30];
    let (x, y, right, bottom, n) =
        pix.extract_rect(tiny_gfx::Rect::from_ltwh(1018.0, 596.0, 5.0, 3.0), &mut out);
    assert_eq!((x, y, right, bottom, n), (1018, 596, 1023, 599, 15));
    assert!(out[..n].iter().all(|p| *p == 0x1234));
}
