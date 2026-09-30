use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const SIZE: Size = Size::new(1024.0, 600.0);
const CLOCK: Point = Point::new(132.0, 278.0);
struct TrackedBackend {
    inner: HeadlessBackend,
    flushes: Vec<Rect>,
}
impl PlatformBackend for TrackedBackend {
    fn screen_size(&self) -> Size {
        SIZE
    }
    fn begin_frame(&mut self) {
        self.inner.begin_frame();
    }
    fn end_frame(&mut self) {
        self.inner.end_frame();
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        self.inner.poll_touch()
    }
    fn flush(&mut self, rect: Rect, pixels: &[u16]) {
        self.flushes.push(rect);
        self.inner.flush(rect, pixels);
    }
}
fn setup() -> (Arc<Mutex<LauncherState>>, App, TrackedBackend) {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    let mut backend = TrackedBackend {
        inner: HeadlessBackend::new(1024, 600),
        flushes: Vec::new(),
    };
    app.step(&mut backend);
    backend.flushes.clear();
    (state, app, backend)
}

#[test]
fn continuous_swipe_keeps_adjacent_pages_and_commits_only_after_halfway_release() {
    let (state, mut app, mut backend) = setup();
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Down(Point::new(780.0, 278.0)),
    );
    for x in [700.0, 560.0, 400.0, 280.0] {
        backend.flushes.clear();
        event(
            &state,
            &mut app,
            &mut backend,
            TouchEvent::Move(Point::new(x, 278.0)),
        );
        let s = state.lock().unwrap();
        assert_eq!(
            s.current_page, 0,
            "page changes on release, not while dragging"
        );
        assert!((s.page_controller.drag_offset() - (x - 780.0)).abs() < 0.01);
        assert!(
            !backend.flushes.is_empty(),
            "each drag must move both pages"
        );
    }
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Up(Point::new(280.0, 278.0)),
    );
    assert_eq!(state.lock().unwrap().current_page, 1);
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    std::thread::sleep(std::time::Duration::from_millis(300));
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    assert_eq!(state.lock().unwrap().page_controller.drag_offset(), 0.0);
    assert!(!state.lock().unwrap().page_controller.is_settling());
}
#[test]
fn short_drag_and_release_without_move_never_activate_a_tile() {
    for (end, include_move) in [
        (Point::new(70.0, 278.0), true),
        (Point::new(132.0, 310.0), false),
    ] {
        let (state, mut app, mut backend) = setup();
        event(&state, &mut app, &mut backend, TouchEvent::Down(CLOCK));
        if include_move {
            event(&state, &mut app, &mut backend, TouchEvent::Move(end));
        }
        event(&state, &mut app, &mut backend, TouchEvent::Up(end));
        assert!(matches!(
            state.lock().unwrap().active_app,
            ActiveApp::Launcher
        ));
        assert_eq!(state.lock().unwrap().current_page, 0);
        std::thread::sleep(std::time::Duration::from_millis(300));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        assert_eq!(state.lock().unwrap().page_controller.drag_offset(), 0.0);
    }
}

fn event(
    state: &Arc<Mutex<LauncherState>>,
    app: &mut App,
    backend: &mut TrackedBackend,
    e: TouchEvent,
) {
    backend.inner.event(e);
    app.step_with_builder(backend, |size| build_launcher_ui(state.clone(), size));
}
#[test]
fn desktop_press_uses_only_the_touched_slot_and_matches_a_fresh_full_frame() {
    let (state, mut app, mut backend) = setup();
    let before = backend.inner.pixels.clone();
    event(&state, &mut app, &mut backend, TouchEvent::Down(CLOCK));
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    assert_ne!(
        before, backend.inner.pixels,
        "Down needs visible feedback before Up"
    );
    assert!(!backend.flushes.is_empty());
    let area: f32 = backend.flushes.iter().map(|r| r.width * r.height).sum();
    assert!(
        area < 60_000.0,
        "a press must not repaint 614400 pixels: {area}"
    );
    for (i, pixel) in backend.inner.pixels.iter().enumerate() {
        let p = Point::new((i % 1024) as f32, (i / 1024) as f32);
        if !backend.flushes.iter().any(|r| r.contains(p)) {
            assert_eq!(*pixel, before[i], "press changed an unrelated pixel");
        }
    }
    // Compare the production clipped update with a newly painted pressed tree.
    let mut root = build_launcher_ui(state.clone(), SIZE).create_render_object();
    root.layout(&BoxConstraints::tight(SIZE));
    root.dispatch_touch(&TouchEvent::Down(CLOCK));
    let mut full = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
    root.paint(&mut Canvas::new(full.as_mut()), Offset::ZERO);
    assert!(
        backend.inner.pixels == full.data(),
        "clipped press differs from full render"
    );
    event(&state, &mut app, &mut backend, TouchEvent::Cancel);
    assert_eq!(
        backend.inner.pixels, before,
        "Cancel must erase shrink, tint and outline"
    );
}
#[test]
fn all_eight_desktop_buttons_expose_their_own_damage_rectangle() {
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let mut root = build_launcher_ui(state, SIZE).create_render_object();
    root.layout(&BoxConstraints::tight(SIZE));
    for y in [278.0, 466.0] {
        for x in [132.0, 348.0, 564.0, 780.0] {
            let p = Point::new(x, y);
            root.dispatch_touch(&TouchEvent::Down(p));
            let r = root
                .hit_rect(p)
                .expect("wallpaper must delegate child damage");
            assert!(r.contains(p));
            assert!(
                r.width <= 180.0 && r.height <= 200.0,
                "got page damage: {r:?}"
            );
            root.dispatch_touch(&TouchEvent::Cancel);
        }
    }
}
#[test]
fn canceled_drag_clears_feedback_and_returning_to_origin_never_opens_an_app() {
    for excursion in [
        Point::new(132.0, 307.0),
        Point::new(132.0, 466.0),
        // Outward on page one: cancels the press without paging. An inward
        // swipe now correctly reaches the requested second page of apps.
        Point::new(174.0, 278.0),
    ] {
        let (state, mut app, mut backend) = setup();
        let before = backend.inner.pixels.clone();
        event(&state, &mut app, &mut backend, TouchEvent::Down(CLOCK));
        event(&state, &mut app, &mut backend, TouchEvent::Move(excursion));
        if excursion.x == CLOCK.x {
            assert!(
                backend.inner.pixels == before,
                "vertical drag must erase feedback"
            );
        } else {
            assert!(
                state.lock().unwrap().page_controller.drag_offset() > 0.0,
                "outward drag should show bounded rubber resistance"
            );
        }
        event(&state, &mut app, &mut backend, TouchEvent::Move(CLOCK));
        event(&state, &mut app, &mut backend, TouchEvent::Up(CLOCK));
        let mut s = state.lock().unwrap();
        assert!(matches!(s.active_app, ActiveApp::Launcher));
        assert!(s.running_apps.is_empty());
        assert!(s.take_commands().is_empty());
        assert!(
            backend.inner.pixels == before,
            "release must restore the unpressed page"
        );
    }
}
#[test]
fn small_touch_jitter_keeps_press_feedback_and_opens_on_release() {
    let (state, mut app, mut backend) = setup();
    event(&state, &mut app, &mut backend, TouchEvent::Down(CLOCK));
    let pressed = backend.inner.pixels.clone();
    let p = Point::new(CLOCK.x + 3.0, CLOCK.y + 4.0);
    event(&state, &mut app, &mut backend, TouchEvent::Move(p));
    assert_eq!(backend.inner.pixels, pressed);
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    event(&state, &mut app, &mut backend, TouchEvent::Up(p));
    assert!(matches!(state.lock().unwrap().active_app, ActiveApp::Clock));
    let revision = state.lock().unwrap().revision;
    // A duplicate/late release must not open another app or repeat the action.
    event(&state, &mut app, &mut backend, TouchEvent::Up(p));
    assert_eq!(state.lock().unwrap().revision, revision);
}

#[test]
fn settling_repaints_only_the_grid_without_rebuilding_the_desktop() {
    let (state, mut app, mut backend) = setup();
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Down(Point::new(780.0, 380.0)),
    );
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Move(Point::new(450.0, 380.0)),
    );
    backend
        .inner
        .event(TouchEvent::Up(Point::new(450.0, 380.0)));
    backend.flushes.clear();
    let mut rebuilds = 0;
    app.step_with_builder(&mut backend, |size| {
        rebuilds += 1;
        build_launcher_ui(state.clone(), size)
    });
    assert_eq!(rebuilds, 0, "release must keep the active tree");
    assert!(state.lock().unwrap().page_controller.is_settling());
    assert!(!backend.flushes.is_empty());
    assert!(backend
        .flushes
        .iter()
        .all(|r| r.y >= 208.0 && r.height <= 376.0 && r.width <= 864.0));
    app.step_with_builder(&mut backend, |size| {
        rebuilds += 1;
        build_launcher_ui(state.clone(), size)
    });
    assert_eq!(rebuilds, 0, "animation frame must not remount widgets");
    std::thread::sleep(std::time::Duration::from_millis(240));
    app.step_with_builder(&mut backend, |size| {
        rebuilds += 1;
        build_launcher_ui(state.clone(), size)
    });
    assert_eq!(
        rebuilds, 1,
        "apply deferred revisions and refresh launch backdrop at rest"
    );
    assert_eq!(state.lock().unwrap().page_controller.drag_offset(), 0.0);
}

#[test]
fn owned_drag_follows_outside_grid_and_regrabbing_edge_bounce_does_not_jump() {
    let (state, mut app, mut backend) = setup();
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Down(Point::new(132.0, 278.0)),
    );
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Move(Point::new(260.0, 278.0)),
    );
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Up(Point::new(260.0, 278.0)),
    );
    let before = state.lock().unwrap().page_controller.drag_offset();
    assert!(before > 1.0);
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Down(Point::new(260.0, 278.0)),
    );
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Move(Point::new(260.0, 278.0)),
    );
    assert!((state.lock().unwrap().page_controller.drag_offset() - before).abs() < 0.01);
    event(&state, &mut app, &mut backend, TouchEvent::Cancel);
    state.lock().unwrap().page_controller.set_page(0);
    app.request_rebuild();
    app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Down(Point::new(700.0, 380.0)),
    );
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Move(Point::new(0.0, 380.0)),
    );
    assert_eq!(state.lock().unwrap().page_controller.drag_offset(), -700.0);
    event(
        &state,
        &mut app,
        &mut backend,
        TouchEvent::Up(Point::new(0.0, 380.0)),
    );
    assert_eq!(state.lock().unwrap().current_page, 1);
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
}
