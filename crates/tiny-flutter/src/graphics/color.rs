use crate::graphics::geometry::Rect;
use crate::tiny_gfx;

/// 32-bit RGBA Color structure compatible with Flutter style definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color::from_rgba(0, 0, 0, 0);
    pub const BLACK: Color = Color::from_rgb(0, 0, 0);
    pub const WHITE: Color = Color::from_rgb(255, 255, 255);
    pub const RED: Color = Color::from_rgb(244, 67, 54);
    pub const GREEN: Color = Color::from_rgb(76, 175, 80);
    pub const BLUE: Color = Color::from_rgb(33, 150, 243);
    pub const CYAN: Color = Color::from_rgb(0, 188, 212);
    pub const YELLOW: Color = Color::from_rgb(255, 235, 59);
    pub const ORANGE: Color = Color::from_rgb(255, 152, 0);
    pub const PURPLE: Color = Color::from_rgb(156, 39, 176);
    pub const GREY: Color = Color::from_rgb(158, 158, 158);
    pub const DARK_GREY: Color = Color::from_rgb(48, 48, 48);
    pub const LIGHT_GREY: Color = Color::from_rgb(220, 220, 220);

    pub const PRIMARY: Color = Color::from_rgb(30, 136, 229);
    pub const ACCENT: Color = Color::from_rgb(255, 64, 129);
    pub const BACKGROUND: Color = Color::from_rgb(18, 18, 18);
    pub const SURFACE: Color = Color::from_rgb(30, 30, 30);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Flutter style 0xAARRGGBB hex representation.
    pub const fn from_argb(argb: u32) -> Self {
        let a = ((argb >> 24) & 0xFF) as u8;
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        Self { r, g, b, a }
    }

    /// 0xRRGGBB hex with full opacity.
    pub const fn from_hex(rgb: u32) -> Self {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        Self { r, g, b, a: 255 }
    }

    /// Modify alpha channel (0.0 to 1.0).
    pub fn with_opacity(self, opacity: f32) -> Self {
        let clamped = opacity.clamp(0.0, 1.0);
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (255.0 * clamped).round() as u8,
        }
    }

    /// Convert to 16-bit RGB565 representation (5-bit Red, 6-bit Green, 5-bit Blue).
    #[inline(always)]
    pub fn to_rgb565(&self) -> u16 {
        rgb888_to_rgb565(self.r, self.g, self.b)
    }

    /// Convert to tiny-gfx Color.
    #[inline(always)]
    pub fn to_gfx(&self) -> tiny_gfx::Color {
        tiny_gfx::Color::from_rgba(self.r, self.g, self.b, self.a)
    }

    /// Alias for backwards compatibility with any remaining to_skia calls.
    #[inline(always)]
    pub fn to_skia(&self) -> tiny_gfx::Color {
        self.to_gfx()
    }
}

/// Convert single 24-bit RGB888 pixel to 16-bit RGB565.
#[inline(always)]
pub fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    (((r as u16) & 0xF8) << 8) | (((g as u16) & 0xFC) << 3) | ((b as u16) >> 3)
}

/// High-speed extraction: Extracts a rectangular area from a Pixmap565
/// and writes contiguous RGB565 pixels into `dst`.
///
/// Returns (x1, y1, x2, y2, pixel_count).
pub fn extract_rect_to_rgb565(
    pixmap: &tiny_gfx::Pixmap565,
    rect: Rect,
    dst: &mut [u16],
) -> (i32, i32, i32, i32, usize) {
    let gfx_rect = tiny_gfx::Rect::from_ltwh(rect.x, rect.y, rect.width, rect.height);
    pixmap.extract_rect(gfx_rect, dst)
}
