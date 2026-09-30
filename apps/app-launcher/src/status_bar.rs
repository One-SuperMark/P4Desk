//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/status_bar.rs (MIT). The background-app icon restore
//! path is retained; width is dynamic and all connection/time indicators use
//! actual device state. Battery percentage is an explicit GPIO20 voltage estimate;
//! native Type-C host connection may show the user's assumed charging indicator.

use crate::app_icons::get_app_icon_asset;
use crate::battery::{BatteryState, ChargeState};
use crate::launcher_state::LauncherState;
use crate::radio::{self, RadioSnapshot, SettingsSection, WifiIndicator};
use crate::widgets::AppIconPainter;
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;

pub const STATUS_BAR_HEIGHT: f32 = 56.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusPanelKind {
    Device,
    Wifi,
}

pub fn build_status_bar(state: Arc<Mutex<LauncherState>>, width: f32) -> impl Widget {
    let (clock, valid, usb, connected, sd, battery, wifi, running) = {
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
            s.radio,
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
            StatusPanelKind::Wifi,
            44.0,
            wifi_glyph(&wifi, 30.0),
        ))
        .left(width - 288.0)
        .top(6.0),
    )
    .push(
        Positioned::new(status_button(
            state.clone(),
            StatusPanelKind::Device,
            44.0,
            glyph(&UI_USB, usb_color, !usb, 30.0),
        ))
        .left(width - 240.0)
        .top(6.0),
    )
    .push(
        Positioned::new(status_button(
            state.clone(),
            StatusPanelKind::Device,
            44.0,
            glyph(&UI_SD_CARD, if sd { MINT } else { MUTED }, !sd, 30.0),
        ))
        .left(width - 192.0)
        .top(6.0),
    )
    .push(
        Positioned::new(status_button(
            state,
            StatusPanelKind::Device,
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

fn wifi_color(wifi: WifiIndicator) -> Color {
    if wifi == WifiIndicator::Connected {
        MINT
    } else {
        MUTED
    }
}
fn wifi_glyph(radio: &RadioSnapshot, side: f32) -> CustomPaint {
    let wifi = radio.wifi_indicator();
    let icon = if wifi == WifiIndicator::Connected {
        match radio.wifi_signal_level() {
            Some(3) => &UI_WIFI_3,
            Some(2) => &UI_WIFI_2,
            Some(1) => &UI_WIFI_1,
            _ => &UI_WIFI_0,
        }
    } else {
        &UI_WIFI_3
    };
    CustomPaint::new(StatusGlyph {
        icon,
        color: wifi_color(wifi),
        unavailable: wifi == WifiIndicator::Disabled,
        // SVG layers are the dot followed by the three arcs. Paint each layer
        // once so gray underpainting cannot leave fringes on green AA edges.
        underlay: (wifi == WifiIndicator::Connected).then_some((
            VectorIcon {
                layers: &UI_WIFI_3.layers[icon.layers.len()..],
                ..UI_WIFI_3
            },
            MUTED,
        )),
    })
    .size(Size::new(side, side))
}

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
    underlay: Option<(VectorIcon, Color)>,
}
impl CustomPainter for StatusGlyph {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let r = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        if let Some((icon, color)) = self.underlay {
            icon.paint(canvas, r, color);
        }
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
        underlay: None,
    })
    .size(Size::new(side, side))
}
fn status_button(
    state: Arc<Mutex<LauncherState>>,
    kind: StatusPanelKind,
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
            s.status_panel_open = !s.status_panel_open || s.status_panel_kind != kind;
            s.status_panel_kind = kind;
            s.changed();
        })
}

/// Modal status card: outside taps dismiss without opening the desktop underneath.
pub fn with_status_panel(
    state: Arc<Mutex<LauncherState>>,
    root: Box<dyn Widget>,
    size: Size,
) -> Box<dyn Widget> {
    if state.lock().unwrap().status_panel_kind == StatusPanelKind::Wifi {
        return with_wifi_panel(state, root, size);
    }
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

fn fit_network_name(name: &str, max_width: f32) -> String {
    let font = Font::default_font();
    if font.measure_text(name, 22.0).width <= max_width {
        return name.to_owned();
    }
    let mut shown = name.to_owned();
    while !shown.is_empty() {
        shown.pop();
        let candidate = format!("{shown}…");
        if font.measure_text(&candidate, 22.0).width <= max_width {
            return candidate;
        }
    }
    "…".into()
}

fn wifi_card(radio: RadioSnapshot, state: Arc<Mutex<LauncherState>>, width: f32) -> Stack {
    let wifi = radio.wifi_indicator();
    let connected = wifi == WifiIndicator::Connected;
    let connecting = wifi == WifiIndicator::Connecting;
    let name = if connected || connecting {
        radio::label(&radio.ssid)
    } else {
        String::new()
    };
    let network = if name.is_empty() {
        "未连接网络".into()
    } else {
        fit_network_name(&name, width - 48.0)
    };
    let ip = if connected {
        radio::label(&radio.ip)
    } else {
        String::new()
    };
    let at = |child: Box<dyn Widget>, x, y| Positioned::new(child).left(x).top(y);
    let text = |s: String, px, c| Box::new(Text::new(s).font_size(px).color(c)) as Box<dyn Widget>;
    Stack::new()
        .push(
            Container::new()
                .width(width)
                .height(308.0)
                .border_radius(20.0)
                .color(Color::from_hex(0x202a37))
                .border(Color::from_hex(0x344252), 1.0),
        )
        .push(at(Box::new(wifi_glyph(&radio, 34.0)), 24.0, 22.0))
        .push(at(text("Wi-Fi".into(), 26.0, INK), 76.0, 22.0))
        .push(at(
            text(wifi.label().into(), 18.0, wifi_color(wifi)),
            width - 104.0,
            29.0,
        ))
        .push(at(text(network, 22.0, INK), 24.0, 91.0))
        .push(at(
            text(
                format!("IP 地址：{}", if ip.is_empty() { "—" } else { &ip }),
                18.0,
                MUTED,
            ),
            24.0,
            131.0,
        ))
        .push(at(
            text(radio.wifi_status().into(), 16.0, MUTED),
            24.0,
            175.0,
        ))
        .push(at(
            text(
                if connected && radio.wifi_signal_level().is_some() {
                    format!("信号：{} dBm", radio.wifi_rssi_dbm)
                } else if connected {
                    "信号：读取中".into()
                } else {
                    String::new()
                },
                16.0,
                MUTED,
            ),
            24.0,
            201.0,
        ))
        .push(at(
            Box::new(
                ElevatedButton::new(Text::new("Wi-Fi 设置").font_size(18.0).color(INK))
                    .style(
                        ButtonStyle::new()
                            .size(width - 48.0, 48.0)
                            .color(Color::from_hex(0x344252))
                            .pressed_color(Color::from_hex(0x45586b))
                            .border_radius(10.0),
                    )
                    .on_pressed(move || {
                        let mut s = state.lock().unwrap();
                        s.settings_view.section = SettingsSection::Wifi;
                        s.settings_view.page = 0;
                        s.settings_view.selected_ble = None;
                        s.settings_view.confirm_forget = false;
                        s.manual_time_open = false;
                        s.open_app("settings");
                    }),
            ),
            24.0,
            232.0,
        ))
}

fn with_wifi_panel(
    state: Arc<Mutex<LauncherState>>,
    root: Box<dyn Widget>,
    size: Size,
) -> Box<dyn Widget> {
    let radio = state.lock().unwrap().radio;
    let width = 376.0f32.min(size.width - 32.0);
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
                        Container::new()
                            .width(width)
                            .height(308.0)
                            .child(wifi_card(radio, state, width)),
                    )
                    .on_tap(|| {}),
                )
                .left(size.width - width - 24.0)
                .top(STATUS_BAR_HEIGHT + 8.0),
            ),
    )
}
