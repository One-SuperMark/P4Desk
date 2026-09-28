use crate::graphics::color::Color;
use crate::graphics::font::Font;
use crate::graphics::geometry::{Point, RRect, Rect};
use crate::tiny_gfx::{self, FillRule, Paint, Path, Pixmap565Mut, Stroke};

/// A 2D drawing canvas wrapping tiny-gfx Pixmap565Mut with a Flutter-like drawing API.
pub struct Canvas<'a> {
    inner: tiny_gfx::Canvas<'a>,
}

impl<'a> Canvas<'a> {
    pub fn new(pixmap: Pixmap565Mut<'a>) -> Self {
        Self {
            inner: tiny_gfx::Canvas::new(pixmap),
        }
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Save current canvas transform and clip state.
    #[inline(always)]
    pub fn save(&mut self) {
        self.inner.save();
    }

    /// Restore the most recently saved state.
    #[inline(always)]
    pub fn restore(&mut self) {
        self.inner.restore();
    }

    /// Translate canvas coordinate origin.
    #[inline(always)]
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.inner.translate(dx, dy);
    }

    /// Scale canvas coordinate system.
    #[inline(always)]
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.inner.scale(sx, sy);
    }

    /// Stroke a custom vector path with specified paint and stroke properties.
    #[inline(always)]
    pub fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) {
        self.inner.stroke_path(path, paint, stroke);
    }

    /// Fill a custom vector path with specified paint and fill rule.
    #[inline(always)]
    pub fn fill_path(&mut self, path: &Path, paint: &Paint, fill_rule: FillRule) {
        self.inner.fill_path(path, paint, fill_rule);
    }

    /// Clip current drawing region to an axis-aligned rectangle.
    #[inline(always)]
    pub fn clip_rect(&mut self, rect: Rect) {
        self.inner.clip_rect(tiny_gfx::Rect::from_ltwh(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        ));
    }

    /// Retrieve the current active clipping rectangle, if any.
    #[inline(always)]
    pub fn current_clip(&self) -> Option<Rect> {
        self.inner
            .current_clip()
            .map(|r| Rect::from_ltwh(r.x, r.y, r.width, r.height))
    }

    /// Clear entire canvas with a single color.
    #[inline(always)]
    pub fn clear(&mut self, color: Color) {
        self.inner.clear(color.to_gfx());
    }

    /// Fast blit of precomputed raw RGB565 pixels onto the canvas.
    #[inline(always)]
    pub fn copy_pixels_from(&mut self, src: &[u16]) {
        self.inner.copy_pixels_from(src);
    }

    /// Fill the entire canvas with a high-fidelity dithered horizontal gradient (eliminates banding).
    #[inline(always)]
    pub fn fill_dithered_horizontal_gradient(&mut self, c0: (u8, u8, u8), c1: (u8, u8, u8)) {
        self.inner.fill_dithered_horizontal_gradient(c0, c1);
    }

    /// Blit an opaque 16-bit RGB565 sub-image onto the canvas.
    #[inline(always)]
    pub fn blit_image_565(&mut self, x: i32, y: i32, w: u32, h: u32, pixels: &[u16]) {
        self.inner.blit_image_565(x, y, w, h, pixels);
    }

    /// Blit a 16-bit RGB565 sub-image with an 8-bit alpha mask onto the canvas.
    #[inline(always)]
    pub fn blit_image_565_with_alpha(
        &mut self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        rgb: &[u16],
        alpha: &[u8],
    ) {
        self.inner.blit_image_565_with_alpha(x, y, w, h, rgb, alpha);
    }

    /// Blit a 16-bit RGB565 sub-image with an 8-bit alpha mask scaled to (dst_w, dst_h) onto the canvas.
    #[inline(always)]
    pub fn blit_image_565_with_alpha_scaled(
        &mut self,
        x: i32,
        y: i32,
        dst_w: u32,
        dst_h: u32,
        src_w: u32,
        src_h: u32,
        rgb: &[u16],
        alpha: &[u8],
    ) {
        self.inner
            .blit_image_565_with_alpha_scaled(x, y, dst_w, dst_h, src_w, src_h, rgb, alpha);
    }

    /// Draw a filled rectangle.
    #[inline(always)]
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.inner.draw_rect(
            tiny_gfx::Rect::from_ltwh(rect.x, rect.y, rect.width, rect.height),
            color.to_gfx(),
        );
    }

    /// Draw a filled rounded rectangle.
    #[inline(always)]
    pub fn draw_rrect(&mut self, rrect: RRect, color: Color) {
        let gfx_rrect = tiny_gfx::RRect::from_rect_xy(
            tiny_gfx::Rect::from_ltwh(
                rrect.rect.x,
                rrect.rect.y,
                rrect.rect.width,
                rrect.rect.height,
            ),
            rrect.radius.x,
            rrect.radius.y,
        );
        self.inner.draw_rrect(gfx_rrect, color.to_gfx());
    }

    /// Draw a stroked rounded rectangle border.
    #[inline(always)]
    pub fn draw_rrect_stroke(&mut self, rrect: RRect, color: Color, stroke_width: f32) {
        let gfx_rrect = tiny_gfx::RRect::from_rect_xy(
            tiny_gfx::Rect::from_ltwh(
                rrect.rect.x,
                rrect.rect.y,
                rrect.rect.width,
                rrect.rect.height,
            ),
            rrect.radius.x,
            rrect.radius.y,
        );
        self.inner
            .draw_rrect_stroke(gfx_rrect, color.to_gfx(), stroke_width);
    }

    /// Draw a filled circle.
    #[inline(always)]
    pub fn draw_circle(&mut self, center: Point, radius: f32, color: Color) {
        self.inner.draw_circle(
            tiny_gfx::Point::new(center.x, center.y),
            radius,
            color.to_gfx(),
        );
    }

    /// Blit an 8-bit alpha mask at the specified coordinates with given color.
    #[inline(always)]
    pub fn blit_mask(&mut self, x: i32, y: i32, w: u32, h: u32, mask: &[u8], color: Color) {
        self.inner.blit_mask(x, y, w, h, mask, color.to_gfx());
    }

    /// Draw a pre-baked static icon at the specified position.
    #[inline(always)]
    pub fn draw_icon(
        &mut self,
        origin: Point,
        icon: &crate::graphics::ui_icons::UiIcon,
        color: Color,
    ) {
        self.blit_mask(
            origin.x.round() as i32,
            origin.y.round() as i32,
            icon.width as u32,
            icon.height as u32,
            icon.bitmap,
            color,
        );
    }

    /// Draw text string with specified font, font size and color.

    pub fn draw_text(&mut self, text: &str, font: &Font, size: f32, origin: Point, color: Color) {
        if text.is_empty() || color.a == 0 {
            return;
        }

        if let Some(clip) = self.current_clip() {
            if origin.y + size < clip.y || origin.y > clip.bottom() {
                return;
            }
        }

        let mut cur_x = origin.x;
        let base_y = origin.y + font.cap_height(size);
        for ch in text.chars() {
            if ch == '\u{200a}' || ch == '\t' {
                cur_x += font.advance(ch, size);
                continue;
            }
            if ch.is_control() {
                continue;
            }
            let glyph = font.glyph(ch, size);
            self.inner.blit_mask(
                (cur_x + glyph.xmin as f32).round() as i32,
                (base_y - glyph.ymin as f32 - glyph.height as f32).round() as i32,
                glyph.width as u32,
                glyph.height as u32,
                glyph.bitmap(),
                color.to_gfx(),
            );
            cur_x += glyph.advance;
        }
    }
}
