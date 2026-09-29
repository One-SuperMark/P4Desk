use crate::color::Color;
use crate::geometry::{Point, RRect, Rect, Transform};
use crate::paint::{FillRule, Paint, Stroke};
use crate::path::Path;
use crate::pixmap::Pixmap565Mut;
use crate::raster;

pub struct Canvas<'a> {
    pixmap: Pixmap565Mut<'a>,
    current_transform: Transform,
    transform_stack: Vec<Transform>,
    clip_stack: Vec<Option<Rect>>,
    current_clip: Option<Rect>,
}

impl<'a> Canvas<'a> {
    pub fn new(pixmap: Pixmap565Mut<'a>) -> Self {
        Self {
            pixmap,
            current_transform: Transform::identity(),
            transform_stack: Vec::new(),
            clip_stack: Vec::new(),
            current_clip: None,
        }
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.pixmap.width
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.pixmap.height
    }

    #[inline(always)]
    pub fn data(&self) -> &[u16] {
        self.pixmap.data()
    }

    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut [u16] {
        self.pixmap.data_mut()
    }

    pub fn save(&mut self) {
        self.transform_stack.push(self.current_transform);
        self.clip_stack.push(self.current_clip);
    }

    pub fn restore(&mut self) {
        if let Some(tf) = self.transform_stack.pop() {
            self.current_transform = tf;
        }
        if let Some(clip) = self.clip_stack.pop() {
            self.current_clip = clip;
        }
    }

    #[inline(always)]
    pub fn current_clip(&self) -> Option<Rect> {
        self.current_clip
    }
    /// Cull local geometry before constructing paths/allocating scanline scratch.
    pub fn is_rect_visible(&self, rect: Rect) -> bool {
        let t = self.current_transform;
        let x1 = t.sx * rect.x + t.tx;
        let x2 = t.sx * rect.right() + t.tx;
        let y1 = t.sy * rect.y + t.ty;
        let y2 = t.sy * rect.bottom() + t.ty;
        let mapped = Rect::from_ltrb(x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2));
        let screen = Rect::from_ltwh(0.0, 0.0, self.width() as f32, self.height() as f32);
        mapped
            .intersect(&self.current_clip.unwrap_or(screen))
            .and_then(|r| r.intersect(&screen))
            .is_some()
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.current_transform = self.current_transform.post_translate(dx, dy);
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current_transform = self.current_transform.post_scale(sx, sy);
    }

    pub fn clip_rect(&mut self, rect: Rect) {
        // Map rect with current transform translation and scale
        let mapped = Rect {
            x: self.current_transform.sx * rect.x + self.current_transform.tx,
            y: self.current_transform.sy * rect.y + self.current_transform.ty,
            width: self.current_transform.sx * rect.width,
            height: self.current_transform.sy * rect.height,
        };
        self.current_clip = match self.current_clip {
            Some(prev) => Some(
                prev.intersect(&mapped)
                    .unwrap_or(Rect::from_ltwh(0.0, 0.0, 0.0, 0.0)),
            ),
            None => Some(mapped),
        };
    }

    pub fn clear(&mut self, color: Color) {
        if self.current_clip.is_none() {
            self.pixmap.fill(color.to_rgb565());
        } else {
            let full_rect = Rect::from_ltwh(
                0.0,
                0.0,
                self.pixmap.width as f32,
                self.pixmap.height as f32,
            );
            raster::fill_rect(&mut self.pixmap, self.current_clip, full_rect, color);
        }
    }

    pub fn copy_pixels_from(&mut self, src: &[u16]) {
        let dst = self.pixmap.data_mut();
        let len = dst.len().min(src.len());
        dst[..len].copy_from_slice(&src[..len]);
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let transformed = Rect {
            x: self.current_transform.sx * rect.x + self.current_transform.tx,
            y: self.current_transform.sy * rect.y + self.current_transform.ty,
            width: self.current_transform.sx * rect.width,
            height: self.current_transform.sy * rect.height,
        };
        raster::fill_rect(&mut self.pixmap, self.current_clip, transformed, color);
    }

    pub fn draw_rrect(&mut self, rrect: RRect, color: Color) {
        let sx = self.current_transform.sx;
        let sy = self.current_transform.sy;
        let transformed = RRect {
            rect: Rect {
                x: sx * rrect.rect.x + self.current_transform.tx,
                y: sy * rrect.rect.y + self.current_transform.ty,
                width: sx * rrect.rect.width,
                height: sy * rrect.rect.height,
            },
            radius: crate::geometry::Radius {
                x: sx * rrect.radius.x,
                y: sy * rrect.radius.y,
            },
        };
        raster::fill_rrect(&mut self.pixmap, self.current_clip, transformed, color);
    }

    pub fn draw_rrect_stroke(&mut self, rrect: RRect, color: Color, stroke_width: f32) {
        let sx = self.current_transform.sx;
        let sy = self.current_transform.sy;
        let transformed = RRect {
            rect: Rect {
                x: sx * rrect.rect.x + self.current_transform.tx,
                y: sy * rrect.rect.y + self.current_transform.ty,
                width: sx * rrect.rect.width,
                height: sy * rrect.rect.height,
            },
            radius: crate::geometry::Radius {
                x: sx * rrect.radius.x,
                y: sy * rrect.radius.y,
            },
        };
        let sw = stroke_width * sx;
        raster::stroke_rrect(&mut self.pixmap, self.current_clip, transformed, color, sw);
    }
    pub fn draw_rrect_aa(&mut self, rrect: RRect, color: Color) {
        let sx = self.current_transform.sx;
        let sy = self.current_transform.sy;
        let transformed = RRect {
            rect: Rect::from_ltwh(
                sx * rrect.rect.x + self.current_transform.tx,
                sy * rrect.rect.y + self.current_transform.ty,
                sx * rrect.rect.width,
                sy * rrect.rect.height,
            ),
            radius: crate::geometry::Radius {
                x: sx * rrect.radius.x,
                y: sy * rrect.radius.y,
            },
        };
        raster::fill_rrect_aa(&mut self.pixmap, self.current_clip, transformed, color);
    }

    pub fn draw_circle(&mut self, center: Point, radius: f32, color: Color) {
        let mapped = self.current_transform.map_point(center);
        let r = radius * self.current_transform.sx;
        raster::fill_circle(&mut self.pixmap, self.current_clip, mapped, r, color);
    }

    pub fn paint_circle(
        &mut self,
        center: Point,
        radius: f32,
        paint: &Paint,
        stroke_width: Option<f32>,
    ) {
        let center = self.current_transform.map_point(center);
        let scale = self.current_transform.sx.abs();
        let paint = self.mapped_paint(paint);
        crate::vector::circle(
            &mut self.pixmap,
            self.current_clip,
            center,
            radius * scale,
            &paint,
            stroke_width.map(|w| w * scale),
        );
    }
    /// Tint a circular region (or its outside) through a narrow glass boundary.
    /// The caller paints its backdrop first. Supports translation/uniform scale.
    pub fn glass_circle(
        &mut self,
        center: Point,
        radius: f32,
        tint: Color,
        fill: crate::GlassFill,
        rim: f32,
        strength: f32,
    ) {
        let center = self.current_transform.map_point(center);
        let scale = self.current_transform.sx.abs();
        crate::glass::composite(
            &mut self.pixmap,
            self.current_clip,
            center,
            radius * scale,
            tint,
            fill,
            rim * scale,
            strength,
        );
    }
    pub fn paint_rrect(&mut self, rect: RRect, paint: &Paint, stroke_width: Option<f32>) {
        let t = self.current_transform;
        let rect = RRect::from_rect_radius(
            Rect::from_ltwh(
                t.sx * rect.rect.x + t.tx,
                t.sy * rect.rect.y + t.ty,
                t.sx * rect.rect.width,
                t.sy * rect.rect.height,
            ),
            crate::geometry::Radius {
                x: t.sx.abs() * rect.radius.x,
                y: t.sy.abs() * rect.radius.y,
            },
        );
        let paint = self.mapped_paint(paint);
        crate::vector::rounded_rect(
            &mut self.pixmap,
            self.current_clip,
            rect,
            &paint,
            stroke_width.map(|w| w * t.sx.abs()),
        );
    }

    pub fn blit_mask(&mut self, x: i32, y: i32, w: u32, h: u32, mask: &[u8], color: Color) {
        let tx = x + (self.current_transform.tx.round() as i32);
        let ty = y + (self.current_transform.ty.round() as i32);
        raster::blit_mask(
            &mut self.pixmap,
            self.current_clip,
            tx,
            ty,
            w,
            h,
            mask,
            color,
        );
    }

    pub fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) {
        let polylines = self.flatten_transformed(path);
        let mut stroke = stroke.clone();
        stroke.width *= self.current_transform.sx.abs();
        let paint = self.mapped_paint(paint);
        crate::vector::stroke(
            &mut self.pixmap,
            self.current_clip,
            &polylines,
            &paint,
            &stroke,
        );
    }

    pub fn fill_path(&mut self, path: &Path, paint: &Paint, fill_rule: FillRule) {
        let polylines = self.flatten_transformed(path);
        let paint = self.mapped_paint(paint);
        crate::vector::fill(
            &mut self.pixmap,
            self.current_clip,
            &polylines,
            &paint,
            fill_rule,
        );
    }

    fn flatten_transformed(&self, path: &Path) -> Vec<Vec<Point>> {
        let scale = self
            .current_transform
            .sx
            .abs()
            .max(self.current_transform.sy.abs())
            .max(0.001);
        path.flatten(0.10 / scale)
            .into_iter()
            .map(|poly| {
                poly.into_iter()
                    .map(|p| self.current_transform.map_point(p))
                    .collect()
            })
            .collect()
    }

    fn mapped_paint<'p>(&self, paint: &Paint<'p>) -> Paint<'p> {
        let mut paint = paint.clone();
        if let crate::paint::Shader::Linear(gradient) = &mut paint.shader {
            let t = self.current_transform;
            let determinant = t.sx * t.sy - t.kx * t.ky;
            if determinant.abs() > 0.000001 {
                let inverse = Transform {
                    sx: t.sy / determinant,
                    sy: t.sx / determinant,
                    kx: -t.kx / determinant,
                    ky: -t.ky / determinant,
                    tx: (t.kx * t.ty - t.sy * t.tx) / determinant,
                    ty: (t.ky * t.tx - t.sx * t.ty) / determinant,
                };
                let g = gradient.transform;
                gradient.transform = Transform {
                    sx: g.sx * inverse.sx + g.kx * inverse.ky,
                    kx: g.sx * inverse.kx + g.kx * inverse.sy,
                    ky: g.ky * inverse.sx + g.sy * inverse.ky,
                    sy: g.ky * inverse.kx + g.sy * inverse.sy,
                    tx: g.sx * inverse.tx + g.kx * inverse.ty + g.tx,
                    ty: g.ky * inverse.tx + g.sy * inverse.ty + g.ty,
                };
            }
        }
        paint
    }

    pub fn fill_dithered_horizontal_gradient(&mut self, c0: (u8, u8, u8), c1: (u8, u8, u8)) {
        raster::fill_dithered_horizontal_gradient(&mut self.pixmap, self.current_clip, c0, c1);
    }

    pub fn blit_image_565(&mut self, x: i32, y: i32, w: u32, h: u32, pixels: &[u16]) {
        let tx = x + (self.current_transform.tx.round() as i32);
        let ty = y + (self.current_transform.ty.round() as i32);
        raster::blit_image_565(&mut self.pixmap, self.current_clip, tx, ty, w, h, pixels);
    }

    pub fn blit_image_565_with_alpha(
        &mut self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        rgb: &[u16],
        alpha: &[u8],
    ) {
        let tx = x + (self.current_transform.tx.round() as i32);
        let ty = y + (self.current_transform.ty.round() as i32);
        raster::blit_image_565_with_alpha(
            &mut self.pixmap,
            self.current_clip,
            tx,
            ty,
            w,
            h,
            rgb,
            alpha,
        );
    }

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
        let tx = x + (self.current_transform.tx.round() as i32);
        let ty = y + (self.current_transform.ty.round() as i32);
        raster::blit_image_565_with_alpha_scaled(
            &mut self.pixmap,
            self.current_clip,
            tx,
            ty,
            dst_w,
            dst_h,
            src_w,
            src_h,
            rgb,
            alpha,
        );
    }
}
