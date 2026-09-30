//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/status_bar.rs (MIT). The background-app icon restore
//! path is retained; width is dynamic and all connection/time indicators use
//! actual device state. Battery percentage is an explicit GPIO20 voltage estimate;
//! native Type-C host connection may show the user's assumed charging indicator.

use crate::battery::{BatteryState, ChargeState};
use crate::launcher_state::LauncherState;
use crate::radio::{self, RadioSnapshot, SettingsSection, WifiIndicator};
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

pub const STATUS_BAR_HEIGHT: f32 = 56.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusPanelKind {
    Device,
    Wifi,
    Control,
}

fn wifi_color(wifi: WifiIndicator) -> Color {
    if wifi == WifiIndicator::Connected {
        Folio::green()
    } else {
        Folio::muted()
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
            Folio::muted(),
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
        Folio::green()
    } else if b.percent.is_some_and(|p| p <= 10) {
        Color::from_hex(0xff7979)
    } else if b.percent.is_some_and(|p| p <= 20) {
        Folio::orange()
    } else if b.percent.is_none() {
        Folio::muted()
    } else {
        Folio::ink()
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
                .antialias(true)
                .size(width, 48.0)
                .color(Color::TRANSPARENT)
                .pressed_color(Color::WHITE.with_opacity(0.10))
                .border_radius(Folio::CARD_RADIUS)
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
    if state.lock().unwrap().status_panel_kind == StatusPanelKind::Control {
        return with_control_center(state, root, size);
    }
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
                .color(Folio::raised())
                .border(Folio::line(), 1.0),
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
        .push(at(
            text("估算剩余电量".into(), 16.0, Folio::muted()),
            100.0,
            59.0,
        ))
        .push(at(
            text("电池电压".into(), 22.0, Folio::muted()),
            24.0,
            102.0,
        ))
        .push(at(
            text(b.voltage_label(), 22.0, Folio::ink()),
            210.0,
            102.0,
        ))
        .push(at(
            text("充电状态".into(), 22.0, Folio::muted()),
            24.0,
            142.0,
        ))
        .push(at(
            text(b.charge_label().into(), 22.0, Folio::ink()),
            210.0,
            142.0,
        ))
        .push(at(
            text(
                match b.charge {
                    ChargeState::Unknown => "未检测到 Type-C 电脑连接",
                    ChargeState::PluggedInAssumed => "按 Type-C 连接显示，不检测充满",
                    _ => "电量根据电压估算",
                }
                .into(),
                16.0,
                Folio::muted(),
            ),
            24.0,
            186.0,
        ))
        .push(at(
            Box::new(
                Container::new()
                    .width(w - 48.0)
                    .height(1.0)
                    .color(Folio::line()),
            ),
            24.0,
            224.0,
        ))
        .push(at(
            Box::new(glyph(
                &UI_USB,
                if connected {
                    Folio::green()
                } else {
                    Folio::muted()
                },
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
                Folio::ink(),
            ),
            66.0,
            242.0,
        ))
        .push(at(
            Box::new(glyph(
                &UI_SD_CARD,
                if sd { Folio::green() } else { Folio::muted() },
                !sd,
                26.0,
            )),
            24.0,
            295.0,
        ))
        .push(at(
            text(
                if sd { "TF 就绪" } else { "TF 未挂载" }.into(),
                22.0,
                Folio::ink(),
            ),
            66.0,
            292.0,
        ))
        .push(at(
            Box::new(
                Container::new()
                    .width(w - 48.0)
                    .height(1.0)
                    .color(Folio::line()),
            ),
            24.0,
            336.0,
        ))
        .push(at(
            text("本次启动".into(), 18.0, Folio::muted()),
            24.0,
            354.0,
        ))
        .push(at(
            text(
                crate::boot_diagnostics::reset_reason_label(reset_reason).into(),
                18.0,
                Folio::ink(),
            ),
            152.0,
            354.0,
        ))
        .push(at(
            text("运行时间".into(), 18.0, Folio::muted()),
            24.0,
            396.0,
        ))
        .push(at(
            text(
                crate::boot_diagnostics::uptime_label(uptime),
                18.0,
                Folio::ink(),
            ),
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
                .color(Folio::raised())
                .border(Folio::line(), 1.0),
        )
        .push(at(Box::new(wifi_glyph(&radio, 34.0)), 24.0, 22.0))
        .push(at(text("Wi-Fi".into(), 26.0, Folio::ink()), 76.0, 22.0))
        .push(at(
            text(wifi.label().into(), 18.0, wifi_color(wifi)),
            width - 104.0,
            29.0,
        ))
        .push(at(text(network, 22.0, Folio::ink()), 24.0, 91.0))
        .push(at(
            text(
                format!("IP 地址：{}", if ip.is_empty() { "—" } else { &ip }),
                18.0,
                Folio::muted(),
            ),
            24.0,
            131.0,
        ))
        .push(at(
            text(radio.wifi_status().into(), 16.0, Folio::muted()),
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
                Folio::muted(),
            ),
            24.0,
            201.0,
        ))
        .push(at(
            Box::new(
                ElevatedButton::new(Text::new("Wi-Fi 设置").font_size(18.0).color(Folio::ink()))
                    .style(
                        ButtonStyle::new()
                            .antialias(true)
                            .size(width - 48.0, 48.0)
                            .color(Folio::line())
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

/// iPhone-style passive battery: remaining charge fills a solid body from the left.
/// The numeric percentage has no percent suffix; no outline or charging bolt.
struct BatteryPercentPainter(BatteryState);
impl CustomPainter for BatteryPercentPainter {
    fn paint(&self, canvas: &mut Canvas, _size: Size) {
        use tiny_gfx::{GradientStop, LinearGradient, Paint, SpreadMode, Transform};
        let body = Rect::from_ltwh(0.0, 7.0, 25.0, 14.0);
        let charged = Folio::pick(Color::WHITE, Color::from_hex(0x263746));
        let remaining = Folio::pick(Color::from_hex(0x98999e), Color::from_hex(0x9eacb6));
        let percent = self.0.percent.unwrap_or(0).min(100);
        let mut fill = Paint::new(if percent == 100 { charged } else { remaining }.to_gfx());
        if percent > 0 && percent < 100 {
            let stop = percent as f32 / 100.0;
            // One coverage pass for the whole silhouette. A subpixel transition
            // avoids integer clipping seams through the rounded top/bottom edge.
            fill.shader = LinearGradient::new(
                tiny_gfx::Point::new(body.x, 0.0),
                tiny_gfx::Point::new(body.right(), 0.0),
                vec![
                    GradientStop::new(0.0, charged.to_gfx()),
                    GradientStop::new((stop - 0.35 / body.width).max(0.0), charged.to_gfx()),
                    GradientStop::new((stop + 0.35 / body.width).min(1.0), remaining.to_gfx()),
                    GradientStop::new(1.0, remaining.to_gfx()),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            )
            .unwrap();
        }
        // The SVG coverage path uses eight vertical samples plus analytic X
        // coverage, including the small terminal. No enlarged framebuffer.
        canvas.paint_rrect(RRect::from_rect_circular(body, 4.0), &fill, None);
        canvas.paint_rrect(
            RRect::from_rect_circular(Rect::from_ltwh(26.0, 11.5, 2.0, 5.0), 1.0),
            &Paint::new(remaining.to_gfx()),
            None,
        );
        let label = self
            .0
            .percent
            .map(|p| p.min(100).to_string())
            .unwrap_or_else(|| "--".into());
        // Two digits stay as large as possible; 100 has its own fitting size.
        let cell_width = if label.len() == 3 { 7.0 } else { 8.0 };
        let cell_height = cell_width * 26.0 / 18.0;
        let left = 12.5 - label.len() as f32 * cell_width * 0.5;
        let top = 14.0 - cell_height * 0.5;
        for (i, ch) in label.bytes().enumerate() {
            let index = if ch.is_ascii_digit() {
                (ch - b'0') as usize
            } else {
                10
            };
            crate::battery_digits_generated::DIGITS[index].paint(
                canvas,
                Rect::from_ltwh(left + i as f32 * cell_width, top, cell_width, cell_height),
                Folio::pick(Color::from_hex(0x08090b), Color::WHITE),
            );
        }
    }
}

/// The top plate is display-only. The separate controls button remains interactive.
pub fn build_status_rail(state: Arc<Mutex<LauncherState>>) -> impl Widget {
    let (r, battery, usb, connected, sd) = {
        let s = state.lock().unwrap();
        (
            s.radio,
            s.battery.clone(),
            s.usb_connected,
            s.connected,
            s.sd_ready,
        )
    };
    let at = |child: Box<dyn Widget>, x, y| Positioned::new(child).left(x).top(y);
    // Split the padded plate into equal cells, then center each actual glyph.
    // Keep the established icon sizes, including the 28x14 battery silhouette.
    let inset = 12.0;
    let cell_width = (92.0 - inset * 2.0) / 2.0;
    let cell_height = (152.0 - inset * 2.0) / 3.0;
    let cell = |child: Box<dyn Widget>, column: usize, row: usize| {
        at(
            Box::new(
                Container::new()
                    .width(cell_width)
                    .height(cell_height)
                    .child(Center::new(child)),
            ),
            inset + column as f32 * cell_width,
            inset + row as f32 * cell_height,
        )
    };
    Stack::new()
        .push(crate::folio_desktop::glass(92.0, 152.0))
        .push(cell(Box::new(wifi_glyph(&r, 28.0)), 1, 0))
        .push(cell(
            Box::new(glyph(
                &UI_USB,
                if connected {
                    Folio::green()
                } else if usb {
                    Folio::orange()
                } else {
                    Folio::muted()
                },
                !usb,
                28.0,
            )),
            0,
            1,
        ))
        .push(cell(
            Box::new(glyph(
                &UI_SD_CARD,
                if sd { Folio::green() } else { Folio::muted() },
                !sd,
                27.0,
            )),
            1,
            1,
        ))
        .push(cell(
            Box::new(glyph(
                &UI_BLUETOOTH,
                if r.bt_ready != 0 {
                    Folio::blue()
                } else {
                    Folio::muted()
                },
                false,
                26.0,
            )),
            0,
            2,
        ))
        .push(cell(
            Box::new(CustomPaint::new(BatteryPercentPainter(battery)).size(Size::new(28.0, 28.0))),
            0,
            0,
        ))
        .push(at(
            Box::new(
                // The pressed overlay shares the glass pill's exact bounds
                // and radius; only its brightness changes during a press.
                crate::folio_desktop::glass(92.0, 48.0).child(Center::new(status_button(
                    state,
                    StatusPanelKind::Control,
                    92.0,
                    glyph(&UI_CONTROLS, Folio::ink(), false, 28.0),
                ))),
            ),
            0.0,
            168.0,
        ))
}

fn with_control_center(
    state: Arc<Mutex<LauncherState>>,
    root: Box<dyn Widget>,
    size: Size,
) -> Box<dyn Widget> {
    use crate::launcher_state::UiCommand;
    use crate::radio::RadioCommand;
    let (r, b, usb, sd, brightness) = {
        let s = state.lock().unwrap();
        (
            s.radio,
            s.battery.clone(),
            s.connected,
            s.sd_ready,
            s.settings.brightness,
        )
    };
    let w = 572.0;
    let backdrop = state.lock().unwrap().control_center_backdrop.clone();
    let root = crate::control_center_glass::FrozenDesktop {
        child: root,
        cache: backdrop.clone(),
    };
    let at = |child: Box<dyn Widget>, x, y| Positioned::new(child).left(x).top(y);
    let text =
        |label: String, px, c| Box::new(Text::new(label).font_size(px).color(c)) as Box<dyn Widget>;
    let mut panel = Stack::new()
        .push(
            CustomPaint::new(crate::control_center_glass::Panel(backdrop))
                .size(Size::new(w, 512.0)),
        )
        .push(at(text("控制中心".into(), 28.0, Folio::ink()), 24.0, 23.0));
    let close = state.clone();
    panel = panel.push(at(
        Box::new(
            ElevatedButton::new(glyph(&UI_CLOSE, Folio::ink(), false, 24.0))
                .style(
                    ButtonStyle::new()
                        .antialias(true)
                        .size(44.0, 44.0)
                        .color(Folio::raised().with_opacity(if Folio::is_light() {
                            0.58
                        } else {
                            0.48
                        }))
                        .pressed_color(Folio::line().with_opacity(0.78))
                        .border_radius(22.0)
                        .padding(EdgeInsets::all(10.0)),
                )
                .on_pressed(move || {
                    let mut s = close.lock().unwrap();
                    s.status_panel_open = false;
                    s.changed();
                }),
        ),
        w - 64.0,
        16.0,
    ));
    for (i, (icon, title, detail, on)) in [
        (
            &UI_WIFI_3,
            "Wi-Fi",
            r.wifi_indicator().label(),
            r.wifi_on != 0,
        ),
        (
            &UI_BLUETOOTH,
            "蓝牙",
            if r.bt_on == 0 {
                "已关闭"
            } else if r.bt_ready == 0 {
                "启动中"
            } else {
                "已开启"
            },
            r.bt_on != 0,
        ),
        (
            &UI_USB,
            "USB 副屏",
            if usb { "Mac 已连接" } else { "未连接" },
            usb,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let s = state.clone();
        let x = 24.0 + i as f32 * 178.0;
        let content = Stack::new()
            .push(at(
                Box::new(glyph(
                    icon,
                    if on {
                        Folio::accent_ink()
                    } else {
                        Folio::muted()
                    },
                    false,
                    30.0,
                )),
                16.0,
                14.0,
            ))
            .push(at(text(title.into(), 22.0, Folio::ink()), 16.0, 52.0))
            .push(at(text(detail.into(), 16.0, Folio::muted()), 16.0, 82.0));
        panel = panel.push(at(
            Box::new(
                ElevatedButton::new(content)
                    .style(
                        ButtonStyle::new()
                            .antialias(true)
                            .size(168.0, 114.0)
                            .color(Folio::raised().with_opacity(if Folio::is_light() {
                                0.58
                            } else {
                                0.48
                            }))
                            .pressed_color(Folio::line().with_opacity(0.78))
                            .border_radius(24.0)
                            .padding(EdgeInsets::ZERO),
                    )
                    .on_pressed(move || {
                        let mut s = s.lock().unwrap();
                        match i {
                            0 => {
                                let on = s.radio.wifi_on == 0;
                                s.queue(UiCommand::Radio(RadioCommand::WifiEnable(on)));
                            }
                            1 => {
                                let on = s.radio.bt_on == 0;
                                s.queue(UiCommand::Radio(RadioCommand::BleEnable(on)));
                            }
                            _ => {
                                let p = crate::folio_desktop::icon_center(size, 6);
                                let side = crate::folio_desktop::ICON_SIZE * 0.94;
                                s.launch_app(
                                    "display",
                                    Rect::from_ltwh(p.x - side * 0.5, p.y - side * 0.5, side, side),
                                );
                            }
                        };
                        s.changed();
                    }),
            ),
            x,
            78.0,
        ));
    }
    panel = panel
        .push(at(
            Box::new(
                Container::new()
                    .width(w - 48.0)
                    .height(116.0)
                    .color(Folio::raised().with_opacity(if Folio::is_light() {
                        0.58
                    } else {
                        0.48
                    }))
                    .border_radius(24.0),
            ),
            24.0,
            204.0,
        ))
        .push(at(text("屏幕亮度".into(), 22.0, Folio::ink()), 44.0, 222.0))
        .push(at(
            text(format!("{brightness}%"), 22.0, Folio::accent_ink()),
            w - 104.0,
            222.0,
        ));
    for (i, value) in [25u8, 50, 75, 100].into_iter().enumerate() {
        let s = state.clone();
        panel = panel.push(at(
            Box::new(
                ElevatedButton::new(
                    Text::new(format!("{value}%"))
                        .font_size(18.0)
                        .color(Folio::ink()),
                )
                .style(
                    ButtonStyle::new()
                        .antialias(true)
                        .size(114.0, 44.0)
                        .color(if brightness == value {
                            Folio::accent()
                        } else {
                            Folio::line()
                        })
                        .pressed_color(Folio::accent())
                        .border_radius(14.0),
                )
                .on_pressed(move || {
                    let mut s = s.lock().unwrap();
                    s.queue(UiCommand::Brightness(value));
                    s.changed();
                }),
            ),
            44.0 + i as f32 * 122.0,
            262.0,
        ));
    }
    let details = state.clone();
    panel = panel.push(at(
        Box::new(
            ElevatedButton::new(
                Row::new()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .push(glyph(battery_icon(&b), battery_color(&b), false, 36.0))
                    .push(
                        Text::new(format!(
                            "  {}  ·  {}   /   {}",
                            b.percent_label(),
                            b.charge_label(),
                            if sd { "TF 就绪" } else { "TF 未挂载" }
                        ))
                        .font_size(18.0)
                        .color(Folio::ink()),
                    ),
            )
            .style(
                ButtonStyle::new()
                    .antialias(true)
                    .size(w - 48.0, 64.0)
                    .color(Folio::raised().with_opacity(if Folio::is_light() {
                        0.58
                    } else {
                        0.48
                    }))
                    .pressed_color(Folio::line().with_opacity(0.78))
                    .border_radius(24.0),
            )
            .on_pressed(move || {
                let mut s = details.lock().unwrap();
                s.status_panel_kind = StatusPanelKind::Device;
                s.changed();
            }),
        ),
        24.0,
        336.0,
    ));
    for (i, (label, id)) in [
        ("设置", "settings"),
        ("Mac 控制", "mac"),
        ("关闭屏幕", "screen"),
    ]
    .into_iter()
    .enumerate()
    {
        let s = state.clone();
        panel = panel.push(at(
            Box::new(
                ElevatedButton::new(Text::new(label).font_size(22.0).color(Folio::ink()))
                    .style(
                        ButtonStyle::new()
                            .antialias(true)
                            .size(168.0, 64.0)
                            .color(Folio::raised().with_opacity(if Folio::is_light() {
                                0.58
                            } else {
                                0.48
                            }))
                            .pressed_color(Folio::line().with_opacity(0.78))
                            .border_radius(24.0),
                    )
                    .on_pressed(move || {
                        let mut s = s.lock().unwrap();
                        if id == "screen" {
                            s.queue(UiCommand::Screen(false));
                            s.status_panel_open = false;
                        } else {
                            s.open_app(id);
                        }
                        s.changed();
                    }),
            ),
            24.0 + i as f32 * 178.0,
            424.0,
        ));
    }
    let dismiss = state.clone();
    Box::new(
        Stack::new()
            .push(root)
            .push(
                GestureDetector::new(
                    Container::new()
                        .width(size.width)
                        .height(size.height)
                        .color(Color::BLACK.with_opacity(0.18)),
                )
                .on_tap(move || {
                    let mut s = dismiss.lock().unwrap();
                    s.status_panel_open = false;
                    s.changed();
                }),
            )
            .push(
                Positioned::new(
                    GestureDetector::new(Container::new().width(w).height(512.0).child(panel))
                        .on_tap(|| {}),
                )
                .left(size.width - 132.0 - w)
                .top(24.0),
            ),
    )
}
