use crate::launcher_state::{ActiveApp, LauncherState, UiCommand};
use crate::timer::{Phase, TimerKind};
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

const BG: Color = Color::from_hex(0x11161e);
const SIDE: Color = Color::from_hex(0x171e28);
const CARD: Color = Color::from_hex(0x202a37);
const LINE: Color = Color::from_hex(0x344252);
const INK: Color = Color::from_hex(0xe8edf2);
const MUTED: Color = Color::from_hex(0x94a4b6);
const MINT: Color = Color::from_hex(0x8edbc5);
const WARM: Color = Color::from_hex(0xffbe82);
const NAV_W: f32 = 178.0;

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
fn launch_button(
    id: &'static str,
    title: &'static str,
    sub: &str,
    w: f32,
    h: f32,
    state: Arc<Mutex<LauncherState>>,
) -> impl Widget {
    let contents = Stack::new()
        .push(at(text(title, 28.0, INK), 18.0, 15.0))
        .push(at(text(sub, 18.0, MUTED), 18.0, 55.0));
    GestureDetector::new(panel(w, h).child(contents))
        .on_tap(move || edit(&state, |s| s.open_app(id)))
}

pub fn build_launcher_ui(state: Arc<Mutex<LauncherState>>, size: Size) -> impl Widget {
    let (active, clock, date, connected, sd_ready, notice, revision) = {
        let s = state.lock().unwrap();
        (
            s.active_app.clone(),
            s.clock.clone(),
            s.date.clone(),
            s.connected,
            s.sd_ready,
            s.notice.clone(),
            s.revision,
        )
    };
    let w = size.width.max(640.0);
    let h = size.height.max(480.0);
    let x = NAV_W + 24.0;
    let bw = w - x - 24.0;
    let title = match &active {
        ActiveApp::Launcher => "桌面助手",
        ActiveApp::Clock => "时钟",
        ActiveApp::Timer => "专注与计时",
        ActiveApp::Notes(_) => "便签",
        ActiveApp::Calculator(_) => "计算器",
        ActiveApp::MacControls => "Mac 控制",
        ActiveApp::Settings => "设置",
    };
    let mut root = Stack::new()
        .push(Container::new().width(w).height(h).color(BG))
        .push(Container::new().width(NAV_W).height(h).color(SIDE))
        .push(at(text("P4Desk", 28.0, MINT), 22.0, 26.0))
        .push(at(text("桌面辅助工具", 18.0, MUTED), 22.0, 67.0))
        .push(at(text(title, 28.0, INK), x, 25.0))
        .push(at(
            text(
                if connected {
                    "Mac 已连接"
                } else {
                    "离线可用"
                },
                18.0,
                if connected { MINT } else { MUTED },
            ),
            w - 158.0,
            28.0,
        ))
        .push(at(
            text(
                if sd_ready {
                    "TF 卡就绪"
                } else {
                    "基本工具模式"
                },
                18.0,
                MUTED,
            ),
            22.0,
            h - 71.0,
        ))
        .push(at(
            text(format!("{}", clock.get(..5).unwrap_or("--:--")), 22.0, INK),
            22.0,
            h - 39.0,
        ));
    for (i, (id, label)) in [
        ("home", "首页"),
        ("clock", "时钟"),
        ("timer", "番茄钟 / 计时"),
        ("notes", "便签"),
        ("calculator", "计算器"),
        ("mac", "Mac 控制"),
        ("settings", "设置"),
    ]
    .into_iter()
    .enumerate()
    {
        let selected = if id == "home" {
            matches!(active, ActiveApp::Launcher)
        } else {
            active.id() == Some(id)
        };
        let s = state.clone();
        root = root.push(at(
            button(label, 146.0, 46.0, selected, move || {
                edit(&s, |s| {
                    if id == "home" {
                        s.background_active_app()
                    } else {
                        s.open_app(id)
                    }
                })
            }),
            16.0,
            116.0 + i as f32 * 53.0,
        ));
    }
    let content: Box<dyn Widget> = match active.clone() {
        ActiveApp::Launcher => Box::new(home(state.clone(), bw, clock, date)),
        ActiveApp::Clock => Box::new(clock_page(state.clone(), bw, clock, date)),
        ActiveApp::Timer => Box::new(timer_page(state.clone(), bw)),
        ActiveApp::Notes(view) => Box::new(notes_page(state.clone(), view, bw)),
        ActiveApp::Calculator(calc) => {
            Box::new(Container::new().width(bw).height(h - 112.0).child(
                calculator::build_calculator_ui(calc, Size::new(bw, h - 112.0)),
            ))
        }
        ActiveApp::MacControls => Box::new(mac_page(state.clone(), bw)),
        ActiveApp::Settings => Box::new(settings_page(state.clone(), bw)),
    };
    root = root.push(at(
        Container::new().width(bw).height(h - 106.0).child(content),
        x,
        88.0,
    ));
    if !notice.is_empty() {
        root = root.push(at(
            Container::new()
                .width(bw)
                .height(38.0)
                .color(LINE)
                .border_radius(8.0)
                .padding(EdgeInsets::all(6.0))
                .child(text(short(&notice, 42), 18.0, WARM)),
            x,
            h - 43.0,
        ));
    }
    let back = state.clone();
    let kill = state.clone();
    let _ = revision;
    BackListener::new(root, move || edit(&back, |s| s.background_active_app()))
        .on_kill(move || edit(&kill, |s| s.kill_active_app()))
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

fn home(state: Arc<Mutex<LauncherState>>, w: f32, clock: String, date: String) -> Stack {
    let top_w = w * 0.60 - 8.0;
    let side_w = w - top_w - 16.0;
    let (timer, notes, running) = {
        let s = state.lock().unwrap();
        (
            s.timer.display(),
            s.snapshot.notes.len(),
            s.running_apps.len(),
        )
    };
    let mut p = Stack::new()
        .push(at(panel(top_w, 202.0), 0.0, 0.0))
        .push(at(text("今天", 18.0, MINT), 22.0, 16.0))
        .push(at(text(clock, 68.0, INK), 22.0, 55.0))
        .push(at(text(date, 18.0, MUTED), 22.0, 156.0))
        .push(at(panel(side_w, 202.0), top_w + 16.0, 0.0))
        .push(at(text("当前专注", 18.0, WARM), top_w + 38.0, 16.0))
        .push(at(text(timer, 54.0, INK), top_w + 38.0, 59.0));
    let s = state.clone();
    p = p.push(at(
        button("打开番茄钟", side_w - 44.0, 46.0, false, move || {
            edit(&s, |s| s.open_app("timer"))
        }),
        top_w + 38.0,
        137.0,
    ));
    p = p.push(at(text("常用工具", 22.0, INK), 0.0, 225.0)).push(at(
        text(
            format!("{notes} 条便签  /  {running} 个后台工具"),
            18.0,
            MUTED,
        ),
        w - 282.0,
        231.0,
    ));
    let cw = (w - 32.0) / 3.0;
    for (i, (id, label, sub)) in [
        ("clock", "时钟", "日期与本地时间"),
        ("timer", "番茄钟", "专注 · 休息 · 倒计时"),
        ("notes", "便签", "Mac 编辑，随时查看"),
        ("calculator", "计算器", "保留上次计算"),
        ("mac", "Mac 控制", "快捷键 · 应用 · 媒体"),
        ("settings", "设置", "屏幕与 USB 副屏"),
    ]
    .into_iter()
    .enumerate()
    {
        p = p.push(at(
            launch_button(id, label, sub, cw, 96.0, state.clone()),
            i as f32 % 3.0 * (cw + 16.0),
            269.0 + (i / 3) as f32 * 112.0,
        ));
    }
    p
}
fn clock_page(state: Arc<Mutex<LauncherState>>, w: f32, clock: String, date: String) -> Stack {
    let s = state.clone();
    Stack::new()
        .push(panel(w, 406.0))
        .push(at(text("本地时间", 22.0, MINT), 28.0, 24.0))
        .push(at(text(clock, 100.0, INK), 28.0, 120.0))
        .push(at(text(date, 28.0, MUTED), 28.0, 259.0))
        .push(at(
            button("从 Mac 校时", 200.0, 50.0, false, move || {
                edit(&s, |s| s.queue(UiCommand::RequestTimeSync))
            }),
            28.0,
            326.0,
        ))
        .push(at(
            text("离线时继续走时；重新上电后连接 Mac 校时", 18.0, MUTED),
            28.0,
            430.0,
        ))
}
fn timer_page(state: Arc<Mutex<LauncherState>>, w: f32) -> Stack {
    let t = state.lock().unwrap().timer.clone();
    let kind = match t.kind {
        TimerKind::Pomodoro => "番茄钟",
        TimerKind::Countdown => "倒计时",
    };
    let phase = match (t.kind, t.phase) {
        (TimerKind::Countdown, _) => "自由计时",
        (_, Phase::Focus) => "专注 25 分钟",
        (_, Phase::Break) => "休息 5 分钟",
    };
    let mut p = Stack::new()
        .push(panel(w, 308.0))
        .push(at(text(kind, 22.0, MINT), 28.0, 22.0))
        .push(at(text(phase, 22.0, MUTED), w - 232.0, 22.0))
        .push(at(text(t.display(), 100.0, INK), 28.0, 82.0))
        .push(at(
            text(format!("已完成 {} 次专注", t.completed_cycles), 18.0, MUTED),
            28.0,
            237.0,
        ));
    if t.finished {
        p = p.push(at(text("计时完成", 28.0, WARM), w - 180.0, 147.0));
    }
    let s = state.clone();
    p = p.push(at(
        button(
            if t.is_running() {
                "暂停"
            } else if t.finished {
                "开始下一段"
            } else {
                "开始"
            },
            220.0,
            58.0,
            true,
            move || edit(&s, |s| s.timer.toggle(s.monotonic_ms)),
        ),
        0.0,
        328.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("重置", 150.0, 58.0, false, move || {
            edit(&s, |s| s.timer.reset())
        }),
        236.0,
        328.0,
    ));
    let s = state.clone();
    p = p.push(at(
        button("番茄钟", 150.0, 54.0, false, move || {
            edit(&s, |s| s.timer.set_pomodoro())
        }),
        0.0,
        406.0,
    ));
    for (i, m) in [1u64, 5, 10, 25].into_iter().enumerate() {
        let s = state.clone();
        p = p.push(at(
            button(format!("{m} 分钟"), 132.0, 54.0, false, move || {
                edit(&s, |s| s.timer.set_countdown(m))
            }),
            166.0 + i as f32 * 148.0,
            406.0,
        ));
    }
    p
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
                Container::new()
                    .width(w - 48.0)
                    .height(79.0)
                    .child(text(&n.title, 28.0, MINT).wrap()),
                24.0,
                17.0,
            ))
            .push(at(
                Container::new().width(w - 48.0).height(271.0).child(
                    SingleChildScrollView::new(text(&n.body, 22.0, INK).wrap()).controller(scroll),
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
            ElevatedButton::new(text(short(&b.label, 26), 18.0, INK).wrap())
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
                format!("资源代次：{generation}    轻触屏幕可唤醒"),
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
