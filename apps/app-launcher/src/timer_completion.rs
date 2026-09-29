//! Bottom pop, circular theme expansion and a center-out glass reveal.
//! Timeline and gesture ownership use the actual monotonic timer deadline.
use crate::launcher_state::{ActiveApp, LauncherState};
use crate::timer::TimerService;
use p4desk_protocol::Mode;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

pub const COMPLETION_DURATION_MS: u64 = 3_100;
const POP_END_MS: f32 = 560.0;
const EXPAND_END_MS: f32 = 1_520.0;
const REVEAL_START_MS: f32 = 1_800.0;
const REVEAL_END_MS: f32 = 3_050.0;
const FRAME_INTERVAL_MS: u64 = 33;
const TAP_SLOP_SQUARED: f32 = 16.0 * 16.0;

struct CompletionTouch {
    origin: Point,
    travel_squared: f32,
}

#[derive(Default)]
pub struct TimerCompletionState {
    started_ms: Option<u64>,
    last_frame: Option<u64>,
    // Keep gesture ownership past the last animation frame. A held finger must
    // never release onto the Start button that appears beneath the overlay.
    touch: Option<CompletionTouch>,
}
impl TimerCompletionState {
    pub fn start(&mut self, deadline: u64, now_ms: u64) {
        self.started_ms =
            (now_ms.saturating_sub(deadline) < COMPLETION_DURATION_MS).then_some(deadline);
        self.last_frame = None;
        self.touch = None;
    }
    pub fn cancel(&mut self) {
        self.started_ms = None;
        self.last_frame = None;
        self.touch = None;
    }
    pub fn progress(&self, now_ms: u64) -> Option<f32> {
        let elapsed = now_ms.saturating_sub(self.started_ms?);
        (elapsed < COMPLETION_DURATION_MS).then_some(elapsed as f32 / COMPLETION_DURATION_MS as f32)
    }
    /// Throttle full-screen repaints and always submit the final clean frame.
    pub fn take_dirty(&mut self, now_ms: u64, screen_size: Size) -> Option<Rect> {
        let elapsed = now_ms.saturating_sub(self.started_ms?);
        if elapsed >= COMPLETION_DURATION_MS {
            self.started_ms = None;
            self.last_frame = None;
        } else {
            let frame = elapsed / FRAME_INTERVAL_MS;
            if self.last_frame == Some(frame) {
                return None;
            }
            self.last_frame = Some(frame);
        }
        Some(Rect::from_ltwh(
            0.0,
            0.0,
            screen_size.width,
            screen_size.height,
        ))
    }
}

fn sample(state: &LauncherState) -> Option<(f32, Color)> {
    (matches!(state.active_app, ActiveApp::Timer)
        && state.mode == Mode::Pad
        && state.settings.screen_on
        && state.timer.finished)
        .then(|| {
            state
                .timer_completion
                .progress(state.monotonic_ms)
                .map(|progress| (progress, completion_color(&state.timer)))
        })
        .flatten()
}

fn completion_color(timer: &TimerService) -> Color {
    crate::pomodoro_ui::accent(timer)
}

/// Wrap the existing timer page, including navigation. The fully covered hold
/// skips child painting; expansion and transparent reveal repaint the backdrop.
pub struct TimerCompletionOverlay {
    state: Arc<Mutex<LauncherState>>,
    child: Box<dyn Widget>,
}
impl TimerCompletionOverlay {
    pub fn new(state: Arc<Mutex<LauncherState>>, child: Box<dyn Widget>) -> Self {
        Self { state, child }
    }
}
impl Widget for TimerCompletionOverlay {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderCompletionOverlay {
            state: self.state.clone(),
            child: self.child.create_render_object(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}
struct RenderCompletionOverlay {
    state: Arc<Mutex<LauncherState>>,
    child: Box<dyn RenderBox>,
    size: Size,
    offset: Offset,
}
impl RenderCompletionOverlay {
    fn bounds(&self) -> Rect {
        Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height)
    }
    fn intercepting(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.timer_completion.touch.is_some() || sample(&state).is_some()
    }
}
impl RenderBox for RenderCompletionOverlay {
    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.size
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let animation = sample(&self.state.lock().unwrap());
        let opaque = animation.is_some_and(|(p, _)| {
            let ms = p * COMPLETION_DURATION_MS as f32;
            (EXPAND_END_MS..REVEAL_START_MS).contains(&ms)
        });
        if !opaque {
            self.child.paint(canvas, offset);
        }
        if let Some((progress, color)) = animation {
            canvas.save();
            canvas.translate(offset.dx, offset.dy);
            canvas.clip_rect(self.bounds());
            paint_completion(canvas, self.size, progress, color);
            canvas.restore();
        }
    }
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }
    fn hit_test(&self, point: Point) -> bool {
        if self.intercepting() {
            self.bounds().contains(point)
        } else {
            self.child.hit_test(point)
        }
    }
    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if self.intercepting() {
            self.bounds().contains(point).then(|| self.bounds())
        } else {
            self.child.hit_rect(point)
        }
    }
    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if !self.intercepting() {
            self.child.set_pressed_at(point, pressed);
        }
    }
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let bounds = self.bounds();
        let mut state = self.state.lock().unwrap();
        let active = sample(&state).is_some();
        let effect = &mut state.timer_completion;
        let owned = effect.touch.is_some();
        let handled = match *event {
            TouchEvent::Down(point) if active && bounds.contains(point) => {
                effect.touch = Some(CompletionTouch {
                    origin: point,
                    travel_squared: 0.0,
                });
                true
            }
            TouchEvent::Move(point) if owned => {
                let touch = effect.touch.as_mut().unwrap();
                touch.travel_squared = touch
                    .travel_squared
                    .max(distance_squared(touch.origin, point));
                true
            }
            TouchEvent::Up(point) if owned => {
                let touch = effect.touch.take().unwrap();
                if bounds.contains(point)
                    && touch
                        .travel_squared
                        .max(distance_squared(touch.origin, point))
                        <= TAP_SLOP_SQUARED
                {
                    effect.cancel();
                    state.changed();
                }
                true
            }
            TouchEvent::Cancel if owned => {
                effect.touch = None;
                true
            }
            // Also swallow an Up from a gesture that began before completion.
            TouchEvent::Up(_) | TouchEvent::Move(_) | TouchEvent::Cancel if active => true,
            _ => false,
        };
        drop(state);
        if handled {
            // Clear any underlying button pressed before the deadline.
            self.child.dispatch_touch(&TouchEvent::Cancel);
            true
        } else {
            self.child.dispatch_touch(event)
        }
    }
    fn needs_rebuild(&self) -> bool {
        self.child.needs_rebuild()
    }
}

fn distance_squared(a: Point, b: Point) -> f32 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.35 * t * t * t + 1.35 * t * t
}
fn paint_completion(canvas: &mut Canvas, size: Size, progress: f32, theme: Color) {
    use tiny_gfx::GlassFill;
    let ms = progress * COMPLETION_DURATION_MS as f32;
    let scale = (size.width / 1024.0).min(size.height / 600.0);
    let center = Point::new(size.width * 0.5, size.height * 0.5);
    let badge_radius = 51.0 * scale;
    let max_radius = (center.x.powi(2) + center.y.powi(2)).sqrt() + 14.0 * scale;
    let rim = 8.0 * scale;

    if ms < POP_END_MS {
        let pop = ease_out_back(ms / POP_END_MS);
        let bottom = size.height + badge_radius * 1.15;
        let position = Point::new(center.x, bottom + (center.y - bottom) * pop);
        let radius = badge_radius * (0.72 + 0.28 * pop);
        paint_badge(canvas, position, radius, theme, smooth(ms / 90.0));
        return;
    }
    if ms < EXPAND_END_MS {
        let wave = smooth((ms - POP_END_MS) / (EXPAND_END_MS - POP_END_MS));
        let radius = badge_radius + (max_radius - badge_radius) * wave;
        canvas.glass_circle(center, radius, theme, GlassFill::Inside, rim, 1.0);
    } else if ms < REVEAL_START_MS {
        canvas.draw_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height), theme);
    } else if ms < REVEAL_END_MS {
        let wave = smooth((ms - REVEAL_START_MS) / (REVEAL_END_MS - REVEAL_START_MS));
        // Outside tint leaves an expanding transparent window over the freshly
        // painted timer. The moving glass rim refracts its original source row.
        canvas.glass_circle(
            center,
            max_radius * wave,
            theme,
            GlassFill::Outside,
            rim,
            1.0,
        );
    }
    // Fade the white check during the full-color hold, before the clear aperture
    // opens. No icon is drawn over the revealed center or left behind at the end.
    if ms < REVEAL_START_MS {
        let opacity = 1.0 - smooth((ms - 1_640.0) / 160.0);
        paint_badge(canvas, center, badge_radius, theme, opacity);
    }
}
fn paint_badge(canvas: &mut Canvas, center: Point, radius: f32, theme: Color, opacity: f32) {
    if opacity <= 0.0 {
        return;
    }
    tiny_flutter::graphics::svg_icons_generated::UI_CHECK_CIRCLE.paint_with_opacity(
        canvas,
        Rect::from_ltwh(
            center.x - radius,
            center.y - radius,
            radius * 2.0,
            radius * 2.0,
        ),
        theme,
        opacity,
    );
}
