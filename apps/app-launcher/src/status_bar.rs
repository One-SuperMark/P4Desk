//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/status_bar.rs (MIT). The background-app icon restore
//! path is retained; width is dynamic and all connection/time indicators use
//! actual device state instead of the upstream sample battery/Wi-Fi values.

use crate::app_icons::get_app_icon_asset;
use crate::launcher_state::LauncherState;
use crate::widgets::AppIconPainter;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

pub const STATUS_BAR_HEIGHT: f32 = 56.0;

pub fn build_status_bar(state: Arc<Mutex<LauncherState>>, width: f32) -> impl Widget {
    let (clock, valid, usb, sd, running) = {
        let s = state.lock().unwrap();
        let mut running: Vec<String> = s.running_apps.keys().cloned().collect();
        running.sort();
        (
            s.clock.clone(),
            s.time_valid,
            s.usb_connected,
            s.sd_ready,
            running,
        )
    };
    let mut bar = Stack::new()
        .push(
            Container::new()
                .width(width)
                .height(STATUS_BAR_HEIGHT)
                .color(Color::from_rgba(5, 12, 20, 235)),
        )
        .push(
            Positioned::new(
                Text::new(if valid { clock } else { "待校时".into() })
                    .font_size(22.0)
                    .color(Color::from_rgb(240, 243, 246)),
            )
            .left(24.0)
            .top(13.0),
        );
    for (index, id) in running.into_iter().take(6).enumerate() {
        let Some(asset) = get_app_icon_asset(&id) else {
            continue;
        };
        let restore = state.clone();
        let chip = GestureDetector::new(
            Container::new()
                .width(36.0)
                .height(36.0)
                .border_radius(8.0)
                .border(Color::from_rgba(255, 255, 255, 55), 1.0)
                .color(Color::from_rgba(255, 255, 255, 20))
                .child(Center::new(
                    CustomPaint::new(AppIconPainter {
                        asset,
                        target_size: 28.0,
                        pressed: None,
                    })
                    .size(Size::new(28.0, 28.0)),
                )),
        )
        .on_tap(move || {
            if let Ok(mut s) = restore.lock() {
                s.open_app(&id);
            }
        });
        bar = bar.push(
            Positioned::new(chip)
                .left(160.0 + index as f32 * 44.0)
                .top(10.0),
        );
    }
    bar.push(
        Positioned::new(
            Text::new(if usb {
                "USB 已连接"
            } else {
                "USB 未连接"
            })
            .font_size(18.0)
            .color(if usb {
                Color::from_hex(0x8edbc5)
            } else {
                Color::from_hex(0x94a4b6)
            }),
        )
        .left(width - 282.0)
        .top(16.0),
    )
    .push(
        Positioned::new(
            Text::new(if sd { "TF 就绪" } else { "TF 未挂载" })
                .font_size(18.0)
                .color(if sd {
                    Color::from_hex(0x8edbc5)
                } else {
                    Color::from_hex(0x94a4b6)
                }),
        )
        .left(width - 130.0)
        .top(16.0),
    )
}
