//! Static SVG geometry, compiled offline and rasterized at the requested size.
//! The firmware stores paths and colors, with no SVG XML parser or bitmap atlas.
use super::{Canvas, Color, Point as UiPoint, RRect, Rect};
use crate::tiny_gfx::{self as gfx, FillRule, PathBuilder, Point, Stroke};

#[derive(Clone, Copy)]
pub enum VectorCommand {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}
#[derive(Clone, Copy)]
pub enum VectorShape {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
    },
    Line(f32, f32, f32, f32),
    Polygon(&'static [(f32, f32)]),
    Path(&'static [VectorCommand]),
}
#[derive(Clone, Copy)]
pub enum VectorPaint {
    Solid(Color),
    Tint(f32),
    MappedGradient {
        matrix: [f32; 6],
        radial: bool,
        stops: &'static [(f32, Color)],
    },
    VerticalGradient {
        y1: f32,
        y2: f32,
        top: Color,
        bottom: Color,
    },
}
#[derive(Clone, Copy)]
pub struct VectorLayer {
    pub shape: VectorShape,
    pub fill: Option<VectorPaint>,
    pub stroke: Option<VectorPaint>,
    pub stroke_width: f32,
    pub line_cap: gfx::LineCap,
    pub line_join: gfx::LineJoin,
    pub fill_rule: FillRule,
}
#[derive(Clone, Copy)]
pub struct VectorIcon {
    pub width: f32,
    pub height: f32,
    pub layers: &'static [VectorLayer],
}
impl VectorIcon {
    pub fn paint(&self, canvas: &mut Canvas, bounds: Rect, tint: Color) {
        self.paint_with_opacity(canvas, bounds, tint, 1.0);
    }
    pub fn paint_with_opacity(&self, canvas: &mut Canvas, bounds: Rect, tint: Color, opacity: f32) {
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }
        let scale = (bounds.width / self.width).min(bounds.height / self.height);
        let x = bounds.x + (bounds.width - self.width * scale) * 0.5;
        let y = bounds.y + (bounds.height - self.height * scale) * 0.5;
        // Include the widest stroke that may extend past the SVG viewBox.
        let outset = self
            .layers
            .iter()
            .map(|layer| layer.stroke_width)
            .fold(0.0, f32::max)
            * scale;
        if opacity <= 0.0
            || !canvas.is_rect_visible(Rect::from_ltwh(
                x - outset,
                y - outset,
                self.width * scale + outset * 2.0,
                self.height * scale + outset * 2.0,
            ))
        {
            return;
        }
        let area = Rect::from_ltwh(
            x - outset - 1.0,
            y - outset - 1.0,
            self.width * scale + outset * 2.0 + 2.0,
            self.height * scale + outset * 2.0 + 2.0,
        );
        super::vector_cache::paint(self, canvas, bounds, area, tint, opacity, |canvas| {
            self.paint_layers(canvas, x, y, scale, tint, opacity);
        });
    }

    pub(super) fn paint_layers(
        &self,
        canvas: &mut Canvas,
        x: f32,
        y: f32,
        scale: f32,
        tint: Color,
        opacity: f32,
    ) {
        // Coordinates are scaled before the Canvas sees them. This preserves the
        // caller's translation and global dirty clip, including fractional origins.
        for layer in self.layers {
            // SVG circles and rounded rectangles have analytic row spans. Avoid
            // rebuilding hundreds of tiny outline polygons on every touch frame.
            match layer.shape {
                VectorShape::Circle {
                    x: cx,
                    y: cy,
                    radius,
                } => {
                    let center = UiPoint::new(x + cx * scale, y + cy * scale);
                    if let Some(fill) = layer.fill {
                        canvas.paint_circle(
                            center,
                            radius * scale,
                            &fill.paint(tint, x, y, scale, opacity),
                            None,
                        );
                    }
                    if let Some(stroke) = layer.stroke {
                        canvas.paint_circle(
                            center,
                            radius * scale,
                            &stroke.paint(tint, x, y, scale, opacity),
                            Some(layer.stroke_width * scale),
                        );
                    }
                    continue;
                }
                VectorShape::Rect {
                    x: bx,
                    y: by,
                    width,
                    height,
                    radius,
                } => {
                    let rect = RRect::from_rect_circular(
                        Rect::from_ltwh(
                            x + bx * scale,
                            y + by * scale,
                            width * scale,
                            height * scale,
                        ),
                        radius * scale,
                    );
                    if let Some(fill) = layer.fill {
                        canvas.paint_rrect(rect, &fill.paint(tint, x, y, scale, opacity), None);
                    }
                    if let Some(stroke) = layer.stroke {
                        canvas.paint_rrect(
                            rect,
                            &stroke.paint(tint, x, y, scale, opacity),
                            Some(layer.stroke_width * scale),
                        );
                    }
                    continue;
                }
                _ => (),
            }
            let mut path = PathBuilder::new();
            let move_to = |px: f32, py: f32| (x + px * scale, y + py * scale);
            match layer.shape {
                VectorShape::Rect { .. } | VectorShape::Circle { .. } => unreachable!(),
                VectorShape::Line(x1, y1, x2, y2) => {
                    let (a, b) = move_to(x1, y1);
                    path.move_to(a, b);
                    let (a, b) = move_to(x2, y2);
                    path.line_to(a, b);
                }
                VectorShape::Polygon(points) => {
                    if let Some(&(px, py)) = points.first() {
                        let (a, b) = move_to(px, py);
                        path.move_to(a, b);
                        for &(px, py) in &points[1..] {
                            let (a, b) = move_to(px, py);
                            path.line_to(a, b);
                        }
                        path.close();
                    }
                }
                VectorShape::Path(commands) => {
                    for command in commands {
                        match *command {
                            VectorCommand::Move(px, py) => {
                                let (a, b) = move_to(px, py);
                                path.move_to(a, b);
                            }
                            VectorCommand::Line(px, py) => {
                                let (a, b) = move_to(px, py);
                                path.line_to(a, b);
                            }
                            VectorCommand::Quad(ax, ay, bx, by) => {
                                let (a, b) = move_to(ax, ay);
                                let (c, d) = move_to(bx, by);
                                path.quad_to(a, b, c, d);
                            }
                            VectorCommand::Cubic(ax, ay, bx, by, cx, cy) => {
                                let (a, b) = move_to(ax, ay);
                                let (c, d) = move_to(bx, by);
                                let (e, f) = move_to(cx, cy);
                                path.cubic_to(a, b, c, d, e, f);
                            }
                            VectorCommand::Close => path.close(),
                        }
                    }
                }
            }
            let Some(path) = path.finish() else {
                continue;
            };
            if let Some(fill) = layer.fill {
                canvas.fill_path(
                    &path,
                    &fill.paint(tint, x, y, scale, opacity),
                    layer.fill_rule,
                );
            }
            if let Some(stroke) = layer.stroke {
                canvas.stroke_path(
                    &path,
                    &stroke.paint(tint, x, y, scale, opacity),
                    &Stroke {
                        width: layer.stroke_width * scale,
                        line_cap: layer.line_cap,
                        line_join: layer.line_join,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
impl VectorPaint {
    fn paint(self, tint: Color, x: f32, y: f32, scale: f32, opacity: f32) -> gfx::Paint<'static> {
        let opacity = opacity.clamp(0.0, 1.0);
        let fade = |color: Color| color.with_opacity(color.a as f32 / 255.0 * opacity);
        match self {
            Self::Solid(color) => gfx::Paint::new(fade(color).to_gfx()),
            Self::Tint(alpha) => gfx::Paint::new(
                Color::from_rgba(
                    tint.r,
                    tint.g,
                    tint.b,
                    (tint.a as f32 * alpha * opacity).round() as u8,
                )
                .to_gfx(),
            ),
            Self::MappedGradient {
                matrix: [a, b, c, d, e, f],
                radial,
                stops,
            } => {
                let transform = gfx::Transform {
                    sx: a / scale,
                    ky: b / scale,
                    kx: c / scale,
                    sy: d / scale,
                    tx: e - (a * x + c * y) / scale,
                    ty: f - (b * x + d * y) / scale,
                };
                let mut paint = gfx::Paint::new(gfx::Color::TRANSPARENT);
                paint.shader = gfx::Shader::Mapped(gfx::paint::MappedGradient {
                    transform,
                    radial,
                    stops: stops
                        .iter()
                        .map(|(t, c)| gfx::GradientStop::new(*t, fade(*c).to_gfx()))
                        .collect(),
                });
                paint
            }
            Self::VerticalGradient {
                y1,
                y2,
                top,
                bottom,
            } => {
                let top = fade(top);
                let bottom = fade(bottom);
                let mut paint = gfx::Paint::new(top.to_gfx());
                paint.shader = gfx::LinearGradient::new(
                    Point::new(0.0, y + y1 * scale),
                    Point::new(0.0, y + y2 * scale),
                    vec![
                        gfx::GradientStop::new(0.0, top.to_gfx()),
                        gfx::GradientStop::new(1.0, bottom.to_gfx()),
                    ],
                    gfx::SpreadMode::Pad,
                    gfx::Transform::identity(),
                )
                .expect("compiled SVG gradient has two stops");
                paint
            }
        }
    }
}
