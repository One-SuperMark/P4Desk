use app_launcher::{build_launcher_ui, headless::HeadlessBackend, ActiveApp, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;
const SIZE: Size = Size::new(1024.0, 600.0);

struct Harness {
    state: Arc<Mutex<LauncherState>>,
    app: App,
    backend: HeadlessBackend,
}
impl Harness {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
        let mut backend = HeadlessBackend::new(1024, 600);
        app.step(&mut backend);
        Self {
            state,
            app,
            backend,
        }
    }
    fn event(&mut self, event: TouchEvent) {
        self.backend.event(event);
        self.app.step_with_builder(&mut self.backend, |s| {
            build_launcher_ui(self.state.clone(), s)
        });
    }
    fn tap(&mut self, x: f32, y: f32) {
        self.event(TouchEvent::Down(Point::new(x, y)));
        self.event(TouchEvent::Up(Point::new(x, y)));
    }
    fn tick(&mut self, ms: u64) {
        self.state.lock().unwrap().tick(ms, 0);
        self.app.request_rebuild();
        self.app.step_with_builder(&mut self.backend, |s| {
            build_launcher_ui(self.state.clone(), s)
        });
    }
}

#[test]
fn second_page_entries_launch_from_their_icons_and_home_preserves_the_page() {
    let mut h = Harness::new();
    // Real swipe, starting in the inter-row gap, cannot launch an application.
    h.event(TouchEvent::Down(Point::new(700.0, 380.0)));
    h.event(TouchEvent::Move(Point::new(200.0, 380.0)));
    h.event(TouchEvent::Up(Point::new(200.0, 380.0)));
    assert_eq!(h.state.lock().unwrap().page_controller.page(), 1);
    assert!(matches!(
        h.state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    std::thread::sleep(std::time::Duration::from_millis(300));
    h.app
        .step_with_builder(&mut h.backend, |s| build_launcher_ui(h.state.clone(), s));
    for (i, (id, x)) in [
        ("file-manager", 132.0),
        ("office-viewer", 348.0),
        ("sub2api-monitor", 564.0),
    ]
    .into_iter()
    .enumerate()
    {
        h.tap(x, 278.0);
        assert_eq!(h.state.lock().unwrap().active_app.id(), Some(id));
        h.tick(i as u64 * 1000 + 130);
        let mut fresh = HeadlessBackend::new(1024, 600);
        h.state
            .lock()
            .unwrap()
            .desktop_backdrop
            .lock()
            .unwrap()
            .take();
        App::new(build_launcher_ui(h.state.clone(), SIZE), SIZE).step(&mut fresh);
        assert!(
            h.backend.pixels == fresh.pixels,
            "second-page launch cache and vector fallback differ"
        );
        h.tick((i as u64 + 1) * 1000);
        h.tap(46.0, 30.0);
        let s = h.state.lock().unwrap();
        assert!(matches!(s.active_app, ActiveApp::Launcher));
        assert_eq!(s.page_controller.page(), 1);
        assert!(s.running_apps.contains_key(id));
        assert!(s.recent_app_ids().any(|recent| recent == id));
    }
    // First page dot remains an explicit route back to the original eight apps.
    h.tap(442.0, 580.0);
    assert_eq!(h.state.lock().unwrap().page_controller.page(), 0);
}

#[test]
fn new_entries_close_and_survive_session_serialization_as_recent_apps() {
    use app_launcher::session::Session;
    let mut s = LauncherState::new();
    for id in ["file-manager", "office-viewer", "sub2api-monitor"] {
        s.open_app(id);
        s.kill_active_app();
    }
    s.open_app("office-viewer");
    let saved: Session =
        serde_json::from_slice(&serde_json::to_vec(&Session::capture(&s)).unwrap()).unwrap();
    let mut restored = LauncherState::new();
    assert!(saved.restore(&mut restored));
    assert_eq!(restored.active_app.id(), Some("office-viewer"));
    assert_eq!(
        restored.recent_app_ids().collect::<Vec<_>>(),
        vec!["file-manager", "sub2api-monitor", "office-viewer"]
    );
    assert!(
        restored.running_apps.is_empty(),
        "closed placeholders must not be resurrected in background"
    );
}

#[test]
fn all_colloid_vectors_render_and_switch_with_appearance() {
    use tiny_flutter::graphics::colloid_icons_generated::ALL_COLLOID_ICONS;
    assert_eq!(ALL_COLLOID_ICONS.len(), 42);
    for (name, icon) in ALL_COLLOID_ICONS {
        for side in [28, 58, 140] {
            let mut pixels = tiny_gfx::Pixmap565::new(side, side).unwrap();
            icon.paint(
                &mut Canvas::new(pixels.as_mut()),
                Rect::from_ltwh(0.0, 0.0, side as f32, side as f32),
                Color::WHITE,
            );
            assert_eq!(pixels.data()[0], 0, "{name} corner not transparent");
            assert!(pixels.data().iter().any(|p| *p != 0), "{name} is empty");
        }
    }
    for (id, _) in app_launcher::launcher_ui::DESKTOP_ENTRIES {
        Folio::configure(false, 65);
        let dark = app_launcher::app_icons::get_app_icon_asset(id).unwrap();
        Folio::configure(true, 65);
        let light = app_launcher::app_icons::get_app_icon_asset(id).unwrap();
        assert!(
            !std::ptr::eq(dark, light),
            "{id} did not select the other theme"
        );
    }
}

#[test]
fn usb_first_frame_uses_explicit_theme_on_the_display_owner_thread() {
    for light in [false, true] {
        let pixels = std::thread::spawn(move || {
            Folio::configure(!light, 65); // Intentionally different UI-thread state.
            let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
            app_launcher::app_launch::paint_usb_display_reveal(
                &mut Canvas::new(pixels.as_mut()),
                SIZE,
                0,
                600,
                light,
                app_launcher::icon_theme::IconTheme::Colloid,
            );
            pixels
        })
        .join()
        .unwrap();
        let color = app_launcher::app_icons::launch_color_for("display", light).to_rgb565();
        assert!(pixels.data().iter().all(|p| *p == color));
    }
}
