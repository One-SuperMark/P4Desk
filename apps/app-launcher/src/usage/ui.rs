//! Landscape monitor: Mac headline, quota strips, rankings and on-demand details.
use super::*;
use crate::{LauncherState, UiCommand};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;
type Shared = Arc<Mutex<LauncherState>>;
#[path = "headline_view.rs"]
mod headline_view;
#[path = "live_metrics.rs"]
mod live_metrics;
#[path = "trend_view.rs"]
mod trend_view;
#[path = "visuals.rs"]
mod visuals;
fn at(w: impl Widget + 'static, x: f32, y: f32) -> Positioned {
    Positioned::new(w).left(x).top(y)
}
fn text(s: impl Into<String>, size: f32, color: Color) -> Text {
    Text::new(s).font_size(size).color(color)
}
fn user_text(s: impl Into<String>, size: f32, color: Color) -> Text {
    text(s, size, color).font(Font::dynamic_font().clone())
}
fn short(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!(
            "{}…",
            s.chars().take(n.saturating_sub(1)).collect::<String>()
        )
    } else {
        s.into()
    }
}
fn fit(s: &str, size: f32, width: f32) -> String {
    fit_with_font(s, size, width, Font::default_font())
}
fn fit_with_font(s: &str, size: f32, width: f32, font: &Font) -> String {
    let mut result = String::new();
    let mut used = 0.;
    for ch in s.chars() {
        let next = font.advance(ch, size);
        if used + next > width {
            let ellipsis = font.advance('…', size);
            while !result.is_empty() && used + ellipsis > width {
                used -= font.advance(result.pop().unwrap(), size);
            }
            result.push('…');
            return result;
        }
        used += next;
        result.push(ch);
    }
    result
}
fn grouped(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
// Place amounts by their measured width, not by the length of a mixed label.
fn right_text(
    s: impl Into<String>,
    size: f32,
    color: Color,
    right: f32,
    y: f32,
    width: f32,
) -> Positioned {
    let label = fit(&s.into(), size, width);
    let measured = Font::default_font().measure_text(&label, size).width;
    at(text(label, size, color), right - measured, y)
}
fn money(value: f64) -> String {
    if value > 0.0 && value < 1.0 {
        format!("${value:.4}")
    } else {
        format!("${value:.2}")
    }
}
fn inset() -> Color {
    Folio::pick(Color::from_hex(0x23272a), Color::from_hex(0xf2f5f8))
}
fn selected_fill() -> Color {
    Folio::pick(Color::from_hex(0x26363b), Color::from_hex(0xe6f0f3))
}
fn badge(label: &str, color: Color) -> Container {
    let width = Font::default_font().measure_text(label, 14.).width + 20.;
    Container::new()
        .width(width)
        .height(26.)
        .color(color.with_opacity(0.13))
        .border_radius(7.)
        .child(Center::new(text(label, 14., color)))
}
fn quota_color(remaining: Option<f64>) -> Color {
    match remaining {
        Some(v) if v <= 10. => Folio::red(),
        Some(v) if v <= 25. => Folio::orange(),
        Some(_) => Folio::green(),
        None => Folio::muted(),
    }
}
fn adaptive_number(label: String, maximum: f32, width: f32) -> Text {
    let font = Font::default_font();
    let mut size = maximum;
    while size > 14. && font.measure_text(&label, size).width > width {
        size -= 2.;
    }
    text(label, size, Folio::ink())
}
fn empty_card(w: f32, h: f32, page: Page, title: &str, hint: &str) -> Container {
    panel(w, h).child(
        Stack::new()
            .push(at(
                Container::new()
                    .width(48.)
                    .height(48.)
                    .color(inset())
                    .border_radius(14.)
                    .child(Center::new(
                        CustomPaint::new(NavGlyph(page, Folio::muted())).size(Size::new(24., 24.)),
                    )),
                w / 2. - 24.,
                h / 2. - 66.,
            ))
            .push(at(
                Container::new()
                    .width(w - 48.)
                    .height(28.)
                    .child(Center::new(text(title, 22., Folio::ink()))),
                24.,
                h / 2.,
            ))
            .push(at(
                Container::new()
                    .width(w - 48.)
                    .height(22.)
                    .child(Center::new(text(hint, 14., Folio::muted()))),
                24.,
                h / 2. + 40.,
            )),
    )
}
fn edit(s: &Shared, f: impl FnOnce(&mut State)) {
    let mut s = s.lock().unwrap();
    f(&mut s.usage);
    s.changed();
}
fn button(
    label: impl Into<String>,
    w: f32,
    h: f32,
    on: bool,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    let fill = if on { Folio::accent() } else { inset() };
    ElevatedButton::new(text(label, 18.0, Folio::on_fill(fill)))
        .style(
            Folio::control_style(w, h, fill)
                .border_radius((h * 0.25).min(10.0))
                .padding(EdgeInsets::symmetric(2.0, 6.0)),
        )
        .on_pressed(f)
}
fn panel(w: f32, h: f32) -> Container {
    Container::new()
        .width(w)
        .height(h)
        .color(Folio::surface())
        .border_radius(18.0)
}
fn control_group(w: f32, h: f32) -> Container {
    panel(w, h).color(inset()).border_radius(12.0)
}
fn rule(w: f32) -> Container {
    Container::new()
        .width(w)
        .height(1.0)
        .color(Folio::line().with_opacity(0.55))
}
fn bar(w: f32, h: f32, ratio: f64, color: Color) -> Stack {
    let mut s = Stack::new().push(
        Container::new()
            .width(w)
            .height(h)
            .color(Folio::raised())
            .border_radius(h * 0.5),
    );
    let fill = w * ratio.clamp(0.0, 1.0) as f32;
    if fill > 0.0 {
        s = s.push(
            Container::new()
                .width(fill)
                .height(h)
                .color(color)
                .border_radius(h * 0.5),
        );
    }
    s
}
fn metric(label: &str, value: String, w: f32) -> Container {
    Container::new().width(w).height(58.0).child(
        Stack::new()
            .push(text(label, 14.0, Folio::muted()))
            .push(at(
                text(fit(&value, 22.0, w - 8.0), 22.0, Folio::ink()),
                0.0,
                24.0,
            )),
    )
}
fn stroke(c: &mut Canvas, points: &[(f32, f32)], color: Color, width: f32) {
    let mut p = tiny_gfx::PathBuilder::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            p.move_to(x, y);
        } else {
            p.line_to(x, y);
        }
    }
    if let Some(p) = p.finish() {
        c.stroke_path(
            &p,
            &tiny_gfx::Paint::new(color.to_gfx()),
            &tiny_gfx::Stroke {
                width,
                line_cap: tiny_gfx::LineCap::Round,
                line_join: tiny_gfx::LineJoin::Round,
                ..Default::default()
            },
        );
    }
}
fn trend_path(points: &[(f32, f32)]) -> tiny_gfx::PathBuilder {
    super::trend::curve(points)
}
struct NavGlyph(Page, Color);
impl CustomPainter for NavGlyph {
    fn paint(&self, c: &mut Canvas, _: Size) {
        let col = self.1;
        match self.0 {
            Page::Overview => {
                stroke(c, &[(2., 3.), (2., 20.), (22., 20.)], col, 1.6);
                stroke(c, &[(5., 15.), (10., 9.), (15., 12.), (21., 4.)], col, 2.);
            }
            Page::Accounts => {
                for y in [3., 10., 17.] {
                    c.draw_rrect_aa(
                        RRect::from_rect_and_radius(
                            Rect::from_ltwh(2., y, 20., 4.),
                            Radius::circular(2.),
                        ),
                        col,
                    );
                }
            }
            Page::Models => {
                for x in [2., 13.] {
                    for y in [2., 13.] {
                        c.draw_rrect_stroke(
                            RRect::from_rect_and_radius(
                                Rect::from_ltwh(x, y, 8., 8.),
                                Radius::circular(2.),
                            ),
                            col,
                            1.6,
                        );
                    }
                }
            }
            Page::Users => {
                c.paint_circle(
                    tiny_flutter::Point::new(9., 6.),
                    3.5,
                    &tiny_gfx::Paint::new(col.to_gfx()),
                    Some(1.6),
                );
                stroke(
                    c,
                    &[
                        (2., 21.),
                        (2., 17.),
                        (5., 13.),
                        (13., 13.),
                        (16., 17.),
                        (16., 21.),
                    ],
                    col,
                    1.6,
                );
                stroke(c, &[(18., 4.), (21., 7.), (18., 10.)], col, 1.6);
                stroke(c, &[(19., 14.), (22., 18.), (22., 21.)], col, 1.6);
            }
            Page::Connection => {
                let mut points = Vec::with_capacity(33);
                for i in 0..=32 {
                    let a = i as f32 * std::f32::consts::TAU / 32.0;
                    let r = if i % 4 == 0 || i % 4 == 3 { 10.5 } else { 8.0 };
                    points.push((12.0 + a.cos() * r, 12.0 + a.sin() * r));
                }
                stroke(c, &points, col, 1.6);
                c.paint_circle(
                    tiny_flutter::Point::new(12.0, 12.0),
                    3.0,
                    &tiny_gfx::Paint::new(col.to_gfx()),
                    Some(1.6),
                );
            }
        }
    }
}
fn navigation(state: Shared, page: Page) -> Stack {
    let mut s = Stack::new().push(control_group(452., 42.));
    for (i, p) in [Page::Overview, Page::Accounts, Page::Models, Page::Users]
        .into_iter()
        .enumerate()
    {
        let fill = if page == p { Folio::accent() } else { inset() };
        let col = if page == p {
            Folio::on_fill(fill)
        } else {
            Folio::muted()
        };
        let st = state.clone();
        let child = Stack::new()
            .push(at(
                CustomPaint::new(NavGlyph(p, col)).size(Size::new(24., 24.)),
                10.,
                7.,
            ))
            .push(at(
                text(
                    if p == Page::Accounts {
                        "额度"
                    } else {
                        p.label()
                    },
                    18.,
                    col,
                ),
                44.,
                7.,
            ));
        s = s.push(at(
            ElevatedButton::new(child)
                .style(
                    Folio::control_style(108., 36., fill)
                        .padding(EdgeInsets::ZERO)
                        .border_radius(9.),
                )
                .on_pressed(move || edit(&st, |u| u.navigate(p, u.period, None))),
            4. + i as f32 * 112.,
            3.,
        ));
    }
    s
}
pub fn build(state: Shared, size: Size) -> Stack {
    let s = state.lock().unwrap();
    let u = &s.usage;
    let (page, period, detail, configured, data, status, busy, stale, index, now) = (
        u.page,
        u.period,
        u.detail,
        u.config.is_some(),
        u.data.clone(),
        u.status.clone(),
        u.busy,
        u.stale,
        u.list_page,
        s.unix_ms,
    );
    drop(s);
    let mut root = Stack::new().push(navigation(state.clone(), page));
    let st = state.clone();
    root = root.push(at(
        ElevatedButton::new(
            CustomPaint::new(NavGlyph(
                Page::Connection,
                if page == Page::Connection {
                    Folio::on_fill(Folio::accent())
                } else {
                    Folio::muted()
                },
            ))
            .size(Size::new(24.0, 24.0)),
        )
        .style(
            Folio::control_style(
                42.,
                42.,
                if page == Page::Connection {
                    Folio::accent()
                } else {
                    Folio::surface()
                },
            )
            .padding(EdgeInsets::ZERO)
            .border_radius(11.0),
        )
        .on_pressed(move || edit(&st, State::edit_connection)),
        464.,
        0.,
    ));
    if page == Page::Connection || !configured {
        return root.push(at(
            connection(state, size.width, size.height - 58.),
            0.,
            58.,
        ));
    }
    let range_x = size.width - 308.;
    root = root.push(at(control_group(218., 42.), range_x, 0.));
    for (i, p) in [Period::Day, Period::Month, Period::Total]
        .into_iter()
        .enumerate()
    {
        let st = state.clone();
        root = root.push(at(
            button(p.label(), 68., 36., period == p, move || {
                edit(&st, |u| u.navigate(u.page, p, u.detail))
            }),
            range_x + 4. + i as f32 * 71.,
            3.,
        ));
    }
    let st = state.clone();
    root = root.push(at(
        button(
            if busy { "刷新中" } else { "刷新" },
            78.,
            42.,
            false,
            move || {
                let mut s = st.lock().unwrap();
                if !s.usage.busy {
                    s.queue(UiCommand::Usage(Command::Refresh));
                    s.changed();
                }
            },
        ),
        size.width - 78.,
        0.,
    ));
    let partial = data
        .as_ref()
        .is_some_and(|d| d.warning.as_deref() == Some(status.as_str()));
    let h = size.height - 100.;
    if let Some(d) = data {
        let body = if detail.is_some() || page == Page::Overview {
            overview(state.clone(), &d, size.width, h, period, detail, now)
        } else if page == Page::Accounts {
            account_list(state.clone(), &d, size.width, h, index, now)
        } else if page == Page::Models {
            model_list(state.clone(), &d, size.width, h, index)
        } else {
            user_list(state.clone(), &d, size.width, h, index, period)
        };
        root = root.push(at(body, 0., 58.));
        if detail.is_some() {
            let st = state.clone();
            root = root.push(at(
                button("‹ 返回列表", 128., 32., false, move || {
                    edit(&st, |u| u.navigate(u.page, u.period, None))
                }),
                size.width - 128.,
                size.height - 32.,
            ));
        } else if page != Page::Overview {
            let per = if page == Page::Accounts { 2 } else { 5 };
            let count = match page {
                Page::Accounts => d.accounts.len(),
                Page::Models => d.models.len(),
                _ => d.users.len(),
            };
            let pages = count.div_ceil(per).max(1);
            if pages > 1 {
                let st = state.clone();
                root = root.push(at(
                    button("‹", 36., 32., false, move || {
                        edit(&st, |u| {
                            u.list_page = u.list_page.saturating_sub(1);
                            u.model_selected = None;
                        })
                    }),
                    size.width - 158.,
                    size.height - 32.,
                ));
                root = root.push(at(
                    text(format!("{} / {}", index + 1, pages), 16., Folio::muted()),
                    size.width - 107.,
                    size.height - 27.,
                ));
                let st = state.clone();
                root = root.push(at(
                    button("›", 36., 32., false, move || {
                        edit(&st, |u| {
                            u.list_page = (u.list_page + 1).min(pages - 1);
                            u.model_selected = None;
                        })
                    }),
                    size.width - 36.,
                    size.height - 32.,
                ));
            }
        }
    } else {
        root = root.push(at(
            empty_card(
                size.width,
                h,
                page,
                if busy {
                    "正在获取用量…"
                } else {
                    "暂无数据"
                },
                if busy {
                    "首次加载后会自动更新"
                } else {
                    "检查 Wi-Fi 与连接设置，或点击刷新"
                },
            ),
            0.,
            58.,
        ));
    }
    let caption = if partial {
        format!("部分未更新 · {}", short(&status, 44))
    } else if stale {
        format!("上次数据 · {}", short(&status, 45))
    } else {
        short(&status, 49)
    };
    if page == Page::Overview && period == Period::Day && detail.is_none() {
        root.push(at(
            live_metrics::LiveText::status(state, size.width - 8.),
            4.,
            size.height - 27.,
        ))
    } else {
        root.push(at(
            text(
                caption,
                14.,
                if stale {
                    Folio::orange()
                } else {
                    Folio::muted()
                },
            ),
            4.,
            size.height - 27.,
        ))
    }
}
fn overview(
    state: Shared,
    d: &Arc<Data>,
    w: f32,
    h: f32,
    period: Period,
    detail: Option<i64>,
    now: i64,
) -> Stack {
    let lw = (w - 16.) * 0.60;
    let rw = w - lw - 16.;
    let title = if let Some(id) = detail {
        let label = {
            let locked = state.lock().unwrap();
            if locked.usage.page == Page::Users {
                locked.usage.user_display_name(id)
            } else {
                format!("账号 #{id}")
            }
        };
        format!(
            "{} · {} TOKENS",
            fit_with_font(&label, 14., lw - 170., Font::dynamic_font()),
            period.label()
        )
    } else {
        "TOTAL TOKENS".into()
    };
    let title = if detail.is_some() {
        user_text(title, 14., Folio::muted())
    } else {
        text(title, 14., Folio::muted())
    };
    let hero = Stack::new()
        .push(at(title, 22., 22.))
        .push(at(
            headline_view::HeadlineView::new(
                state.clone(),
                lw - 44.0,
                headline_view::NUMBER_HEIGHT,
            ),
            22.,
            59.,
        ))
        .push(at(
            live_metrics::LiveText::cost(state.clone(), lw - 44.),
            22.,
            172.,
        ));
    let mut root = Stack::new().push(panel(lw, 208.).child(hero)).push(at(
        chart(state.clone(), d, lw, h - 222.),
        0.,
        222.,
    ));
    if detail.is_none() {
        let quota = if let Some(a) = d.accounts.first() {
            Stack::new()
                .push(at(text("账号额度", 18., Folio::ink()), 18., 14.))
                .push(at(text("查看全部  ›", 14., Folio::muted()), rw - 114., 17.))
                .push(at(
                    text(
                        fit(
                            &format!("{}  {}", platform(&a.platform), a.plan_label),
                            16.,
                            rw - 74.,
                        ),
                        16.,
                        Folio::ink(),
                    ),
                    52.,
                    45.,
                ))
                .push(at(visuals::provider(&a.platform, 24.), 18., 44.))
                .push(at(
                    compact_quota(&a.five, "Session · 5h", rw - 36., now, 300),
                    18.,
                    76.,
                ))
                .push(at(
                    compact_quota(&a.seven, "Weekly · 7d", rw - 36., now, 10080),
                    18.,
                    127.,
                ))
                .push(at(
                    text(
                        a.seven
                            .as_ref()
                            .map(|v| v.caption(now))
                            .unwrap_or("重置时间未知".into()),
                        14.,
                        Folio::muted(),
                    ),
                    18.,
                    177.,
                ))
        } else {
            Stack::new()
                .push(at(text("账号额度", 18., Folio::ink()), 18., 14.))
                .push(at(text("暂无可用额度", 18., Folio::muted()), 18., 78.))
        };
        let st = state.clone();
        root = root.push(at(
            GestureDetector::new(panel(rw, 208.).child(quota))
                .on_tap(move || edit(&st, |u| u.navigate(Page::Accounts, u.period, None))),
            lw + 16.,
            0.,
        ));
    } else {
        let mut summary = Stack::new().push(at(text("模型构成", 18., Folio::ink()), 18., 14.));
        let selected = state
            .lock()
            .unwrap()
            .usage
            .model_selected
            .unwrap_or(0)
            .min(d.models.len().saturating_sub(1));
        if let Some(m) = d.models.get(selected) {
            summary = summary
                .push(at(text(short(&m.name, 24), 18., Folio::blue()), 18., 46.))
                .push(at(composition(m, rw - 36.), 18., 80.));
        }
        root = root.push(at(panel(rw, 208.).child(summary), lw + 16., 0.));
    }
    root.push(at(
        model_ranking(state, d, rw, h - 220., detail.is_some()),
        lw + 16.,
        220.,
    ))
}
fn platform(s: &str) -> &str {
    match s {
        "openai" => "Codex",
        "anthropic" => "Claude",
        _ => s,
    }
}
fn compact_quota(window: &Option<Window>, label: &str, w: f32, now: i64, minutes: u32) -> Stack {
    let remaining = window.as_ref().map(Window::remaining);
    let mut row = Stack::new()
        .push(text(label, 14., Folio::muted()))
        .push(right_text(
            remaining
                .map(|v| format!("{v:.0}% 剩余"))
                .unwrap_or("未提供".into()),
            14.,
            quota_color(remaining),
            w,
            0.,
            100.,
        ))
        .push(at(
            bar(
                w,
                5.,
                remaining.unwrap_or(0.) / 100.,
                quota_color(remaining),
            ),
            0.,
            25.,
        ));
    if let Some(time) = window
        .as_ref()
        // Static quota chrome follows the monitor's minute wall-clock rebuild,
        // rather than the independent five-second headline sampling cadence.
        .and_then(|v| v.remaining_time_ratio(now.div_euclid(60_000) * 60_000, minutes))
    {
        row = row.push(at(
            bar(w, 2., time, Folio::blue().with_opacity(0.65)),
            0.,
            34.,
        ));
    }
    row
}

fn model_ranking(state: Shared, d: &Data, w: f32, h: f32, detail: bool) -> Container {
    let mut card = Stack::new().push(at(text("模型 TOP", 18., Folio::ink()), 18., 10.));
    if !detail {
        card = card.push(right_text(
            "查看全部  ›",
            14.,
            Folio::muted(),
            w - 18.,
            13.,
            106.,
        ));
    }
    let sum = d.models.iter().map(|m| m.totals.tokens as f64).sum::<f64>();
    if d.models.is_empty() {
        card = card.push(at(
            text("当前范围暂无模型用量", 16., Folio::muted()),
            18.,
            72.,
        ));
    }
    let first = if detail {
        state.lock().unwrap().usage.list_page * 3
    } else {
        0
    };
    if detail && d.models.len() > 3 {
        let pages = d.models.len().div_ceil(3);
        for (forward, x) in [(false, w - 88.), (true, w - 44.)] {
            let st = state.clone();
            card = card.push(at(
                button(
                    if forward { "›" } else { "‹" },
                    34.,
                    30.,
                    false,
                    move || {
                        edit(&st, |u| {
                            u.list_page = if forward {
                                (u.list_page + 1).min(pages - 1)
                            } else {
                                u.list_page.saturating_sub(1)
                            };
                            u.model_selected = Some(u.list_page * 3);
                        })
                    },
                ),
                x,
                4.,
            ));
        }
    }
    for (local, m) in d.models.iter().skip(first).take(3).enumerate() {
        let index = first + local;
        let st = state.clone();
        let ratio = if sum > 0. {
            m.totals.tokens as f64 / sum
        } else {
            0.
        };
        let color = visuals::identity_color(&m.name);
        let row = Stack::new()
            .push(at(
                Container::new()
                    .width(5.)
                    .height(5.)
                    .color(color)
                    .border_radius(2.5),
                0.,
                9.,
            ))
            .push(at(
                text(fit(&m.name, 16., w - 163.), 16., Folio::ink()),
                13.,
                0.,
            ))
            .push(right_text(
                tokens(m.totals.tokens),
                14.,
                Folio::ink(),
                w - 90.,
                1.,
                76.,
            ))
            .push(right_text(
                format!("{:.0}%", ratio * 100.),
                14.,
                Folio::muted(),
                w - 36.,
                1.,
                48.,
            ))
            .push(at(bar(w - 36., 4., ratio, color), 0., 26.));
        card = card.push(at(
            GestureDetector::new(Container::new().width(w - 36.).height(41.).child(row)).on_tap(
                move || {
                    edit(&st, |u| {
                        if !detail {
                            u.navigate(Page::Models, u.period, None);
                        }
                        u.model_selected = Some(index);
                    });
                },
            ),
            18.,
            42. + local as f32 * 44.,
        ));
    }
    panel(w, h).child(card)
}
fn chart(state: Shared, d: &Arc<Data>, w: f32, h: f32) -> trend_view::LiveTrendCard {
    trend_view::LiveTrendCard::new(state, d.clone(), Size::new(w, h))
}
fn chart_snapshot(state: Shared, d: &Data, w: f32, h: f32) -> Container {
    let (selected, axis) = {
        let state = state.lock().unwrap();
        (
            state.usage.chart_selected,
            trend_view::TrendAxis::read(&state.usage),
        )
    };
    let index = selected
        .unwrap_or(d.trend.len().saturating_sub(1))
        .min(d.trend.len().saturating_sub(1));
    let comparison = d
        .yesterday_trend
        .iter()
        .take(if axis == trend_view::TrendAxis::Day {
            d.yesterday_trend.len()
        } else {
            d.trend.len()
        })
        .cloned()
        .collect::<Vec<_>>();
    let has_previous = !comparison.is_empty();
    let mut p = Stack::new().push(at(
        text(
            if has_previous {
                "全站 Token 趋势"
            } else {
                &d.trend_label
            },
            18.0,
            Folio::ink(),
        ),
        18.0,
        12.0,
    ));
    p = p.push(at(
        text(
            if has_previous { "今日" } else { "Token" },
            14.0,
            Folio::blue(),
        ),
        w - 130.0,
        16.0,
    ));
    p = p.push(at(
        Container::new()
            .width(10.)
            .height(3.)
            .color(Folio::blue())
            .border_radius(1.5),
        w - 148.,
        26.,
    ));
    if has_previous {
        p = p
            .push(at(
                Container::new()
                    .width(10.)
                    .height(3.)
                    .color(Folio::muted())
                    .border_radius(1.5),
                w - 80.,
                26.,
            ))
            .push(at(text("昨日", 14., Folio::muted()), w - 62., 16.));
    }
    let peak = d
        .trend
        .iter()
        .chain(comparison.iter())
        .map(|p| p.totals.tokens)
        .max()
        .unwrap_or(0);
    p = p.push(at(
        text(
            if d.trend.is_empty() && comparison.is_empty() {
                "暂无完整趋势记录".into()
            } else {
                format!("峰值 {}", tokens(peak))
            },
            14.,
            Folio::muted(),
        ),
        18.,
        42.,
    ));
    p = p.push(at(
        trend_view::InteractiveTrend {
            state: state.clone(),
            points: Arc::new(d.trend.clone()),
            previous: Arc::new(comparison),
            selected: index,
            size: Size::new(w - 36., h - 113.),
        },
        18.,
        64.,
    ));
    if axis == trend_view::TrendAxis::Day {
        for (i, label) in ["00:00", "06:00", "12:00", "18:00", "24:00"]
            .into_iter()
            .enumerate()
        {
            let label_width = Font::default_font().measure_text(label, 14.).width;
            let x = match i {
                0 => 18.,
                4 => w - 18. - label_width,
                _ => 20. + i as f32 / 4. * (w - 40.) - label_width / 2.,
            };
            p = p.push(at(text(label, 14., Folio::muted()), x, h - 45.));
        }
    } else if let (Some(first), Some(last)) = (d.trend.first(), d.trend.last()) {
        p = p
            .push(at(
                text(axis_label(&first.date), 14., Folio::muted()),
                18.,
                h - 45.,
            ))
            .push(at(
                text(axis_label(&last.date), 14., Folio::muted()),
                w - 70.,
                h - 45.,
            ));
    }
    if let Some(v) = d.trend.get(index) {
        p = p.push(at(
            text(
                fit(
                    &format!(
                        "{}   {} Token   {}",
                        axis_label(&v.date),
                        tokens(v.totals.tokens),
                        money(v.totals.cost)
                    ),
                    14.,
                    w - if has_previous { 164. } else { 36. },
                ),
                14.,
                Folio::ink(),
            ),
            18.,
            h - 23.,
        ));
        let previous = if axis == trend_view::TrendAxis::Day {
            trend_view::minute_of_day(&v.date).and_then(|minute| {
                d.yesterday_trend
                    .iter()
                    .find(|p| trend_view::minute_of_day(&p.date) == Some(minute))
            })
        } else {
            d.yesterday_trend.get(index)
        };
        if let Some(y) = previous {
            p = p.push(right_text(
                format!("昨日 {}", tokens(y.totals.tokens)),
                14.,
                Folio::muted(),
                w - 18.,
                h - 23.,
                132.,
            ));
        }
    }
    panel(w, h).child(p)
}
fn axis_label(date: &str) -> String {
    if date.len() >= 16 && matches!(date.as_bytes().get(10), Some(b' ') | Some(b'T')) {
        date.get(11..16).unwrap_or(date).into()
    } else if date.len() >= 10 {
        date.get(5..10).unwrap_or(date).replace('-', "/")
    } else {
        date.into()
    }
}
fn quota_window(window: &Option<Window>, label: &str, w: f32, now: i64, minutes: u32) -> Stack {
    let remaining = window.as_ref().map(Window::remaining);
    let mut row = Stack::new()
        .push(text(label, 14., Folio::muted()))
        .push(at(
            text(
                remaining.map(|v| format!("{v:.0}%")).unwrap_or("—".into()),
                34.,
                Folio::ink(),
            ),
            0.,
            26.,
        ))
        .push(at(
            text(
                if remaining.is_some() {
                    "剩余额度"
                } else {
                    "未提供"
                },
                14.,
                Folio::muted(),
            ),
            96.,
            43.,
        ))
        .push(at(
            bar(
                w,
                7.,
                remaining.unwrap_or(0.) / 100.,
                quota_color(remaining),
            ),
            0.,
            75.,
        ))
        .push(at(
            text(
                fit(
                    &window
                        .as_ref()
                        .map(|v| v.caption(now))
                        .unwrap_or("重置时间未知".into()),
                    14.,
                    w - 48.,
                ),
                14.,
                Folio::muted(),
            ),
            0.,
            95.,
        ));
    if let Some(time) = window
        .as_ref()
        .and_then(|v| v.remaining_time_ratio(now, minutes))
    {
        row = row
            .push(right_text(
                format!("{:.0}%", time * 100.),
                14.,
                Folio::muted(),
                w,
                95.,
                46.,
            ))
            .push(at(
                bar(w, 2., time, Folio::blue().with_opacity(0.65)),
                0.,
                122.,
            ));
    }
    row
}

fn account_list(state: Shared, d: &Data, w: f32, h: f32, index: usize, now: i64) -> Stack {
    let mut root = Stack::new();
    if d.accounts.is_empty() {
        return root.push(empty_card(
            w,
            h,
            Page::Accounts,
            "暂无账号",
            "当前范围没有可显示的账号额度",
        ));
    }
    let visible = d
        .accounts
        .iter()
        .skip(index * 2)
        .take(2)
        .collect::<Vec<_>>();
    let single = visible.len() == 1;
    let cw = if single { w } else { (w - 16.) / 2. };
    for (i, a) in visible.into_iter().enumerate() {
        let st = state.clone();
        let id = a.id;
        let plan = fit(&a.plan_label, 14., 128.);
        let plan_w = Font::default_font().measure_text(&plan, 14.).width + 20.;
        let mut card = Stack::new()
            .push(at(visuals::provider(&a.platform, 34.), 22., 21.))
            .push(at(
                text(
                    fit(platform(&a.platform), 24., cw - plan_w - 110.),
                    24.,
                    Folio::ink(),
                ),
                68.,
                17.,
            ))
            .push(at(
                badge(&plan, Folio::accent_ink()),
                cw - 22. - plan_w,
                21.,
            ))
            .push(at(
                text(fit(&a.label, 14., cw - 92.), 14., Folio::muted()),
                68.,
                51.,
            ))
            .push(at(rule(cw - 44.), 22., 82.));
        if single {
            let qw = (cw - 60.) / 2.;
            for (x, window, label, minutes) in [
                (22., &a.five, "Session · 5 小时", 300),
                (cw / 2. + 8., &a.seven, "Weekly · 7 天", 10080),
            ] {
                card = card.push(at(
                    Container::new()
                        .width(qw)
                        .height(194.)
                        .color(inset())
                        .border_radius(12.)
                        .child(Stack::new().push(at(
                            quota_window(window, label, qw - 36., now, minutes),
                            18.,
                            22.,
                        ))),
                    x,
                    104.,
                ));
            }
            card = card.push(at(
                text(
                    "绿色：剩余额度    蓝色：距离重置的时间",
                    14.,
                    Folio::muted(),
                ),
                22.,
                320.,
            ));
        } else {
            card = card
                .push(at(
                    quota_window(&a.five, "Session · 5 小时", cw - 44., now, 300),
                    22.,
                    104.,
                ))
                .push(at(rule(cw - 44.), 22., 244.))
                .push(at(
                    quota_window(&a.seven, "Weekly · 7 天", cw - 44., now, 10080),
                    22.,
                    256.,
                ));
        }
        let updated = a
            .usage_updated_ms
            .map(|ms| {
                let tm = chrono::DateTime::from_timestamp_millis(ms.saturating_add(
                    state.lock().unwrap().settings.timezone_minutes as i64 * 60_000,
                ));
                tm.map(|v| format!("更新于 {}", v.format("%H:%M")))
                    .unwrap_or("额度尚未更新".into())
            })
            .unwrap_or("额度尚未更新".into());
        if single {
            card = card
                .push(at(text(updated, 14., Folio::muted()), 22., h - 42.))
                .push(right_text(
                    "查看模型与趋势  ›",
                    16.,
                    Folio::accent_ink(),
                    cw - 22.,
                    h - 44.,
                    216.,
                ));
        }
        root = root.push(at(
            GestureDetector::new(panel(cw, h).child(card))
                .on_tap(move || edit(&st, |u| u.navigate(Page::Accounts, u.period, Some(id)))),
            i as f32 * (cw + 16.),
            0.,
        ));
    }
    root
}

fn composition(m: &Model, w: f32) -> Stack {
    let sum = m.input as f64 + m.cache as f64 + m.output as f64;
    let mut rows = Stack::new().push(visuals::composition_bar(m.input, m.cache, m.output, w));
    for (i, (label, value, color)) in [
        ("输入", m.input, Folio::blue()),
        ("缓存读取", m.cache, Folio::green()),
        ("输出", m.output, Folio::orange()),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 24. + i as f32 * 30.;
        rows = rows
            .push(at(
                Container::new()
                    .width(6.)
                    .height(6.)
                    .color(color)
                    .border_radius(3.),
                0.,
                y + 8.,
            ))
            .push(at(text(label, 14., Folio::muted()), 14., y))
            .push(right_text(
                tokens(value),
                16.,
                Folio::ink(),
                w - 62.,
                y - 1.,
                w - 152.,
            ))
            .push(right_text(
                if sum > 0. {
                    format!("{:.0}%", value as f64 / sum * 100.)
                } else {
                    "—".into()
                },
                14.,
                Folio::muted(),
                w,
                y,
                52.,
            ));
    }
    rows
}

fn model_list(state: Shared, d: &Data, w: f32, h: f32, index: usize) -> Stack {
    if d.models.is_empty() {
        return Stack::new().push(empty_card(
            w,
            h,
            Page::Models,
            "暂无模型用量",
            "切换时间范围或等待下一次更新",
        ));
    }
    let lw = (w - 16.) * 0.49;
    let rw = w - lw - 16.;
    let first = index * 5;
    let selected = state
        .lock()
        .unwrap()
        .usage
        .model_selected
        .unwrap_or(first)
        .min(d.models.len() - 1);
    let max = d
        .models
        .iter()
        .map(|m| m.totals.tokens)
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    let mut list = Stack::new()
        .push(at(text("模型排名", 18., Folio::ink()), 18., 13.))
        .push(right_text(
            format!("{} 个模型", d.models.len()),
            14.,
            Folio::muted(),
            lw - 18.,
            17.,
            120.,
        ));
    for (i, m) in d.models.iter().skip(first).take(5).enumerate() {
        let n = first + i;
        let st = state.clone();
        let color = visuals::identity_color(&m.name);
        let fill = if n == selected {
            selected_fill()
        } else {
            Folio::surface()
        };
        let mut child = Stack::new()
            .push(at(
                text(
                    format!("{:02}", n + 1),
                    14.,
                    if n == selected { color } else { Folio::muted() },
                ),
                12.,
                13.,
            ))
            .push(at(
                text(fit(&m.name, 18., lw - 186.), 18., Folio::ink()),
                42.,
                6.,
            ))
            .push(right_text(
                tokens(m.totals.tokens),
                16.,
                Folio::ink(),
                lw - 34.,
                7.,
                102.,
            ))
            .push(at(text("Token", 14., Folio::muted()), 42., 31.))
            .push(right_text(
                money(m.totals.cost),
                14.,
                Folio::muted(),
                lw - 34.,
                32.,
                112.,
            ))
            .push(at(
                bar(lw - 82., 3., m.totals.tokens as f64 / max, color),
                42.,
                55.,
            ));
        if n == selected {
            child = child.push(at(
                Container::new()
                    .width(3.)
                    .height(28.)
                    .color(color)
                    .border_radius(1.5),
                0.,
                16.,
            ));
        }
        list = list.push(at(
            ElevatedButton::new(child)
                .style(
                    Folio::control_style(lw - 20., 62., fill)
                        .padding(EdgeInsets::ZERO)
                        .border_radius(9.),
                )
                .on_pressed(move || edit(&st, |u| u.model_selected = Some(n))),
            10.,
            47. + i as f32 * 68.,
        ));
    }
    let m = &d.models[selected];
    let base = m.input as f64 + m.cache as f64;
    let hit = if base > 0. {
        format!("{:.1}%", m.cache as f64 / base * 100.)
    } else {
        "—".into()
    };
    let name_color = visuals::identity_color(&m.name);
    let mut detail = Stack::new()
        .push(at(text("模型详情", 14., Folio::muted()), 22., 17.))
        .push(at(visuals::avatar(&m.name, 30., name_color), 22., 46.))
        .push(at(
            text(fit(&m.name, 22., rw - 86.), 22., Folio::ink()),
            64.,
            46.,
        ))
        .push(at(
            adaptive_number(grouped(m.totals.tokens), 36., rw - 44.),
            22.,
            92.,
        ))
        .push(at(
            text(
                format!("TOTAL TOKENS   ≈ {}", tokens(m.totals.tokens)),
                14.,
                Folio::muted(),
            ),
            22.,
            142.,
        ));
    let mw = (rw - 56.) / 2.;
    for (x, label, value) in [
        (22., "用户扣费", money(m.totals.cost)),
        (34. + mw, "缓存命中率", hit),
    ] {
        detail = detail.push(at(
            Container::new()
                .width(mw)
                .height(77.)
                .color(inset())
                .border_radius(10.)
                .child(Stack::new().push(at(metric(label, value, mw - 28.), 14., 10.))),
            x,
            177.,
        ));
    }
    detail = detail
        .push(at(text("Token 构成", 14., Folio::muted()), 22., 266.))
        .push(at(composition(m, rw - 44.), 22., 290.));
    if let Some(t) = &d.totals {
        let share = if t.tokens > 0 {
            m.totals.tokens as f64 / t.tokens as f64 * 100.
        } else {
            0.
        };
        detail = detail.push(right_text(
            format!("范围占比 {share:.1}%"),
            14.,
            Folio::accent_ink(),
            rw - 22.,
            17.,
            180.,
        ));
    }
    Stack::new()
        .push(panel(lw, h).child(list))
        .push(at(panel(rw, h).child(detail), lw + 16., 0.))
}
fn user_list(state: Shared, d: &Data, w: f32, h: f32, index: usize, period: Period) -> Stack {
    if d.users.is_empty() {
        return Stack::new().push(empty_card(
            w,
            h,
            Page::Users,
            "暂无用户用量",
            "当前时间范围没有可显示的用户记录",
        ));
    }
    let lw = 244.;
    let rw = w - lw - 16.;
    let summary = Stack::new()
        .push(at(
            text(
                format!("{} · 用户用量", period.label()),
                16.,
                Folio::muted(),
            ),
            22.,
            20.,
        ))
        .push(at(
            adaptive_number(
                d.totals
                    .as_ref()
                    .map(|t| tokens(t.tokens))
                    .unwrap_or("—".into()),
                36.,
                lw - 44.,
            ),
            22.,
            64.,
        ))
        .push(at(
            text(fit(&d.total_label, 14., lw - 44.), 14., Folio::muted()),
            22.,
            118.,
        ))
        .push(at(rule(lw - 44.), 22., 156.))
        .push(at(
            metric(
                if d.users.len() >= 200 {
                    "返回用户（前 200）"
                } else {
                    "活跃用户"
                },
                format!("{} 位", d.users.len()),
                lw - 44.,
            ),
            22.,
            181.,
        ))
        .push(at(
            metric(
                "用户扣费",
                d.totals
                    .as_ref()
                    .map(|t| money(t.cost))
                    .unwrap_or("—".into()),
                lw - 44.,
            ),
            22.,
            267.,
        ))
        .push(at(
            text("点击查看模型与趋势", 14., Folio::muted()),
            22.,
            h - 38.,
        ));
    let mut list = Stack::new()
        .push(at(text("用户排名", 18., Folio::ink()), 18., 13.))
        .push(right_text(
            "Token / 用户扣费",
            14.,
            Folio::muted(),
            rw - 18.,
            17.,
            164.,
        ));
    let max = d
        .users
        .iter()
        .map(|u| u.totals.tokens)
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    for (i, u) in d.users.iter().skip(index * 5).take(5).enumerate() {
        let n = index * 5 + i;
        let st = state.clone();
        let id = u.id;
        let display_name = u.display_name();
        let color = visuals::identity_color(&format!("user:{id}"));
        let child = Stack::new()
            .push(at(
                text(format!("{:02}", n + 1), 14., Folio::muted()),
                10.,
                15.,
            ))
            .push(at(visuals::user_avatar(&display_name, 32., color), 40., 10.))
            .push(at(
                user_text(
                    fit_with_font(&display_name, 18., rw - 270., Font::dynamic_font()),
                    18.,
                    Folio::ink(),
                ),
                84.,
                15.,
            ))
            .push(right_text(
                tokens(u.totals.tokens),
                18.,
                Folio::ink(),
                rw - 56.,
                6.,
                140.,
            ))
            .push(right_text(
                money(u.totals.cost),
                14.,
                Folio::muted(),
                rw - 56.,
                34.,
                140.,
            ))
            .push(at(text("›", 24., Folio::muted()), rw - 40., 12.))
            .push(at(
                bar(rw - 118., 3., u.totals.tokens as f64 / max, color),
                84.,
                56.,
            ));
        list = list.push(at(
            ElevatedButton::new(child)
                .style(
                    Folio::control_style(rw - 20., 64., Folio::surface())
                        .padding(EdgeInsets::ZERO)
                        .border_radius(9.),
                )
                .on_pressed(move || edit(&st, |u| u.navigate(Page::Users, u.period, Some(id)))),
            10.,
            47. + i as f32 * 68.,
        ));
    }
    Stack::new()
        .push(panel(lw, h).child(summary))
        .push(at(panel(rw, h).child(list), lw + 16., 0.))
}

fn connection(state: Shared, w: f32, h: f32) -> Stack {
    let s = state.lock().unwrap();
    let u = &s.usage;
    let e = &u.editor;
    let site = e.site.clone();
    let key_len = e.key.len();
    let field = e.field;
    let keyboard = e.keyboard;
    let upper = e.uppercase;
    let symbols = e.symbols;
    let busy = u.busy;
    let configured = u.config.is_some();
    let status = u.status.clone();
    drop(s);
    let mut p = Stack::new().push(text("连接你的 Sub2API", 26.0, Folio::ink()));
    p = p.push(at(
        text(
            "通过 Wi-Fi 独立刷新，断开 Mac 后仍可使用",
            16.0,
            Folio::muted(),
        ),
        0.0,
        40.0,
    ));
    for (i, label) in [
        format!("站点  {}", short(&site, 51)),
        format!(
            "管理员密钥  {}",
            if key_len == 0 {
                if configured {
                    "留空则保留当前密钥".into()
                } else {
                    "点击输入".into()
                }
            } else {
                "•".repeat(key_len.min(28))
            }
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let st = state.clone();
        p = p.push(at(
            button(label, w, 48.0, keyboard && field == i, move || {
                edit(&st, |u| {
                    if !u.busy {
                        u.editor.field = i;
                        u.editor.keyboard = true;
                    }
                })
            }),
            0.0,
            74.0 + i as f32 * 58.0,
        ));
    }
    if keyboard {
        let rows = if symbols {
            vec!["!@#$%^&*()", "-_=+[]{}:;", "/?.~,<>|\\", "`'\"012345"]
        } else {
            vec!["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"]
        };
        let kw = (w - 18.0) / 10.0;
        for (r, row) in rows.iter().enumerate() {
            for (i, ch) in row.chars().enumerate() {
                let ch = if upper { ch.to_ascii_uppercase() } else { ch };
                let st = state.clone();
                p = p.push(at(
                    button(ch.to_string(), kw, 36.0, false, move || {
                        edit(&st, |u| {
                            if !u.busy {
                                let dst = if u.editor.field == 0 {
                                    &mut u.editor.site
                                } else {
                                    &mut u.editor.key
                                };
                                if dst.len() < if u.editor.field == 0 { 240 } else { 512 } {
                                    dst.push(ch);
                                }
                            }
                        })
                    }),
                    i as f32 * (kw + 2.0),
                    192.0 + r as f32 * 40.0,
                ));
            }
        }
        for (i, label) in [
            if upper { "小写" } else { "大写" },
            if symbols { "ABC" } else { "符号" },
            "退格",
            "清空",
            "收起",
        ]
        .into_iter()
        .enumerate()
        {
            let st = state.clone();
            p = p.push(at(
                button(label, 92.0, 32.0, false, move || {
                    edit(&st, |u| {
                        if u.busy {
                            return;
                        }
                        match i {
                            0 => u.editor.uppercase = !u.editor.uppercase,
                            1 => u.editor.symbols = !u.editor.symbols,
                            2 => {
                                if u.editor.field == 0 {
                                    u.editor.site.pop();
                                } else {
                                    u.editor.key.pop();
                                }
                            }
                            3 => {
                                if u.editor.field == 0 {
                                    u.editor.site.clear()
                                } else {
                                    u.editor.key.clear()
                                }
                            }
                            _ => u.editor.keyboard = false,
                        }
                    })
                }),
                i as f32 * 103.0,
                356.0,
            ));
        }
    } else {
        p = p
            .push(at(
                text(
                    "使用管理员 API Key，模型调用密钥无法读取管理统计。",
                    18.0,
                    Folio::muted(),
                ),
                0.0,
                206.0,
            ))
            .push(at(
                text(
                    "配置保存在设备内部存储，密钥不会写入 TF 数据缓存。",
                    18.0,
                    Folio::muted(),
                ),
                0.0,
                241.0,
            ))
            .push(at(
                text(
                    "每 60 秒刷新；失败保留旧数据并标记状态。",
                    18.0,
                    Folio::muted(),
                ),
                0.0,
                276.0,
            ))
            .push(at(
                text(
                    "也可在 Mac 的 P4 Desk「用量监控」中配置。",
                    18.0,
                    Folio::muted(),
                ),
                0.0,
                311.0,
            ));
    }
    let st = state.clone();
    p = p.push(at(
        button(
            if busy {
                "正在验证…"
            } else {
                "验证并保存"
            },
            156.0,
            44.0,
            true,
            move || {
                let mut s = st.lock().unwrap();
                if s.usage.busy {
                    return;
                }
                let e = &s.usage.editor;
                let key = if e.key.is_empty() {
                    s.usage
                        .config
                        .as_ref()
                        .filter(|c| Config::new(&e.site, &c.key).is_ok_and(|v| v.site == c.site))
                        .map(|c| c.key.as_str())
                        .unwrap_or("")
                } else {
                    e.key.as_str()
                };
                match Config::new(&e.site, key) {
                    Ok(c) => s.queue(UiCommand::Usage(Command::Save(c))),
                    Err(msg) => s.usage.status = msg.into(),
                };
                s.changed();
            },
        ),
        0.0,
        h - 56.0,
    ));
    let st = state;
    p = p.push(at(
        button("断开并清除密钥", 190.0, 44.0, false, move || {
            let mut s = st.lock().unwrap();
            s.queue(UiCommand::Usage(Command::Forget));
            s.changed();
        }),
        174.0,
        h - 56.0,
    ));
    p.push(at(
        text(short(&status, 29), 14.0, Folio::muted()),
        380.0,
        h - 44.0,
    ))
}
