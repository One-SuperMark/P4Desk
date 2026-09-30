//! Minimal landscape timer, rendered by the same Rust widgets as the firmware.
use crate::flip_clock::{ClockControl, ClockControlPainter};
use crate::launcher_state::LauncherState;
use crate::timer::{Phase, TimerKind, TimerService};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

pub const TIMER_BG: Color = Folio::BG;

const FOCUS: Color = Color::from_hex(0xd07969);

const LONG_BREAK: Color = Color::from_hex(0x9dbded);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    Kind(TimerKind),
    Phase(Phase),
    SecondsPreset(u64),
    Preset(u64),
    Toggle,
    Reset,
    Next,
}
#[derive(Debug, Clone)]
pub struct TimerKey {
    pub action: TimerAction,
    pub rect: Rect,
}
pub struct TimerLayout {
    pub keys: Vec<TimerKey>,
    pub digits: Rect,
    pub progress: Rect,
    pub status: Rect,
    pub cycles: Rect,
    pub scale: f32,
}
impl TimerLayout {
    pub fn key(&self, action: TimerAction) -> Option<&TimerKey> {
        self.keys.iter().find(|key| key.action == action)
    }
}
pub fn timer_layout(timer: &TimerService, size: Size) -> TimerLayout {
    let scale = (size.width / 976.0).min(size.height / 506.0).max(0.1);
    let centered = |width: f32, y: f32, height: f32| {
        Rect::from_ltwh(
            (size.width - width * scale) * 0.5,
            y * scale,
            width * scale,
            height * scale,
        )
    };
    let choices: Vec<_> = match timer.kind {
        TimerKind::Pomodoro => Phase::ALL.into_iter().map(TimerAction::Phase).collect(),
        TimerKind::Countdown => vec![
            TimerAction::SecondsPreset(10),
            TimerAction::Preset(1),
            TimerAction::Preset(5),
            TimerAction::Preset(10),
            TimerAction::Preset(25),
        ],
    };
    let key_width = if timer.kind == TimerKind::Pomodoro {
        142.0
    } else {
        108.0
    };
    let strip = centered(choices.len() as f32 * (key_width + 12.0) - 12.0, 20.0, 46.0);
    let mut keys: Vec<_> = choices
        .into_iter()
        .enumerate()
        .map(|(i, action)| TimerKey {
            action,
            rect: Rect::from_ltwh(
                strip.x + i as f32 * (key_width + 12.0) * scale,
                strip.y,
                key_width * scale,
                strip.height,
            ),
        })
        .collect();
    let primary = centered(232.0, 368.0, 64.0);
    keys.push(TimerKey {
        action: TimerAction::Toggle,
        rect: primary,
    });
    keys.push(TimerKey {
        action: TimerAction::Reset,
        rect: Rect::from_ltwh(
            primary.x - 120.0 * scale,
            primary.y + 4.0 * scale,
            100.0 * scale,
            56.0 * scale,
        ),
    });
    if timer.kind == TimerKind::Pomodoro {
        keys.push(TimerKey {
            action: TimerAction::Next,
            rect: Rect::from_ltwh(
                primary.right() + 20.0 * scale,
                primary.y + 4.0 * scale,
                100.0 * scale,
                56.0 * scale,
            ),
        });
    }
    TimerLayout {
        keys,
        digits: centered(720.0, 88.0, 190.0),
        progress: centered(512.0, 291.0, 4.0),
        status: centered(720.0, 310.0, 38.0),
        cycles: centered(512.0, 460.0, 30.0),
        scale,
    }
}
pub fn timer_mode_bounds(width: f32) -> Rect {
    Rect::from_ltwh(width - 328.0, 9.0, 232.0, 44.0)
}
pub fn timer_mode_keys(size: Size) -> Vec<TimerKey> {
    [TimerKind::Pomodoro, TimerKind::Countdown]
        .into_iter()
        .enumerate()
        .map(|(i, kind)| TimerKey {
            action: TimerAction::Kind(kind),
            rect: Rect::from_ltwh(
                i as f32 * (size.width + 8.0) * 0.5,
                0.0,
                (size.width - 8.0) * 0.5,
                size.height,
            ),
        })
        .collect()
}
pub(crate) fn accent(timer: &TimerService) -> Color {
    match (timer.kind, timer.phase) {
        (TimerKind::Countdown, _) | (_, Phase::Focus) => FOCUS,
        (_, Phase::Break) => Folio::accent_ink(),
        (_, Phase::LongBreak) => LONG_BREAK,
    }
}
fn phase_accent(phase: Phase) -> Color {
    match phase {
        Phase::Focus => FOCUS,
        Phase::Break => Folio::accent_ink(),
        Phase::LongBreak => LONG_BREAK,
    }
}
pub fn primary_label(timer: &TimerService) -> &'static str {
    if timer.is_running() {
        "暂停"
    } else if timer.finished && timer.kind == TimerKind::Pomodoro {
        match timer.next_phase() {
            Phase::Focus => "开始专注",
            Phase::Break => "开始短休息",
            Phase::LongBreak => "开始长休息",
        }
    } else if timer.finished {
        "再次开始"
    } else if timer.remaining_ms < timer.duration_ms() {
        "继续"
    } else {
        "开始"
    }
}
pub fn status_label(timer: &TimerService) -> &'static str {
    if timer.finished {
        if timer.kind == TimerKind::Countdown {
            return "计时完成";
        }
        return match timer.next_phase() {
            Phase::Focus => "休息结束 · 准备专注",
            Phase::Break => "专注完成 · 准备短休息",
            Phase::LongBreak => "专注完成 · 准备长休息",
        };
    }
    if !timer.is_running() && timer.remaining_ms < timer.duration_ms() {
        return "已暂停";
    }
    match (timer.kind, timer.phase, timer.is_running()) {
        (TimerKind::Countdown, _, true) => "倒计时进行中",
        (TimerKind::Countdown, _, false) => "准备开始",
        (_, Phase::Focus, true) => "保持专注",
        (_, Phase::Focus, false) => "准备专注",
        (_, _, true) => "放松一下",
        (_, _, false) => "准备休息",
    }
}
fn apply(state: &Arc<Mutex<LauncherState>>, action: TimerAction) {
    if let Ok(mut state) = state.lock() {
        let now = state.monotonic_ms;
        match action {
            TimerAction::Kind(kind) if kind == state.timer.kind => return,
            TimerAction::Kind(TimerKind::Pomodoro) => state.timer.set_pomodoro(),
            TimerAction::Kind(TimerKind::Countdown) => {
                let seconds = state.timer.countdown_ms / 1_000;
                state.timer.set_countdown_seconds(seconds);
            }
            TimerAction::Phase(phase) => state.timer.set_phase(phase),
            TimerAction::SecondsPreset(seconds) => {
                if state.timer.countdown_ms == seconds * 1_000 {
                    return;
                }
                state.timer.set_countdown_seconds(seconds);
            }
            TimerAction::Preset(minutes) => {
                if state.timer.countdown_ms == minutes * 60_000 {
                    return;
                }
                state.timer.set_countdown(minutes);
            }
            TimerAction::Toggle => state.timer.toggle(now),
            TimerAction::Reset => state.timer.reset(),
            TimerAction::Next => state.timer.next(now),
        }
        state.changed();
    }
}
fn at(child: impl Widget + 'static, rect: Rect) -> Positioned {
    Positioned::new(child).left(rect.x).top(rect.y)
}
struct LabelPainter {
    label: String,
    px: f32,
    color: Color,
}
impl CustomPainter for LabelPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let font = Font::default_font();
        let upper = self
            .label
            .chars()
            .map(|ch| {
                let g = font.glyph(ch, self.px);
                g.ymin + g.height as i32
            })
            .max()
            .unwrap_or(0) as f32;
        let lower = self
            .label
            .chars()
            .map(|ch| font.glyph(ch, self.px).ymin)
            .min()
            .unwrap_or(0) as f32;
        canvas.draw_text(
            &self.label,
            font,
            self.px,
            Point::new(
                (size.width - font.measure_text(&self.label, self.px).width) * 0.5,
                (size.height - upper + lower) * 0.5 + upper - font.cap_height(self.px),
            ),
            self.color,
        );
    }
}
fn label(label: impl Into<String>, px: f32, color: Color, size: Size) -> CustomPaint {
    CustomPaint::new(LabelPainter {
        label: label.into(),
        px,
        color,
    })
    .size(size)
}
fn key_button(
    state: Arc<Mutex<LauncherState>>,
    key: &TimerKey,
    caption: String,
    px: f32,
    color: Color,
    fill: Color,
) -> ElevatedButton {
    let action = key.action;
    ElevatedButton::new(label(
        caption,
        px,
        color,
        Size::new(key.rect.width, key.rect.height),
    ))
    .style(
        ButtonStyle::new()
            .size(key.rect.width, key.rect.height)
            .color(fill)
            .pressed_color(Folio::pressed(fill))
            .border_radius(key.rect.height * 0.5)
            .antialias(true)
            .padding(EdgeInsets::ZERO),
    )
    .on_pressed(move || apply(&state, action))
}
pub fn build_timer_navigation(state: Arc<Mutex<LauncherState>>, width: f32) -> Stack {
    let timer = state.lock().unwrap().timer.clone();
    let bounds = timer_mode_bounds(width);
    let mut bar = Stack::new().push(at(
        Container::new()
            .width(bounds.width + 8.0)
            .height(bounds.height + 8.0)
            .color(Folio::raised())
            .border_radius((bounds.height + 8.0) * 0.5),
        Rect::from_ltwh(
            bounds.x - 4.0,
            bounds.y - 4.0,
            bounds.width + 8.0,
            bounds.height + 8.0,
        ),
    ));
    for (control, x) in [
        (ClockControl::Home, 24.0),
        (ClockControl::Close, width - 72.0),
    ] {
        let state = state.clone();
        let icon = ElevatedButton::new(
            CustomPaint::new(ClockControlPainter::new(control)).size(Size::new(24.0, 24.0)),
        )
        .style(Folio::navigation_style())
        .on_pressed(move || {
            if let Ok(mut state) = state.lock() {
                match control {
                    ClockControl::Home => state.background_active_app(),
                    ClockControl::Close => state.kill_active_app(),
                }
            }
        });
        bar = bar.push(at(icon, Rect::from_ltwh(x, 6.0, 48.0, 48.0)));
    }
    for mut key in timer_mode_keys(Size::new(bounds.width, bounds.height)) {
        let TimerAction::Kind(kind) = key.action else {
            unreachable!()
        };
        key.rect = key.rect.shift(Offset::new(bounds.x, bounds.y));
        let selected = timer.kind == kind;
        bar = bar.push(at(
            key_button(
                state.clone(),
                &key,
                match kind {
                    TimerKind::Pomodoro => "番茄钟",
                    TimerKind::Countdown => "倒计时",
                }
                .into(),
                18.0,
                if selected {
                    Folio::ink()
                } else {
                    Folio::muted()
                },
                if selected {
                    Folio::raised()
                } else {
                    Color::TRANSPARENT
                },
            ),
            key.rect,
        ));
    }
    bar
}
pub fn build_timer_ui(state: Arc<Mutex<LauncherState>>, size: Size) -> Stack {
    let timer = state.lock().unwrap().timer.clone();
    let layout = timer_layout(&timer, size);
    let mut page = Stack::new().push(
        CustomPaint::new(TimerFace {
            timer: timer.clone(),
        })
        .size(size),
    );
    for key in layout.keys {
        let (caption, color, fill, px) = match key.action {
            TimerAction::Phase(phase) => (
                phase.label().into(),
                if timer.phase == phase {
                    phase_accent(phase)
                } else {
                    Folio::muted()
                },
                if timer.phase == phase {
                    Folio::raised()
                } else {
                    Color::TRANSPARENT
                },
                22.0,
            ),
            TimerAction::SecondsPreset(_) | TimerAction::Preset(_) => {
                let (caption, duration_ms) = match key.action {
                    TimerAction::SecondsPreset(seconds) => {
                        (format!("{seconds} 秒"), seconds * 1_000)
                    }
                    TimerAction::Preset(minutes) => (format!("{minutes} 分钟"), minutes * 60_000),
                    _ => unreachable!(),
                };
                (
                    caption,
                    if timer.countdown_ms == duration_ms {
                        FOCUS
                    } else {
                        Folio::muted()
                    },
                    if timer.countdown_ms == duration_ms {
                        Folio::raised()
                    } else {
                        Color::TRANSPARENT
                    },
                    22.0,
                )
            }
            TimerAction::Toggle => (
                primary_label(&timer).into(),
                Folio::on_fill(accent(&timer)),
                accent(&timer),
                24.0,
            ),
            TimerAction::Reset => ("重置".into(), Folio::ink(), Folio::raised(), 18.0),
            TimerAction::Next => ("下一段".into(), Folio::ink(), Folio::raised(), 18.0),
            TimerAction::Kind(_) => unreachable!(),
        };
        page = page.push(at(
            key_button(state.clone(), &key, caption, px * layout.scale, color, fill),
            key.rect,
        ));
    }
    page
}
struct TimerFace {
    timer: TimerService,
}
impl CustomPainter for TimerFace {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let layout = timer_layout(&self.timer, size);
        canvas.draw_rect(
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            Folio::bg(),
        );
        let mut choices = layout.keys.iter().filter(|key| {
            matches!(
                key.action,
                TimerAction::Phase(_) | TimerAction::Preset(_) | TimerAction::SecondsPreset(_)
            )
        });
        let first = choices.next();
        let last = choices.last().or(first);
        if let (Some(first), Some(last)) = (first, last) {
            let inset = 4.0 * layout.scale;
            canvas.draw_rrect_aa(
                RRect::from_rect_circular(
                    Rect::from_ltwh(
                        first.rect.x - inset,
                        first.rect.y - inset,
                        last.rect.right() - first.rect.x + 2.0 * inset,
                        first.rect.height + 2.0 * inset,
                    ),
                    first.rect.height * 0.5 + inset,
                ),
                Folio::raised(),
            );
        }
        canvas.draw_rrect_aa(
            RRect::from_rect_circular(
                Rect::from_ltwh(
                    40.0 * layout.scale,
                    80.0 * layout.scale,
                    size.width - 80.0 * layout.scale,
                    268.0 * layout.scale,
                ),
                Folio::CARD_RADIUS,
            ),
            Folio::raised(),
        );
        let font = Font::default_font();
        let px = 180.0 * layout.scale;
        let cell = ('0'..='9')
            // Measure unused digits without rasterizing all ten large masks
            // during the first fade into the timer page.
            .map(|ch| font.advance(ch, px))
            .fold(0.0, f32::max);
        let value = self.timer.display();
        let width = value
            .chars()
            .map(|ch| if ch == ':' { cell * 0.48 } else { cell })
            .sum::<f32>();
        let mut x = layout.digits.x + (layout.digits.width - width) * 0.5;
        let g = font.glyph('0', px);
        let baseline = layout.digits.y
            + (layout.digits.height - g.height as f32) * 0.5
            + g.ymin as f32
            + g.height as f32;
        for ch in value.chars() {
            let advance = if ch == ':' { cell * 0.48 } else { cell };
            let glyph = font.glyph(ch, px);
            // Fixed digit cells keep the timer stable, even with proportional fonts.
            canvas.draw_text(
                &ch.to_string(),
                font,
                px,
                Point::new(
                    x + (advance - glyph.advance) * 0.5,
                    baseline - font.cap_height(px),
                ),
                Folio::ink(),
            );
            x += advance;
        }
        canvas.draw_rrect_aa(
            RRect::from_rect_circular(layout.progress, layout.progress.height * 0.5),
            Folio::line(),
        );
        let elapsed = 1.0
            - self.timer.remaining_ms.min(self.timer.duration_ms()) as f32
                / self.timer.duration_ms().max(1) as f32;
        if elapsed > 0.0 {
            canvas.draw_rrect_aa(
                RRect::from_rect_circular(
                    Rect::from_ltwh(
                        layout.progress.x,
                        layout.progress.y,
                        layout.progress.width * elapsed,
                        layout.progress.height,
                    ),
                    layout.progress.height * 0.5,
                ),
                accent(&self.timer),
            );
        }
        canvas.save();
        canvas.translate(layout.status.x, layout.status.y);
        LabelPainter {
            label: status_label(&self.timer).into(),
            px: 22.0 * layout.scale,
            color: if self.timer.finished {
                accent(&self.timer)
            } else {
                Folio::muted()
            },
        }
        .paint(canvas, Size::new(layout.status.width, layout.status.height));
        canvas.restore();
        if self.timer.kind == TimerKind::Pomodoro {
            let count = if self.timer.completed_cycles > 0
                && (self.timer.phase != Phase::Focus || self.timer.finished)
            {
                (self.timer.completed_cycles - 1) % 4 + 1
            } else {
                self.timer.completed_cycles % 4
            };
            let summary = format!("已完成 {} 次专注", self.timer.completed_cycles);
            let px = 18.0 * layout.scale;
            let width = font.measure_text(&summary, px).width + 122.0 * layout.scale;
            let x = (size.width - width) * 0.5;
            for i in 0..4 {
                let r = Rect::from_ltwh(
                    x + i as f32 * 24.0 * layout.scale,
                    layout.cycles.y + 9.0 * layout.scale,
                    12.0 * layout.scale,
                    12.0 * layout.scale,
                );
                let active = i == count && self.timer.phase == Phase::Focus && !self.timer.finished;
                canvas.draw_rrect_aa(
                    RRect::from_rect_circular(r, r.width * 0.5),
                    if i < count || active {
                        FOCUS
                    } else {
                        Folio::line()
                    },
                );
                if active {
                    canvas.draw_rrect_aa(
                        RRect::from_rect_circular(r.deflate(2.0 * layout.scale), r.width * 0.5),
                        Folio::bg(),
                    );
                }
            }
            canvas.draw_text(
                &summary,
                font,
                px,
                Point::new(
                    x + 122.0 * layout.scale,
                    layout.cycles.y + 2.0 * layout.scale,
                ),
                Folio::muted(),
            );
        }
    }
}
