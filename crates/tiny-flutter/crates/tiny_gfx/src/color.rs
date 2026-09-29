#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::from_rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);

    #[inline(always)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline(always)]
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub fn red(&self) -> f32 {
        self.r as f32 / 255.0
    }

    #[inline(always)]
    pub fn green(&self) -> f32 {
        self.g as f32 / 255.0
    }

    #[inline(always)]
    pub fn blue(&self) -> f32 {
        self.b as f32 / 255.0
    }

    #[inline(always)]
    pub fn alpha(&self) -> f32 {
        self.a as f32 / 255.0
    }

    #[inline(always)]
    pub fn to_color_u8(&self) -> ColorU8 {
        ColorU8 {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    #[inline(always)]
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    #[inline(always)]
    pub fn to_rgb565(&self) -> u16 {
        rgb888_to_rgb565(self.r, self.g, self.b)
    }

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        self.a == 255
    }

    #[inline(always)]
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// Convert 8-bit RGB to 16-bit RGB565.
#[inline(always)]
pub const fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | ((b as u16) >> 3)
}

/// Convert 16-bit RGB565 to 8-bit RGB tuple (R, G, B).
#[inline(always)]
pub const fn rgb565_to_rgb888(c: u16) -> (u8, u8, u8) {
    let r = (((c >> 11) & 0x1F) * 255 / 31) as u8;
    let g = (((c >> 5) & 0x3F) * 255 / 63) as u8;
    let b = ((c & 0x1F) * 255 / 31) as u8;
    (r, g, b)
}

/// High-speed alpha blend between two RGB565 pixels:
/// result = src * alpha + dst * (255 - alpha).
#[inline(always)]
pub fn blend_rgb565(dst: u16, src: u16, alpha: u8) -> u16 {
    if alpha == 255 {
        return src;
    }
    if alpha == 0 {
        return dst;
    }
    let a = alpha as u32;
    let inv_a = 255 - a;

    let d = dst as u32;
    let s = src as u32;

    let r_d = (d >> 11) & 0x1F;
    let g_d = (d >> 5) & 0x3F;
    let b_d = d & 0x1F;

    let r_s = (s >> 11) & 0x1F;
    let g_s = (s >> 5) & 0x3F;
    let b_s = s & 0x1F;

    let r = ((r_s * a + r_d * inv_a + 127) / 255) as u16;
    let g = ((g_s * a + g_d * inv_a + 127) / 255) as u16;
    let b = ((b_s * a + b_d * inv_a + 127) / 255) as u16;

    (r << 11) | (g << 5) | b
}

/// Exact channel lookup for a constant color/alpha over a large pixel span.
/// 256 stack bytes replace repeated multiply/divide operations per pixel.
pub(crate) struct SolidBlend565 {
    red: [u16; 32],
    green: [u16; 64],
    blue: [u16; 32],
}
impl SolidBlend565 {
    pub(crate) fn new(src: u16, alpha: u8) -> Self {
        let a = u32::from(alpha);
        let inverse = 255 - a;
        let channel = |source: u16, destination: usize, shift: u32| {
            (((u32::from(source) * a + destination as u32 * inverse + 127) / 255) as u16) << shift
        };
        Self {
            red: std::array::from_fn(|d| channel((src >> 11) & 31, d, 11)),
            green: std::array::from_fn(|d| channel((src >> 5) & 63, d, 5)),
            blue: std::array::from_fn(|d| channel(src & 31, d, 0)),
        }
    }
    pub(crate) fn apply(&self, pixels: &mut [u16]) {
        for pixel in pixels {
            let d = *pixel;
            *pixel = self.red[(d >> 11) as usize]
                | self.green[((d >> 5) & 63) as usize]
                | self.blue[(d & 31) as usize];
        }
    }
}

#[cfg(test)]
mod solid_blend_tests {
    use super::*;

    #[test]
    fn constant_blend_matches_original_rounding_for_all_channel_values_and_alpha() {
        for (limit, shift) in [(32, 11), (64, 5), (32, 0)] {
            for source in 0..limit {
                for alpha in 0..=255 {
                    let src = source << shift;
                    let table = SolidBlend565::new(src, alpha);
                    let mut pixels: Vec<u16> = (0..limit).map(|d| d << shift).collect();
                    let expected: Vec<u16> = pixels
                        .iter()
                        .map(|&d| blend_rgb565(d, src, alpha))
                        .collect();
                    table.apply(&mut pixels);
                    assert_eq!(pixels, expected);
                }
            }
        }
        // Mixed RGB words protect masking and the independently shifted tables.
        for src in [0, 0xffff, 0x18c3, 0x2945, 0xf800, 0x07e0, 0x001f] {
            for alpha in [0, 1, 17, 64, 127, 128, 213, 254, 255] {
                let mut pixels: Vec<u16> = (0..=u16::MAX).collect();
                SolidBlend565::new(src, alpha).apply(&mut pixels);
                for (dst, actual) in pixels.into_iter().enumerate() {
                    assert_eq!(actual, blend_rgb565(dst as u16, src, alpha));
                }
            }
        }
    }
}

/// Blend an RGB888 color with alpha onto an RGB565 destination pixel.
#[inline(always)]
pub fn blend_rgb888_onto_rgb565(dst: u16, r: u8, g: u8, b: u8, alpha: u8) -> u16 {
    if alpha == 255 {
        return rgb888_to_rgb565(r, g, b);
    }
    if alpha == 0 {
        return dst;
    }
    let a = alpha as u32;
    let inv_a = 255 - a;

    let d = dst as u32;
    let r_d = (d >> 11) & 0x1F;
    let g_d = (d >> 5) & 0x3F;
    let b_d = d & 0x1F;

    let r_s = (r as u32) >> 3;
    let g_s = (g as u32) >> 2;
    let b_s = (b as u32) >> 3;

    let r_out = ((r_s * a + r_d * inv_a + 127) / 255) as u16;
    let g_out = ((g_s * a + g_d * inv_a + 127) / 255) as u16;
    let b_out = ((b_s * a + b_d * inv_a + 127) / 255) as u16;

    (r_out << 11) | (g_out << 5) | b_out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorU8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorU8 {
    #[inline(always)]
    pub const fn red(&self) -> u8 {
        self.r
    }

    #[inline(always)]
    pub const fn green(&self) -> u8 {
        self.g
    }

    #[inline(always)]
    pub const fn blue(&self) -> u8 {
        self.b
    }

    #[inline(always)]
    pub const fn alpha(&self) -> u8 {
        self.a
    }
}
