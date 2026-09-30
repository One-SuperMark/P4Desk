//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/widgets.rs (MIT). Icon scale and geometry follow the
//! actual screen size; the wallpaper below is an original geometric drawing.

use crate::app_icons::AppIconAsset;
use crate::live_clock_icon::{paint_app_icon, ClockTime};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

pub struct GlyphPainter(pub &'static tiny_flutter::VectorIcon, pub Color);
impl CustomPainter for GlyphPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        self.0.paint(
            canvas,
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            self.1,
        );
    }
}
pub fn glyph(icon: &'static tiny_flutter::VectorIcon, size: f32, color: Color) -> CustomPaint {
    CustomPaint::new(GlyphPainter(icon, color)).size(Size::new(size, size))
}
pub fn app_badge(id: &str, size: f32) -> CustomPaint {
    CustomPaint::new(AppIconPainter {
        asset: crate::app_icons::get_app_icon_asset(id).expect("known application badge"),
        target_size: size,
        pressed: None,
        clock: None,
    })
    .size(Size::new(size, size))
}

pub struct AppIconPainter {
    pub asset: &'static AppIconAsset,
    pub target_size: f32,
    pub pressed: Option<Arc<AtomicBool>>,
    pub clock: Option<ClockTime>,
}
impl CustomPainter for AppIconPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let pressed = self
            .pressed
            .as_ref()
            .is_some_and(|p| p.load(Ordering::Relaxed));
        let target = self.target_size.min(size.width).min(size.height).max(1.0)
            * if pressed { 0.94 } else { 1.0 };
        paint_app_icon(
            self.asset,
            canvas,
            Rect::from_ltwh(
                (size.width - target) * 0.5,
                (size.height - target) * 0.5,
                target,
                target,
            ),
            self.clock,
            1.0,
        );
        if pressed {
            // Colloid rounded tile; feedback follows the supplied SVG outline.
            let tile = target * 112.0 / 128.0;
            let rect = RRect::from_rect_circular(
                Rect::from_ltwh(
                    (size.width - tile) * 0.5,
                    (size.height - tile) * 0.5,
                    tile,
                    tile,
                ),
                if matches!(
                    crate::icon_theme::current(),
                    crate::icon_theme::IconTheme::Colloid | crate::icon_theme::IconTheme::WhiteSur
                ) {
                    tile * 13.0 / 56.0
                } else {
                    tile * 0.5
                },
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
    clock: Option<ClockTime>,
    on_tap: impl Fn(Rect) + Send + Sync + 'static,
) -> impl Widget {
    icon_slot(
        label,
        asset,
        icon_size,
        (icon_size + 32.0).max(160.0),
        icon_size + 44.0,
        clock,
        on_tap,
    )
}

pub fn build_dock_icon(
    asset: &'static AppIconAsset,
    icon_size: f32,
    clock: Option<ClockTime>,
    on_tap: impl Fn(Rect) + Send + Sync + 'static,
) -> impl Widget {
    icon_slot("", asset, icon_size, 84.0, icon_size, clock, on_tap)
}
fn icon_slot(
    label: &'static str,
    asset: &'static AppIconAsset,
    icon_size: f32,
    width: f32,
    height: f32,
    clock: Option<ClockTime>,
    on_tap: impl Fn(Rect) + Send + Sync + 'static,
) -> impl Widget {
    let pressed = Arc::new(AtomicBool::new(false));
    let bounds = Arc::new(Mutex::new(Rect::ZERO));
    let tapped_bounds = bounds.clone();
    let button = GestureDetector::new(
        Container::new().width(width).height(height).child(
            Column::new()
                .main_axis_alignment(MainAxisAlignment::Start)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .push(
                    CustomPaint::new(AppIconPainter {
                        asset,
                        target_size: icon_size,
                        pressed: Some(pressed.clone()),
                        clock,
                    })
                    .size(Size::new(icon_size, icon_size)),
                )
                .push(SizedBox::square(8.0))
                .push(Text::new(label).font_size(22.0).color(Folio::ink())),
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
        canvas.cache_opaque_surface(
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            Folio::is_light() as u32,
            |canvas| {
                Self::paint_uncached(canvas, size);
            },
        );
    }
}
impl WallpaperPainter {
    fn paint_uncached(canvas: &mut Canvas, size: Size) {
        // Original vector landscape, inspired by Folio's muted natural palette.
        // Static and cacheable: no full-screen blur or animated wallpaper buffer.
        if Folio::is_light() {
            canvas.fill_dithered_horizontal_gradient((226, 231, 245), (182, 217, 232));
        } else {
            canvas.fill_dithered_horizontal_gradient((116, 111, 116), (62, 88, 109));
        }
        let colors = if Folio::is_light() {
            [0xcbd3e4, 0xb4cbdc, 0x92b8ce, 0x749daf]
        } else {
            [0x797b83, 0x525f70, 0x354f60, 0x253b4d]
        }
        .map(Color::from_hex);
        if size.width == 1024.0 && size.height == 600.0 {
            const SPANS: &[u8] = include_bytes!("../../../assets/wallpaper/folio-waves.spans");
            for row in SPANS.chunks_exact(8) {
                let y = u16::from_le_bytes([row[0], row[1]]) as f32;
                let x = u16::from_le_bytes([row[2], row[3]]) as f32;
                let width = u16::from_le_bytes([row[4], row[5]]) as f32;
                let rect = Rect::from_ltwh(x, y, width, 1.0);
                if canvas.is_rect_visible(rect) {
                    canvas.draw_rect(
                        rect,
                        colors[row[7] as usize].with_opacity(row[6] as f32 / 255.0),
                    );
                }
            }
            return;
        }
        let w = size.width;
        let h = size.height;
        for (level, color) in [
            (0.30, colors[0]),
            (0.46, colors[1]),
            (0.63, colors[2]),
            (0.82, colors[3]),
        ] {
            let mut path = tiny_gfx::PathBuilder::new();
            path.move_to(-20.0, h * level);
            path.cubic_to(
                w * 0.26,
                h * (level - 0.34),
                w * 0.52,
                h * (level + 0.38),
                w + 20.0,
                h * (level - 0.05),
            );
            path.line_to(w + 20.0, h + 20.0);
            path.line_to(-20.0, h + 20.0);
            path.close();
            if let Some(path) = path.finish() {
                canvas.fill_path(
                    &path,
                    &tiny_gfx::Paint::new(color.to_gfx()),
                    tiny_gfx::FillRule::Winding,
                );
            }
        }
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

#[cfg(test)]
mod wallpaper_cache_tests {
    use super::*;
    #[test]
    fn cached_wallpaper_matches_procedural_output_for_both_themes_and_clips() {
        for light in [false, true, false] {
            Folio::configure(light, 65);
            for clip in [None, None, Some(Rect::from_ltwh(97.0, 210.0, 455.0, 133.0))] {
                let render = |cached| {
                    let mut image = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
                    let mut canvas = Canvas::new(image.as_mut());
                    canvas.clear(Color::RED);
                    if let Some(clip) = clip {
                        canvas.clip_rect(clip);
                    }
                    if cached {
                        WallpaperPainter.paint(&mut canvas, Size::new(1024.0, 600.0));
                    } else {
                        WallpaperPainter::paint_uncached(&mut canvas, Size::new(1024.0, 600.0));
                    }
                    image
                };
                assert_eq!(render(true), render(false));
            }
        }
    }
}
