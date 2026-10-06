//! Exercise production hit-testing and rebuilds, including chart gestures and pagination.
use app_launcher::{build_launcher_ui, headless::HeadlessBackend, usage::*, LauncherState};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point as P, Size, TouchEvent};
fn fixture(page: Page) -> Arc<Mutex<LauncherState>> {
    let mut s = LauncherState::new();
    s.open_app("sub2api-monitor");
    s.usage.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    s.usage.page = page;
    let mut d = Data::default();
    for i in 0..7 {
        d.models.push(Model {
            name: format!("model-{i}"),
            totals: Totals {
                tokens: 700 - i * 100,
                cost: 1.5,
                requests: Some(3),
            },
            input: 100,
            cache: 400,
            output: 200,
        });
    }
    for i in 0..4 {
        d.trend.push(Point {
            date: format!("2026-10-06 {i:02}:00"),
            totals: Totals {
                tokens: i * 100,
                cost: 0.5,
                requests: Some(2),
            },
        });
    }
    s.usage.data = Some(Arc::new(d));
    Arc::new(Mutex::new(s))
}
fn tap(app: &mut App, b: &mut HeadlessBackend, state: &Arc<Mutex<LauncherState>>, x: f32, y: f32) {
    for e in [TouchEvent::Down(P::new(x, y)), TouchEvent::Up(P::new(x, y))] {
        b.event(e);
        app.step_with_builder(b, |size| build_launcher_ui(state.clone(), size));
    }
}
#[test]
fn model_selection_and_bidirectional_pagination_show_visible_details() {
    let state = fixture(Page::Models);
    let size = Size::new(1024., 600.);
    let mut b = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut b);
    tap(&mut app, &mut b, &state, 230., 276.);
    assert_eq!(state.lock().unwrap().usage.model_selected, Some(1));
    tap(&mut app, &mut b, &state, 982., 568.);
    assert_eq!(state.lock().unwrap().usage.list_page, 1);
    assert_eq!(state.lock().unwrap().usage.model_selected, None);
    tap(&mut app, &mut b, &state, 230., 208.);
    assert_eq!(state.lock().unwrap().usage.model_selected, Some(5));
    tap(&mut app, &mut b, &state, 860., 568.);
    assert_eq!(state.lock().unwrap().usage.list_page, 0);
    assert_eq!(state.lock().unwrap().usage.model_selected, None);
}
#[test]
fn chart_selects_nearest_bucket_but_drag_does_not_become_a_tap() {
    let state = fixture(Page::Overview);
    let size = Size::new(1024., 600.);
    let mut b = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut b);
    tap(&mut app, &mut b, &state, 44., 446.);
    assert_eq!(state.lock().unwrap().usage.chart_selected, Some(0));
    for e in [
        TouchEvent::Down(P::new(560., 446.)),
        TouchEvent::Move(P::new(410., 446.)),
        TouchEvent::Up(P::new(410., 446.)),
    ] {
        b.event(e);
        app.step_with_builder(&mut b, |size| build_launcher_ui(state.clone(), size));
    }
    assert_eq!(state.lock().unwrap().usage.chart_selected, Some(0));
    tap(&mut app, &mut b, &state, 576., 446.);
    assert_eq!(state.lock().unwrap().usage.chart_selected, Some(3));
}
#[test]
fn headline_top_model_opens_that_model_instead_of_default_first() {
    let state = fixture(Page::Overview);
    let size = Size::new(1024., 600.);
    let mut b = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut b);
    tap(&mut app, &mut b, &state, 755., 456.);
    let s = state.lock().unwrap();
    assert_eq!(s.usage.page, Page::Models);
    assert_eq!(s.usage.model_selected, Some(1));
}

#[test]
fn refresh_clamps_detail_model_paging_by_models_instead_of_user_list() {
    let state = fixture(Page::Users);
    let mut s = state.lock().unwrap();
    s.usage.detail = Some(9);
    s.usage.list_page = 1;
    s.usage.model_selected = Some(4);
    s.usage.clamp_selection();
    assert_eq!(s.usage.list_page, 1);
    assert_eq!(s.usage.model_selected, Some(4));
    let mut d = (*s.usage.data.as_ref().unwrap().as_ref()).clone();
    d.models.truncate(2);
    s.usage.data = Some(Arc::new(d));
    s.usage.clamp_selection();
    assert_eq!(s.usage.list_page, 0);
    assert_eq!(s.usage.model_selected, None);
}
