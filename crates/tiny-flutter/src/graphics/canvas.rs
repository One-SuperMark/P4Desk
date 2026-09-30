use crate::graphics::color::Color;
use crate::graphics::font::Font;
use crate::graphics::geometry::{Point, RRect, Rect};
use crate::tiny_gfx::{self, FillRule, Paint, Path, Pixmap565Mut, Stroke};

/// A 2D drawing canvas wrapping tiny-gfx Pixmap565Mut with a Flutter-like drawing API.
pub struct Canvas<'a> {
    inner: tiny_gfx::Canvas<'a>,
}

/// Quarter-resolution, twice box-filtered capture used by modal frosted glass.
pub struct BlurredBackdrop {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    revision: u32,
}

impl<'a> Canvas<'a> {
    pub fn capture_blurred_backdrop(&self) -> Option<BlurredBackdrop> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(1);
        let w = (self.width() as usize + 3) / 4;
        let h = (self.height() as usize + 3) / 4;
        if w == 0 || h == 0 {
            return None;
        }
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(w * h * 3).ok()?;
        pixels.resize(w * h * 3, 0);
        let mut scratch = Vec::new();
        scratch.try_reserve_exact(pixels.len()).ok()?;
        scratch.resize(pixels.len(), 0);
        for y in 0..h {
            for x in 0..w {
                let mut sum = [0u32; 3];
                let mut count = 0;
                for yy in y * 4..((y + 1) * 4).min(self.height() as usize) {
                    for xx in x * 4..((x + 1) * 4).min(self.width() as usize) {
                        let (r, g, b) = tiny_gfx::rgb565_to_rgb888(
                            self.pixels_rgb565()[yy * self.pixel_stride() + xx],
                        );
                        sum[0] += r as u32;
                        sum[1] += g as u32;
                        sum[2] += b as u32;
                        count += 1;
                    }
                }
                for c in 0..3 {
                    pixels[(y * w + x) * 3 + c] = (sum[c] / count) as u8;
                }
            }
        }
        // Sliding sums keep the filter linear in pixel count, independent of radius.
        for _ in 0..2 {
            blur_axis(&pixels, &mut scratch, w, h, true);
            blur_axis(&scratch, &mut pixels, w, h, false);
        }
        Some(BlurredBackdrop {
            pixels,
            width: w as u32,
            height: h as u32,
            revision: NEXT.fetch_add(1, Ordering::Relaxed),
        })
    }

    pub fn frosted_backdrop(&mut self, rect: RRect, tint: Color, source: &BlurredBackdrop) {
        use crate::theme::Folio;
        if Folio::glass_amount() == 0 {
            self.draw_rrect_aa(rect, tint);
            return;
        }
        let signature = [
            rect.rect.x.to_bits(),
            rect.rect.y.to_bits(),
            rect.rect.width.to_bits(),
            rect.rect.height.to_bits(),
            rect.radius.x.to_bits(),
            u32::from_le_bytes([tint.r, tint.g, tint.b, tint.a]),
            Folio::glass_amount() as u32,
            source.width,
            source.height,
            source.revision,
            0,
            0,
        ];
        super::vector_cache::surface(
            self,
            rect.rect,
            source.pixels.as_ptr() as usize,
            3,
            signature,
            false,
            |canvas| {
                canvas.inner.glass_panel_with_finish(
                    tiny_gfx::Rect::from_ltwh(
                        rect.rect.x,
                        rect.rect.y,
                        rect.rect.width,
                        rect.rect.height,
                    ),
                    rect.radius.x,
                    tint.to_gfx(),
                    &source.pixels,
                    source.width,
                    source.height,
                    Folio::glass_amount(),
                    false,
                    tiny_gfx::GlassFinish::Frosted,
                );
            },
        );
    }
    pub fn liquid_glass(&mut self, rect: RRect, tint: Color, pressed: bool) {
        self.liquid_glass_material(rect, tint, pressed, crate::theme::GlassMaterial::Page);
    }
    pub fn liquid_glass_material(
        &mut self,
        rect: RRect,
        tint: Color,
        pressed: bool,
        material: crate::theme::GlassMaterial,
    ) {
        use crate::theme::{Folio, GlassMaterial};
        if Folio::glass_amount() == 0 {
            let fill = if pressed { Folio::pressed(tint) } else { tint };
            self.draw_rrect_aa(rect, Color::from_rgb(fill.r, fill.g, fill.b));
            return;
        }
        let background = Folio::bg();
        let solid = [background.r, background.g, background.b];
        let wallpaper = if material != GlassMaterial::Page {
            Folio::backdrop()
        } else {
            None
        };
        let (pixels, width, height) = wallpaper
            .map(|b| (b.pixels, b.width, b.height))
            .unwrap_or((&solid, 1, 1));
        let signature = [
            rect.rect.x.to_bits(),
            rect.rect.y.to_bits(),
            rect.rect.width.to_bits(),
            rect.rect.height.to_bits(),
            rect.radius.x.to_bits(),
            u32::from_le_bytes([tint.r, tint.g, tint.b, tint.a]),
            Folio::glass_amount() as u32,
            pressed as u32,
            material as u32,
            width,
            height,
            u32::from_le_bytes([solid[0], solid[1], solid[2], 0]),
        ];
        super::vector_cache::surface(
            self,
            rect.rect,
            wallpaper.map_or(0, |b| b.pixels.as_ptr() as usize),
            1,
            signature,
            false,
            |canvas| {
                canvas.paint_glass_surface(rect, tint, pixels, width, height, pressed, material);
            },
        );
    }

    fn paint_glass_surface(
        &mut self,
        rect: RRect,
        tint: Color,
        pixels: &[u8],
        width: u32,
        height: u32,
        pressed: bool,
        material: crate::theme::GlassMaterial,
    ) {
        use crate::theme::{Folio, GlassMaterial};
        self.inner.glass_panel_with_finish(
            tiny_gfx::Rect::from_ltwh(rect.rect.x, rect.rect.y, rect.rect.width, rect.rect.height),
            rect.radius.x,
            tint.to_gfx(),
            pixels,
            width,
            height,
            Folio::glass_amount(),
            pressed,
            if material == GlassMaterial::DesktopCard {
                tiny_gfx::GlassFinish::Lens
            } else {
                tiny_gfx::GlassFinish::Soft
            },
        );
    }

    /// Cache a known opaque procedural surface. `revision` identifies all source
    /// and color changes. Geometry and clip are included in the bounded cache.
    pub fn cache_opaque_surface(
        &mut self,
        rect: Rect,
        revision: u32,
        draw: impl FnOnce(&mut Canvas),
    ) {
        if self.translation_only() != Some((0.0, 0.0))
            || rect != Rect::from_ltwh(0.0, 0.0, self.width() as f32, self.height() as f32)
        {
            draw(self);
            return;
        }
        super::vector_cache::surface(
            self,
            rect,
            0,
            2,
            [
                rect.x.to_bits(),
                rect.y.to_bits(),
                rect.width.to_bits(),
                rect.height.to_bits(),
                revision,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            true,
            draw,
        );
    }
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
    pub fn is_rect_visible(&self, rect: Rect) -> bool {
        self.inner.is_rect_visible(tiny_gfx::Rect::from_ltwh(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        ))
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
    /// Read-only pixels for a bounded, in-memory repaint cache.
    pub fn pixels_rgb565(&self) -> &[u16] {
        self.inner.data()
    }

    pub(crate) fn pixels_rgb565_mut(&mut self) -> &mut [u16] {
        self.inner.data_mut()
    }

    pub(crate) fn pixel_stride(&self) -> usize {
        self.inner.pixel_stride()
    }

    pub(crate) fn translation_only(&self) -> Option<(f32, f32)> {
        self.inner.translation_only()
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

    pub(crate) fn blit_image_565_spans(
        &mut self,
        x: i32,
        y: i32,
        rgb: &[u16],
        alpha: &[u8],
        spans: &[tiny_gfx::raster::ImageSpan565],
    ) {
        self.inner.blit_image_565_spans(x, y, rgb, alpha, spans);
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
    pub fn draw_rrect_aa(&mut self, rrect: RRect, color: Color) {
        let shape = tiny_gfx::RRect::from_rect_xy(
            tiny_gfx::Rect::from_ltwh(
                rrect.rect.x,
                rrect.rect.y,
                rrect.rect.width,
                rrect.rect.height,
            ),
            rrect.radius.x,
            rrect.radius.y,
        );
        self.inner.draw_rrect_aa(shape, color.to_gfx());
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

    /// Analytic SVG circle fill or centered outline with coverage antialiasing.
    pub fn paint_circle(
        &mut self,
        center: Point,
        radius: f32,
        paint: &Paint,
        stroke_width: Option<f32>,
    ) {
        self.inner.paint_circle(
            tiny_gfx::Point::new(center.x, center.y),
            radius,
            paint,
            stroke_width,
        );
    }
    pub fn glass_circle(
        &mut self,
        center: Point,
        radius: f32,
        tint: Color,
        fill: tiny_gfx::GlassFill,
        rim: f32,
        strength: f32,
    ) {
        self.inner.glass_circle(
            tiny_gfx::Point::new(center.x, center.y),
            radius,
            tint.to_gfx(),
            fill,
            rim,
            strength,
        );
    }
    /// Analytic SVG rounded rectangle fill or centered outline.
    pub fn paint_rrect(&mut self, rect: RRect, paint: &Paint, stroke_width: Option<f32>) {
        self.inner.paint_rrect(
            tiny_gfx::RRect::from_rect_xy(
                tiny_gfx::Rect::from_ltwh(
                    rect.rect.x,
                    rect.rect.y,
                    rect.rect.width,
                    rect.rect.height,
                ),
                rect.radius.x,
                rect.radius.y,
            ),
            paint,
            stroke_width,
        );
    }

    /// Blit an 8-bit alpha mask at the specified coordinates with given color.
    #[inline(always)]
    pub fn blit_mask(&mut self, x: i32, y: i32, w: u32, h: u32, mask: &[u8], color: Color) {
        self.inner.blit_mask(x, y, w, h, mask, color.to_gfx());
    }

    /// Draw the icon's SVG geometry at its default size.
    #[inline(always)]
    pub fn draw_icon(
        &mut self,
        origin: Point,
        icon: &crate::graphics::ui_icons::UiIcon,
        color: Color,
    ) {
        icon.vector.paint(
            self,
            Rect::from_ltwh(origin.x, origin.y, icon.width as f32, icon.height as f32),
            color,
        );
    }

    /// Draw text string with specified font, font size and color.

    pub fn draw_text(&mut self, text: &str, font: &Font, size: f32, origin: Point, color: Color) {
        if text.is_empty() || color.a == 0 {
            return;
        }

        // Text origin is local; the saved clip is in framebuffer coordinates.
        // The raster blitter clips each transformed glyph in the same space.

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

fn blur_axis(src: &[u8], dst: &mut [u8], w: usize, h: usize, horizontal: bool) {
    let (lines, length, stride) = if horizontal { (h, w, 3) } else { (w, h, w * 3) };
    for line in 0..lines {
        let base = if horizontal { line * w * 3 } else { line * 3 };
        for c in 0..3 {
            let at =
                |i: isize| src[base + i.clamp(0, length as isize - 1) as usize * stride + c] as u32;
            let mut sum: u32 = (-4..=4).map(at).sum();
            for i in 0..length {
                dst[base + i * stride + c] = (sum / 9) as u8;
                sum = sum + at(i as isize + 5) - at(i as isize - 4);
            }
        }
    }
}

#[cfg(test)]
mod surface_cache_tests {
    use super::*;
    use crate::theme::{Folio, GlassBackdrop, GlassMaterial};
    #[test]
    fn captured_backdrop_blurs_detail_and_keeps_uniform_colors() {
        let mut p = tiny_gfx::Pixmap565::new(64, 64).unwrap();
        for y in 0..64 {
            for x in 0..64 {
                p.data_mut()[y * 64 + x] = if (x / 8 + y / 8) % 2 == 0 { 0 } else { 0xffff };
            }
        }
        let blur = Canvas::new(p.as_mut()).capture_blurred_backdrop().unwrap();
        assert!(
            blur.pixels.iter().all(|v| *v > 35 && *v < 220),
            "fine details should become a soft color field"
        );
        p.fill(Color::from_hex(0x427dab).to_rgb565());
        let blur = Canvas::new(p.as_mut()).capture_blurred_backdrop().unwrap();
        let (r, g, b) = tiny_gfx::rgb565_to_rgb888(p.data()[0]);
        assert!(blur.pixels.chunks_exact(3).all(|c| c == [r, g, b]));
    }
    #[test]
    fn cached_glass_matches_direct_for_themes_press_backdrops_and_partial_updates() {
        static RED: [u8; 12] = [240, 30, 20, 140, 20, 60, 30, 80, 90, 240, 120, 70];
        static BLUE: [u8; 12] = [20, 30, 240, 60, 20, 140, 90, 80, 30, 70, 120, 240];
        for light in [false, true] {
            for amount in [35, 65, 100] {
                Folio::configure(light, amount);
                for pixels in [&RED, &BLUE] {
                    Folio::set_backdrop(GlassBackdrop {
                        pixels,
                        width: 2,
                        height: 2,
                    });
                    for material in [
                        GlassMaterial::Page,
                        GlassMaterial::Desktop,
                        GlassMaterial::DesktopCard,
                    ] {
                        for pressed in [false, true, false] {
                            for clip in [None, None, Some(Rect::from_ltwh(31.0, 16.0, 81.0, 68.0))]
                            {
                                let render = |cached| {
                                    let mut image = tiny_gfx::Pixmap565::new(160, 120).unwrap();
                                    let mut canvas = Canvas::new(image.as_mut());
                                    canvas.clear(Color::from_hex(0x314259));
                                    if let Some(clip) = clip {
                                        canvas.clip_rect(clip);
                                    }
                                    let rect = RRect::from_rect_circular(
                                        Rect::from_ltwh(10.5, 7.5, 134.0, 99.0),
                                        22.0,
                                    );
                                    let tint = Folio::glass();
                                    if cached {
                                        canvas.liquid_glass_material(rect, tint, pressed, material);
                                    } else {
                                        let bg = Folio::bg();
                                        let solid = [bg.r, bg.g, bg.b];
                                        let page = material == GlassMaterial::Page;
                                        canvas.paint_glass_surface(
                                            rect,
                                            tint,
                                            if page { &solid } else { pixels },
                                            if page { 1 } else { 2 },
                                            if page { 1 } else { 2 },
                                            pressed,
                                            material,
                                        );
                                    }
                                    image
                                };
                                assert_eq!(render(true), render(false));
                            }
                        }
                    }
                }
            }
        }
    }
}
