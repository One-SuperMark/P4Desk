//! Coverage rasterization for vector fills and strokes. Eight vertical samples
//! and analytic horizontal coverage use a bounded row, never a supersized canvas.
use crate::color::blend_rgb565;
use crate::geometry::{Point, RRect, Radius, Rect};
use crate::paint::{FillRule, LineCap, LineJoin, Paint, Shader, Stroke};
use crate::pixmap::Pixmap565Mut;

#[derive(Clone)]
enum Shape {
    Polygon(Vec<Point>),
    Circle(Point, f32),
    Ring(Point, f32, f32),
    RoundedRect(RRect, Option<RRect>),
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
    rasterize(pixmap, clip, &[shape], paint, None);
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
    rasterize(
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
    rasterize(
        pixmap,
        clip,
        &polygons
            .iter()
            .cloned()
            .map(Shape::Polygon)
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
            shapes.push(Shape::Polygon(vec![
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
                        shapes.push(Shape::Polygon(vec![
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
                    let mut join = vec![p, a];
                    if stroke.line_join == LineJoin::Miter {
                        let sum =
                            Point::new((before.x + after.x) * side, (before.y + after.y) * side);
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
                    shapes.push(Shape::Polygon(join));
                }
            }
        }
    }
    rasterize(pixmap, clip, &shapes, paint, None);
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
fn rasterize(
    pixmap: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    shapes: &[Shape],
    paint: &Paint,
    fill_rule: Option<FillRule>,
) {
    let mut min = Point::new(f32::INFINITY, f32::INFINITY);
    let mut max = Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for shape in shapes {
        match shape {
            Shape::Polygon(points) => {
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
                    Shape::Polygon(points) => {
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
