//! Synthetic USB first-frame handoff using the actual Rust compositor; no capture data.
use app_launcher::app_launch::paint_usb_display_reveal;
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, LauncherState};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tiny_flutter::{App, Canvas, Point, Size, TouchEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/usb-launch-direct".into()),
    );
    std::fs::create_dir_all(&out)?;
    let size = Size::new(1024.0, 600.0);
    let state = Arc::new(Mutex::new(LauncherState::new()));
    state.lock().unwrap().connected = true;
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    // A static synthetic desktop makes it evident that each reveal frame must
    // start from an untouched decode, not from the previous tinted framebuffer.
    let image: Vec<u16> = (0..600)
        .flat_map(|y| {
            (0..1024).map(move |x| {
                let rgb = if y < 26 {
                    0xe7e9ee
                } else if x > 150 && x < 874 && y > 115 && y < 480 {
                    if y < 156 {
                        0x263743
                    } else if y % 48 < 16 && x > 195 && x < 710 {
                        0xa0b5bf
                    } else {
                        0xecf2f5
                    }
                } else {
                    0x1f5271 + ((x / 64) * 0x000400)
                };
                tiny_flutter::Color::from_hex(rgb).to_rgb565()
            })
        })
        .collect();
    for frame in 0..180 {
        let now = frame * 16;
        if now < 1600 {
            let (changed, dirty) = {
                let mut s = state.lock().unwrap();
                let changed = s.tick(now, 0);
                (changed, s.take_launch_animation_dirty(size))
            };
            if changed {
                app.request_rebuild();
            }
            if let Some(rect) = dirty {
                app.mark_dirty(rect);
            }
            if frame == 23 {
                backend.event(TouchEvent::Down(Point::new(564.0, 466.0)));
            }
            if frame == 25 {
                backend.event(TouchEvent::Up(Point::new(564.0, 466.0)));
            }
            app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        } else {
            backend.pixels.copy_from_slice(&image);
            let mut canvas = Canvas::new(tiny_flutter::tiny_gfx::Pixmap565Mut::new(
                &mut backend.pixels,
                1024,
                600,
            ));
            paint_usb_display_reveal(
                &mut canvas,
                size,
                (now - 1600) as u32,
                600,
                false,
                app_launcher::icon_theme::IconTheme::Colloid,
            );
        }
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{frame:03}.png")))?;
    }
    Ok(())
}
