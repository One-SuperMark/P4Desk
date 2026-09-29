//! Synthetic frames from an actual desktop press and the firmware's paint path.
//! cargo run -p app-launcher --features screenshots --example app-launch-preview -- artifacts/app-launch
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, LauncherState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let out = PathBuf::from(args.next().unwrap_or_else(|| "artifacts/app-launch".into()));
    let step_ms = args
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(33)
        .clamp(1, 100);
    let id = args.next().unwrap_or_else(|| "timer".into());
    let down_frame = 363u64.div_ceil(step_ms);
    let up_frame = 396u64.div_ceil(step_ms);
    std::fs::create_dir_all(&out)?;
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    let point = match id.as_str() {
        "clock" => Point::new(151.0, 177.0),
        "timer" => Point::new(392.0, 177.0),
        "notes" => Point::new(632.0, 177.0),
        "calculator" => Point::new(873.0, 177.0),
        "mac" => Point::new(151.0, 403.0),
        "settings" => Point::new(392.0, 403.0),
        "display" => Point::new(632.0, 403.0),
        _ => return Err("unknown app".into()),
    };
    for frame in 0..=1_914 / step_ms {
        let (changed, dirty) = {
            let mut s = state.lock().unwrap();
            let changed = s.tick(frame * step_ms, 0);
            (changed, s.take_launch_animation_dirty(size))
        };
        if changed {
            app.request_rebuild();
        }
        if let Some(rect) = dirty {
            app.mark_dirty(rect);
        }
        if frame == down_frame {
            backend.event(TouchEvent::Down(point));
        }
        if frame == up_frame {
            backend.event(TouchEvent::Up(point));
        }
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{frame:03}.png")))?;
    }
    Ok(())
}
