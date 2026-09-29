//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/status_bar.rs (MIT). The background-app icon restore
//! path is retained; width is dynamic and all connection/time indicators use
//! actual device state. Battery percentage is an explicit GPIO20 voltage estimate;
//! native Type-C host connection may show the user's assumed charging indicator.

use crate::app_icons::get_app_icon_asset;
use crate::battery::{BatteryState, ChargeState};
use crate::launcher_state::LauncherState;
use crate::widgets::AppIconPainter;
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;

pub const STATUS_BAR_HEIGHT: f32 = 56.0;

pub fn build_status_bar(state: Arc<Mutex<LauncherState>>, width: f32) -> impl Widget {
    let (clock, valid, usb, connected, sd, battery, running) = {
        let s = state.lock().unwrap();
        let mut running: Vec<String> = s.running_apps.keys().cloned().collect();
        running.sort();
        (
            s.clock.clone(),
            s.time_valid,
            s.usb_connected,
            s.connected,
            s.sd_ready,
            s.battery.clone(),
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
    let usb_color = if connected {
        MINT
    } else if usb {
        AMBER
    } else {
        MUTED
    };
    bar.push(
        Positioned::new(status_button(
            state.clone(),
            44.0,
            glyph(&UI_USB, usb_color, !usb, 30.0),
        ))
        .left(width - 240.0)
        .top(6.0),
    )
    .push(
        Positioned::new(status_button(
            state.clone(),
            44.0,
            glyph(&UI_SD_CARD, if sd { MINT } else { MUTED }, !sd, 30.0),
        ))
        .left(width - 192.0)
        .top(6.0),
    )
    .push(
        Positioned::new(status_button(
            state,
            124.0,
            Row::new()
                .main_axis_alignment(MainAxisAlignment::Center)
                .push(glyph(
                    battery_icon(&battery),
                    battery_color(&battery),
                    false,
                    40.0,
                ))
                .push(SizedBox::from_size(Size::new(6.0, 1.0)))
                .push(
                    Text::new(battery.percent_label())
                        .font_size(18.0)
                        .color(battery_color(&battery)),
                ),
        ))
        .left(width - 148.0)
        .top(6.0),
    )
}

const INK: Color = Color::from_hex(0xe8edf2);
const MUTED: Color = Color::from_hex(0x94a4b6);
const MINT: Color = Color::from_hex(0x8edbc5);
const AMBER: Color = Color::from_hex(0xffbe82);

pub fn battery_icon(b: &BatteryState) -> &'static VectorIcon {
    if matches!(
        b.charge,
        ChargeState::Charging | ChargeState::PluggedInAssumed
    ) {
        return &UI_BATTERY_BOLT;
    }
    match b.percent {
        Some(88..=100) => &UI_BATTERY_100,
        Some(63..=87) => &UI_BATTERY_75,
        Some(38..=62) => &UI_BATTERY_50,
        Some(13..=37) => &UI_BATTERY_25,
        _ => &UI_BATTERY_0,
    }
}
fn battery_color(b: &BatteryState) -> Color {
    if matches!(
        b.charge,
        ChargeState::Charging | ChargeState::PluggedInAssumed | ChargeState::Full
    ) {
        MINT
    } else if b.percent.is_some_and(|p| p <= 10) {
        Color::from_hex(0xff7979)
    } else if b.percent.is_some_and(|p| p <= 20) {
        AMBER
    } else if b.percent.is_none() {
        MUTED
    } else {
        INK
    }
}
struct StatusGlyph {
    icon: &'static VectorIcon,
    color: Color,
    unavailable: bool,
}
impl CustomPainter for StatusGlyph {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let r = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        self.icon.paint(canvas, r, self.color);
        if self.unavailable {
            UI_STATUS_SLASH.paint(canvas, r, self.color);
        }
    }
}
fn glyph(icon: &'static VectorIcon, color: Color, unavailable: bool, side: f32) -> CustomPaint {
    CustomPaint::new(StatusGlyph {
        icon,
        color,
        unavailable,
    })
    .size(Size::new(side, side))
}
fn status_button(
    state: Arc<Mutex<LauncherState>>,
    width: f32,
    child: impl Widget + 'static,
) -> ElevatedButton {
    ElevatedButton::new(child)
        .style(
            ButtonStyle::new()
                .size(width, 44.0)
                .color(Color::TRANSPARENT)
                .pressed_color(Color::from_hex(0x344252))
                .border_radius(10.0)
                .padding(EdgeInsets::all(2.0)),
        )
        .on_pressed(move || {
            let mut s = state.lock().unwrap();
            s.status_panel_open = !s.status_panel_open;
            s.changed();
        })
}

/// Modal status card: outside taps dismiss without opening the desktop underneath.
pub fn with_status_panel(
    state: Arc<Mutex<LauncherState>>,
    root: Box<dyn Widget>,
    size: Size,
) -> Box<dyn Widget> {
    let (b, usb, connected, sd, reset_reason, uptime) = {
        let s = state.lock().unwrap();
        (
            s.battery.clone(),
            s.usb_connected,
            s.connected,
            s.sd_ready,
            s.reset_reason,
            s.monotonic_ms,
        )
    };
    let w = 376.0f32.min(size.width - 32.0);
    let panel_height = 448.0;
    let at = |child: Box<dyn Widget>, x, y| Positioned::new(child).left(x).top(y);
    let text =
        |s: String, px, color| Box::new(Text::new(s).font_size(px).color(color)) as Box<dyn Widget>;
    let card = Stack::new()
        .push(
            Container::new()
                .width(w)
                .height(panel_height)
                .border_radius(20.0)
                .color(Color::from_hex(0x202a37))
                .border(Color::from_hex(0x344252), 1.0),
        )
        .push(at(
            Box::new(glyph(battery_icon(&b), battery_color(&b), false, 64.0)),
            20.0,
            14.0,
        ))
        .push(at(
            text(b.percent_label(), 36.0, battery_color(&b)),
            100.0,
            16.0,
        ))
        .push(at(text("估算剩余电量".into(), 16.0, MUTED), 100.0, 59.0))
        .push(at(text("电池电压".into(), 22.0, MUTED), 24.0, 102.0))
        .push(at(text(b.voltage_label(), 22.0, INK), 210.0, 102.0))
        .push(at(text("充电状态".into(), 22.0, MUTED), 24.0, 142.0))
        .push(at(text(b.charge_label().into(), 22.0, INK), 210.0, 142.0))
        .push(at(
            text(
                match b.charge {
                    ChargeState::Unknown => "未检测到 Type-C 电脑连接",
                    ChargeState::PluggedInAssumed => "按 Type-C 连接显示，不检测充满",
                    _ => "电量根据电压估算",
                }
                .into(),
                16.0,
                MUTED,
            ),
            24.0,
            186.0,
        ))
        .push(at(
            Box::new(
                Container::new()
                    .width(w - 48.0)
                    .height(1.0)
                    .color(Color::from_hex(0x344252)),
            ),
            24.0,
            224.0,
        ))
        .push(at(
            Box::new(glyph(
                &UI_USB,
                if connected { MINT } else { MUTED },
                !usb,
                26.0,
            )),
            24.0,
            245.0,
        ))
        .push(at(
            text(
                if connected {
                    "Mac 已连接"
                } else if usb {
                    "等待 Mac 应用"
                } else {
                    "USB 未连接"
                }
                .into(),
                22.0,
                INK,
            ),
            66.0,
            242.0,
        ))
        .push(at(
            Box::new(glyph(&UI_SD_CARD, if sd { MINT } else { MUTED }, !sd, 26.0)),
            24.0,
            295.0,
        ))
        .push(at(
            text(if sd { "TF 就绪" } else { "TF 未挂载" }.into(), 22.0, INK),
            66.0,
            292.0,
        ))
        .push(at(
            Box::new(
                Container::new()
                    .width(w - 48.0)
                    .height(1.0)
                    .color(Color::from_hex(0x344252)),
            ),
            24.0,
            336.0,
        ))
        .push(at(text("本次启动".into(), 18.0, MUTED), 24.0, 354.0))
        .push(at(
            text(
                crate::boot_diagnostics::reset_reason_label(reset_reason).into(),
                18.0,
                INK,
            ),
            152.0,
            354.0,
        ))
        .push(at(text("运行时间".into(), 18.0, MUTED), 24.0, 396.0))
        .push(at(
            text(crate::boot_diagnostics::uptime_label(uptime), 18.0, INK),
            152.0,
            396.0,
        ));
    let dismiss = state.clone();
    Box::new(
        Stack::new()
            .push(root)
            .push(
                GestureDetector::new(
                    Container::new()
                        .width(size.width)
                        .height(size.height)
                        .color(Color::from_rgba(0, 0, 0, 40)),
                )
                .on_tap(move || {
                    let mut s = dismiss.lock().unwrap();
                    s.status_panel_open = false;
                    s.changed();
                }),
            )
            .push(
                Positioned::new(
                    GestureDetector::new(
                        Container::new().width(w).height(panel_height).child(card),
                    )
                    .on_tap(|| {}),
                )
                .left(size.width - w - 24.0)
                .top(STATUS_BAR_HEIGHT + 8.0),
            ),
    )
}
