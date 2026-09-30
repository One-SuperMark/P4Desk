//! Rounded glass over an immutable prefiltered wallpaper layer.
//! No live-frame feedback, full-screen blur, allocation or per-pixel square root.
use crate::pixmap::Pixmap565Mut;
use crate::{blend_rgb565, rgb888_to_rgb565, Color, Rect};

#[derive(Clone, Copy)]
pub enum GlassFinish {
    Soft,
    Lens,
    Frosted,
}

pub(crate) fn panel(
    pixels: &mut Pixmap565Mut<'_>,
    clip: Option<Rect>,
    rect: Rect,
    radius: f32,
    tint: Color,
    source: &[u8],
    sw: u32,
    sh: u32,
    amount: u8,
    pressed: bool,
    finish: GlassFinish,
) {
    if sw < 1
        || sh < 1
        || sw > 2048
        || sh > 2048
        || source.len() != sw as usize * sh as usize * 3
        || !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
        || !radius.is_finite()
    {
        return;
    }
    let screen = Rect::from_ltwh(0.0, 0.0, pixels.width as f32, pixels.height as f32);
    let Some(bounds) = rect
        .intersect(&clip.unwrap_or(screen))
        .and_then(|r| r.intersect(&screen))
    else {
        return;
    };
    let r = radius.clamp(0.0, rect.width.min(rect.height) * 0.5);
    let sx = ((sw - 1) << 8) / pixels.width.max(1);
    let sy = ((sh - 1) << 8) / pixels.height.max(1);
    let color = [tint.r, tint.g, tint.b];
    let frosted = matches!(finish, GlassFinish::Frosted);
    let lens = matches!(finish, GlassFinish::Lens | GlassFinish::Frosted);
    let opacity = if lens {
        (34 + amount.min(100) as u16 * 116 / 100) as u8
    } else {
        (68 + amount.min(100) as u16 * 155 / 100) as u8
    };
    let solid = (sw == 1 && sh == 1).then(|| [source[0], source[1], source[2]]);
    let sample = |x: i32, y: i32| {
        if let Some(color) = solid {
            return color;
        }
        let u = (x.max(0) as u32 * sx).min((sw - 1) << 8);
        let v = (y.max(0) as u32 * sy).min((sh - 1) << 8);
        let x0 = u >> 8;
        let y0 = v >> 8;
        let at = |x: u32, y: u32| {
            let i = (y * sw + x) as usize * 3;
            [source[i], source[i + 1], source[i + 2]]
        };
        let a = at(x0, y0);
        let b = at((x0 + 1).min(sw - 1), y0);
        let c = at(x0, (y0 + 1).min(sh - 1));
        let d = at((x0 + 1).min(sw - 1), (y0 + 1).min(sh - 1));
        let mix = |a: u8, b: u8, t: u32| ((a as u32 * (256 - t) + b as u32 * t) >> 8) as u8;
        std::array::from_fn::<_, 3, _>(|i| {
            mix(mix(a[i], b[i], u & 255), mix(c[i], d[i], u & 255), v & 255)
        })
    };
    for y in bounds.y.floor().max(0.0) as usize..bounds.bottom().ceil() as usize {
        let dy = (y as f32 + 0.5 - rect.y).min(rect.bottom() - y as f32 - 0.5);
        let corner = (r - dy).max(0.0);
        let inset = if corner > 0.0 {
            r - (r * r - corner * corner).max(0.0).sqrt()
        } else {
            0.0
        };
        let left = rect.x + inset;
        let right = rect.right() - inset;
        let start = bounds.x.max(left - 0.5).floor().max(0.0) as usize;
        let end = bounds
            .right()
            .min(right + 0.5)
            .ceil()
            .min(pixels.width as f32) as usize;
        let row = pixels.row_mut(y as u32);
        let vertical = ((y as f32 - rect.y) / rect.height).clamp(0.0, 1.0);
        for x in start..end {
            let dx = (x as f32 + 0.5 - left).min(right - x as f32 - 0.5);
            let edge = dx.min(dy);
            let coverage = ((edge + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            if coverage == 0 {
                continue;
            }
            let bend = if lens {
                ((1.0 - edge.max(0.0) / 16.0).max(0.0) * 15.0) as i32
            } else {
                ((1.0 - edge.max(0.0) / 9.0).max(0.0) * 7.0) as i32
            };
            let bx = if (x as f32) < rect.x + rect.width * 0.5 {
                -bend
            } else {
                bend
            };
            let by = if vertical < 0.5 { -bend } else { bend };
            // A mild center magnification makes the card read as a curved lens.
            let magnify_x = if lens {
                ((x as f32 - rect.x - rect.width * 0.5) * 0.035) as i32
            } else {
                0
            };
            let magnify_y = if lens {
                ((y as f32 - rect.y - rect.height * 0.5) * 0.08) as i32
            } else {
                0
            };
            let refracted = sample(x as i32 + bx - magnify_x, y as i32 + by - magnify_y);

            // A thin directional reflection, then a broad soft inner highlight.
            let rim = if r > 0.0 {
                (1.0 - edge.max(0.0) / 1.4).max(0.0)
            } else {
                0.0
            };
            let highlight = (if frosted {
                // A broad, weak reflection with a continuous shoulder. The
                // modal must not have the bright upper cap of a small lens.
                let top = (1.0 - vertical / 0.80).clamp(0.0, 1.0);
                let soft_top = top * top * (3.0 - 2.0 * top);
                rim * (28.0 - vertical * 14.0) + soft_top * 6.0
            } else if lens {
                let top = (1.0 - vertical / 0.80).clamp(0.0, 1.0);
                let soft_top = top * top * (3.0 - 2.0 * top);
                let lower_edge = ((vertical - 0.80) / 0.20).max(0.0);
                rim * (38.0 - vertical * 18.0) + soft_top * 9.0 + lower_edge * 3.0
            } else {
                rim * (if vertical < 0.35 { 52.0 } else { 20.0 }) + (1.0 - vertical) * 9.0
            } + if pressed { 24.0 } else { 0.0 }) as u8;
            let rgb: [u8; 3] = std::array::from_fn(|i| {
                let base = (refracted[i] as u32 * (255 - opacity as u32)
                    + color[i] as u32 * opacity as u32
                    + 127)
                    / 255;
                ((base * (255 - highlight as u32) + 255 * highlight as u32 + 127) / 255) as u8
            });
            // Quantize once, with screen-anchored ordered dithering. This avoids
            // broad RGB565 terraces and stays identical under clipped repaint.
            const BAYER: [i32; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
            let noise = BAYER[(y & 3) * 4 + (x & 3)] - 8;
            let out = rgb888_to_rgb565(
                (rgb[0] as i32 + noise / 2).clamp(0, 255) as u8,
                (rgb[1] as i32 + noise / 4).clamp(0, 255) as u8,
                (rgb[2] as i32 + noise / 2).clamp(0, 255) as u8,
            );
            row[x] = if coverage == 255 {
                out
            } else {
                blend_rgb565(row[x], out, coverage)
            };
        }
    }
}
