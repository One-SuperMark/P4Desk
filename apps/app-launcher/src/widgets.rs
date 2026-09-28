//! Ported from esp32-rust-ui 0b75870835902cbf950505d1539dbb8f6e0a7197,
//! apps/app-launcher/src/widgets.rs (MIT). Icon scale and geometry follow the
//! actual screen size; the wallpaper below is an original geometric drawing.

use crate::app_icons::AppIconAsset;
use tiny_flutter::prelude::*;

pub struct AppIconPainter {
    pub asset: &'static AppIconAsset,
    pub target_size: f32,
}
impl CustomPainter for AppIconPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let rgb = self.asset.get_rgb565_slice();
        let target = self.target_size.min(size.width).min(size.height).max(1.0);
        let scale = target / self.asset.width.max(self.asset.height) as f32;
        let w = (self.asset.width as f32 * scale).round().max(1.0) as u32;
        let h = (self.asset.height as f32 * scale).round().max(1.0) as u32;
        let x = ((size.width - w as f32) * 0.5).round() as i32;
        let y = ((size.height - h as f32) * 0.5).round() as i32;
        if w == self.asset.width && h == self.asset.height {
            canvas.blit_image_565_with_alpha(x, y, w, h, rgb, self.asset.alpha);
        } else {
            canvas.blit_image_565_with_alpha_scaled(
                x,
                y,
                w,
                h,
                self.asset.width,
                self.asset.height,
                rgb,
                self.asset.alpha,
            );
        }
    }
}

/// Original icon + label launcher slot, sized for the landscape desktop grid.
pub fn build_app_icon(
    label: &'static str,
    asset: &'static AppIconAsset,
    icon_size: f32,
    on_tap: impl Fn() + Send + Sync + 'static,
) -> impl Widget {
    GestureDetector::new(
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
                        })
                        .size(Size::new(icon_size, icon_size)),
                    )
                    .push(SizedBox::square(8.0))
                    .push(Text::new(label).font_size(22.0).color(Color::WHITE)),
            ),
    )
    .on_tap(on_tap)
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
