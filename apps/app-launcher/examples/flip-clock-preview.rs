//! Render only synthetic clock data with the same Pad widget and dirty path.
//! cargo run -p app-launcher --features screenshots --example flip-clock-preview -- artifacts/flip-clock

use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, LauncherState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/flip-clock".into()),
    );
    std::fs::create_dir_all(&out)?;
    let state = Arc::new(Mutex::new(LauncherState::new()));
    // 2026-09-29 20:06:58 UTC: show a seconds turn and the minute carry.
    // The time is deliberately fixed for reproducible UI review.
    let start = 1_790_712_418_000;
    {
        let mut s = state.lock().unwrap();
        s.settings.timezone_minutes = 0;
        s.tick(0, start);
        s.open_app("clock");
    }
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    // Finish the last turn before the preview loops back to its fixed start.
    for frame in 0..=190 {
        let now = frame * 20;
        let (changed, dirty) = {
            let mut s = state.lock().unwrap();
            let changed = s.tick(now, start + now as i64);
            (changed, s.take_clock_animation_dirty(size))
        };
        if changed {
            app.request_rebuild();
        }
        if let Some(rect) = dirty {
            app.mark_dirty(rect);
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{frame:03}.png")))?;
    }
    #[cfg(feature = "screenshots")]
    backend.screenshot(&out.join("clock.png"))?;
    Ok(())
}
