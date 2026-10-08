//! Verify list text, avatar initials and detail titles use the TF face,
//! including a character already available in the built-in UI subset.
use app_launcher::{build_launcher_ui, headless::HeadlessBackend, usage::*, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::font::{
    install_typeface_provider, Glyph, TypefaceMetrics, TypefaceProvider,
};
use tiny_flutter::{App, Point as TouchPoint, Size, TouchEvent};

struct TestFace(Arc<Mutex<Vec<(char, u16)>>>);
impl TypefaceProvider for TestFace {
    fn metrics(&self, ch: char, size: u16) -> Option<TypefaceMetrics> {
        matches!(ch, '钟' | '邃').then_some(TypefaceMetrics {
            width: 10,
            height: 10,
            xmin: 0,
            ymin: 0,
            advance: size as f32 + 4.,
        })
    }
    fn rasterize(&self, ch: char, size: u16) -> Option<Glyph> {
        let metrics = self.metrics(ch, size)?;
        self.0.lock().unwrap().push((ch, size));
        Glyph::from_alpha8(metrics, vec![211; 100]).ok()
    }
}
struct ProviderGuard;
impl Drop for ProviderGuard {
    fn drop(&mut self) {
        install_typeface_provider(None);
    }
}

#[test]
fn user_list_avatar_and_detail_title_resolve_the_same_tf_typeface() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    install_typeface_provider(Some(Arc::new(TestFace(calls.clone()))));
    let _guard = ProviderGuard;
    let mut state = LauncherState::new();
    state.open_app("sub2api-monitor");
    state.usage.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    state.usage.page = Page::Users;
    state.usage.data = Some(Arc::new(Data {
        users: vec![User {
            id: 17,
            name: Some("钟邃".into()),
            label: "用户#17".into(),
            totals: Totals {
                tokens: 1200,
                cost: 0.15,
                requests: Some(3),
            },
        }],
        ..Data::default()
    }));
    state.usage.remember_page(
        &Scope {
            page: Page::Users,
            period: Period::Day,
            detail: None,
            date: "2026-10-08".into(),
            timezone_minutes: 480,
        },
        state.usage.data.clone().unwrap(),
    );
    let state = Arc::new(Mutex::new(state));
    let size = Size::new(1024., 600.);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    {
        let calls = calls.lock().unwrap();
        assert!(
            calls.contains(&('钟', 18)),
            "name must use TF even when UI has the character"
        );
        assert!(
            calls.contains(&('邃', 18)),
            "name must render beyond the UI subset"
        );
        assert!(
            calls.contains(&('钟', 14)),
            "avatar must use the same TF face"
        );
    }
    for event in [
        TouchEvent::Down(TouchPoint::new(410., 212.)),
        TouchEvent::Up(TouchPoint::new(410., 212.)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert_eq!(state.lock().unwrap().usage.detail, Some(17));
    // Detail responses contain models/trends, while the name comes from the
    // retained users snapshot; render the same shape as a completed response.
    state.lock().unwrap().usage.data = Some(Arc::new(Data::default()));
    let mut detail = App::new(build_launcher_ui(state.clone(), size), size);
    detail.step(&mut backend);
    assert!(
        calls.lock().unwrap().contains(&('邃', 14)),
        "detail title must render the TF name"
    );
}
