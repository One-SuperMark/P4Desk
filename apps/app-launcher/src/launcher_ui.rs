//! Launcher shell ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197
//! (MIT): icon slots, PageView pages, page dots, background-app status bar and
//! full-screen sub-app routing. Layout follows the actual panel size; local tools,
//! Chinese text and touch Back/Kill controls extend the original shell. Each page
//! contains four columns and two rows; system shortcuts share the original slots.

use crate::app_icons::get_app_icon_asset;
use crate::app_launch::AppLaunchOverlay;
use crate::flip_clock::{
    ClockControl, ClockControlPainter, FlipClockPainter, CLOCK_CONTENT_ORIGIN,
};
use crate::launcher_state::{ActiveApp, LauncherState, UiCommand};
use crate::pomodoro_ui::{build_timer_navigation, build_timer_ui};
use crate::status_bar::STATUS_BAR_HEIGHT;
use crate::timer_completion::TimerCompletionOverlay;
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

const GRID_COLUMNS: usize = 4;
const GRID_ROWS: usize = 2;
pub const DESKTOP_PAGE_CAPACITY: usize = GRID_COLUMNS * GRID_ROWS;
pub const DESKTOP_ENTRIES: &[(&str, &str)] = &[
    ("clock", "时钟"),
    ("timer", "番茄钟"),
    ("notes", "便签"),
    ("calculator", "计算器"),
    ("mac", "Mac 控制"),
    ("settings", "设置"),
    ("display", "USB 副屏"),
    ("screen", "关闭屏幕"),
    ("file-manager", "文件管理"),
    ("office-viewer", "Office"),
    ("sub2api-monitor", "用量监控"),
];

fn at(child: impl Widget + 'static, x: f32, y: f32) -> Positioned {
    Positioned::new(child).left(x).top(y)
}
fn text(label: impl Into<String>, px: f32, color: Color) -> Text {
    Text::new(label).font_size(px).color(color)
}
fn panel(w: f32, h: f32) -> Container {
    Container::new()
        .width(w)
        .height(h)
        .color(Folio::surface())
        .border_radius(Folio::GROUP_RADIUS)
}
fn button(
    label: impl Into<String>,
    w: f32,
    h: f32,
    accent: bool,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(text(label, 22.0, Folio::ink()).wrap())
        .style(
            Folio::control_style(
                w,
                h,
                if accent {
                    Folio::accent()
                } else {
                    Folio::raised()
                },
            )
            .padding(EdgeInsets::all(8.0)),
        )
        .on_pressed(f)
}
fn edit(state: &Arc<Mutex<LauncherState>>, f: impl FnOnce(&mut LauncherState)) {
    if let Ok(mut s) = state.lock() {
        f(&mut s);
        s.changed();
    }
}
fn clock_icon_button(
    control: ClockControl,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(
        CustomPaint::new(ClockControlPainter::new(control)).size(Size::new(24.0, 24.0)),
    )
    .style(Folio::navigation_style())
    .on_pressed(f)
}
fn clock_navigation(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let home = state.clone();
    Stack::new()
        .push(
            Container::new()
                .width(w)
                .height(STATUS_BAR_HEIGHT)
                .color(Folio::bg()),
        )
        .push(at(
            clock_icon_button(ClockControl::Home, move || {
                edit(&home, |s| s.background_active_app())
            }),
            24.0,
            6.0,
        ))
        .push(at(
            clock_icon_button(ClockControl::Close, move || {
                edit(&state, |s| s.kill_active_app())
            }),
            w - 72.0,
            6.0,
        ))
}
pub fn calculator_mode_selector_bounds(w: f32) -> Rect {
    let right = w - 96.0;
    let width = (right - 96.0).clamp(1.0, 340.0);
    Rect::from_ltwh(right - width, 9.0, width, 42.0)
}
fn calculator_navigation(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let home = state.clone();
    let calc = match &state.lock().unwrap().active_app {
        ActiveApp::Calculator(calc) => calc.clone(),
        _ => unreachable!("calculator navigation requires an active calculator"),
    };
    let selector = calculator_mode_selector_bounds(w);
    let icon = |control, action: Box<dyn Fn() + Send + Sync>| {
        ElevatedButton::new(
            CustomPaint::new(ClockControlPainter::new(control)).size(Size::new(24.0, 24.0)),
        )
        .style(Folio::navigation_style())
        .on_pressed(action)
    };
    Stack::new()
        .push(
            Container::new()
                .width(w)
                .height(STATUS_BAR_HEIGHT)
                .color(Folio::bg()),
        )
        .push(at(
            icon(
                ClockControl::Home,
                Box::new(move || edit(&home, |s| s.background_active_app())),
            ),
            24.0,
            6.0,
        ))
        .push(at(
            calculator::build_mode_selector(calc, Size::new(selector.width, selector.height)),
            selector.x,
            selector.y,
        ))
        .push(at(
            icon(
                ClockControl::Close,
                Box::new(move || edit(&state, |s| s.kill_active_app())),
            ),
            w - 72.0,
            6.0,
        ))
}
pub fn build_launcher_ui(state: Arc<Mutex<LauncherState>>, size: Size) -> Box<dyn Widget> {
    let (active, date, notice) = {
        let s = state.lock().unwrap();
        crate::appearance::configure(&s.settings);
        (
            s.active_app.clone(),
            if s.time_estimated {
                format!("{} · 待校准", s.date)
            } else {
                s.date.clone()
            },
            s.notice.clone(),
        )
    };
    let w = size.width.max(1.0);
    let h = size.height.max(1.0);
    let mut root: Box<dyn Widget> = if matches!(active, ActiveApp::Settings) {
        Box::new(crate::settings_ui::build_settings(
            state.clone(),
            Size::new(w, h),
        ))
    } else if matches!(active, ActiveApp::Launcher) {
        Box::new(build_desktop(state.clone(), Size::new(w, h)))
    } else {
        let title = match &active {
            ActiveApp::Clock => "时钟",
            ActiveApp::Timer => "专注与计时",
            ActiveApp::Notes(_) => "便签",
            ActiveApp::Calculator(_) => "计算器",
            ActiveApp::MacControls => "Mac 控制",
            ActiveApp::Settings => "设置",
            ActiveApp::Usage => "用量监控",
            ActiveApp::Planned(app) => app.title(),
            ActiveApp::DisplaySetup => "USB 副屏",
            ActiveApp::Launcher => unreachable!(),
        };
        let content_w = (w - 48.0).max(1.0);
        let content_h = (h - 94.0).max(1.0);
        let content: Box<dyn Widget> = match active.clone() {
            ActiveApp::Clock => Box::new(clock_page(
                state.clone(),
                Size::new(content_w, content_h),
                date,
            )),
            ActiveApp::Timer => Box::new(build_timer_ui(
                state.clone(),
                Size::new(content_w, content_h),
            )),
            ActiveApp::Notes(view) => Box::new(notes_page(state.clone(), view, content_w)),
            ActiveApp::Calculator(calc) => Box::new(calculator::build_calculator_ui(
                calc,
                Size::new(content_w, content_h),
            )),
            ActiveApp::MacControls => Box::new(mac_page(state.clone(), content_w)),
            ActiveApp::Settings => unreachable!(),
            ActiveApp::Usage => Box::new(crate::usage::ui::build(state.clone(), Size::new(content_w, content_h))),
            ActiveApp::Planned(app) => Box::new(crate::planned_apps::build(
                app,
                Size::new(content_w, content_h),
            )),
            ActiveApp::DisplaySetup => Box::new(display_setup_page(
                state.clone(),
                Size::new(content_w, content_h),
            )),
            ActiveApp::Launcher => unreachable!(),
        };
        let bar = if matches!(active, ActiveApp::Calculator(_)) {
            calculator_navigation(state.clone(), w)
        } else if matches!(active, ActiveApp::Timer) {
            build_timer_navigation(state.clone(), w)
        } else {
            let nav = clock_navigation(state.clone(), w);
            if matches!(active, ActiveApp::Clock | ActiveApp::DisplaySetup) {
                nav
            } else {
                nav.push(at(text(title, 28.0, Folio::ink()), 88.0, 15.0))
            }
        };
        Box::new(
            Stack::new()
                .push(Container::new().width(w).height(h).color(Folio::bg()))
                .push(at(bar, 0.0, 0.0))
                .push(at(
                    Container::new()
                        .width(content_w)
                        .height(content_h)
                        .child(content),
                    CLOCK_CONTENT_ORIGIN.dx,
                    CLOCK_CONTENT_ORIGIN.dy,
                )),
        )
    };
    if !notice.is_empty()
        && !(matches!(active, ActiveApp::Launcher)
            && notice == "Mac 未连接，请连接 USB 和 Mac 应用")
        && !matches!(
            active,
            ActiveApp::Clock
                | ActiveApp::Calculator(_)
                | ActiveApp::Timer
                | ActiveApp::DisplaySetup
                | ActiveApp::Usage
        )
    {
        // The slim gap between cards and the grid keeps notices clear of app icons.
        let on_desktop = matches!(active, ActiveApp::Launcher);
        let notice_width = if on_desktop {
            crate::folio_desktop::workspace_width(Size::new(w, h))
        } else {
            w - 48.0
        };
        let notice_chars = if on_desktop {
            ((notice_width - 12.0) / 14.0).floor().max(4.0) as usize
        } else {
            42
        };
        root = Box::new(
            Stack::new().push(root).push(at(
                Container::new()
                    .width(notice_width)
                    .height(if on_desktop { 24.0 } else { 38.0 })
                    .color(Folio::line())
                    .border_radius(8.0)
                    .padding(EdgeInsets::all(if on_desktop { 4.0 } else { 6.0 }))
                    .child(text(
                        short(&notice, notice_chars),
                        if on_desktop { 14.0 } else { 18.0 },
                        Folio::orange(),
                    )),
                24.0,
                if on_desktop { 182.0 } else { h - 43.0 },
            )),
        );
    }
    let keep_control_backdrop = {
        let s = state.lock().unwrap();
        matches!(active, ActiveApp::Launcher)
            && s.status_panel_open
            && s.status_panel_kind == crate::status_bar::StatusPanelKind::Control
    };
    if !keep_control_backdrop {
        state
            .lock()
            .unwrap()
            .control_center_backdrop
            .lock()
            .unwrap()
            .take();
    }
    if matches!(active, ActiveApp::Launcher) && state.lock().unwrap().status_panel_open {
        root = crate::status_bar::with_status_panel(state.clone(), root, Size::new(w, h));
    }
    if matches!(active, ActiveApp::Timer) {
        root = Box::new(TimerCompletionOverlay::new(state.clone(), root));
    }
    let launching = {
        let state = state.lock().unwrap();
        state.app_launch.frame(state.monotonic_ms).is_some()
    };
    let desktop = launching
        .then(|| Box::new(build_desktop(state.clone(), Size::new(w, h))) as Box<dyn Widget>);
    let backdrop = state.lock().unwrap().desktop_backdrop.clone();
    root = Box::new(AppLaunchOverlay::new(
        state.clone(),
        root,
        desktop,
        backdrop,
        matches!(active, ActiveApp::Launcher),
    ));
    let back = state.clone();
    let kill = state.clone();
    Box::new(
        BackListener::new(root, move || edit(&back, |s| s.back_active_app()))
            .on_kill(move || edit(&kill, |s| s.kill_active_app())),
    )
}

pub(crate) fn build_desktop(state: Arc<Mutex<LauncherState>>, size: Size) -> impl Widget {
    crate::folio_desktop::build(state, size)
}

fn short(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!(
            "{}...",
            s.chars().take(max.saturating_sub(3)).collect::<String>()
        )
    } else {
        s.into()
    }
}

fn content_excerpt(value: &str, px: f32, width: f32, lines: usize) -> String {
    let font = Font::content_font();
    let layout = font.layout_text(value, px, width, true);
    if layout.lines.len() <= lines {
        return layout.lines.join("\n");
    }
    let mut shown: Vec<_> = layout.lines.into_iter().take(lines).collect();
    if let Some(last) = shown.last_mut() {
        while !last.is_empty() && font.measure_text(&format!("{last}..."), px).width > width {
            last.pop();
        }
        last.push_str("...");
    }
    shown.join("\n")
}

fn clock_page(state: Arc<Mutex<LauncherState>>, size: Size, date: String) -> Stack {
    let w = size.width;
    Stack::new()
        .push(CustomPaint::new(FlipClockPainter::new(state)).size(size))
        .push(at(
            Container::new()
                .width(560.0_f32.min(w))
                .height(42.0)
                .color(Folio::surface())
                .border_radius(21.0)
                .child(Center::new(text(date, 22.0, Folio::muted()))),
            (w - 560.0_f32.min(w)) * 0.5,
            size.height - 106.0,
        ))
}
fn notes_page(
    state: Arc<Mutex<LauncherState>>,
    view: Arc<Mutex<crate::launcher_state::NotesView>>,
    w: f32,
) -> Stack {
    let notes = state.lock().unwrap().snapshot.notes.clone();
    let mut v = view.lock().unwrap();
    v.selected = v.selected.min(notes.len().saturating_sub(1));
    let selected = v.selected;
    let scroll = v.scroll.clone();
    let confirm = v.confirm_delete;
    drop(v);
    let mut p = Stack::new().push(panel(w, 382.0));
    if let Some(n) = notes.get(selected) {
        let title = content_excerpt(&n.title, 28.0, w - 108.0, 2);
        let title_top = if title.contains('\n') { 10.0 } else { 28.0 };
        p = p
            .push(at(crate::widgets::app_badge("notes", 52.0), 16.0, 17.0))
            .push(at(
                Container::new().width(w - 108.0).height(76.0).child(
                    text(title, 28.0, Folio::ink())
                        .font(Font::content_font().clone())
                        .wrap(),
                ),
                84.0,
                title_top,
            ))
            .push(at(
                Container::new()
                    .width(w - 48.0)
                    .height(1.0)
                    .color(Folio::separator()),
                24.0,
                94.0,
            ))
            .push(at(
                Container::new().width(w - 48.0).height(252.0).child(
                    SingleChildScrollView::new(
                        text(&n.body, 22.0, Folio::ink())
                            .font(Font::content_font().clone())
                            .wrap(),
                    )
                    .controller(scroll),
                ),
                24.0,
                112.0,
            ));
        let s = state.clone();
        let note_id = n.id.clone();
        let vc = view.clone();
        p = p.push(at(
            ElevatedButton::new(text(
                if confirm {
                    "确认删除"
                } else {
                    "删除便签"
                },
                22.0,
                if confirm {
                    Folio::ink()
                } else {
                    Folio::destructive()
                },
            ))
            .style(Folio::control_style(
                166.0,
                54.0,
                if confirm {
                    Folio::destructive_fill()
                } else {
                    Folio::raised()
                },
            ))
            .on_pressed(move || {
                if let Ok(mut v) = vc.lock() {
                    if v.confirm_delete {
                        v.confirm_delete = false;
                        edit(&s, |s| s.queue(UiCommand::DeleteNote(note_id.clone())));
                    } else {
                        v.confirm_delete = true;
                    }
                }
            }),
            w - 166.0,
            402.0,
        ));
        p = p.push(at(
            text(
                format!("{} / {}", selected + 1, notes.len()),
                22.0,
                Folio::muted(),
            ),
            319.0,
            416.0,
        ));
    } else {
        p = p
            .push(at(
                crate::widgets::app_badge("notes", 88.0),
                (w - 88.0) * 0.5,
                60.0,
            ))
            .push(at(
                Container::new()
                    .width(w)
                    .height(42.0)
                    .child(Center::new(text("还没有便签", 28.0, Folio::ink()))),
                0.0,
                174.0,
            ))
            .push(at(
                Container::new()
                    .width(w)
                    .height(36.0)
                    .child(Center::new(text(
                        "在 Mac 上编辑并同步到这里",
                        22.0,
                        Folio::muted(),
                    ))),
                0.0,
                230.0,
            ));
    }
    if notes.is_empty() {
        return p;
    }
    for (i, (label, delta)) in [("上一条", -1isize), ("下一条", 1isize)]
        .into_iter()
        .enumerate()
    {
        let vc = view.clone();
        let count = notes.len();
        p = p.push(at(
            button(label, 142.0, 54.0, false, move || {
                if let Ok(mut v) = vc.lock() {
                    if count > 0 {
                        v.selected =
                            (v.selected as isize + delta).rem_euclid(count as isize) as usize;
                    }
                    v.confirm_delete = false;
                    v.scroll.set_offset(0.0);
                }
            }),
            i as f32 * 158.0,
            402.0,
        ));
    }
    p
}
fn mac_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (buttons, page, connected) = {
        let s = state.lock().unwrap();
        (s.snapshot.buttons.clone(), s.mac_page, s.connected)
    };
    let mut p = Stack::new()
        .push(at(
            crate::widgets::glyph(
                &UI_USB,
                22.0,
                if connected {
                    Folio::green()
                } else {
                    Folio::muted()
                },
            ),
            0.0,
            2.0,
        ))
        .push(at(
            text(
                if connected {
                    "轻触执行 Mac 动作"
                } else {
                    "连接 Mac 后可执行快捷键与启动应用"
                },
                22.0,
                Folio::muted(),
            ),
            32.0,
            0.0,
        ));
    let cw = (w - 32.0) / 3.0;
    for (i, b) in buttons.iter().skip(page * 12).take(12).enumerate() {
        let s = state.clone();
        let id = b.id.clone();
        let action_icon = match &b.action {
            p4desk_protocol::Action::Shortcut { .. } => &UI_SYSTEM,
            p4desk_protocol::Action::Application { .. } => &UI_TERMINAL,
            p4desk_protocol::Action::Media { .. } => &UI_PLAY_CIRCLE,
        };
        p = p.push(at(
            ElevatedButton::new(
                Stack::new()
                    .push(at(
                        crate::widgets::glyph(action_icon, 26.0, Folio::accent_ink()),
                        16.0,
                        20.0,
                    ))
                    .push(at(
                        Container::new()
                            .width(cw - 76.0)
                            .height(52.0)
                            .child(Center::new(
                                text(
                                    content_excerpt(&b.label, 18.0, cw - 76.0, 2),
                                    18.0,
                                    Folio::ink(),
                                )
                                .font(Font::content_font().clone())
                                .wrap(),
                            )),
                        56.0,
                        7.0,
                    )),
            )
            .style(
                ButtonStyle::new()
                    .antialias(true)
                    .size(cw, 66.0)
                    .color(Folio::surface())
                    .pressed_color(Folio::pressed(Folio::surface()))
                    .border_radius(Folio::GROUP_RADIUS)
                    .padding(EdgeInsets::ZERO),
            )
            .on_pressed(move || edit(&s, |s| s.queue(UiCommand::Action(id.clone())))),
            (i % 3) as f32 * (cw + 16.0),
            44.0 + (i / 3) as f32 * 77.0,
        ));
    }
    if buttons.is_empty() {
        p = p.push(at(
            panel(w, 292.0).child(
                Stack::new()
                    .push(at(
                        crate::widgets::app_badge("mac", 88.0),
                        (w - 88.0) * 0.5,
                        28.0,
                    ))
                    .push(at(
                        Container::new()
                            .width(w - 96.0)
                            .height(124.0)
                            .child(Center::new(
                                text(
                                    "在 Mac 应用中添加快捷键或应用启动按钮，再同步到设备。",
                                    22.0,
                                    Folio::muted(),
                                )
                                .wrap(),
                            )),
                        48.0,
                        142.0,
                    )),
            ),
            0.0,
            44.0,
        ));
    }
    let pages = buttons.len().div_ceil(12).max(1);
    if pages > 1 {
        let s = state.clone();
        p = p.push(at(
            button(
                format!("动作 {} / {}  下一页", page + 1, pages),
                276.0,
                46.0,
                false,
                move || edit(&s, |s| s.mac_page = (s.mac_page + 1) % pages),
            ),
            0.0,
            363.0,
        ));
    } else if !buttons.is_empty() {
        p = p.push(at(
            text(format!("{} / {}", page + 1, pages), 18.0, Folio::muted()),
            16.0,
            376.0,
        ));
    }
    p = p.push(at(
        Container::new()
            .width(w)
            .height(1.0)
            .color(Folio::separator()),
        0.0,
        416.0,
    ));
    for (i, (label, usage)) in [
        ("上一首", 0xb6u16),
        ("播放 / 暂停", 0xcd),
        ("下一首", 0xb5),
        ("音量 -", 0xea),
        ("音量 +", 0xe9),
    ]
    .into_iter()
    .enumerate()
    {
        let s = state.clone();
        p = p.push(at(
            button(label, (w - 48.0) / 5.0, 52.0, false, move || {
                edit(&s, |s| s.queue(UiCommand::Media(usage)))
            }),
            i as f32 * ((w - 48.0) / 5.0 + 12.0),
            429.0,
        ));
    }
    p
}
pub(crate) fn manual_time_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (values, timezone) = {
        let s = state.lock().unwrap();
        (s.manual_clock.values(), s.settings.timezone_minutes)
    };
    let mut p = Stack::new().push(panel(w, 334.0)).push(at(
        text("手动设置日期与时间", 28.0, Folio::accent_ink()),
        24.0,
        20.0,
    ));
    let cw = (w - 64.0) / 5.0;
    for (i, label) in ["年", "月", "日", "时", "分"].into_iter().enumerate() {
        let x = 24.0 + i as f32 * (cw + 4.0);
        p = p
            .push(at(text(label, 22.0, Folio::muted()), x, 72.0))
            .push(at(
                text(format!("{:02}", values[i]), 36.0, Folio::ink()),
                x,
                117.0,
            ));
        let s = state.clone();
        p = p.push(at(
            button("+", cw - 12.0, 52.0, false, move || {
                edit(&s, |s| s.manual_clock.adjust(i, 1))
            }),
            x,
            184.0,
        ));
        let s = state.clone();
        p = p.push(at(
            button("-", cw - 12.0, 52.0, false, move || {
                edit(&s, |s| s.manual_clock.adjust(i, -1))
            }),
            x,
            250.0,
        ));
    }
    let s = state.clone();
    p = p.push(at(
        button("保存时间", 220.0, 58.0, true, move || {
            edit(&s, |s| {
                let ms = s.manual_clock.unix_ms(s.settings.timezone_minutes);
                s.queue(UiCommand::SetTime(ms, s.settings.timezone_minutes));
                s.manual_time_open = false;
            })
        }),
        0.0,
        356.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("从 Mac 校时", 220.0, 58.0, false, move || {
            edit(&s, |s| s.queue(UiCommand::RequestTimeSync))
        }),
        236.0,
        356.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("返回", 150.0, 58.0, false, move || {
            edit(&s, |s| s.manual_time_open = false)
        }),
        472.0,
        356.0,
    ));
    p.push(at(
        text(
            format!("时区偏移：{} 分钟，重新上电后需再次校时", timezone),
            18.0,
            Folio::muted(),
        ),
        0.0,
        438.0,
    ))
}

fn display_setup_page(state: Arc<Mutex<LauncherState>>, size: Size) -> Stack {
    let (connected, notice) = {
        let s = state.lock().unwrap();
        (s.connected, s.notice.clone())
    };
    let cx = size.width * 0.5;
    let cy = size.height * 0.5;
    let status = if notice.is_empty() {
        if connected {
            "等待 Mac 应用".to_owned()
        } else {
            "Mac 未连接，请连接 USB 和 Mac 应用".to_owned()
        }
    } else {
        notice
    };
    Stack::new()
        .push(at(panel(size.width - 120.0, 384.0), 60.0, cy - 192.0))
        .push(at(
            Container::new()
                .width(size.width - 168.0)
                .height(1.0)
                .color(Folio::separator()),
            84.0,
            cy + 70.0,
        ))
        .push(at(
            CustomPaint::new(crate::widgets::AppIconPainter {
                asset: get_app_icon_asset("display").unwrap(),
                target_size: 112.0,
                pressed: None,
                clock: None,
            })
            .size(Size::new(112.0, 112.0)),
            cx - 56.0,
            cy - 164.0,
        ))
        .push(at(
            Container::new()
                .width(size.width)
                .height(40.0)
                .child(Center::new(text("USB 副屏", 28.0, Folio::ink()))),
            0.0,
            cy - 24.0,
        ))
        .push(at(
            Container::new()
                .width(size.width - 184.0)
                .height(70.0)
                .child(Center::new(
                    text(
                        status,
                        18.0,
                        if connected {
                            Folio::accent_ink()
                        } else {
                            Folio::muted()
                        },
                    )
                    .wrap(),
                )),
            92.0,
            cy + 6.0,
        ))
        .push(at(
            button("进入副屏", 180.0, 48.0, true, move || {
                edit(&state, |s| s.request_display_mode())
            }),
            cx - 90.0,
            cy + 86.0,
        ))
}
