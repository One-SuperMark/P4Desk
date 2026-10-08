//! Folio grouped settings, sized for a 1024x600 touch display.
use crate::radio::{self, RadioCommand as Cmd, SettingsSection as Section, WifiJoin};
use crate::{LauncherState, UiCommand};
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;
use tiny_flutter::VectorIcon;

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
        .border_radius(Folio::GROUP_RADIUS)
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
    ElevatedButton::new(text(s, 18.0, Folio::ink()))
        .style(
            Folio::control_style(
                w,
                h,
                if accent {
                    Folio::accent()
                } else {
                    Folio::line()
                },
            )
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
    ElevatedButton::new(icon(i, 24.0, Folio::ink()))
        .style(
            Folio::navigation_style()
                .color(c)
                .pressed_color(Folio::pressed(c)),
        )
        .on_pressed(f)
}
const SWITCH_WIDTH: f32 = 72.0;
const SWITCH_HEIGHT: f32 = 48.0;
const SWITCH_RIGHT_INSET: f32 = 18.0;
fn switch_x(panel_width: f32) -> f32 {
    panel_width - SWITCH_WIDTH - SWITCH_RIGHT_INSET
}
fn switch_center(right: bool) -> f32 {
    if right {
        SWITCH_WIDTH - 21.0
    } else {
        21.0
    }
}
fn paint_switch(c: &mut Canvas, right: bool, color: Color) {
    c.draw_rrect_aa(
        RRect::from_rect_circular(Rect::from_ltwh(5.0, 8.0, SWITCH_WIDTH - 10.0, 32.0), 16.0),
        color,
    );
    c.draw_rrect_aa(
        RRect::from_rect_circular(
            Rect::from_ltwh(switch_center(right) - 14.0, 10.0, 28.0, 28.0),
            14.0,
        ),
        Color::WHITE,
    );
}
fn switch_control(
    painter: impl CustomPainter + 'static,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(CustomPaint::new(painter).size(Size::new(SWITCH_WIDTH, SWITCH_HEIGHT)))
        .style(
            ButtonStyle::new()
                .antialias(true)
                .size(SWITCH_WIDTH, SWITCH_HEIGHT)
                .color(Color::TRANSPARENT)
                .pressed_color(Folio::pressed(Color::TRANSPARENT))
                .border_radius(24.0)
                .padding(EdgeInsets::all(0.0)),
        )
        .on_pressed(f)
}
struct Toggle(bool);
impl CustomPainter for Toggle {
    fn paint(&self, c: &mut Canvas, _: Size) {
        paint_switch(
            c,
            self.0,
            if self.0 {
                Folio::green()
            } else {
                Folio::line()
            },
        );
    }
}
fn toggle(on: bool, f: impl Fn() + Send + Sync + 'static) -> ElevatedButton {
    switch_control(Toggle(on), f)
}
struct IconPaletteSwitch(bool);
impl CustomPainter for IconPaletteSwitch {
    fn paint(&self, c: &mut Canvas, _: Size) {
        let light = self.0;
        paint_switch(
            c,
            !light,
            Color::from_hex(if light { 0xe7c788 } else { 0x475579 }),
        );
        let (active, inactive, tint) = if light {
            (&UI_SUN, &UI_MOON, Color::from_hex(0xb77820))
        } else {
            (&UI_MOON, &UI_SUN, Color::from_hex(0x475579))
        };
        inactive.paint(
            c,
            Rect::from_ltwh(switch_center(light) - 6.0, 18.0, 12.0, 12.0),
            Color::from_hex(if light { 0x967342 } else { 0xb9c6e4 }),
        );
        active.paint(
            c,
            Rect::from_ltwh(switch_center(!light) - 9.0, 15.0, 18.0, 18.0),
            tint,
        );
    }
}
fn icon_palette_switch(light: bool, f: impl Fn() + Send + Sync + 'static) -> ElevatedButton {
    switch_control(IconPaletteSwitch(light), f)
}
fn section_icon(s: Section) -> (&'static VectorIcon, Color) {
    match s {
        Section::Wifi => (&UI_WIFI_3, Folio::accent()),
        Section::Bluetooth => (&UI_BLUETOOTH, Folio::accent()),
        Section::Appearance => (&UI_PALETTE, Color::from_hex(0xa375d3)),
        Section::Display => (&DESKTOP_DISPLAY, Color::from_hex(0x7458d6)),
        Section::DateTime => (&UI_CALENDAR, Color::from_hex(0xf05454)),
        Section::Storage => (&UI_STORAGE, Color::from_hex(0x85837e)),
        Section::About => (&UI_INFO, Color::from_hex(0x747c8c)),
    }
}
fn section_badge(section: Section) -> Container {
    let name = match section {
        Section::Wifi => "wifi",
        Section::Bluetooth => "bluetooth",
        Section::Display => "display-settings",
        Section::DateTime => "date-time",
        Section::Storage => "storage",
        Section::About => "about",
        Section::Appearance => "appearance",
    };
    if let Some(asset) = crate::app_icons::get_settings_icon(name) {
        Container::new()
            .width(30.0)
            .height(30.0)
            .child(icon(asset, 30.0, Color::WHITE))
    } else {
        let (glyph, color) = section_icon(section);
        panel(30.0, 30.0, color).child(Center::new(icon(glyph, 23.0, Folio::on_fill(color))))
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
                .color(Folio::bg()),
        )
        .push(
            Container::new()
                .width(SIDEBAR)
                .height(size.height)
                .color(Folio::surface())
                .glass(true),
        );
    let s = state.clone();
    p = p
        .push(at(
            panel(SIDEBAR - 24.0, 128.0, Folio::raised()),
            12.0,
            82.0,
        ))
        .push(at(
            panel(SIDEBAR - 24.0, 314.0, Folio::raised()),
            12.0,
            218.0,
        ))
        .push(at(
            Container::new()
                .width(1.0)
                .height(size.height)
                .color(Folio::separator()),
            SIDEBAR - 1.0,
            0.0,
        ));
    p = p
        .push(at(
            icon_button(&UI_HOME, Folio::raised(), move || {
                edit(&s, |s| s.background_active_app())
            }),
            14.0,
            14.0,
        ))
        .push(at(text("设置", 28.0, Folio::ink()), 74.0, 24.0));
    let s = state.clone();
    p = p.push(at(
        icon_button(&UI_CLOSE, Folio::raised(), move || {
            edit(&s, |s| s.kill_active_app())
        }),
        size.width - 60.0,
        14.0,
    ));
    for (i, section) in Section::ALL.into_iter().enumerate() {
        let row = Stack::new()
            .push(at(section_badge(section), 12.0, 10.0))
            .push(at(text(section.title(), 22.0, Folio::ink()), 55.0, 13.0));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .antialias(true)
                        .size(SIDEBAR - 40.0, 50.0)
                        .color(if selected == section {
                            Folio::accent()
                        } else {
                            Color::TRANSPARENT
                        })
                        .pressed_color(Folio::pressed(if selected == section {
                            Folio::accent()
                        } else {
                            Color::TRANSPARENT
                        }))
                        .border_radius(Folio::CONTROL_RADIUS)
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
            20.0,
            90.0 + i as f32 * 62.0 + if i >= 2 { 12.0 } else { 0.0 },
        ));
    }
    p = p
        .push(at(
            text("P4Desk", 18.0, Folio::muted()),
            24.0,
            size.height - 42.0,
        ))
        .push(at(
            text(selected.title(), 28.0, Folio::ink()),
            SIDEBAR + 24.0,
            24.0,
        ));
    let detail = match selected {
        Section::Wifi => wifi_page(state.clone(), w),
        Section::Bluetooth => ble_page(state.clone(), w),
        Section::Appearance => appearance_page(state.clone(), w, size.height - 100.0),
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
        .push(panel(w, 96.0, Folio::raised()))
        .push(at(
            Container::new().width(42.0).height(42.0).child(icon(
                crate::app_icons::get_settings_icon(if title == "Wi-Fi" {
                    "wifi"
                } else {
                    "bluetooth"
                })
                .unwrap_or(glyph),
                42.0,
                Color::WHITE,
            )),
            18.0,
            24.0,
        ))
        .push(at(text(title, 24.0, Folio::ink()), 76.0, 17.0))
        .push(at(text(subtitle, 18.0, Folio::muted()), 76.0, 53.0))
        .push(at(toggle(on, f), switch_x(w), 21.0))
}

struct ThemeIconPreview {
    theme: crate::icon_theme::IconTheme,
    id: &'static str,
}
impl CustomPainter for ThemeIconPreview {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let r = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        crate::app_icons::get_app_icon_for(self.theme, self.id)
            .unwrap()
            .paint(canvas, r, Color::WHITE);
        if self.id == "clock" {
            crate::live_clock_icon::paint_clock_hands_for_theme(
                canvas,
                r,
                crate::live_clock_icon::ClockTime::parse("10:09:30"),
                1.0,
                self.theme,
            );
        }
    }
}
fn theme_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    use crate::icon_theme::IconTheme;
    let selected = state.lock().unwrap().settings.icon_theme;
    let light_icons = state.lock().unwrap().settings.light_icons();
    let color_state = state.clone();
    let mut p = Stack::new()
        .push(panel(w, 422.0, Folio::raised()))
        .push(at(text("图标主题", 24.0, Folio::ink()), 24.0, 24.0))
        .push(at(text("图标配色", 18.0, Folio::ink()), 24.0, 65.0))
        .push(at(
            text(
                if light_icons { "浅色" } else { "深色" },
                18.0,
                Folio::muted(),
            ),
            switch_x(w) - 52.0,
            65.0,
        ))
        .push(at(
            icon_palette_switch(light_icons, move || {
                edit(&color_state, |s| {
                    s.queue(UiCommand::IconLight(!light_icons));
                });
            }),
            switch_x(w),
            51.0,
        ));
    let cw = (w - 84.0) / 4.0;
    for (i, theme) in IconTheme::ALL.into_iter().enumerate() {
        let chosen = selected == theme;
        let mut content = Stack::new();
        for (j, id) in ["clock", "timer", "calculator", "settings"]
            .into_iter()
            .enumerate()
        {
            content = content.push(at(
                CustomPaint::new(ThemeIconPreview { theme, id }).size(Size::new(54.0, 54.0)),
                (cw - 116.0) * 0.5 + (j % 2) as f32 * 62.0,
                16.0 + (j / 2) as f32 * 64.0,
            ));
        }
        content = content
            .push(at(
                Container::new()
                    .width(cw)
                    .height(30.0)
                    .child(Center::new(text(theme.label(), 18.0, Folio::ink()))),
                0.0,
                150.0,
            ))
            .push(at(
                Container::new()
                    .width(cw)
                    .height(26.0)
                    .child(Center::new(text(theme.subtitle(), 18.0, Folio::muted()))),
                0.0,
                181.0,
            ));
        if chosen {
            content = content.push(at(
                icon(&UI_CHECK_CIRCLE, 22.0, Folio::ink()),
                cw - 30.0,
                7.0,
            ));
        }
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(content)
                .style(
                    Folio::control_style(
                        cw,
                        224.0,
                        if chosen {
                            Folio::accent()
                        } else {
                            Folio::surface()
                        },
                    )
                    .padding(EdgeInsets::ZERO),
                )
                .on_pressed(move || {
                    edit(&s, |s| {
                        if s.settings.icon_theme != theme {
                            s.queue(UiCommand::IconTheme(theme));
                        }
                    })
                }),
            24.0 + i as f32 * (cw + 12.0),
            108.0,
        ));
    }
    p.push(at(
        text("图标配色独立于外观，选择会自动保存", 18.0, Folio::muted()),
        24.0,
        374.0,
    ))
}

struct AppearancePreview {
    light: bool,
}
impl CustomPainter for AppearancePreview {
    fn paint(&self, c: &mut Canvas, size: Size) {
        let r = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        c.draw_rrect_aa(
            RRect::from_rect_circular(r, 10.0),
            Color::from_hex(if self.light { 0xc7ddeb } else { 0x425d79 }),
        );
        c.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(10.0, 12.0, size.width - 20.0, size.height - 20.0),
                8.0,
            ),
            Color::from_hex(if self.light { 0xf4f6fa } else { 0x242a34 }),
        );
        for (i, color) in [0xff756d, 0xffcf67, 0x66cf92].into_iter().enumerate() {
            c.paint_circle(
                Point::new(20.0 + i as f32 * 9.0, 20.0),
                2.5,
                &tiny_gfx::Paint::new(Color::from_hex(color).to_gfx()),
                None,
            );
        }
        c.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(17.0, 30.0, size.width * 0.27, size.height - 43.0),
                4.0,
            ),
            Color::from_hex(if self.light { 0xdce5ec } else { 0x37404b }),
        );
        c.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(size.width * 0.43, 32.0, size.width * 0.41, 5.0),
                2.5,
            ),
            Color::from_hex(if self.light { 0x788a9b } else { 0xa4b0be }),
        );
        c.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(size.width * 0.43, 45.0, size.width * 0.30, 5.0),
                2.5,
            ),
            Color::from_hex(if self.light { 0xb5c2ce } else { 0x626f7f }),
        );
    }
}
struct GlassPreview;
impl CustomPainter for GlassPreview {
    fn paint(&self, c: &mut Canvas, size: Size) {
        let bg = Folio::pick(Color::from_hex(0x253b4d), Color::from_hex(0x98baca));
        c.draw_rrect_aa(
            RRect::from_rect_circular(Rect::from_ltwh(0.0, 0.0, size.width, size.height), 20.0),
            bg,
        );
        c.paint_circle(
            Point::new(size.width * 0.80, 33.0),
            22.0,
            &tiny_gfx::Paint::new(Color::from_hex(0xe0b998).to_gfx()),
            None,
        );
        c.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(10.0, 56.0, size.width - 20.0, size.height - 66.0),
                28.0,
            ),
            Folio::pick(Color::from_hex(0x466977), Color::from_hex(0xbcd6d7)),
        );
        c.liquid_glass(
            RRect::from_rect_circular(Rect::from_ltwh(16.0, 18.0, size.width - 32.0, 44.0), 22.0),
            Folio::glass(),
            false,
        );
        UI_HOME.paint(c, Rect::from_ltwh(32.0, 29.0, 22.0, 22.0), Folio::ink());
        UI_WIFI_3.paint(c, Rect::from_ltwh(76.0, 29.0, 22.0, 22.0), Folio::ink());
        UI_PALETTE.paint(c, Rect::from_ltwh(120.0, 29.0, 22.0, 22.0), Folio::ink());
        c.liquid_glass(
            RRect::from_rect_circular(
                Rect::from_ltwh(46.0, size.height - 54.0, size.width - 92.0, 38.0),
                19.0,
            ),
            Folio::glass(),
            false,
        );
        c.draw_text(
            "P4Desk",
            Font::default_font(),
            18.0,
            Point::new(size.width * 0.5 - 31.0, size.height - 44.0),
            Folio::ink(),
        );
    }
}
fn appearance_page(state: Arc<Mutex<LauncherState>>, w: f32, h: f32) -> Stack {
    let (light, amount) = {
        let s = state.lock().unwrap();
        (s.settings.light_appearance, s.settings.glass_amount)
    };
    let mut p = Stack::new()
        .push(panel(w, 174.0, Folio::raised()))
        .push(at(text("外观模式", 24.0, Folio::ink()), 24.0, 26.0))
        .push(at(
            text("选择适合你的明暗", 18.0, Folio::muted()),
            24.0,
            66.0,
        ));
    for (i, (is_light, label)) in [(true, "浅色"), (false, "深色")].into_iter().enumerate() {
        let selected = light == is_light;
        let content = Stack::new()
            .push(at(
                CustomPaint::new(AppearancePreview { light: is_light })
                    .size(Size::new(144.0, 78.0)),
                12.0,
                10.0,
            ))
            .push(at(text(label, 18.0, Folio::ink()), 65.0, 99.0));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(content)
                .style(
                    Folio::control_style(
                        168.0,
                        136.0,
                        if selected {
                            Folio::accent()
                        } else {
                            Folio::surface()
                        },
                    )
                    .padding(EdgeInsets::ZERO),
                )
                .on_pressed(move || {
                    edit(&s, |s| {
                        s.queue(UiCommand::Appearance {
                            light: is_light,
                            glass: amount,
                        })
                    })
                }),
            w - 376.0 + i as f32 * 180.0,
            18.0,
        ));
    }
    let s = state.clone();
    p = p
        .push(at(panel(w, 282.0, Folio::raised()), 0.0, 192.0))
        .push(at(text("Liquid Glass", 26.0, Folio::ink()), 24.0, 215.0))
        .push(at(
            toggle(amount > 0, move || {
                edit(&s, |s| {
                    s.queue(UiCommand::Appearance {
                        light,
                        glass: if amount > 0 { 0 } else { 65 },
                    })
                })
            }),
            switch_x(w),
            205.0,
        ))
        .push(at(
            text("柔化背景，让控件更轻盈", 18.0, Folio::muted()),
            24.0,
            271.0,
        ))
        .push(at(
            CustomPaint::new(GlassPreview).size(Size::new(224.0, 146.0)),
            w - 248.0,
            264.0,
        ));
    for (i, (value, label)) in [(25, "通透"), (65, "均衡"), (100, "柔和")]
        .into_iter()
        .enumerate()
    {
        let s = state.clone();
        p = p.push(at(
            button(label, 124.0, 48.0, amount == value, move || {
                edit(&s, |s| {
                    s.queue(UiCommand::Appearance {
                        light,
                        glass: value,
                    })
                })
            }),
            24.0 + i as f32 * 132.0,
            318.0,
        ));
    }
    p = p
        .push(at(
            text("向上滑动选择图标主题", 18.0, Folio::muted()),
            24.0,
            427.0,
        ))
        .push(at(theme_page(state.clone(), w), 0.0, 494.0));
    let controller = state
        .lock()
        .unwrap()
        .settings_view
        .appearance_scroll
        .clone();
    Stack::new().push(
        Container::new().width(w).height(h).child(
            SingleChildScrollView::new(Container::new().width(w).height(940.0).child(p))
                .controller(controller)
                .raster_cache(
                    (crate::icon_theme::raster_key() as u64) << 8 | amount as u64,
                    Folio::bg(),
                ),
        ),
    )
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
        text(format!("{} / {}", page + 1, pages), 16.0, Folio::muted()),
        w - 180.0,
        482.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("上一页", 76.0, 44.0, false, move || {
            edit(&s, |s| {
                s.settings_view.page = s.settings_view.page.saturating_sub(1)
            })
        }),
        w - 284.0,
        474.0,
    ));
    p.push(at(
        button("下一页", 76.0, 44.0, false, move || {
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
            Folio::muted(),
        ),
        8.0,
        116.0,
    ));
    let mut current = Stack::new().push(panel(w, 80.0, Folio::raised()));
    if r.wifi_phase == 5 || r.wifi_phase == 4 {
        current = current
            .push(at(
                text(short(&radio::label(&r.ssid), 29), 22.0, Folio::ink()),
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
                    if r.wifi_phase == 5 {
                        Folio::green()
                    } else {
                        Folio::muted()
                    },
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
            .push(at(text(short(&saved, 22), 22.0, Folio::ink()), 20.0, 10.0))
            .push(at(
                text("开启 Wi-Fi 后可自动连接", 16.0, Folio::muted()),
                20.0,
                44.0,
            ));
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
            .push(at(
                text("选择下方网络以加入", 22.0, Folio::ink()),
                20.0,
                13.0,
            ))
            .push(at(
                text("支持 2.4 GHz 网络，密码在本机输入", 16.0, Folio::muted()),
                20.0,
                47.0,
            ));
    }
    p = p.push(at(current, 0.0, 143.0)).push(at(
        text("其他网络", 18.0, Folio::muted()),
        8.0,
        245.0,
    ));
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
    p = p.push(at(panel(w, 184.0, Folio::raised()), 0.0, 282.0));
    for (i, ap) in r.aps().iter().skip(page * 4).take(4).copied().enumerate() {
        let connected = r.wifi_phase == 5 && ap.label() == radio::label(&r.ssid);
        let mut row = Stack::new().push(at(
            text(short(&ap.label(), 29), 22.0, Folio::ink()),
            46.0,
            10.0,
        ));
        if connected {
            row = row.push(at(icon(&UI_CHECK_CIRCLE, 20.0, Folio::green()), 16.0, 13.0));
        }
        if ap.security != 0 {
            row = row.push(at(icon(&UI_LOCK, 20.0, Folio::muted()), w - 106.0, 13.0));
        }
        let wifi = if ap.rssi >= -55 {
            &UI_WIFI_3
        } else if ap.rssi >= -70 {
            &UI_WIFI_2
        } else {
            &UI_WIFI_1
        };
        row = row.push(at(icon(wifi, 26.0, Folio::muted()), w - 68.0, 10.0));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .antialias(true)
                        .size(w, 46.0)
                        .color(Color::TRANSPARENT)
                        .pressed_color(Folio::pressed(Color::TRANSPARENT))
                        .border_radius(Folio::GROUP_RADIUS)
                        .padding(EdgeInsets::all(0.0)),
                )
                .on_pressed(move || {
                    edit(&s, |s| {
                        if s.radio.wifi_on == 0 {
                            return;
                        }
                        if ap.security == 2 {
                            s.show_notice(
                                crate::launcher_state::SystemNotice::UnsupportedWifiSecurity,
                            );
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
        if i + 1 < count.saturating_sub(page * 4).min(4) {
            p = p.push(at(
                Container::new()
                    .width(w - 40.0)
                    .height(1.0)
                    .color(Folio::separator()),
                20.0,
                282.0 + (i + 1) as f32 * 46.0,
            ));
        }
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
                Folio::muted(),
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
        .push(at(panel(w, 92.0, Folio::raised()), 0.0, 115.0))
        .push(at(text("本机 · P4Desk", 22.0, Folio::ink()), 20.0, 131.0))
        .push(at(
            text(
                if r.phone_connected != 0 {
                    "手机或其他中心设备已连接"
                } else {
                    "手机可通过 BLE 应用发现并连接本机"
                },
                18.0,
                if r.phone_connected != 0 {
                    Folio::green()
                } else {
                    Folio::muted()
                },
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
    p = p.push(at(text("附近的设备", 18.0, Folio::muted()), 8.0, 240.0));
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
    p = p.push(at(panel(w, 196.0, Folio::raised()), 0.0, 274.0));
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
            .push(at(icon(&UI_BLUETOOTH, 23.0, Folio::muted()), 16.0, 13.0))
            .push(at(
                text(short(&d.label(), 25), 22.0, Folio::ink()),
                52.0,
                11.0,
            ))
            .push(at(
                text(
                    caption,
                    16.0,
                    if d.connected != 0 {
                        Folio::green()
                    } else {
                        Folio::muted()
                    },
                ),
                w - 84.0,
                16.0,
            ));
        let s = state.clone();
        p = p.push(at(
            ElevatedButton::new(row)
                .style(
                    ButtonStyle::new()
                        .antialias(true)
                        .size(w, 49.0)
                        .color(Color::TRANSPARENT)
                        .pressed_color(Folio::pressed(Color::TRANSPARENT))
                        .border_radius(Folio::GROUP_RADIUS)
                        .padding(EdgeInsets::all(0.0)),
                )
                .on_pressed(move || edit(&s, |s| s.settings_view.selected_ble = Some(d.id))),
            0.0,
            274.0 + i as f32 * 49.0,
        ));
        if i + 1 < count.saturating_sub(page * 4).min(4) {
            p = p.push(at(
                Container::new()
                    .width(w - 68.0)
                    .height(1.0)
                    .color(Folio::separator()),
                52.0,
                274.0 + (i + 1) as f32 * 49.0,
            ));
        }
    }
    if count == 0 {
        p = p.push(at(
            text(
                if r.bt_on == 0 {
                    "开启蓝牙以查找附近设备"
                } else {
                    "将 BLE 设备设为可发现，再轻触“搜索设备”"
                },
                22.0,
                Folio::muted(),
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
            text("设备列表已更新，请返回重新选择", 22.0, Folio::muted()),
            20.0,
            194.0,
        ));
    };
    p = p
        .push(at(panel(w, 138.0, Folio::raised()), 0.0, 166.0))
        .push(at(
            text(short(&d.label(), 32), 26.0, Folio::ink()),
            20.0,
            183.0,
        ))
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
                Folio::muted(),
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
        p = p.push(at(text("GATT 服务", 18.0, Folio::muted()), 8.0, 323.0));
        if r.services_count == 0 {
            p = p.push(at(text("正在读取服务…", 22.0, Folio::muted()), 20.0, 360.0));
        }
        for (i, uuid) in r.services().iter().take(5).enumerate() {
            p = p.push(at(
                text(short(&radio::label(uuid), 39), 18.0, Folio::ink()),
                20.0,
                359.0 + i as f32 * 29.0,
            ));
        }
    } else if r.bt_peer_id == id {
        p = p.push(at(
            text("正在连接，请稍候…", 18.0, Folio::muted()),
            20.0,
            267.0,
        ));
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
        .push(panel(w, 190.0, Folio::raised()))
        .push(at(text("屏幕亮度", 24.0, Folio::ink()), 22.0, 22.0))
        .push(at(
            text(format!("{value}%"), 28.0, Folio::ink()),
            w - 105.0,
            20.0,
        ));
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
        .push(at(
            text("轻触屏幕即可唤醒", 18.0, Folio::muted()),
            180.0,
            229.0,
        ));
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
        text("副屏中三指长按一秒，可返回 Pad", 18.0, Folio::muted()),
        20.0,
        392.0,
    ))
}
fn time_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (open, clock, date, valid, connected, sync, timezone, estimated) = {
        let s = state.lock().unwrap();
        (
            s.manual_time_open,
            s.clock.clone(),
            s.date.clone(),
            s.time_valid,
            s.radio.wifi_indicator() == radio::WifiIndicator::Connected,
            s.radio.time_sync,
            s.settings.timezone_minutes,
            s.time_estimated,
        )
    };
    if open {
        return crate::launcher_ui::manual_time_page(state, w);
    }
    let offset = format!(
        "UTC{}{:02}:{:02}",
        if timezone < 0 { "−" } else { "+" },
        timezone.abs() / 60,
        timezone.abs() % 60
    );
    let can_sync = sync.can_request(connected);
    let mut sync_button = ElevatedButton::new(text(
        if sync.phase == 1 && connected {
            "正在对时…"
        } else {
            "立即对时"
        },
        18.0,
        if can_sync {
            Folio::ink()
        } else {
            Folio::disabled()
        },
    ))
    .style(
        Folio::control_style(
            154.0,
            46.0,
            if can_sync {
                Folio::accent()
            } else {
                Folio::line()
            },
        )
        .padding(EdgeInsets::all(6.0)),
    );
    if can_sync {
        let s = state.clone();
        sync_button = sync_button.on_pressed(move || {
            edit(&s, |s| {
                // Re-check at dispatch time, including after a disconnect or a previous tap.
                if s.radio
                    .time_sync
                    .can_request(s.radio.wifi_indicator() == radio::WifiIndicator::Connected)
                {
                    s.notice.clear();
                    s.queue(UiCommand::Radio(Cmd::WifiTimeSync));
                }
            })
        });
    }
    let p = Stack::new()
        .push(panel(w, 152.0, Folio::raised()))
        .push(at(
            text(
                if valid { clock } else { "待校时".into() },
                36.0,
                Folio::ink(),
            ),
            24.0,
            24.0,
        ))
        .push(at(text(offset, 18.0, Folio::muted()), w - 158.0, 36.0))
        .push(at(text(date, 22.0, Folio::muted()), 24.0, 82.0))
        .push(at(
            text(
                if estimated {
                    "已恢复上次时间，等待校准"
                } else {
                    ""
                },
                18.0,
                Folio::orange(),
            ),
            24.0,
            119.0,
        ))
        .push(at(panel(w, 198.0, Folio::raised()), 0.0, 170.0))
        .push(at(icon(&UI_WIFI_3, 28.0, Folio::accent_ink()), 24.0, 195.0))
        .push(at(text("网络自动对时", 22.0, Folio::ink()), 64.0, 196.0))
        .push(at(sync_button, w - 178.0, 188.0))
        .push(at(
            text(
                sync.status(connected),
                18.0,
                if connected && sync.phase == 2 {
                    Folio::green()
                } else if connected && sync.phase == 3 {
                    Folio::orange()
                } else {
                    Folio::muted()
                },
            ),
            24.0,
            244.0,
        ))
        .push(at(
            text(sync.last_sync_label(timezone), 18.0, Folio::muted()),
            24.0,
            280.0,
        ))
        .push(at(
            Container::new()
                .width(w - 48.0)
                .height(1.0)
                .color(Folio::separator()),
            24.0,
            315.0,
        ))
        .push(at(
            text(
                "连接 Wi-Fi 后自动校准，之后每小时更新",
                18.0,
                Folio::muted(),
            ),
            24.0,
            334.0,
        ));
    let s = state.clone();
    let p = p.push(at(
        button(
            "手动设置日期与时间",
            260.0,
            50.0,
            false,
            move || edit(&s, |s| s.manual_time_open = true),
        ),
        0.0,
        392.0,
    ));
    let s = state.clone();
    p.push(at(
        button("从 Mac 校时", 180.0, 50.0, false, move || {
            edit(&s, |s| s.queue(UiCommand::RequestTimeSync))
        }),
        280.0,
        392.0,
    ))
}
fn storage_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let s = state.lock().unwrap();
    Stack::new()
        .push(panel(w, 220.0, Folio::raised()))
        .push(at(icon(&UI_SD_CARD, 46.0, Folio::ink()), 24.0, 28.0))
        .push(at(text("TF 卡", 26.0, Folio::ink()), 90.0, 28.0))
        .push(at(
            text(
                if s.sd_ready { "已就绪" } else { "未挂载" },
                22.0,
                if s.sd_ready {
                    Folio::green()
                } else {
                    Folio::orange()
                },
            ),
            90.0,
            69.0,
        ))
        .push(at(
            text(
                format!("资源代次：{}", s.snapshot.generation),
                22.0,
                Folio::ink(),
            ),
            24.0,
            124.0,
        ))
        .push(at(
            text(
                "便签、图标与字形资源保存在 /sdcard/p4desk",
                18.0,
                Folio::muted(),
            ),
            24.0,
            174.0,
        ))
        .push(at(panel(w, 176.0, Folio::raised()), 0.0, 238.0))
        .push(at(text("断电恢复", 26.0, Folio::ink()), 24.0, 262.0))
        .push(at(
            text(
                s.persistence_status.label(),
                18.0,
                if s.persistence_status == crate::session::PersistenceStatus::Failed {
                    Folio::orange()
                } else {
                    Folio::muted()
                },
            ),
            24.0,
            310.0,
        ))
        .push(at(
            text(
                "应用状态保存在板载闪存，计时恢复后暂停",
                18.0,
                Folio::muted(),
            ),
            24.0,
            354.0,
        ))
}
fn about_page(w: f32) -> Stack {
    Stack::new()
        .push(panel(w, 278.0, Folio::raised()))
        .push(at(crate::widgets::app_badge("settings", 64.0), 24.0, 26.0))
        .push(at(text("P4Desk", 34.0, Folio::ink()), 108.0, 29.0))
        .push(at(
            text("桌面助手 · 版本 0.1.0", 18.0, Folio::muted()),
            108.0,
            76.0,
        ))
        .push(at(
            text("ESP32-P4  /  1024 × 600", 22.0, Folio::ink()),
            24.0,
            134.0,
        ))
        .push(at(
            text("Wi-Fi 6 · 2.4 GHz   /   Bluetooth LE", 22.0, Folio::ink()),
            24.0,
            176.0,
        ))
        .push(at(
            text("界面字体：HarmonyOS Sans", 18.0, Folio::muted()),
            24.0,
            225.0,
        ))
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
        .push(at(
            panel(w, 554.0, Folio::surface()).border_radius(28.0),
            x,
            22.0,
        ))
        .push(at(
            text(format!("加入 {}", short(&name, 28)), 26.0, Folio::ink()),
            x + 24.0,
            44.0,
        ));
    let s = state.clone();
    p = p.push(at(
        icon_button(&UI_CLOSE, Folio::raised(), move || {
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
            panel(w - 48.0, 55.0, Folio::raised()).child(Center::new(text(
                shown,
                24.0,
                Folio::ink(),
            ))),
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
            text("8–63 位字符，支持大小写、数字和符号", 18.0, Folio::muted()),
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
        p = p.push(at(
            text("此网络不需要密码", 24.0, Folio::muted()),
            x + 24.0,
            125.0,
        ));
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
