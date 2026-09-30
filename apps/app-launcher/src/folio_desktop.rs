//! Landscape adaptation of Folio's widgets, glass status rail and side dock.
//! Uses selected Colloid SVGs and original wallpaper; attribution: docs/third-party.md.
use crate::app_icons::get_app_icon_asset;
use crate::launcher_state::{LauncherState, UiCommand};
use crate::launcher_ui::{DESKTOP_ENTRIES, DESKTOP_PAGE_CAPACITY};
use crate::live_clock_icon::ClockTime;
use crate::status_bar::build_status_rail;
use crate::widgets::{build_app_icon, build_dock_icon, AppIconPainter, WallpaperPainter};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::{Folio, GlassMaterial};

pub const ICON_SIZE: f32 = 140.0;
const GRID_TOP: f32 = 208.0;
const GRID_ROW_HEIGHT: f32 = 188.0;
pub fn workspace_width(size: Size) -> f32 {
    size.width - 160.0
}
pub fn icon_center(size: Size, index: usize) -> Point {
    Point::new(
        24.0 + workspace_width(size) / 4.0 * (index % 4) as f32 + workspace_width(size) / 8.0,
        GRID_TOP + (index / 4) as f32 * GRID_ROW_HEIGHT + ICON_SIZE * 0.5,
    )
}
pub(crate) fn recent_apps(state: &LauncherState) -> Vec<String> {
    state
        .recent_app_ids()
        .filter(|id| get_app_icon_asset(id).is_some())
        .map(str::to_owned)
        .collect()
}
struct DockLayout {
    panel: Rect,
    pitch: f32,
    icon_size: f32,
}
fn dock_layout(size: Size, count: usize) -> Option<DockLayout> {
    if count == 0 {
        return None;
    }
    // The recent-app history keeps at most four entries. Anchor the oldest at
    // the top; new entries extend the Dock downward without resizing icons.
    let pitch = 72.0;
    let height = count as f32 * pitch + 24.0;
    Some(DockLayout {
        panel: Rect::from_ltwh(size.width - 116.0, 264.0, 92.0, height),
        pitch,
        icon_size: pitch - 4.0,
    })
}
fn at(child: impl Widget + 'static, x: f32, y: f32) -> Positioned {
    Positioned::new(child).left(x).top(y)
}
fn text(label: impl Into<String>, px: f32, color: Color) -> Text {
    Text::new(label).font_size(px).color(color)
}
pub fn glass(w: f32, h: f32) -> Container {
    Container::new()
        .width(w)
        .height(h)
        .color(Folio::glass())
        .glass_material(GlassMaterial::Desktop)
        .border_radius(Folio::CARD_RADIUS)
}
struct CardTime(String);
impl CustomPainter for CardTime {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let font = Font::default_font();
        let px = 48.0;
        let baseline = font.cap_height(px);
        let (mut top, mut bottom) = (f32::INFINITY, f32::NEG_INFINITY);
        for ch in self.0.chars() {
            let glyph = font.glyph(ch, px);
            if glyph.height != 0 {
                top = top.min(baseline - glyph.ymin as f32 - glyph.height as f32);
                bottom = bottom.max(baseline - glyph.ymin as f32);
            }
        }
        if top.is_finite() && bottom.is_finite() {
            let y = (size.height - (bottom - top)) * 0.5 - top;
            canvas.draw_text(&self.0, font, px, Point::new(0.0, y), Folio::ink());
        }
    }
}
fn widget_card(
    state: Arc<Mutex<LauncherState>>,
    id: &'static str,
    rect: Rect,
    heading: String,
    value: String,
    caption: String,
    accent: Color,
    clock_time: Option<ClockTime>,
) -> impl Widget {
    let source = Rect::from_ltwh(rect.right() - 82.0, rect.y + 24.0, 58.0, 58.0);
    let content = Stack::new()
        .push(at(text(heading, 18.0, accent), 22.0, 15.0))
        // Visible glyphs are centered between the heading and caption, not in
        // a font line box that includes invisible ascender/descender spacing.
        .push(at(
            CustomPaint::new(CardTime(value)).size(Size::new(rect.width - 120.0, 70.0)),
            22.0,
            42.0,
        ))
        .push(at(text(caption, 18.0, Folio::ink()), 22.0, 120.0))
        .push(at(
            CustomPaint::new(AppIconPainter {
                asset: get_app_icon_asset(id).unwrap(),
                target_size: 58.0,
                pressed: None,
                clock: clock_time,
            })
            .size(Size::new(58.0, 58.0)),
            rect.width - 82.0,
            24.0,
        ));
    ElevatedButton::new(content)
        .style(
            ButtonStyle::new()
                .size(rect.width, rect.height)
                .color(Folio::glass())
                .glass_material(GlassMaterial::DesktopCard)
                .pressed_color(Folio::raised())
                .border_radius(Folio::CARD_RADIUS)
                .antialias(true)
                .padding(EdgeInsets::ZERO),
        )
        .on_pressed(move || {
            let mut s = state.lock().unwrap();
            s.launch_app(id, source);
            s.changed();
        })
}
pub fn build(state: Arc<Mutex<LauncherState>>, size: Size) -> impl Widget {
    let (controller, clock, date, timer, dock, clock_time, estimated) = {
        let mut s = state.lock().unwrap();
        s.total_pages = DESKTOP_ENTRIES.len().div_ceil(DESKTOP_PAGE_CAPACITY).max(1);
        s.current_page = s.page_controller.page().min(s.total_pages - 1);
        if s.page_controller.page() != s.current_page {
            s.page_controller.set_page(s.current_page);
        }
        // During launch, keep the exact pre-launch Dock in the fallback backdrop.
        // Opening an app updates the recent-app order immediately.
        let recent = s
            .app_launch
            .frame(s.monotonic_ms)
            .and_then(|_| s.app_launch.desktop_dock())
            .map(<[String]>::to_vec)
            .unwrap_or_else(|| recent_apps(&s));
        (
            s.page_controller.clone(),
            if s.time_valid {
                s.clock.clone()
            } else {
                "--:--:--".into()
            },
            if s.time_valid {
                s.date.clone()
            } else {
                "待校时".into()
            },
            s.timer.clone(),
            recent,
            s.app_launch
                .frame(s.monotonic_ms)
                .map(|f| f.clock)
                .unwrap_or_else(|| {
                    if s.time_valid {
                        ClockTime::parse(&s.clock)
                    } else {
                        None
                    }
                }),
            s.time_estimated,
        )
    };
    let w = workspace_width(size);
    let half = (w - 16.0) * 0.5;
    let mut root = Stack::new()
        .push(at(
            widget_card(
                state.clone(),
                "clock",
                Rect::from_ltwh(24.0, 24.0, half, 156.0),
                if estimated {
                    "时间待校准"
                } else {
                    "本地时间"
                }
                .into(),
                clock,
                date,
                Folio::ink(),
                clock_time,
            ),
            24.0,
            24.0,
        ))
        .push(at(
            widget_card(
                state.clone(),
                "timer",
                Rect::from_ltwh(40.0 + half, 24.0, half, 156.0),
                if timer.kind == crate::timer::TimerKind::Countdown {
                    "倒计时".into()
                } else {
                    timer.phase.label().into()
                },
                timer.display(),
                crate::pomodoro_ui::status_label(&timer).into(),
                crate::pomodoro_ui::accent(&timer),
                clock_time,
            ),
            40.0 + half,
            24.0,
        ));
    let pages: Vec<_> = DESKTOP_ENTRIES
        .chunks(DESKTOP_PAGE_CAPACITY)
        .map(|entries| {
            let mut page =
                Stack::new().push(Container::new().width(w).height(GRID_ROW_HEIGHT * 2.0));
            for (i, &(id, label)) in entries.iter().enumerate() {
                let s = state.clone();
                let center = icon_center(size, i);
                page = page.push(at(
                    build_app_icon(
                        label,
                        get_app_icon_asset(id).unwrap(),
                        ICON_SIZE,
                        clock_time,
                        move |source| {
                            let mut s = s.lock().unwrap();
                            if id == "screen" {
                                s.queue(UiCommand::Screen(false));
                            } else {
                                s.launch_app(id, source);
                            }
                            s.changed();
                        },
                    ),
                    center.x - 24.0 - (ICON_SIZE + 32.0).max(160.0) * 0.5,
                    center.y - GRID_TOP - ICON_SIZE * 0.5,
                ));
            }
            page
        })
        .collect();
    let changed = state.clone();
    let pages = PageView::new(pages)
        .controller(controller.clone())
        .transition(PageTransition::Slide)
        .raster_cache_key(
            crate::icon_theme::raster_key(),
        )
        .on_page_changed(move |page| {
            let mut s = changed.lock().unwrap();
            s.current_page = page;
            s.changed();
        });
    root = root
        .push(at(
            Container::new()
                .width(w)
                .height(GRID_ROW_HEIGHT * 2.0)
                .child(pages),
            24.0,
            GRID_TOP,
        ))
        .push(at(
            CustomPaint::new(SlidingClock {
                controller,
                clock: clock_time,
            })
            .size(Size::new(w, GRID_ROW_HEIGHT * 2.0)),
            24.0,
            GRID_TOP,
        ))
        .push(at(
            build_status_rail(state.clone()),
            size.width - 116.0,
            24.0,
        ));
    let (page, count) = {
        let s = state.lock().unwrap();
        (s.current_page, s.total_pages)
    };
    if count > 1 {
        for index in 0..count {
            let s = state.clone();
            root = root.push(at(
                ElevatedButton::new(Center::new(
                    Container::new()
                        .width(7.0)
                        .height(7.0)
                        .border_radius(3.5)
                        .color(if index == page {
                            Folio::ink()
                        } else {
                            Folio::muted().with_opacity(0.45)
                        }),
                ))
                .style(
                    ButtonStyle::new()
                        .size(28.0, 28.0)
                        .color(Color::TRANSPARENT)
                        .pressed_color(Folio::pressed(Color::TRANSPARENT))
                        .border_radius(14.0)
                        .antialias(true)
                        .padding(EdgeInsets::ZERO),
                )
                .on_pressed(move || {
                    let mut s = s.lock().unwrap();
                    s.page_controller.set_page(index);
                    s.current_page = index;
                    s.desktop_backdrop.lock().unwrap().take();
                    s.changed();
                }),
                24.0 + w * 0.5 - count as f32 * 14.0 + index as f32 * 28.0,
                566.0,
            ));
        }
    }
    if let Some(layout) = dock_layout(size, dock.len()) {
        root = root.push(at(
            glass(layout.panel.width, layout.panel.height),
            layout.panel.x,
            layout.panel.y,
        ));
        for (i, id) in dock.into_iter().enumerate() {
            let Some(asset) = get_app_icon_asset(&id) else {
                continue;
            };
            let s = state.clone();
            root = root.push(at(
                build_dock_icon(asset, layout.icon_size, clock_time, move |source| {
                    let mut s = s.lock().unwrap();
                    s.launch_app(&id, source);
                    s.changed();
                }),
                size.width - 112.0,
                layout.panel.y + 12.0 + i as f32 * layout.pitch,
            ));
        }
    }
    CustomPaint::new(WallpaperPainter).size(size).child(root)
}

/// Reconstruct just the launch icon patch, including its glass surface. The
/// cached backdrop must not expose a rectangular wallpaper hole in cards/Dock.
pub(crate) fn paint_source_background(
    canvas: &mut Canvas,
    size: Size,
    source: Rect,
    dock_count: usize,
) {
    WallpaperPainter.paint(canvas, size);
    let center = Point::new(
        source.x + source.width * 0.5,
        source.y + source.height * 0.5,
    );
    let half = (workspace_width(size) - 16.0) * 0.5;
    for r in [
        Some(Rect::from_ltwh(24.0, 24.0, half, 156.0)),
        Some(Rect::from_ltwh(40.0 + half, 24.0, half, 156.0)),
        dock_layout(size, dock_count).map(|layout| layout.panel),
    ]
    .into_iter()
    .flatten()
    {
        if r.contains(center) {
            canvas.liquid_glass_material(
                RRect::from_rect_circular(r, Folio::CARD_RADIUS),
                Folio::glass(),
                false,
                if r.y == 24.0 {
                    GlassMaterial::DesktopCard
                } else {
                    GlassMaterial::Desktop
                },
            );
        }
    }
}

struct SlidingClock {
    controller: PageController,
    clock: Option<ClockTime>,
}
impl CustomPainter for SlidingClock {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let drag = self.controller.drag_offset();
        let key = crate::icon_theme::raster_key();
        if drag.abs() < 0.01 || !self.controller.has_cached_pages(key, size) {
            return;
        }
        let page = self.controller.page();
        let shift = if page == 0 {
            drag
        } else if page == 1 && drag > 0.0 {
            drag - size.width
        } else {
            return;
        };
        canvas.save();
        canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height));
        crate::live_clock_icon::paint_clock_hands(
            canvas,
            Rect::from_ltwh(
                size.width / 8.0 - ICON_SIZE * 0.5 + shift,
                0.0,
                ICON_SIZE,
                ICON_SIZE,
            ),
            self.clock,
            1.0,
        );
        canvas.restore();
    }
}
