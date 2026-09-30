//! Synthetic host timings for the production settings scroll path, not device FPS.
use app_launcher::{
    build_launcher_ui, headless::HeadlessBackend, radio::SettingsSection, LauncherState,
};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tiny_flutter::prelude::*;

fn main() {
    let mut s = LauncherState::new();
    s.open_app("settings");
    s.settings.icon_theme = app_launcher::icon_theme::IconTheme::WhiteSur;
    s.settings_view.section = SettingsSection::Appearance;
    let state = Arc::new(Mutex::new(s));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    let now = Instant::now();
    app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
    let cold_us = now.elapsed().as_micros();
    backend.event(TouchEvent::Down(Point::new(740.0, 530.0)));
    app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
    app.take_frame_metrics();
    let mut timings = Vec::new();
    for y in (0..=42)
        .map(|i| 530.0 - i as f32 * 10.0)
        .chain((0..42).rev().map(|i| 530.0 - i as f32 * 10.0))
    {
        backend.event(TouchEvent::Move(Point::new(740.0, y)));
        let start = Instant::now();
        app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
        timings.push(start.elapsed().as_micros());
    }
    timings.sort();
    let metrics = app.take_frame_metrics();
    println!(
        "{}",
        serde_json::json!({
            "scope":"arm64 synthetic actual settings event/layout/render/headless output; not device FPS",
            "cold_us":cold_us,"samples":timings.len(),"median_us":timings[timings.len()/2],
            "p95_us":timings[timings.len()*95/100],"max_us":timings.last(),
            "rendered_frames":metrics.frames,"draw_avg_us":metrics.draw_us/metrics.frames.max(1),
            "output_avg_us":metrics.output_us/metrics.frames.max(1)
        })
    );
}
