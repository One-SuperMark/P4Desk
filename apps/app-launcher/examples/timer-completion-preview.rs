//! Reproducible completion frames, using the production full-screen repaint path.
//! cargo run -p app-launcher --features screenshots --example timer-completion-preview -- artifacts/timer-completion
use app_launcher::headless::HeadlessBackend;
use app_launcher::timer_completion::COMPLETION_DURATION_MS;
use app_launcher::{build_launcher_ui, LauncherState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/timer-completion".into()),
    );
    std::fs::create_dir_all(&out)?;
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        s.open_app("timer");
        s.timer.set_countdown_seconds(10);
        s.timer.toggle(0);
        s.tick(9_500, 0);
    }
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    let frames = (500 + COMPLETION_DURATION_MS + 1_000).div_ceil(33);
    for frame in 0..=frames {
        let now = 9_500 + frame * 33;
        let (changed, dirty) = {
            let mut s = state.lock().unwrap();
            let changed = s.tick(now, 0);
            (changed, s.take_timer_animation_dirty(size))
        };
        if changed {
            app.request_rebuild();
        }
        if let Some(r) = dirty {
            app.mark_dirty(r);
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{frame:03}.png")))?;
    }
    Ok(())
}
