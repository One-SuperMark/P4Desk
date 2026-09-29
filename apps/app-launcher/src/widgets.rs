//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/widgets.rs (MIT). Icon scale and geometry follow the
//! actual screen size; the wallpaper below is an original geometric drawing.

use crate::app_icons::AppIconAsset;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tiny_flutter::prelude::*;

pub struct AppIconPainter {
    pub asset: &'static AppIconAsset,
    pub target_size: f32,
    pub pressed: Option<Arc<AtomicBool>>,
}
impl CustomPainter for AppIconPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let pressed = self
            .pressed
            .as_ref()
            .is_some_and(|p| p.load(Ordering::Relaxed));
        let target = self.target_size.min(size.width).min(size.height).max(1.0)
            * if pressed { 0.94 } else { 1.0 };
        self.asset.paint(
            canvas,
            Rect::from_ltwh(
                (size.width - target) * 0.5,
                (size.height - target) * 0.5,
                target,
                target,
            ),
            Color::WHITE,
        );
        if pressed {
            // Follow the SVG's 8..120 tile silhouette, preserving the vector edge.
            let tile = target * 112.0 / 128.0;
            let rect = RRect::from_rect_circular(
                Rect::from_ltwh(
                    (size.width - tile) * 0.5,
                    (size.height - tile) * 0.5,
                    tile,
                    tile,
                ),
                target * 27.0 / 128.0,
            );
            canvas.draw_rrect_aa(rect, Color::WHITE.with_opacity(0.14));
            canvas.paint_rrect(
                rect,
                &tiny_gfx::Paint::new(Color::WHITE.with_opacity(0.7).to_gfx()),
                Some(1.3),
            );
        }
    }
}

/// Original icon + label launcher slot, sized for the landscape desktop grid.
pub fn build_app_icon(
    label: &'static str,
    asset: &'static AppIconAsset,
    icon_size: f32,
    on_tap: impl Fn(Rect) + Send + Sync + 'static,
) -> impl Widget {
    let pressed = Arc::new(AtomicBool::new(false));
    let bounds = Arc::new(Mutex::new(Rect::ZERO));
    let tapped_bounds = bounds.clone();
    let button = GestureDetector::new(
        Container::new()
            .width((icon_size + 32.0).max(160.0))
            .height(icon_size + 44.0)
            .child(
                Column::new()
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .push(
                        CustomPaint::new(AppIconPainter {
                            asset,
                            target_size: icon_size,
                            pressed: Some(pressed.clone()),
                        })
                        .size(Size::new(icon_size, icon_size)),
                    )
                    .push(SizedBox::square(8.0))
                    .push(Text::new(label).font_size(22.0).color(Color::WHITE)),
            ),
    )
    .on_tap(move || on_tap(*tapped_bounds.lock().unwrap()));
    IconPressFeedback {
        child: Box::new(button),
        pressed,
        bounds,
        icon_size,
    }
}

struct IconPressFeedback {
    child: Box<dyn Widget>,
    pressed: Arc<AtomicBool>,
    bounds: Arc<Mutex<Rect>>,
    icon_size: f32,
}
impl Widget for IconPressFeedback {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderIconPressFeedback {
            child: self.child.create_render_object(),
            pressed: self.pressed.clone(),
            bounds: self.bounds.clone(),
            icon_size: self.icon_size,
            origin: None,
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}
struct RenderIconPressFeedback {
    child: Box<dyn RenderBox>,
    pressed: Arc<AtomicBool>,
    bounds: Arc<Mutex<Rect>>,
    icon_size: f32,
    origin: Option<Point>,
    size: Size,
    offset: Offset,
}
impl RenderBox for RenderIconPressFeedback {
    fn layout(&mut self, c: &BoxConstraints) -> Size {
        self.size = self.child.layout(c);
        self.child.set_offset(Offset::ZERO);
        self.size
    }
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, o: Offset) {
        self.offset = o;
    }
    fn paint(&self, canvas: &mut Canvas, o: Offset) {
        let target = self.icon_size
            * if self.pressed.load(Ordering::Relaxed) {
                0.94
            } else {
                1.0
            };
        // `o` is the global painted slot origin, including the current page.
        // Record the visible SVG box, so launch animation starts at this icon.
        *self.bounds.lock().unwrap() = Rect::from_ltwh(
            o.dx + (self.size.width - target) * 0.5,
            o.dy + (self.icon_size - target) * 0.5,
            target,
            target,
        );
        self.child.paint(canvas, o);
    }
    fn hit_rect(&self, p: Point) -> Option<Rect> {
        self.child.hit_rect(p)
    }
    fn set_pressed_at(&mut self, p: Point, pressed: bool) {
        self.origin = pressed.then_some(p);
        self.pressed
            .store(pressed && self.hit_test(p), Ordering::Relaxed);
        self.child.set_pressed_at(p, pressed);
    }
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let handled = self.child.dispatch_touch(event);
        let was_pressed = self.pressed.load(Ordering::Relaxed);
        match *event {
            TouchEvent::Down(p) if handled => {
                self.origin = Some(p);
                self.pressed.store(true, Ordering::Relaxed);
            }
            TouchEvent::Move(p) => {
                if !self.hit_test(p)
                    || self
                        .origin
                        .is_some_and(|o| (p.x - o.x).abs() > 12.0 || (p.y - o.y).abs() > 12.0)
                {
                    self.pressed.store(false, Ordering::Relaxed);
                }
            }
            TouchEvent::Up(_) | TouchEvent::Cancel => {
                self.origin = None;
                self.pressed.store(false, Ordering::Relaxed);
            }
            _ => (),
        }
        handled || was_pressed
    }
}

/// Original WallpaperPainter role, without the upstream Sierra photograph.
pub struct WallpaperPainter;
impl CustomPainter for WallpaperPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        canvas.fill_dithered_horizontal_gradient((12, 28, 46), (18, 63, 78));
        canvas.draw_circle(
            Point::new(size.width * 0.91, size.height * 0.04),
            size.height * 0.58,
            Color::from_rgba(39, 106, 113, 48),
        );
        canvas.draw_circle(
            Point::new(size.width * 0.12, size.height * 1.02),
            size.height * 0.57,
            Color::from_rgba(56, 83, 137, 58),
        );
        canvas.draw_rect(
            Rect::from_ltwh(0.0, size.height - 44.0, size.width, 44.0),
            Color::from_rgba(7, 18, 29, 105),
        );
    }
}

/// Upstream active pill / inactive dot page indicators.
pub fn make_dot(active: bool) -> impl Widget {
    Container::new()
        .width(if active { 18.0 } else { 7.0 })
        .height(7.0)
        .border_radius(3.5)
        .color(if active {
            Color::from_rgb(240, 243, 246)
        } else {
            Color::from_rgba(255, 255, 255, 120)
        })
}
