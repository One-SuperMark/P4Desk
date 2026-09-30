//! macOS-inspired settings, sized for a 1024x600 touch display.
use crate::radio::{self, RadioCommand as Cmd, SettingsSection as Section, WifiJoin};
use crate::{LauncherState, UiCommand};
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;
use tiny_flutter::VectorIcon;

const BG: Color = Color::from_hex(0x2d2b2a);
const SIDE: Color = Color::from_hex(0x242323);
const CARD: Color = Color::from_hex(0x393736);
const LINE: Color = Color::from_hex(0x484644);
const TEXT: Color = Color::from_hex(0xf3f2f1);
const SUB: Color = Color::from_hex(0xb7b5b3);
const BLUE: Color = Color::from_hex(0x087dff);
const GREEN: Color = Color::from_hex(0x32ce68);
const AMBER: Color = Color::from_hex(0xffc078);
pub const SIDEBAR: f32 = 244.0;
fn at(w: impl Widget + 'static, x: f32, y: f32) -> Positioned {
    Positioned::new(w).left(x).top(y)
}
fn text(s: impl Into<String>, px: f32, c: Color) -> Text {
    Text::new(s).font_size(px).color(c)
}
fn panel(w: f32, h: f32, c: Color) -> Container {
    Container::new()
        .width(w)
        .height(h)
        .color(c)
        .border_radius(13.0)
}
fn edit(s: &Arc<Mutex<LauncherState>>, f: impl FnOnce(&mut LauncherState)) {
    let mut s = s.lock().unwrap();
    f(&mut s);
    s.changed();
}
fn command(s: &Arc<Mutex<LauncherState>>, c: Cmd) {
    edit(s, |s| {
        s.notice.clear();
        s.queue(UiCommand::Radio(c))
    });
}
fn short(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    } else {
        s.into()
    }
}
fn button(
    s: impl Into<String>,
    w: f32,
    h: f32,
    accent: bool,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(text(s, 18.0, TEXT))
        .style(
            ButtonStyle::new()
                .size(w, h)
                .color(if accent { BLUE } else { LINE })
                .pressed_color(Color::from_hex(0x62605f))
                .border_radius(9.0)
                .padding(EdgeInsets::all(6.0)),
        )
        .on_pressed(f)
}
struct Glyph(&'static VectorIcon, Color);
impl CustomPainter for Glyph {
    fn paint(&self, c: &mut Canvas, s: Size) {
        self.0
            .paint(c, Rect::from_ltwh(0.0, 0.0, s.width, s.height), self.1);
    }
}
fn icon(i: &'static VectorIcon, sz: f32, c: Color) -> CustomPaint {
    CustomPaint::new(Glyph(i, c)).size(Size::new(sz, sz))
}
fn icon_button(
    i: &'static VectorIcon,
    c: Color,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(icon(i, 24.0, TEXT))
        .style(
            ButtonStyle::new()
                .size(44.0, 44.0)
                .color(c)
                .pressed_color(LINE)
                .border_radius(10.0)
                .padding(EdgeInsets::all(10.0)),
        )
        .on_pressed(f)
}
struct Toggle(bool);
impl CustomPainter for Toggle {
    fn paint(&self, c: &mut Canvas, _: Size) {
        c.draw_rrect_aa(
            RRect::from_rect_circular(Rect::from_ltwh(4.0, 9.0, 54.0, 30.0), 15.0),
            if self.0 {
                BLUE
            } else {
                Color::from_hex(0x64615f)
            },
        );
        c.draw_circle(
            Point::new(if self.0 { 43.0 } else { 19.0 }, 24.0),
            12.0,
            TEXT,
        );
    }
}
fn toggle(on: bool, f: impl Fn() + Send + Sync + 'static) -> ElevatedButton {
    ElevatedButton::new(CustomPaint::new(Toggle(on)).size(Size::new(62.0, 48.0)))
        .style(
            ButtonStyle::new()
                .size(62.0, 48.0)
                .color(CARD)
                .pressed_color(LINE)
                .border_radius(12.0)
                .padding(EdgeInsets::all(0.0)),
        )
        .on_pressed(f)
}
fn section_icon(s: Section) -> (&'static VectorIcon, Color) {
    match s {
        Section::Wifi => (&UI_WIFI_3, BLUE),
        Section::Bluetooth => (&UI_BLUETOOTH, BLUE),
        Section::Display => (&DESKTOP_DISPLAY, Color::from_hex(0x7458d6)),
        Section::DateTime => (&UI_CALENDAR, Color::from_hex(0xf05454)),
        Section::Storage => (&UI_STORAGE, Color::from_hex(0x85837e)),
        Section::About => (&UI_INFO, Color::from_hex(0x747c8c)),
    }
}
pub fn build_settings(state: Arc<Mutex<LauncherState>>, size: Size) -> Stack {
    let selected = state.lock().unwrap().settings_view.section;
    let w = (size.width - SIDEBAR - 48.0).max(620.0);
    let mut p = Stack::new()
        .push(
            Container::new()
                .width(size.width)
                .height(size.height)
                .color(BG),
        )
        .push(
            Container::new()
                .width(SIDEBAR)
                .height(size.height)
                .color(SIDE),
        );
    let s = state.clone();
    p = p
        .push(at(
            icon_button(&UI_HOME, SIDE, move || {
                edit(&s, |s| s.background_active_app())
            }),
            14.0,
            14.0,
        ))
        .push(at(text("设置", 24.0, TEXT), 74.0, 24.0));
    let s = state.clone();
    p = p.push(at(
        icon_button(&UI_CLOSE, BG, move || edit(&s, |s| s.kill_active_app())),
        size.width - 60.0,
        14.0,
    ));
    for (i, section) in Section::ALL.into_iter().enumerate() {
        let (glyph, color) = section_icon(section);
        let row = Stack::new()
            .push(at(
                panel(30.0, 30.0, color).child(Center::new(icon(glyph, 23.0, TEXT))),
                12.0,
                10.0,
            ))
            .push(at(text(section.title(), 22.0, TEXT), 55.0, 13.0));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .size(SIDEBAR - 24.0, 50.0)
                        .color(if selected == section { BLUE } else { SIDE })
                        .pressed_color(LINE)
                        .border_radius(10.0)
                        .padding(EdgeInsets::all(0.0)),
                )
                .on_pressed(move || {
                    edit(&s, |s| {
                        s.settings_view.section = section;
                        s.settings_view.page = 0;
                        s.settings_view.selected_ble = None;
                        s.settings_view.join = None;
                        s.settings_view.confirm_forget = false;
                        s.manual_time_open = false;
                    })
                }),
            12.0,
            90.0 + i as f32 * 62.0,
        ));
    }
    p = p
        .push(at(text("P4Desk", 18.0, SUB), 24.0, size.height - 42.0))
        .push(at(text(selected.title(), 28.0, TEXT), SIDEBAR + 24.0, 24.0));
    let detail = match selected {
        Section::Wifi => wifi_page(state.clone(), w),
        Section::Bluetooth => ble_page(state.clone(), w),
        Section::Display => display_page(state.clone(), w),
        Section::DateTime => time_page(state.clone(), w),
        Section::Storage => storage_page(state.clone(), w),
        Section::About => about_page(w),
    };
    p = p.push(at(detail, SIDEBAR + 24.0, 76.0));
    if state.lock().unwrap().settings_view.join.is_some() {
        p = p.push(join_overlay(state, size));
    }
    p
}
fn hero(
    title: &str,
    subtitle: &str,
    glyph: &'static VectorIcon,
    w: f32,
    on: bool,
    f: impl Fn() + Send + Sync + 'static,
) -> Stack {
    Stack::new()
        .push(panel(w, 96.0, CARD))
        .push(at(
            panel(42.0, 42.0, BLUE).child(Center::new(icon(glyph, 30.0, TEXT))),
            18.0,
            24.0,
        ))
        .push(at(text(title, 24.0, TEXT), 76.0, 17.0))
        .push(at(text(subtitle, 18.0, SUB), 76.0, 53.0))
        .push(at(toggle(on, f), w - 80.0, 21.0))
}
fn pagination(mut p: Stack, state: Arc<Mutex<LauncherState>>, w: f32, count: usize) -> Stack {
    let page = state
        .lock()
        .unwrap()
        .settings_view
        .page
        .min(count.saturating_sub(1) / 4);
    let pages = count.div_ceil(4).max(1);
    p = p.push(at(
        text(format!("{} / {}", page + 1, pages), 16.0, SUB),
        w - 180.0,
        482.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("上一页", 76.0, 32.0, false, move || {
            edit(&s, |s| {
                s.settings_view.page = s.settings_view.page.saturating_sub(1)
            })
        }),
        w - 284.0,
        474.0,
    ));
    p.push(at(
        button("下一页", 76.0, 32.0, false, move || {
            edit(&state, |s| {
                s.settings_view.page = (s.settings_view.page + 1).min(pages - 1)
            })
        }),
        w - 84.0,
        474.0,
    ))
}
fn wifi_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (r, page, confirm) = {
        let s = state.lock().unwrap();
        (
            s.radio,
            s.settings_view.page,
            s.settings_view.confirm_forget,
        )
    };
    let s = state.clone();
    let mut p = hero(
        "Wi-Fi",
        r.wifi_status(),
        &UI_WIFI_3,
        w,
        r.wifi_on != 0,
        move || command(&s, Cmd::WifiEnable(r.wifi_on == 0)),
    );
    let saved = radio::label(&r.saved_ssid);
    p = p.push(at(
        text(
            if r.wifi_phase == 5 || r.wifi_phase == 4 {
                "当前网络"
            } else {
                "已保存的网络"
            },
            18.0,
            SUB,
        ),
        8.0,
        116.0,
    ));
    let mut current = Stack::new().push(panel(w, 80.0, CARD));
    if r.wifi_phase == 5 || r.wifi_phase == 4 {
        current = current
            .push(at(
                text(short(&radio::label(&r.ssid), 29), 22.0, TEXT),
                20.0,
                10.0,
            ))
            .push(at(
                text(
                    if r.wifi_phase == 5 {
                        format!("已连接  ·  {}", radio::label(&r.ip))
                    } else {
                        "正在连接…".into()
                    },
                    16.0,
                    if r.wifi_phase == 5 { GREEN } else { SUB },
                ),
                20.0,
                44.0,
            ));
        let s = state.clone();
        current = current.push(at(
            button("断开", 76.0, 36.0, false, move || {
                command(&s, Cmd::WifiDisconnect)
            }),
            w - 94.0,
            22.0,
        ));
    } else if !saved.is_empty() {
        current = current
            .push(at(text(short(&saved, 22), 22.0, TEXT), 20.0, 10.0))
            .push(at(text("开启 Wi-Fi 后可自动连接", 16.0, SUB), 20.0, 44.0));
        let s = state.clone();
        current = current.push(at(
            button("连接", 72.0, 36.0, false, move || {
                command(&s, Cmd::WifiSavedConnect)
            }),
            w - 226.0,
            22.0,
        ));
        let s = state.clone();
        current = current.push(at(
            button(
                if confirm {
                    "确认忘记"
                } else {
                    "忘记网络"
                },
                120.0,
                36.0,
                false,
                move || {
                    edit(&s, |s| {
                        if s.settings_view.confirm_forget {
                            s.queue(UiCommand::Radio(Cmd::WifiForget));
                        }
                        s.settings_view.confirm_forget = !s.settings_view.confirm_forget;
                    })
                },
            ),
            w - 138.0,
            22.0,
        ));
    } else {
        current = current
            .push(at(text("选择下方网络以加入", 22.0, TEXT), 20.0, 13.0))
            .push(at(
                text("支持 2.4 GHz 网络，密码在本机输入", 16.0, SUB),
                20.0,
                47.0,
            ));
    }
    p = p
        .push(at(current, 0.0, 143.0))
        .push(at(text("其他网络", 18.0, SUB), 8.0, 245.0));
    let s = state.clone();
    p = p.push(at(
        button(
            if r.wifi_scan != 0 {
                "搜索中…"
            } else {
                "重新搜索"
            },
            112.0,
            34.0,
            false,
            move || {
                if r.wifi_on != 0 && r.wifi_scan == 0 {
                    command(&s, Cmd::WifiScan)
                }
            },
        ),
        w - 112.0,
        237.0,
    ));
    let count = r.aps().len();
    let page = page.min(count.saturating_sub(1) / 4);
    p = p.push(at(panel(w, 184.0, CARD), 0.0, 282.0));
    for (i, ap) in r.aps().iter().skip(page * 4).take(4).copied().enumerate() {
        let mut row = Stack::new().push(at(text(short(&ap.label(), 31), 22.0, TEXT), 18.0, 10.0));
        if ap.security != 0 {
            row = row.push(at(icon(&UI_LOCK, 20.0, SUB), w - 106.0, 13.0));
        }
        let wifi = if ap.rssi >= -55 {
            &UI_WIFI_3
        } else if ap.rssi >= -70 {
            &UI_WIFI_2
        } else {
            &UI_WIFI_1
        };
        row = row.push(at(icon(wifi, 26.0, SUB), w - 68.0, 10.0));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .size(w, 46.0)
                        .color(CARD)
                        .pressed_color(LINE)
                        .border_radius(10.0)
                        .padding(EdgeInsets::all(0.0)),
                )
                .on_pressed(move || {
                    edit(&s, |s| {
                        if s.radio.wifi_on == 0 {
                            return;
                        }
                        if ap.security == 2 {
                            s.notice = "暂不支持企业认证或旧式加密网络".into();
                        } else if ap.saved != 0 {
                            s.queue(UiCommand::Radio(Cmd::WifiSavedConnect));
                        } else {
                            s.settings_view.join = Some(WifiJoin::new(&ap));
                        }
                    })
                }),
            0.0,
            282.0 + i as f32 * 46.0,
        ));
    }
    if count == 0 {
        p = p.push(at(
            text(
                if r.wifi_on == 0 {
                    "开启 Wi-Fi 以搜索附近网络"
                } else if r.wifi_scan != 0 {
                    "正在查找附近的无线网络…"
                } else {
                    "暂无网络，轻触“重新搜索”"
                },
                22.0,
                SUB,
            ),
            24.0,
            348.0,
        ));
    }
    pagination(p, state, w, count)
}
fn ble_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (r, page, selected) = {
        let s = state.lock().unwrap();
        (s.radio, s.settings_view.page, s.settings_view.selected_ble)
    };
    let s = state.clone();
    let mut p = hero(
        "蓝牙",
        r.ble_status(),
        &UI_BLUETOOTH,
        w,
        r.bt_on != 0,
        move || command(&s, Cmd::BleEnable(r.bt_on == 0)),
    );
    if let Some(id) = selected {
        return ble_details(p, state, w, r, id);
    }
    p = p
        .push(at(panel(w, 92.0, CARD), 0.0, 115.0))
        .push(at(text("本机 · P4Desk", 22.0, TEXT), 20.0, 131.0))
        .push(at(
            text(
                if r.phone_connected != 0 {
                    "手机或其他中心设备已连接"
                } else {
                    "手机可通过 BLE 应用发现并连接本机"
                },
                18.0,
                if r.phone_connected != 0 { GREEN } else { SUB },
            ),
            20.0,
            171.0,
        ));
    if r.phone_connected != 0 {
        let s = state.clone();
        p = p.push(at(
            button("断开", 76.0, 34.0, false, move || {
                command(&s, Cmd::BleDisconnect)
            }),
            w - 94.0,
            127.0,
        ));
    }
    p = p.push(at(text("附近的设备", 18.0, SUB), 8.0, 240.0));
    let s = state.clone();
    p = p.push(at(
        button(
            if r.bt_scan != 0 {
                "搜索中…"
            } else {
                "搜索设备"
            },
            112.0,
            34.0,
            false,
            move || {
                if r.bt_on != 0 && r.bt_ready != 0 && r.bt_scan == 0 {
                    command(&s, Cmd::BleScan)
                }
            },
        ),
        w - 112.0,
        232.0,
    ));
    let count = r.devices().len();
    let page = page.min(count.saturating_sub(1) / 4);
    p = p.push(at(panel(w, 196.0, CARD), 0.0, 274.0));
    for (i, d) in r
        .devices()
        .iter()
        .skip(page * 4)
        .take(4)
        .copied()
        .enumerate()
    {
        let caption = if d.connected != 0 {
            "已连接"
        } else if d.connectable != 0 {
            "查看"
        } else {
            "广播"
        };
        let row = Stack::new()
            .push(at(icon(&UI_BLUETOOTH, 23.0, SUB), 16.0, 13.0))
            .push(at(text(short(&d.label(), 25), 22.0, TEXT), 52.0, 11.0))
            .push(at(
                text(caption, 16.0, if d.connected != 0 { GREEN } else { SUB }),
                w - 84.0,
                16.0,
            ));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .size(w, 49.0)
                        .color(CARD)
                        .pressed_color(LINE)
                        .border_radius(10.0)
                        .padding(EdgeInsets::all(0.0)),
                )
                .on_pressed(move || edit(&s, |s| s.settings_view.selected_ble = Some(d.id))),
            0.0,
            274.0 + i as f32 * 49.0,
        ));
    }
    if count == 0 {
        p = p.push(at(
            text(
                if r.bt_on == 0 {
                    "开启蓝牙以查找附近设备"
                } else {
                    "将 BLE 设备设为可发现，再轻触“搜索设备”"
                },
                20.0,
                SUB,
            ),
            20.0,
            352.0,
        ));
    }
    pagination(p, state, w, count)
}
fn ble_details(
    mut p: Stack,
    state: Arc<Mutex<LauncherState>>,
    w: f32,
    r: radio::RadioSnapshot,
    id: u32,
) -> Stack {
    let s = state.clone();
    p = p.push(at(
        button("返回设备列表", 148.0, 36.0, false, move || {
            edit(&s, |s| s.settings_view.selected_ble = None)
        }),
        0.0,
        112.0,
    ));
    let Some(d) = r.devices().iter().find(|d| d.id == id) else {
        return p.push(at(
            text("设备列表已更新，请返回重新选择", 22.0, SUB),
            20.0,
            194.0,
        ));
    };
    p = p
        .push(at(panel(w, 138.0, CARD), 0.0, 166.0))
        .push(at(text(short(&d.label(), 32), 26.0, TEXT), 20.0, 183.0))
        .push(at(
            text(
                format!(
                    "信号 {} dBm · {}",
                    d.rssi,
                    if d.connected != 0 {
                        if r.bt_secure != 0 {
                            "连接已加密"
                        } else {
                            "已连接"
                        }
                    } else if r.bt_peer_id == id {
                        "正在连接"
                    } else if d.connectable != 0 {
                        "可连接"
                    } else {
                        "仅广播，无法连接"
                    }
                ),
                18.0,
                SUB,
            ),
            20.0,
            225.0,
        ));
    if d.connected != 0 {
        for (i, (label, c)) in [
            ("断开", Cmd::BleDisconnect),
            ("配对", Cmd::BlePair),
            ("忘记配对", Cmd::BleForget),
        ]
        .into_iter()
        .enumerate()
        {
            let s = state.clone();
            p = p.push(at(
                button(label, 112.0, 34.0, i == 1, move || command(&s, c.clone())),
                20.0 + i as f32 * 128.0,
                260.0,
            ));
        }
        p = p.push(at(text("GATT 服务", 18.0, SUB), 8.0, 323.0));
        if r.services_count == 0 {
            p = p.push(at(text("正在读取服务…", 20.0, SUB), 20.0, 360.0));
        }
        for (i, uuid) in r.services().iter().take(5).enumerate() {
            p = p.push(at(
                text(short(&radio::label(uuid), 39), 18.0, TEXT),
                20.0,
                359.0 + i as f32 * 29.0,
            ));
        }
    } else if r.bt_peer_id == id {
        p = p.push(at(text("正在连接，请稍候…", 18.0, SUB), 20.0, 267.0));
    } else if d.connectable != 0 {
        let s = state.clone();
        p = p.push(at(
            button("连接设备", 132.0, 34.0, true, move || {
                command(&s, Cmd::BleConnect(id))
            }),
            20.0,
            260.0,
        ));
    }
    p
}
fn display_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let value = state.lock().unwrap().settings.brightness;
    let mut p = Stack::new()
        .push(panel(w, 190.0, CARD))
        .push(at(text("屏幕亮度", 24.0, TEXT), 22.0, 22.0))
        .push(at(text(format!("{value}%"), 28.0, TEXT), w - 105.0, 20.0));
    let bw = (w - 72.0) / 4.0;
    for (i, v) in [25u8, 50, 75, 100].into_iter().enumerate() {
        let s = state.clone();
        p = p.push(at(
            button(format!("{v}%"), bw, 50.0, value == v, move || {
                edit(&s, |s| s.queue(UiCommand::Brightness(v)))
            }),
            22.0 + i as f32 * (bw + 9.0),
            88.0,
        ));
    }
    let s = state.clone();
    p = p
        .push(at(
            button("关闭屏幕", 160.0, 46.0, false, move || {
                edit(&s, |s| s.queue(UiCommand::Screen(false)))
            }),
            0.0,
            216.0,
        ))
        .push(at(text("轻触屏幕即可唤醒", 18.0, SUB), 180.0, 229.0));
    let s = state.clone();
    p.push(at(
        button("进入 USB 副屏", w, 60.0, true, move || {
            edit(&s, |s| {
                s.queue(UiCommand::RequestMode(p4desk_protocol::Mode::Display))
            })
        }),
        0.0,
        313.0,
    ))
    .push(at(
        text("副屏中三指长按一秒，可返回 Pad", 18.0, SUB),
        20.0,
        392.0,
    ))
}
fn time_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (open, clock, date, valid) = {
        let s = state.lock().unwrap();
        (
            s.manual_time_open,
            s.clock.clone(),
            s.date.clone(),
            s.time_valid,
        )
    };
    if open {
        return crate::launcher_ui::manual_time_page(state, w);
    }
    let s = state.clone();
    let p = Stack::new()
        .push(panel(w, 170.0, CARD))
        .push(at(
            text(if valid { clock } else { "待校时".into() }, 36.0, TEXT),
            24.0,
            24.0,
        ))
        .push(at(text(date, 22.0, SUB), 24.0, 86.0))
        .push(at(text("连接 Mac 后自动校时", 18.0, SUB), 24.0, 126.0));
    let p = p.push(at(
        button(
            "手动设置日期与时间",
            260.0,
            50.0,
            true,
            move || edit(&s, |s| s.manual_time_open = true),
        ),
        0.0,
        194.0,
    ));
    let s = state.clone();
    p.push(at(
        button("从 Mac 校时", 180.0, 50.0, false, move || {
            edit(&s, |s| s.queue(UiCommand::RequestTimeSync))
        }),
        280.0,
        194.0,
    ))
}
fn storage_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let s = state.lock().unwrap();
    Stack::new()
        .push(panel(w, 220.0, CARD))
        .push(at(icon(&UI_SD_CARD, 46.0, TEXT), 24.0, 28.0))
        .push(at(text("TF 卡", 26.0, TEXT), 90.0, 28.0))
        .push(at(
            text(
                if s.sd_ready { "已就绪" } else { "未挂载" },
                20.0,
                if s.sd_ready { GREEN } else { AMBER },
            ),
            90.0,
            69.0,
        ))
        .push(at(
            text(format!("资源代次：{}", s.snapshot.generation), 22.0, TEXT),
            24.0,
            124.0,
        ))
        .push(at(
            text("便签、图标与字形资源保存在 /sdcard/p4desk", 18.0, SUB),
            24.0,
            174.0,
        ))
}
fn about_page(w: f32) -> Stack {
    Stack::new()
        .push(panel(w, 278.0, CARD))
        .push(at(icon(&DESKTOP_SETTINGS, 64.0, TEXT), 24.0, 26.0))
        .push(at(text("P4Desk", 34.0, TEXT), 108.0, 29.0))
        .push(at(text("桌面助手 · 版本 0.1.0", 18.0, SUB), 108.0, 76.0))
        .push(at(text("ESP32-P4  /  1024 × 600", 22.0, TEXT), 24.0, 134.0))
        .push(at(
            text("Wi-Fi 6 · 2.4 GHz   /   Bluetooth LE", 22.0, TEXT),
            24.0,
            176.0,
        ))
        .push(at(text("界面字体：HarmonyOS Sans", 18.0, SUB), 24.0, 225.0))
}
fn join_overlay(state: Arc<Mutex<LauncherState>>, size: Size) -> Stack {
    let (name, secured, password, upper, symbols, reveal, valid) = {
        let s = state.lock().unwrap();
        let j = s.settings_view.join.as_ref().unwrap();
        (
            j.name.clone(),
            j.secured,
            j.password.clone(),
            j.uppercase,
            j.symbols,
            j.reveal,
            j.valid(),
        )
    };
    let x = 88.0;
    let w = size.width - 176.0;
    let blocker = GestureDetector::new(
        Container::new()
            .width(size.width)
            .height(size.height)
            .color(Color::from_rgba(0, 0, 0, 185)),
    )
    .on_tap(|| {});
    let mut p = Stack::new()
        .push(blocker)
        .push(at(panel(w, 554.0, BG), x, 22.0))
        .push(at(
            text(format!("加入 {}", short(&name, 28)), 26.0, TEXT),
            x + 24.0,
            44.0,
        ));
    let s = state.clone();
    p = p.push(at(
        icon_button(&UI_CLOSE, BG, move || {
            edit(&s, |s| s.settings_view.join = None)
        }),
        x + w - 62.0,
        32.0,
    ));
    if secured {
        let shown = if password.is_empty() {
            "输入 Wi-Fi 密码".into()
        } else if reveal {
            password
                .chars()
                .rev()
                .take(42)
                .collect::<String>()
                .chars()
                .rev()
                .collect()
        } else {
            "•".repeat(password.len().min(42))
        };
        p = p.push(at(
            panel(w - 48.0, 55.0, CARD).child(Center::new(text(shown, 24.0, TEXT))),
            x + 24.0,
            91.0,
        ));
        let s = state.clone();
        p = p.push(at(
            button(
                if reveal {
                    "隐藏密码"
                } else {
                    "显示密码"
                },
                112.0,
                34.0,
                false,
                move || {
                    edit(&s, |s| {
                        if let Some(j) = s.settings_view.join.as_mut() {
                            j.reveal = !j.reveal;
                        }
                    })
                },
            ),
            x + w - 136.0,
            156.0,
        ));
        p = p.push(at(
            text("8–63 位字符，支持大小写、数字和符号", 18.0, SUB),
            x + 24.0,
            163.0,
        ));
        let rows: Vec<Vec<char>> = if symbols {
            vec![
                "!@#$%^&*()".chars().collect(),
                "-_=+[]{}\\|".chars().collect(),
                ";:'\",.<>/?".chars().collect(),
                "`~01234567".chars().collect(),
            ]
        } else {
            vec![
                "1234567890".chars().collect(),
                "qwertyuiop".chars().collect(),
                "asdfghjkl".chars().collect(),
                "zxcvbnm".chars().collect(),
            ]
        };
        let kw = (w - 68.0) / 10.0;
        for (ri, row) in rows.iter().enumerate() {
            let offset = (10 - row.len()) as f32 * (kw + 2.0) / 2.0;
            for (ci, ch) in row.iter().enumerate() {
                let c = if upper { ch.to_ascii_uppercase() } else { *ch };
                let s = state.clone();
                p = p.push(at(
                    button(c.to_string(), kw, 52.0, false, move || {
                        edit(&s, |s| {
                            if let Some(j) = s.settings_view.join.as_mut() {
                                j.push(c);
                            }
                        })
                    }),
                    x + 24.0 + offset + ci as f32 * (kw + 2.0),
                    204.0 + ri as f32 * 60.0,
                ));
            }
        }
        let s = state.clone();
        p = p.push(at(
            button(
                if upper { "小写" } else { "大写" },
                92.0,
                44.0,
                upper,
                move || {
                    edit(&s, |s| {
                        if let Some(j) = s.settings_view.join.as_mut() {
                            j.uppercase = !j.uppercase;
                        }
                    })
                },
            ),
            x + 24.0,
            452.0,
        ));
        let s = state.clone();
        p = p.push(at(
            button(
                if symbols { "ABC" } else { "符号" },
                92.0,
                44.0,
                symbols,
                move || {
                    edit(&s, |s| {
                        if let Some(j) = s.settings_view.join.as_mut() {
                            j.symbols = !j.symbols;
                        }
                    })
                },
            ),
            x + 128.0,
            452.0,
        ));
        let s = state.clone();
        p = p.push(at(
            button("空格", 200.0, 44.0, false, move || {
                edit(&s, |s| {
                    if let Some(j) = s.settings_view.join.as_mut() {
                        j.push(' ');
                    }
                })
            }),
            x + 232.0,
            452.0,
        ));
        let s = state.clone();
        p = p.push(at(
            button("退格", 92.0, 44.0, false, move || {
                edit(&s, |s| {
                    if let Some(j) = s.settings_view.join.as_mut() {
                        j.password.pop();
                    }
                })
            }),
            x + 444.0,
            452.0,
        ));
    } else {
        p = p.push(at(text("此网络不需要密码", 24.0, SUB), x + 24.0, 125.0));
    }
    let s = state.clone();
    p = p.push(at(
        button("取消", 112.0, 44.0, false, move || {
            edit(&s, |s| s.settings_view.join = None)
        }),
        x + w - 260.0,
        514.0,
    ));
    p.push(at(
        button("加入", 112.0, 44.0, valid, move || {
            edit(&state, |s| {
                if !s.settings_view.join.as_ref().is_some_and(|j| j.valid()) {
                    return;
                }
                let j = s.settings_view.join.take().unwrap();
                s.queue(UiCommand::Radio(Cmd::WifiConnect {
                    id: j.id,
                    password: j.password,
                }));
            })
        }),
        x + w - 136.0,
        514.0,
    ))
}
