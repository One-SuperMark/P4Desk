//! Synthetic screenshots from the actual 1024x600 Rust firmware UI.
//! cargo run -p app-launcher --features screenshots --example pomodoro-preview -- docs/images
use app_launcher::headless::HeadlessBackend;
use app_launcher::timer::Phase;
use app_launcher::{build_launcher_ui, LauncherState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/pomodoro".into()),
    );
    std::fs::create_dir_all(&out)?;
    for name in [
        "focus",
        "running",
        "paused",
        "short-break",
        "long-break",
        "finished",
        "countdown",
    ] {
        let mut state = LauncherState::new();
        state.open_app("timer");
        match name {
            "running" | "paused" => {
                state.timer.toggle(0);
                state.tick(312_000, 0);
                if name == "paused" {
                    state.timer.toggle(312_000);
                }
            }
            "short-break" => state.timer.set_phase(Phase::Break),
            "long-break" => {
                state.timer.completed_cycles = 4;
                state.timer.set_phase(Phase::LongBreak);
            }
            "finished" => {
                state.timer.completed_cycles = 3;
                state.timer.toggle(0);
                state.tick(1_500_000, 0);
            }
            "countdown" => state.timer.set_countdown_seconds(10),
            _ => {}
        }
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(Arc::new(Mutex::new(state)), size), size).step(&mut backend);
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("pomodoro-{name}.png")))?;
    }
    Ok(())
}
