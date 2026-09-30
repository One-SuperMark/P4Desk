//! Reproducible desktop preview through the real Pad UI, using synthetic state.
//! cargo run -p app-launcher --features screenshots --example pad-icons-preview -- artifacts/pad-icons.png

use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, LauncherState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/pad-icons.png".into()),
    );
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        s.settings.timezone_minutes = 0;
        s.tick(0, 1_790_712_418_000);
        s.connected = true;
        s.usb_connected = true;
        s.sd_ready = true;
        // Include small background-app icons as well as the eight desktop icons.
        s.open_app("clock");
        s.open_app("timer");
        s.background_active_app();
    }
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    if std::env::args().nth(2).as_deref() == Some("pressed") {
        backend.event(TouchEvent::Down(Point::new(348.0, 278.0)));
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    #[cfg(feature = "screenshots")]
    backend.screenshot(&out)?;
    Ok(())
}
