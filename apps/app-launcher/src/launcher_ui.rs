//! Launcher shell ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197
//! (MIT): icon slots, PageView pages, page dots, background-app status bar and
//! full-screen sub-app routing. Layout follows the actual panel size; local tools,
//! Chinese text and touch Back/Kill controls extend the original shell. Each page
//! contains four columns and two rows; system shortcuts share the original slots.

use crate::app_icons::{get_app_icon_asset, DESKTOP_ICON_SIDE};
use crate::app_launch::AppLaunchOverlay;
use crate::flip_clock::{
    ClockControl, ClockControlPainter, FlipClockPainter, CLOCK_CONTENT_ORIGIN,
};
use crate::launcher_state::{ActiveApp, LauncherState, UiCommand};
use crate::pomodoro_ui::{build_timer_navigation, build_timer_ui, TIMER_BG};
use crate::status_bar::{build_status_bar, STATUS_BAR_HEIGHT};
use crate::timer_completion::TimerCompletionOverlay;
use crate::widgets::{build_app_icon, make_dot, WallpaperPainter};
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const BG: Color = Color::from_hex(0x11161e);
const CARD: Color = Color::from_hex(0x202a37);
const LINE: Color = Color::from_hex(0x344252);
const INK: Color = Color::from_hex(0xe8edf2);
const MUTED: Color = Color::from_hex(0x94a4b6);
const MINT: Color = Color::from_hex(0x8edbc5);
const WARM: Color = Color::from_hex(0xffbe82);

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
        .color(CARD)
        .border_radius(18.0)
}
fn button(
    label: impl Into<String>,
    w: f32,
    h: f32,
    accent: bool,
    f: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    ElevatedButton::new(text(label, 22.0, if accent { BG } else { INK }).wrap())
        .style(
            ButtonStyle::new()
                .size(w, h)
                .color(if accent { MINT } else { CARD })
                .pressed_color(LINE)
                .border_radius(12.0)
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
        CustomPaint::new(ClockControlPainter::new(control)).size(Size::new(32.0, 32.0)),
    )
    .style(
        ButtonStyle::new()
            .size(48.0, 48.0)
            .color(Color::BLACK)
            .pressed_color(Color::from_hex(0x222222))
            .border_radius(12.0)
            .padding(EdgeInsets::all(8.0)),
    )
    .on_pressed(f)
}
fn clock_navigation(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let home = state.clone();
    Stack::new()
        .push(
            Container::new()
                .width(w)
                .height(STATUS_BAR_HEIGHT)
                .color(Color::BLACK),
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
    Rect::from_ltwh(right - width, 13.0, width, 34.0)
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
            CustomPaint::new(ClockControlPainter::new(control)).size(Size::new(32.0, 32.0)),
        )
        .style(
            ButtonStyle::new()
                .size(48.0, 48.0)
                .color(calculator::CALCULATOR_BG)
                .pressed_color(Color::from_hex(0x5b5958))
                .border_radius(24.0)
                .padding(EdgeInsets::all(8.0)),
        )
        .on_pressed(action)
    };
    Stack::new()
        .push(
            Container::new()
                .width(w)
                .height(STATUS_BAR_HEIGHT)
                .color(calculator::CALCULATOR_BG),
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
    let (active, date, notice, manual_time_open) = {
        let s = state.lock().unwrap();
        (
            s.active_app.clone(),
            s.date.clone(),
            s.notice.clone(),
            s.manual_time_open,
        )
    };
    let w = size.width.max(1.0);
    let h = size.height.max(1.0);
    let mut root: Box<dyn Widget> = if matches!(active, ActiveApp::Launcher) {
        Box::new(build_desktop(state.clone(), Size::new(w, h)))
    } else {
        let title = match &active {
            ActiveApp::Clock => "时钟",
            ActiveApp::Timer => "专注与计时",
            ActiveApp::Notes(_) => "便签",
            ActiveApp::Calculator(_) => "计算器",
            ActiveApp::MacControls => "Mac 控制",
            ActiveApp::Settings => "设置",
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
            ActiveApp::Settings => Box::new(settings_page(state.clone(), content_w)),
            ActiveApp::DisplaySetup => Box::new(display_setup_page(
                state.clone(),
                Size::new(content_w, content_h),
            )),
            ActiveApp::Launcher => unreachable!(),
        };
        let back = state.clone();
        let kill = state.clone();
        let bar = if matches!(active, ActiveApp::Clock | ActiveApp::DisplaySetup) {
            clock_navigation(state.clone(), w)
        } else if matches!(active, ActiveApp::Calculator(_)) {
            calculator_navigation(state.clone(), w)
        } else if matches!(active, ActiveApp::Timer) {
            build_timer_navigation(state.clone(), w)
        } else {
            Stack::new()
                .push(
                    Container::new()
                        .width(w)
                        .height(STATUS_BAR_HEIGHT)
                        .color(CARD),
                )
                .push(at(text(title, 28.0, INK), 24.0, 10.0))
                .push(at(
                    button(
                        if matches!(active, ActiveApp::Settings) && manual_time_open {
                            "返回设置"
                        } else {
                            "回桌面"
                        },
                        152.0,
                        44.0,
                        false,
                        move || edit(&back, |s| s.back_active_app()),
                    ),
                    w - 304.0,
                    6.0,
                ))
                .push(at(
                    button("结束应用", 108.0, 44.0, false, move || {
                        edit(&kill, |s| s.kill_active_app())
                    }),
                    w - 132.0,
                    6.0,
                ))
        };
        Box::new(
            Stack::new()
                .push(Container::new().width(w).height(h).color(
                    if matches!(active, ActiveApp::Clock | ActiveApp::DisplaySetup) {
                        Color::BLACK
                    } else if matches!(active, ActiveApp::Calculator(_)) {
                        calculator::CALCULATOR_BG
                    } else if matches!(active, ActiveApp::Timer) {
                        TIMER_BG
                    } else {
                        BG
                    },
                ))
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
        && !matches!(
            active,
            ActiveApp::Clock
                | ActiveApp::Calculator(_)
                | ActiveApp::Timer
                | ActiveApp::DisplaySetup
        )
    {
        // Desktop notices occupy the left of the footer, leaving page dots visible.
        let on_desktop = matches!(active, ActiveApp::Launcher);
        let notice_width = if on_desktop { w * 0.38 } else { w - 48.0 };
        let notice_chars = if on_desktop {
            ((notice_width - 12.0) / 18.0).floor().max(4.0) as usize
        } else {
            42
        };
        root = Box::new(
            Stack::new().push(root).push(at(
                Container::new()
                    .width(notice_width)
                    .height(38.0)
                    .color(LINE)
                    .border_radius(8.0)
                    .padding(EdgeInsets::all(6.0))
                    .child(text(short(&notice, notice_chars), 18.0, WARM)),
                24.0,
                h - 43.0,
            )),
        );
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

/// Upstream icon-slot structure, expanded to 4 × 2 with stable empty slots.
fn build_app_grid_page(mut icons: Vec<Box<dyn Widget>>, icon_size: f32) -> impl Widget {
    let mut slots: Vec<Box<dyn Widget>> = Vec::with_capacity(DESKTOP_PAGE_CAPACITY);
    for _ in 0..DESKTOP_PAGE_CAPACITY {
        slots.push(if icons.is_empty() {
            Box::new(SizedBox::from_size(Size::new(
                (icon_size + 32.0).max(160.0),
                icon_size + 44.0,
            )))
        } else {
            icons.remove(0)
        });
    }
    let mut slots = slots.into_iter();
    let mut grid = Column::new()
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .cross_axis_alignment(CrossAxisAlignment::Center);
    for _ in 0..GRID_ROWS {
        let mut row = Row::new()
            .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .cross_axis_alignment(CrossAxisAlignment::Center);
        for _ in 0..GRID_COLUMNS {
            row = row.push(slots.next().unwrap());
        }
        grid = grid.push(row);
    }
    grid
}

pub(crate) fn build_desktop(state: Arc<Mutex<LauncherState>>, size: Size) -> impl Widget {
    let (controller, current_page, total_pages) = {
        let mut s = state.lock().unwrap();
        s.total_pages = DESKTOP_ENTRIES.len().div_ceil(DESKTOP_PAGE_CAPACITY).max(1);
        let saved_page = s.page_controller.page();
        s.current_page = saved_page.min(s.total_pages - 1);
        if s.current_page != saved_page {
            s.page_controller.set_page(s.current_page);
        }
        (s.page_controller.clone(), s.current_page, s.total_pages)
    };
    let page_y = STATUS_BAR_HEIGHT + 12.0;
    let page_h = (size.height - page_y - 44.0).max(1.0);
    let icon_size = (page_h * 0.30).clamp(96.0, DESKTOP_ICON_SIDE as f32);
    let entry = |id: &'static str, label: &'static str| -> Box<dyn Widget> {
        let launch = state.clone();
        Box::new(build_app_icon(
            label,
            get_app_icon_asset(id).expect("missing built-in app icon"),
            icon_size,
            move |source| {
                edit(&launch, |s| match id {
                    "screen" => s.queue(UiCommand::Screen(false)),
                    _ => s.launch_app(id, source),
                })
            },
        ))
    };
    let grid_pages: Vec<_> = DESKTOP_ENTRIES
        .chunks(DESKTOP_PAGE_CAPACITY)
        .map(|entries| {
            build_app_grid_page(
                entries
                    .iter()
                    .map(|&(id, label)| entry(id, label))
                    .collect(),
                icon_size,
            )
        })
        .collect();
    let changed = state.clone();
    let pages = PageView::new(grid_pages)
        .controller(controller)
        .transition(PageTransition::None)
        .on_page_changed(move |page| edit(&changed, |s| s.current_page = page));
    let mut dots = Row::new()
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center);
    let indicator_count = if total_pages > 1 { total_pages } else { 0 };
    for page in 0..indicator_count {
        let select = state.clone();
        dots = dots.push(
            GestureDetector::new(
                Container::new()
                    .width(44.0)
                    .height(28.0)
                    .child(Center::new(make_dot(current_page == page))),
            )
            .on_tap(move || {
                edit(&select, |s| {
                    s.page_controller.set_page(page);
                    s.current_page = page;
                })
            }),
        );
    }
    CustomPaint::new(WallpaperPainter).size(size).child(
        Stack::new()
            .push(at(build_status_bar(state, size.width), 0.0, 0.0))
            .push(at(
                Container::new()
                    .width(size.width)
                    .height(page_h)
                    .child(pages),
                0.0,
                page_y,
            ))
            .push(at(
                Container::new()
                    .width(size.width)
                    .height(44.0)
                    .child(Center::new(dots)),
                0.0,
                size.height - 44.0,
            )),
    )
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

fn clock_page(state: Arc<Mutex<LauncherState>>, size: Size, date: String) -> Stack {
    let w = size.width;
    Stack::new()
        .push(CustomPaint::new(FlipClockPainter::new(state)).size(size))
        .push(at(
            Container::new()
                .width(w)
                .height(34.0)
                .child(Center::new(text(date, 22.0, MUTED))),
            0.0,
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
        p = p
            .push(at(
                Container::new().width(w - 48.0).height(79.0).child(
                    text(&n.title, 28.0, MINT)
                        .font(Font::content_font().clone())
                        .wrap(),
                ),
                24.0,
                17.0,
            ))
            .push(at(
                Container::new().width(w - 48.0).height(271.0).child(
                    SingleChildScrollView::new(
                        text(&n.body, 22.0, INK)
                            .font(Font::content_font().clone())
                            .wrap(),
                    )
                    .controller(scroll),
                ),
                24.0,
                96.0,
            ));
        let s = state.clone();
        let note_id = n.id.clone();
        let vc = view.clone();
        p = p.push(at(
            button(
                if confirm {
                    "确认删除"
                } else {
                    "删除便签"
                },
                166.0,
                54.0,
                confirm,
                move || {
                    if let Ok(mut v) = vc.lock() {
                        if v.confirm_delete {
                            v.confirm_delete = false;
                            edit(&s, |s| s.queue(UiCommand::DeleteNote(note_id.clone())));
                        } else {
                            v.confirm_delete = true;
                        }
                    }
                },
            ),
            w - 166.0,
            402.0,
        ));
        p = p.push(at(
            text(format!("{} / {}", selected + 1, notes.len()), 22.0, MUTED),
            319.0,
            416.0,
        ));
    } else {
        p = p
            .push(at(text("还没有便签", 36.0, INK), 26.0, 110.0))
            .push(at(
                text("在 Mac 上编辑并同步到这里", 22.0, MUTED),
                26.0,
                175.0,
            ));
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
    let mut p = Stack::new().push(at(
        text(
            if connected {
                "轻触执行 Mac 动作"
            } else {
                "连接 Mac 后可执行快捷键与启动应用"
            },
            22.0,
            MUTED,
        ),
        0.0,
        0.0,
    ));
    let cw = (w - 32.0) / 3.0;
    for (i, b) in buttons.iter().skip(page * 12).take(12).enumerate() {
        let s = state.clone();
        let id = b.id.clone();
        p = p.push(at(
            ElevatedButton::new(
                text(short(&b.label, 26), 18.0, INK)
                    .font(Font::content_font().clone())
                    .wrap(),
            )
            .style(
                ButtonStyle::new()
                    .size(cw, 66.0)
                    .color(CARD)
                    .pressed_color(LINE)
                    .border_radius(12.0)
                    .padding(EdgeInsets::all(8.0)),
            )
            .on_pressed(move || edit(&s, |s| s.queue(UiCommand::Action(id.clone())))),
            (i % 3) as f32 * (cw + 16.0),
            44.0 + (i / 3) as f32 * 77.0,
        ));
    }
    if buttons.is_empty() {
        p = p.push(at(
            panel(w, 250.0).padding(EdgeInsets::all(24.0)).child(
                text(
                    "在 Mac 应用中添加快捷键或应用启动按钮，再同步到设备。",
                    28.0,
                    INK,
                )
                .wrap(),
            ),
            0.0,
            44.0,
        ));
    }
    let pages = buttons.len().div_ceil(12).max(1);
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
fn settings_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    if state.lock().unwrap().manual_time_open {
        return manual_time_page(state, w);
    }
    let (brightness, sd, valid, generation, connected) = {
        let s = state.lock().unwrap();
        (
            s.settings.brightness,
            s.sd_ready,
            s.time_valid,
            s.snapshot.generation,
            s.connected,
        )
    };
    let mut p = Stack::new()
        .push(panel(w, 180.0))
        .push(at(text("屏幕亮度", 28.0, INK), 24.0, 18.0))
        .push(at(
            text(format!("{brightness}%"), 36.0, MINT),
            w - 145.0,
            17.0,
        ));
    for (i, value) in [25u8, 50, 75, 100].into_iter().enumerate() {
        let s = state.clone();
        p = p.push(at(
            button(
                format!("{value}%"),
                150.0,
                54.0,
                value == brightness,
                move || edit(&s, |s| s.queue(UiCommand::Brightness(value))),
            ),
            24.0 + i as f32 * 169.0,
            88.0,
        ));
    }
    let s = state.clone();
    p = p.push(at(
        button("关闭屏幕", 220.0, 58.0, false, move || {
            edit(&s, |s| s.queue(UiCommand::Screen(false)))
        }),
        0.0,
        200.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("手动校时", 220.0, 58.0, false, move || {
            edit(&s, |s| s.manual_time_open = true)
        }),
        236.0,
        200.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("进入 USB 副屏", w, 68.0, true, move || {
            edit(&s, |s| s.queue(UiCommand::RequestMode(Mode::Display)))
        }),
        0.0,
        280.0,
    ));
    p = p
        .push(at(
            text("副屏中三指长按 1 秒可回到 Pad", 22.0, MUTED),
            0.0,
            364.0,
        ))
        .push(at(
            text(
                format!(
                    "Mac：{}    校时：{}    TF：{}",
                    if connected { "已连接" } else { "未连接" },
                    if valid { "有效" } else { "等待" },
                    if sd { "就绪" } else { "未挂载" }
                ),
                18.0,
                MUTED,
            ),
            0.0,
            411.0,
        ))
        .push(at(
            text(
                format!("资源代次：{generation}    轻触屏幕可唤醒    字体：HarmonyOS Sans"),
                18.0,
                MUTED,
            ),
            0.0,
            448.0,
        ));
    p
}

fn manual_time_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let (values, timezone) = {
        let s = state.lock().unwrap();
        (s.manual_clock.values(), s.settings.timezone_minutes)
    };
    let mut p = Stack::new().push(panel(w, 334.0)).push(at(
        text("手动设置日期与时间", 28.0, MINT),
        24.0,
        20.0,
    ));
    let cw = (w - 64.0) / 5.0;
    for (i, label) in ["年", "月", "日", "时", "分"].into_iter().enumerate() {
        let x = 24.0 + i as f32 * (cw + 4.0);
        p = p.push(at(text(label, 22.0, MUTED), x, 72.0)).push(at(
            text(format!("{:02}", values[i]), 36.0, INK),
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
            MUTED,
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
        .push(at(
            CustomPaint::new(crate::widgets::AppIconPainter {
                asset: get_app_icon_asset("display").unwrap(),
                target_size: 112.0,
                pressed: None,
            })
            .size(Size::new(112.0, 112.0)),
            cx - 56.0,
            cy - 164.0,
        ))
        .push(at(
            Container::new()
                .width(size.width)
                .height(40.0)
                .child(Center::new(text("USB 副屏", 28.0, INK))),
            0.0,
            cy - 24.0,
        ))
        .push(at(
            Container::new()
                .width(size.width)
                .height(40.0)
                .child(Center::new(text(status, 22.0, MUTED))),
            0.0,
            cy + 26.0,
        ))
        .push(at(
            button("进入副屏", 180.0, 48.0, true, move || {
                edit(&state, |s| s.request_display_mode())
            }),
            cx - 90.0,
            cy + 86.0,
        ))
}
