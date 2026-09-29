//! Circular tint/reveal compositor with a narrow refractive, borderless rim.
//! Samples the untouched source row before writing; scratch is one RGB565 row.
use crate::color::{blend_rgb565, Color};
use crate::geometry::{Point, Rect};
use crate::pixmap::Pixmap565Mut;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GlassFill {
    Inside,
    Outside,
    EdgeOnly,
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub(crate) fn composite(
    pixels: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    center: Point,
    radius: f32,
    tint: Color,
    fill: GlassFill,
    rim: f32,
    strength: f32,
) {
    if !center.x.is_finite()
        || !center.y.is_finite()
        || !radius.is_finite()
        || radius < 0.0
        || !rim.is_finite()
        || rim <= 0.0
    {
        return;
    }
    let screen = Rect::from_ltwh(0.0, 0.0, pixels.width as f32, pixels.height as f32);
    let Some(clip) = clip.map_or(Some(screen), |r| screen.intersect(&r)) else {
        return;
    };
    let left = clip.x.ceil() as usize;
    let right = clip.right().floor() as usize;
    let top = clip.y.ceil() as usize;
    let bottom = clip.bottom().floor() as usize;
    let outer = radius + rim + 2.0;
    let inner = (radius - rim - 2.0).max(0.0);
    let color = tint.to_rgb565();
    let strength = strength.clamp(0.0, 1.0);
    let mut source = vec![0u16; pixels.width as usize];
    for y in top..bottom {
        let dy = y as f32 + 0.5 - center.y;
        let row = pixels.row_mut(y as u32);
        source.copy_from_slice(row);
        let outer_x = (outer * outer - dy * dy).max(0.0).sqrt();
        let inner_x = (inner * inner - dy * dy).max(0.0).sqrt();
        let start = ((center.x - outer_x).floor() as i32).clamp(left as i32, right as i32) as usize;
        let end = ((center.x + outer_x).ceil() as i32).clamp(left as i32, right as i32) as usize;
        let core_start =
            ((center.x - inner_x).ceil() as i32).clamp(start as i32, end as i32) as usize;
        let core_end =
            ((center.x + inner_x).floor() as i32).clamp(core_start as i32, end as i32) as usize;
        let tint_span = |row: &mut [u16], a: usize, b: usize| {
            if tint.a == 255 {
                row[a..b].fill(color);
            } else if tint.a != 0 {
                for x in a..b {
                    row[x] = blend_rgb565(source[x], color, tint.a);
                }
            }
        };
        if fill == GlassFill::Outside {
            tint_span(row, left, start);
            tint_span(row, end, right);
        }
        if dy.abs() >= outer {
            // Outside mode covers this entire row; Inside/EdgeOnly leave it alone.
            if fill == GlassFill::Outside {
                tint_span(row, left, right);
            }
            continue;
        }
        if fill == GlassFill::Inside {
            tint_span(row, core_start, core_end);
        }
        for (a, b) in [(start, core_start), (core_end, end)] {
            for x in a..b {
                let dx = x as f32 + 0.5 - center.x;
                let distance = (dx * dx + dy * dy).sqrt();
                let signed = distance - radius;
                let near = 1.0 - smooth(signed.abs() / rim);
                // Horizontal bend uses only the original row. A read cannot
                // see an earlier tint written by this compositor.
                let bend = (dx / distance.max(1.0) * near * 5.0 * strength).round() as i32;
                let sample = (x as i32 + bend).clamp(0, source.len() as i32 - 1) as usize;
                let blurred = blend_rgb565(
                    source[sample],
                    source[(sample + 1).min(source.len() - 1)],
                    64,
                );
                let refracted = blend_rgb565(source[x], blurred, (near * strength * 170.0) as u8);
                let coverage = smooth((radius - distance + 1.0) * 0.5);
                let amount = match fill {
                    GlassFill::Inside => coverage,
                    GlassFill::Outside => 1.0 - coverage,
                    GlassFill::EdgeOnly => 0.0,
                };
                // Refraction and coverage create the moving edge. Do not add
                // a white lip, inner reflection or a contrasting outline.
                row[x] = blend_rgb565(refracted, color, (amount * tint.a as f32).round() as u8);
            }
        }
    }
}
