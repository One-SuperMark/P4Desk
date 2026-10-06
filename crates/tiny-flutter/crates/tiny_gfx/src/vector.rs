//! Coverage rasterization for vector fills and strokes. Eight vertical samples
//! and analytic horizontal coverage use a bounded row, never a supersized canvas.
use crate::color::blend_rgb565;
use crate::geometry::{Point, RRect, Radius, Rect};
use crate::paint::{FillRule, LineCap, LineJoin, Paint, Shader, Stroke};
use crate::pixmap::Pixmap565Mut;

// Fill paths borrow their flattened polygons. Stroke segments and joins have
// at most four vertices, so they never need an individual heap allocation.
#[derive(Clone)]
enum Shape<'a> {
    Polygon {
        points: &'a [Point],
        min_y: f32,
        max_y: f32,
    },
    InlinePolygon {
        points: [Point; 4],
        len: u8,
        min_y: f32,
        max_y: f32,
    },
    Circle(Point, f32),
    Ring(Point, f32, f32),
    RoundedRect(RRect, Option<RRect>),
}
impl<'a> Shape<'a> {
    fn polygon(points: &'a [Point]) -> Self {
        let (min_y, max_y) = vertical_bounds(points);
        Self::Polygon {
            points,
            min_y,
            max_y,
        }
    }
    fn quad(points: [Point; 4]) -> Self {
        Self::small_polygon(points, 4)
    }
    fn small_polygon(points: [Point; 4], len: u8) -> Self {
        let (min_y, max_y) = vertical_bounds(&points[..len as usize]);
        Self::InlinePolygon {
            points,
            len,
            min_y,
            max_y,
        }
    }
    fn polygon_points(&self) -> Option<(&[Point], f32, f32)> {
        match self {
            Self::Polygon {
                points,
                min_y,
                max_y,
            } => Some((points, *min_y, *max_y)),
            Self::InlinePolygon {
                points,
                len,
                min_y,
                max_y,
            } => Some((&points[..*len as usize], *min_y, *max_y)),
            _ => None,
        }
    }
}
fn vertical_bounds(points: &[Point]) -> (f32, f32) {
    points
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), p| {
            (min.min(p.y), max.max(p.y))
        })
}

pub(crate) fn circle(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    center: Point,
    radius: f32,
    paint: &Paint,
    width: Option<f32>,
) {
    let shape = if let Some(width) = width {
        if width <= 0.0 {
            return;
        }
        Shape::Ring(
            center,
            radius + width * 0.5,
            (radius - width * 0.5).max(0.0),
        )
    } else {
        Shape::Circle(center, radius)
    };
    rasterize::<true>(pixmap, clip, &[shape], paint, None);
}
pub(crate) fn rounded_rect(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    rect: RRect,
    paint: &Paint,
    width: Option<f32>,
) {
    let (outer, inner) = if let Some(width) = width {
        if width <= 0.0 {
            return;
        }
        let half = width * 0.5;
        let outer = RRect::from_rect_radius(
            Rect::from_ltwh(
                rect.rect.x - half,
                rect.rect.y - half,
                rect.rect.width + width,
                rect.rect.height + width,
            ),
            Radius {
                x: rect.radius.x + half,
                y: rect.radius.y + half,
            },
        );
        let inner = (rect.rect.width > width && rect.rect.height > width).then(|| {
            RRect::from_rect_radius(
                Rect::from_ltwh(
                    rect.rect.x + half,
                    rect.rect.y + half,
                    rect.rect.width - width,
                    rect.rect.height - width,
                ),
                Radius {
                    x: (rect.radius.x - half).max(0.0),
                    y: (rect.radius.y - half).max(0.0),
                },
            )
        });
        (outer, inner)
    } else {
        (rect, None)
    };
    rasterize::<true>(
        pixmap,
        clip,
        &[Shape::RoundedRect(outer, inner)],
        paint,
        None,
    );
}

fn rrect_span(rect: RRect, y: f32) -> Option<(f32, f32)> {
    if y < rect.rect.y
        || y >= rect.rect.bottom()
        || rect.rect.width <= 0.0
        || rect.rect.height <= 0.0
    {
        return None;
    }
    let rx = rect.radius.x.min(rect.rect.width * 0.5).max(0.0);
    let ry = rect.radius.y.min(rect.rect.height * 0.5).max(0.0);
    let inset = if rx > 0.0 && ry > 0.0 {
        let dy = (rect.rect.y + ry - y)
            .max(y - (rect.rect.bottom() - ry))
            .max(0.0);
        rx * (1.0 - (1.0 - (dy / ry).powi(2)).max(0.0).sqrt())
    } else {
        0.0
    };
    Some((rect.rect.x + inset, rect.rect.right() - inset))
}

pub(crate) fn fill(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    polygons: &[Vec<Point>],
    paint: &Paint,
    rule: FillRule,
) {
    rasterize::<true>(
        pixmap,
        clip,
        &polygons
            .iter()
            .map(|points| Shape::polygon(points))
            .collect::<Vec<_>>(),
        paint,
        Some(rule),
    );
}

pub(crate) fn stroke(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    polylines: &[Vec<Point>],
    paint: &Paint,
    stroke: &Stroke,
) {
    let half = stroke.width * 0.5;
    if half <= 0.0 || !half.is_finite() {
        return;
    }
    let mut shapes = Vec::new();
    for points in polylines {
        if points.len() < 2 {
            continue;
        }
        let closed = points.first() == points.last();
        let mut normals = Vec::with_capacity(points.len() - 1);
        for pair in points.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let length = (dx * dx + dy * dy).sqrt();
            if length < 0.00001 {
                normals.push(Point::ZERO);
                continue;
            }
            let n = Point::new(-dy * half / length, dx * half / length);
            normals.push(n);
            shapes.push(Shape::quad([
                Point::new(a.x + n.x, a.y + n.y),
                Point::new(a.x - n.x, a.y - n.y),
                Point::new(b.x - n.x, b.y - n.y),
                Point::new(b.x + n.x, b.y + n.y),
            ]));
        }
        let count = if closed {
            points.len() - 1
        } else {
            points.len()
        };
        for i in 0..count {
            let p = points[i];
            let endpoint = !closed && (i == 0 || i == count - 1);
            if endpoint {
                match stroke.line_cap {
                    LineCap::Round => shapes.push(Shape::Circle(p, half)),
                    LineCap::Square => {
                        let n = if i == 0 {
                            normals[0]
                        } else {
                            *normals.last().unwrap()
                        };
                        let direction = if i == 0 { -1.0 } else { 1.0 };
                        let t = Point::new(n.y * direction, -n.x * direction);
                        shapes.push(Shape::quad([
                            Point::new(p.x + n.x, p.y + n.y),
                            Point::new(p.x - n.x, p.y - n.y),
                            Point::new(p.x - n.x + t.x, p.y - n.y + t.y),
                            Point::new(p.x + n.x + t.x, p.y + n.y + t.y),
                        ]));
                    }
                    LineCap::Butt => (),
                }
            } else if stroke.line_join == LineJoin::Round {
                shapes.push(Shape::Circle(p, half));
            } else {
                let before = normals[(i + normals.len() - 1) % normals.len()];
                let after = normals[i % normals.len()];
                for side in [-1.0, 1.0] {
                    let a = Point::new(p.x + before.x * side, p.y + before.y * side);
                    let b = Point::new(p.x + after.x * side, p.y + after.y * side);
                    let mut join = [p, a, b, Point::ZERO];
                    let mut len = 3;
                    if stroke.line_join == LineJoin::Miter {
                        let sum =
                            Point::new((before.x + after.x) * side, (before.y + after.y) * side);
                        let dot = sum.x * after.x * side + sum.y * after.y * side;
                        if dot.abs() > 0.00001 {
                            let factor = half * half / dot;
                            let mx = sum.x * factor;
                            let my = sum.y * factor;
                            if mx * mx + my * my <= (half * stroke.miter_limit).powi(2) {
                                join[2] = Point::new(p.x + mx, p.y + my);
                                join[3] = b;
                                len = 4;
                            }
                        }
                    }
                    shapes.push(Shape::small_polygon(join, len));
                }
            }
        }
    }
    rasterize::<true>(pixmap, clip, &shapes, paint, None);
}

fn crossings(points: &[Point], y: f32, output: &mut Vec<(f32, i32)>) {
    if points.len() < 3 {
        return;
    }
    let mut a = *points.last().unwrap();
    for &b in points {
        if (a.y <= y && b.y > y) || (b.y <= y && a.y > y) {
            let x = a.x + (y - a.y) * (b.x - a.x) / (b.y - a.y);
            output.push((x, if b.y > a.y { 1 } else { -1 }));
        }
        a = b;
    }
}
fn spans_from_crossings(
    crossings: &mut [(f32, i32)],
    rule: FillRule,
    output: &mut Vec<(f32, f32)>,
) {
    crossings.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    let mut winding = 0i32;
    let mut start = 0.0;
    for &(x, direction) in crossings.iter() {
        let was_inside = match rule {
            FillRule::EvenOdd => winding % 2 != 0,
            FillRule::Winding => winding != 0,
        };
        winding += direction;
        let inside = match rule {
            FillRule::EvenOdd => winding % 2 != 0,
            FillRule::Winding => winding != 0,
        };
        if !was_inside && inside {
            start = x;
        }
        if was_inside && !inside && x > start {
            output.push((start, x));
        }
    }
}
fn rasterize<const CULL_POLYGONS: bool>(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    shapes: &[Shape<'_>],
    paint: &Paint,
    fill_rule: Option<FillRule>,
) {
    let mut min = Point::new(f32::INFINITY, f32::INFINITY);
    let mut max = Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for shape in shapes {
        match shape {
            Shape::Polygon { .. } | Shape::InlinePolygon { .. } => {
                let (points, _, _) = shape.polygon_points().unwrap();
                for p in points {
                    min.x = min.x.min(p.x);
                    min.y = min.y.min(p.y);
                    max.x = max.x.max(p.x);
                    max.y = max.y.max(p.y);
                }
            }
            Shape::Circle(p, r) | Shape::Ring(p, r, _) => {
                min.x = min.x.min(p.x - r);
                min.y = min.y.min(p.y - r);
                max.x = max.x.max(p.x + r);
                max.y = max.y.max(p.y + r);
            }
            Shape::RoundedRect(rect, _) => {
                min.x = min.x.min(rect.rect.x);
                min.y = min.y.min(rect.rect.y);
                max.x = max.x.max(rect.rect.right());
                max.y = max.y.max(rect.rect.bottom());
            }
        }
    }
    if !min.x.is_finite() || !min.y.is_finite() || !max.x.is_finite() || !max.y.is_finite() {
        return;
    }
    let screen = Rect::from_ltwh(0.0, 0.0, pixmap.width as f32, pixmap.height as f32);
    let Some(bounds) = Rect::from_ltrb(min.x, min.y, max.x, max.y).intersect(&screen) else {
        return;
    };
    let bounds = if let Some(clip) = clip {
        let Some(bounds) = bounds.intersect(&clip) else {
            return;
        };
        bounds
    } else {
        bounds
    };
    let left = bounds.x.floor() as i32;
    let right = bounds.right().ceil() as i32;
    let width = (right - left) as usize;
    let mut coverage = vec![0.0f32; width];
    let mut intersections = Vec::new();
    let mut spans = Vec::new();
    let samples = if paint.anti_alias { 8 } else { 1 };
    for y in bounds.y.floor() as i32..bounds.bottom().ceil() as i32 {
        coverage.fill(0.0);
        for sample in 0..samples {
            let sy = y as f32 + (sample as f32 + 0.5) / samples as f32;
            if sy < bounds.y || sy >= bounds.bottom() {
                continue;
            }
            spans.clear();
            intersections.clear();
            for shape in shapes {
                match shape {
                    Shape::Polygon { .. } | Shape::InlinePolygon { .. } => {
                        let (points, min_y, max_y) = shape.polygon_points().unwrap();
                        // The crossing rule already excludes a polygon outside
                        // this half-open range. Skip its edges without changing
                        // the order of the surviving segment/cap/join spans.
                        if CULL_POLYGONS && (sy < min_y || sy >= max_y) {
                            continue;
                        }
                        if fill_rule.is_none() {
                            intersections.clear();
                        }
                        crossings(points, sy, &mut intersections);
                        if fill_rule.is_none() {
                            spans_from_crossings(&mut intersections, FillRule::Winding, &mut spans);
                        }
                    }
                    Shape::Circle(p, radius) => {
                        let dy = sy - p.y;
                        if dy.abs() < *radius {
                            let dx = (radius * radius - dy * dy).sqrt();
                            spans.push((p.x - dx, p.x + dx));
                        }
                    }
                    Shape::Ring(p, outer, inner) => {
                        let dy = sy - p.y;
                        if dy.abs() < *outer {
                            let dx = (outer * outer - dy * dy).sqrt();
                            let inside = (inner * inner - dy * dy).max(0.0).sqrt();
                            spans.push((p.x - dx, p.x - inside));
                            spans.push((p.x + inside, p.x + dx));
                        }
                    }
                    Shape::RoundedRect(outer, inner) => {
                        if let Some((a, b)) = rrect_span(*outer, sy) {
                            if let Some((c, d)) = inner.and_then(|r| rrect_span(r, sy)) {
                                spans.push((a, c));
                                spans.push((d, b));
                            } else {
                                spans.push((a, b));
                            }
                        }
                    }
                }
            }
            if let Some(rule) = fill_rule {
                spans_from_crossings(&mut intersections, rule, &mut spans);
            }
            // Union the stroke's segment/cap/join spans before applying alpha.
            // Overlapping pieces cannot darken a translucent joint twice.
            spans.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
            let mut merged: Option<(f32, f32)> = None;
            for &(a, b) in &spans {
                let a = a.max(bounds.x);
                let b = b.min(bounds.right());
                if b <= a {
                    continue;
                }
                match merged {
                    Some((start, end)) if a <= end => merged = Some((start, end.max(b))),
                    Some((start, end)) => {
                        accumulate(&mut coverage, left, start, end, samples);
                        merged = Some((a, b));
                    }
                    None => merged = Some((a, b)),
                }
            }
            if let Some((a, b)) = merged {
                accumulate(&mut coverage, left, a, b, samples);
            }
        }
        let row = pixmap.row_mut(y as u32);
        // A vertical SVG gradient has one color per scanline. Its RGB565
        // dither repeats every four columns; keep exact colors/AA while avoiding
        // a gradient transform, division and stop search for every solid pixel.
        let row_color = match &paint.shader {
            Shader::SolidColor(color) => Some(*color),
            Shader::Linear(g) if g.start.x == g.end.x && g.transform.ky == 0.0 => {
                Some(g.color_at(left as f32 + 0.5, y as f32 + 0.5))
            }
            _ => None,
        };
        let row_rgb = row_color.map(|color| {
            if matches!(paint.shader, Shader::Linear(_) | Shader::Mapped(_)) {
                std::array::from_fn::<_, 4, _>(|x| dither(color, x as i32, y))
            } else {
                [color.to_rgb565(); 4]
            }
        });
        for (i, &amount) in coverage.iter().enumerate() {
            if amount <= 0.0 {
                continue;
            }
            let x = left + i as i32;
            let color = match row_color {
                Some(color) => color,
                None => match &paint.shader {
                    Shader::Linear(gradient) => gradient.color_at(x as f32 + 0.5, y as f32 + 0.5),
                    Shader::Mapped(gradient) => gradient.color_at(x as f32 + 0.5, y as f32 + 0.5),
                    _ => continue,
                },
            };
            let amount = if paint.anti_alias {
                amount.min(1.0)
            } else {
                f32::from(amount >= 0.5)
            };
            let alpha = (amount * color.a as f32).round() as u8;
            if alpha == 0 {
                continue;
            }
            let rgb = if let Some(colors) = row_rgb {
                colors[x as usize & 3]
            } else if matches!(paint.shader, Shader::Linear(_) | Shader::Mapped(_)) {
                // Preserve gradient precision on the panel's RGB565 output.
                dither(color, x, y)
            } else {
                color.to_rgb565()
            };
            row[x as usize] = if alpha == 255 {
                rgb
            } else {
                blend_rgb565(row[x as usize], rgb, alpha)
            };
        }
    }
}
fn accumulate(coverage: &mut [f32], left: i32, a: f32, b: f32, samples: i32) {
    let begin = a.floor() as i32;
    let end = b.ceil() as i32;
    if begin >= end {
        return;
    }
    if begin + 1 == end {
        coverage[(begin - left) as usize] += (b - a) / samples as f32;
        return;
    }
    coverage[(begin - left) as usize] += ((begin + 1) as f32 - a) / samples as f32;
    let full = 1.0 / samples as f32;
    for value in &mut coverage[(begin + 1 - left) as usize..(end - 1 - left) as usize] {
        *value += full;
    }
    coverage[(end - 1 - left) as usize] += (b - (end - 1) as f32) / samples as f32;
}
fn dither(color: crate::color::Color, x: i32, y: i32) -> u16 {
    const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let noise = (BAYER[y as usize % 4][x as usize % 4] as f32 + 0.5) / 16.0 - 0.5;
    let quantize = |value: u8, levels: f32| {
        (value as f32 * levels / 255.0 + 0.5 + noise)
            .floor()
            .clamp(0.0, levels) as u16
    };
    (quantize(color.r, 31.0) << 11) | (quantize(color.g, 63.0) << 5) | quantize(color.b, 31.0)
}

#[cfg(test)]
mod coverage_compatibility {
    #[test]
    fn optimized_span_matches_original_overlap_accumulation_exactly() {
        for samples in [1, 8] {
            for start in 0..80 {
                let mut old = vec![0.0f32; 128];
                let mut new = old.clone();
                for row in 0..8 {
                    let a = start as f32 * 0.375 + row as f32 * 0.137;
                    let b = (a + 0.05 + row as f32 * 8.413).min(128.0);
                    for x in a.floor() as i32..b.ceil() as i32 {
                        let overlap = b.min((x + 1) as f32) - a.max(x as f32);
                        if overlap > 0.0 {
                            old[x as usize] += overlap / samples as f32;
                        }
                    }
                    super::accumulate(&mut new, 0, a, b, samples);
                }
                assert_eq!(old, new);
            }
        }
    }
}

#[cfg(test)]
mod allocation_and_culling_compatibility {
    use super::*;
    use crate::{Color, GradientStop, LinearGradient, Pixmap565, SpreadMode, Transform};

    // Keep the original allocation-heavy mesh builder as an independent oracle
    // for segment coordinates, vertex order and cap/join construction. The
    // unculled rasterizer also checks the new per-polygon row rejection.
    enum LegacyShape {
        Polygon(Vec<Point>),
        Circle(Point, f32),
    }
    fn legacy_stroke(
        pixmap: &mut Pixmap565Mut<'_>,
        clip: Option<Rect>,
        polylines: &[Vec<Point>],
        paint: &Paint,
        stroke: &Stroke,
    ) {
        let half = stroke.width * 0.5;
        if half <= 0.0 || !half.is_finite() {
            return;
        }
        let mut shapes = Vec::new();
        for points in polylines {
            if points.len() < 2 {
                continue;
            }
            let closed = points.first() == points.last();
            let mut normals = Vec::new();
            for pair in points.windows(2) {
                let a = pair[0];
                let b = pair[1];
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let length = (dx * dx + dy * dy).sqrt();
                if length < 0.00001 {
                    normals.push(Point::ZERO);
                    continue;
                }
                let n = Point::new(-dy * half / length, dx * half / length);
                normals.push(n);
                shapes.push(LegacyShape::Polygon(vec![
                    Point::new(a.x + n.x, a.y + n.y),
                    Point::new(a.x - n.x, a.y - n.y),
                    Point::new(b.x - n.x, b.y - n.y),
                    Point::new(b.x + n.x, b.y + n.y),
                ]));
            }
            let count = if closed {
                points.len() - 1
            } else {
                points.len()
            };
            for i in 0..count {
                let p = points[i];
                let endpoint = !closed && (i == 0 || i == count - 1);
                if endpoint {
                    match stroke.line_cap {
                        LineCap::Round => shapes.push(LegacyShape::Circle(p, half)),
                        LineCap::Square => {
                            let n = if i == 0 {
                                normals[0]
                            } else {
                                *normals.last().unwrap()
                            };
                            let direction = if i == 0 { -1.0 } else { 1.0 };
                            let t = Point::new(n.y * direction, -n.x * direction);
                            shapes.push(LegacyShape::Polygon(vec![
                                Point::new(p.x + n.x, p.y + n.y),
                                Point::new(p.x - n.x, p.y - n.y),
                                Point::new(p.x - n.x + t.x, p.y - n.y + t.y),
                                Point::new(p.x + n.x + t.x, p.y + n.y + t.y),
                            ]));
                        }
                        LineCap::Butt => (),
                    }
                } else if stroke.line_join == LineJoin::Round {
                    shapes.push(LegacyShape::Circle(p, half));
                } else {
                    let before = normals[(i + normals.len() - 1) % normals.len()];
                    let after = normals[i % normals.len()];
                    for side in [-1.0, 1.0] {
                        let a = Point::new(p.x + before.x * side, p.y + before.y * side);
                        let b = Point::new(p.x + after.x * side, p.y + after.y * side);
                        let mut join = vec![p, a];
                        if stroke.line_join == LineJoin::Miter {
                            let sum = Point::new(
                                (before.x + after.x) * side,
                                (before.y + after.y) * side,
                            );
                            let dot = sum.x * after.x * side + sum.y * after.y * side;
                            if dot.abs() > 0.00001 {
                                let factor = half * half / dot;
                                let mx = sum.x * factor;
                                let my = sum.y * factor;
                                if mx * mx + my * my <= (half * stroke.miter_limit).powi(2) {
                                    join.push(Point::new(p.x + mx, p.y + my));
                                }
                            }
                        }
                        join.push(b);
                        shapes.push(LegacyShape::Polygon(join));
                    }
                }
            }
        }
        let borrowed: Vec<_> = shapes
            .iter()
            .map(|shape| match shape {
                LegacyShape::Polygon(points) => Shape::polygon(points),
                LegacyShape::Circle(center, radius) => Shape::Circle(*center, *radius),
            })
            .collect();
        rasterize::<false>(pixmap, clip, &borrowed, paint, None);
    }

    fn paints() -> Vec<Paint<'static>> {
        let mut gradient = Paint::new(Color::RED);
        gradient.shader = LinearGradient::new(
            Point::new(1.0, 7.0),
            Point::new(72.0, 61.0),
            vec![
                GradientStop::new(0.0, Color::from_rgba(20, 150, 244, 173)),
                GradientStop::new(1.0, Color::from_rgba(230, 84, 42, 219)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();
        vec![Paint::new(Color::from_rgba(231, 245, 249, 137)), gradient]
    }
    fn transformed_polylines(transform: Transform) -> Vec<Vec<Point>> {
        let lines = [
            vec![
                Point::new(-4.3, 9.6),
                Point::new(19.7, 51.4),
                Point::new(19.7, 51.4),
                Point::new(50.6, 8.2),
                Point::new(56.2, 41.6),
                Point::new(85.3, 55.1),
            ],
            vec![
                Point::new(11.5, 17.8),
                Point::new(57.1, 18.1),
                Point::new(56.6, 48.8),
                Point::new(11.5, 17.8),
            ],
            vec![
                Point::new(22.1, 44.3),
                Point::new(34.8, 11.8),
                Point::new(22.1, 44.3),
            ],
        ];
        lines
            .into_iter()
            .map(|line| line.into_iter().map(|p| transform.map_point(p)).collect())
            .collect()
    }
    #[test]
    fn inline_mesh_and_row_culling_match_original_stroke_pixels_exactly() {
        let transforms = [
            Transform::identity(),
            Transform::from_translate(3.25, -4.6),
            Transform::from_scale(1.125, 0.875),
            Transform::from_row(-1.0, 0.0, 0.0, -1.0, 85.4, 65.8),
            Transform::from_row(1.1, 0.15, -0.35, 0.85, 8.3, -4.25),
        ];
        let clips = [
            None,
            Some(Rect::from_ltwh(12.25, 13.5, 49.75, 42.25)),
            Some(Rect::from_ltwh(-12.5, -8.75, 37.3, 29.6)),
            Some(Rect::from_ltwh(90.0, 71.0, 20.0, 15.0)),
        ];
        for transform in transforms {
            let lines = transformed_polylines(transform);
            for mut paint in paints() {
                for aa in [false, true] {
                    paint.anti_alias = aa;
                    for cap in [LineCap::Butt, LineCap::Round, LineCap::Square] {
                        for join in [LineJoin::Round, LineJoin::Miter, LineJoin::Bevel] {
                            for (width, miter_limit) in [(1.3, 1.1), (9.6, 4.0)] {
                                let style = Stroke {
                                    width,
                                    line_cap: cap,
                                    line_join: join,
                                    miter_limit,
                                };
                                for clip in clips {
                                    let mut actual = Pixmap565::new(96, 72).unwrap();
                                    actual.fill(0x3186);
                                    let mut expected = actual.clone();
                                    stroke(&mut actual.as_mut(), clip, &lines, &paint, &style);
                                    legacy_stroke(
                                        &mut expected.as_mut(),
                                        clip,
                                        &lines,
                                        &paint,
                                        &style,
                                    );
                                    assert_eq!(actual, expected, "cap={cap:?} join={join:?} aa={aa} width={width} clip={clip:?}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn borrowed_compound_fills_match_unculled_winding_and_even_odd_pixels() {
        for transform in [
            Transform::identity(),
            Transform::from_translate(-5.25, 8.75),
        ] {
            let polygons = transformed_polylines(transform);
            let shapes: Vec<_> = polygons
                .iter()
                .map(|points| Shape::polygon(points))
                .collect();
            for rule in [FillRule::Winding, FillRule::EvenOdd] {
                for mut paint in paints() {
                    for aa in [false, true] {
                        paint.anti_alias = aa;
                        for clip in [None, Some(Rect::from_ltwh(9.25, 5.5, 56.1, 48.3))] {
                            let mut actual = Pixmap565::new(96, 72).unwrap();
                            actual.fill(0x3186);
                            let mut expected = actual.clone();
                            fill(&mut actual.as_mut(), clip, &polygons, &paint, rule);
                            rasterize::<false>(
                                &mut expected.as_mut(),
                                clip,
                                &shapes,
                                &paint,
                                Some(rule),
                            );
                            assert_eq!(actual, expected);
                        }
                    }
                }
            }
        }
    }
}
