//! Pre-baked bitmap font atlas (离线烘焙字库).
//! Zero runtime TTF parsing overhead. Loads compact .bin files via include_bytes!.

#![allow(dead_code)]

/// Raw glyph entry stored in binary font file (.bin), 4-byte natural alignment (20 Bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawGlyph {
    pub ch: u32,
    pub width: u8,
    pub height: u8,
    pub xmin: i8,
    pub ymin: i8,
    pub advance: f32,
    pub bitmap_offset: u32,
    pub bitmap_len: u16,
    pub _reserved: u16,
}

/// Resolved glyph view containing a zero-copy slice to static bitmap data.
#[derive(Clone, Copy, Debug)]
pub struct BakedGlyph {
    pub ch: char,
    pub width: u16,
    pub height: u16,
    pub xmin: i16,
    pub ymin: i16,
    pub advance_width: f32,
    pub bitmap: &'static [u8],
}

/// Pre-baked font asset loaded from binary asset data.
pub struct BakedFont {
    pub size: u16,
    pub cap_height: f32,
    ascii_count: u16,
    extra_count: u16,
    bitmap_pool_offset: usize,
    data: &'static [u8],
}

impl BakedFont {
    /// Parse and validate pre-baked binary font asset at compile time.
    pub const fn from_bytes(data: &'static [u8]) -> Self {
        assert!(data.len() >= 16, "Invalid font data: smaller than header");
        assert!(
            data[0] == b'A' && data[1] == b'R' && data[2] == b'F' && data[3] == b'N',
            "Invalid font magic: expected ARFN"
        );

        let size = u16::from_le_bytes([data[4], data[5]]);
        let cap_height = f32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let ascii_count = u16::from_le_bytes([data[10], data[11]]);
        let extra_count = u16::from_le_bytes([data[12], data[13]]);
        let total_glyphs = (ascii_count + extra_count) as usize;

        let glyph_table_start = 16;
        let glyph_table_bytes = total_glyphs * std::mem::size_of::<RawGlyph>();
        let bitmap_pool_offset = glyph_table_start + glyph_table_bytes;
        assert!(data.len() >= bitmap_pool_offset, "Font data truncated");

        Self {
            size,
            cap_height,
            ascii_count,
            extra_count,
            bitmap_pool_offset,
            data,
        }
    }

    #[inline(always)]
    fn raw_glyph_at(&self, index: usize) -> RawGlyph {
        let offset = 16 + index * std::mem::size_of::<RawGlyph>();
        unsafe { std::ptr::read_unaligned(self.data.as_ptr().add(offset) as *const RawGlyph) }
    }

    #[inline(always)]
    fn build_glyph(&self, raw: RawGlyph, ch: char) -> BakedGlyph {
        let bmp_start = self.bitmap_pool_offset + raw.bitmap_offset as usize;
        let bmp_end = bmp_start + raw.bitmap_len as usize;
        let bitmap = if bmp_start <= self.data.len() && bmp_end <= self.data.len() {
            unsafe {
                std::slice::from_raw_parts(self.data.as_ptr().add(bmp_start), bmp_end - bmp_start)
            }
        } else {
            &[]
        };

        BakedGlyph {
            ch,
            width: raw.width as u16,
            height: raw.height as u16,
            xmin: raw.xmin as i16,
            ymin: raw.ymin as i16,
            advance_width: raw.advance,
            bitmap,
        }
    }

    /// Retrieve a baked glyph with O(1) ASCII lookup or O(log N) binary search for extra symbols.
    #[inline(always)]
    pub fn get_glyph(&self, ch: char) -> Option<BakedGlyph> {
        let code = ch as u32;
        if code >= 32 && code <= 126 {
            let idx = (code - 32) as usize;
            if idx < self.ascii_count as usize {
                let raw = self.raw_glyph_at(idx);
                return Some(self.build_glyph(raw, ch));
            }
        }

        // Binary search in sorted extra symbols
        let extra_start = self.ascii_count as usize;
        let mut low = 0;
        let mut high = self.extra_count as usize;
        while low < high {
            let mid = (low + high) / 2;
            let raw = self.raw_glyph_at(extra_start + mid);
            if raw.ch == code {
                return Some(self.build_glyph(raw, ch));
            } else if raw.ch < code {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        None
    }

    /// Measure text dimensions (width, cap_height) with pre-baked font metrics.
    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let mut width = 0.0f32;
        for ch in text.chars() {
            if ch == '\u{200A}' {
                width += (self.size as f32 * 0.068).round().max(2.0);
                continue;
            }
            if let Some(g) = self.get_glyph(ch) {
                width += g.advance_width;
            } else {
                width += self.size as f32 * 0.6;
            }
        }
        (width, self.cap_height)
    }
}

pub static FONT_14PX: BakedFont = BakedFont::from_bytes(include_bytes!(
    "../../../../assets/fonts/baked/font_14px.bin"
));
pub static FONT_18PX: BakedFont = BakedFont::from_bytes(include_bytes!(
    "../../../../assets/fonts/baked/font_18px.bin"
));
pub static FONT_22PX: BakedFont = BakedFont::from_bytes(include_bytes!(
    "../../../../assets/fonts/baked/font_22px.bin"
));
pub static FONT_34PX: BakedFont = BakedFont::from_bytes(include_bytes!(
    "../../../../assets/fonts/baked/font_34px.bin"
));

/// Retrieve a pre-baked font for the given pixel size if available.
pub fn get_baked_font(px_size: f32) -> Option<&'static BakedFont> {
    let s = px_size.round() as u16;
    match s {
        14 => Some(&FONT_14PX),
        18 => Some(&FONT_18PX),
        22 => Some(&FONT_22PX),
        34 => Some(&FONT_34PX),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baked_fonts_initialization_and_metadata() {
        for &size in &[14.0, 18.0, 22.0, 34.0] {
            let font = get_baked_font(size).expect("Font size must be present");
            assert_eq!(font.size, size as u16);
            assert!(font.cap_height > 0.0);
            assert_eq!(font.ascii_count, 95);
            assert_eq!(font.extra_count, 10);
        }

        assert!(get_baked_font(16.0).is_none());
        assert!(get_baked_font(30.0).is_none());
    }

    #[test]
    fn test_ascii_and_extra_glyph_lookup() {
        let font = get_baked_font(34.0).unwrap();

        // Space glyph
        let space = font.get_glyph(' ').expect("Space must exist");
        assert_eq!(space.width, 0);
        assert_eq!(space.height, 0);
        assert_eq!(space.bitmap.len(), 0);
        assert!(space.advance_width > 0.0);

        // Standard ASCII glyph
        let glyph_a = font.get_glyph('A').expect("A must exist");
        assert!(glyph_a.width > 0);
        assert!(glyph_a.height > 0);
        assert_eq!(
            glyph_a.bitmap.len(),
            glyph_a.width as usize * glyph_a.height as usize
        );
        assert!(glyph_a.advance_width > 0.0);

        // Extra UI symbols (Calculator, Cursor, Keyboard, Arrows)
        let extra_chars = ['×', '÷', '±', '√', '█', '⇧', '⬆', '▲', '⌫', '↵'];
        for ch in extra_chars {
            let g = font
                .get_glyph(ch)
                .unwrap_or_else(|| panic!("Extra glyph {} must exist", ch));
            assert!(g.width > 0, "Glyph {} width should be > 0", ch);
            assert!(g.height > 0, "Glyph {} height should be > 0", ch);
            assert_eq!(
                g.bitmap.len(),
                g.width as usize * g.height as usize,
                "Glyph {} bitmap size mismatch",
                ch
            );
            assert!(g.advance_width > 0.0);
        }

        // Non-existent glyphs
        assert!(font.get_glyph('中').is_none());
        assert!(font.get_glyph('🦀').is_none());
    }

    #[test]
    fn test_text_measurement() {
        let font = get_baked_font(18.0).unwrap();
        let (w1, h1) = font.measure_text("Hello");
        let (w2, h2) = font.measure_text("Hello World");
        assert!(w2 > w1);
        assert_eq!(h1, font.cap_height);
        assert_eq!(h2, font.cap_height);

        // Measurement with extra symbols
        let (w_calc, _) = font.measure_text("125×8÷2");
        assert!(w_calc > 0.0);
    }
}
