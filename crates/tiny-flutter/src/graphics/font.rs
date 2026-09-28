use crate::graphics::baked_font::get_baked_font;
use crate::graphics::fontpack::{FontPack, PackGlyphRef};
use crate::graphics::geometry::Size;
use fontdue::{Font as InnerFont, FontSettings, Metrics};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

static DEFAULT_FONT_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/source/Roboto-Subset.ttf");
static UI_FONT_BYTES: &[u8] = include_bytes!("../../../../assets/generated/ui.p4f");
static DEFAULT_FONT: OnceLock<Font> = OnceLock::new();
static UI_PACK: OnceLock<Option<Arc<FontPack>>> = OnceLock::new();
static ACTIVE_PACK: RwLock<Option<Arc<FontPack>>> = RwLock::new(None);
const CACHE_BYTES: usize = 512 * 1024;
const CACHE_GLYPHS: usize = 512;

/// Atomically replace the owned, file-backed font provider. Existing glyph references remain valid.
pub fn install_fontpack(pack: Option<Arc<FontPack>>) {
    *ACTIVE_PACK.write().unwrap_or_else(|p| p.into_inner()) = pack;
}

fn ui_pack() -> Option<&'static Arc<FontPack>> {
    UI_PACK
        .get_or_init(|| {
            FontPack::from_bytes(UI_FONT_BYTES.to_vec())
                .ok()
                .map(Arc::new)
        })
        .as_ref()
}

#[derive(Clone)]
enum Bitmap {
    Static(&'static [u8]),
    Owned(Arc<[u8]>),
    Pack(PackGlyphRef),
}

#[derive(Clone)]
pub struct Glyph {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance: f32,
    bitmap: Bitmap,
}
impl Glyph {
    pub fn bitmap(&self) -> &[u8] {
        match &self.bitmap {
            Bitmap::Static(v) => v,
            Bitmap::Owned(v) => v,
            Bitmap::Pack(v) => v.bitmap(),
        }
    }
    fn from_pack(g: PackGlyphRef) -> Self {
        Self {
            width: g.width as usize,
            height: g.height as usize,
            xmin: g.xmin as i32,
            ymin: g.ymin as i32,
            advance: g.advance,
            bitmap: Bitmap::Pack(g),
        }
    }
}

#[derive(Default)]
struct GlyphCache {
    entries: HashMap<(char, u32), Glyph>,
    order: VecDeque<(char, u32)>,
    bytes: usize,
}

#[derive(Clone)]
pub struct Font {
    inner: Arc<InnerFont>,
    cache: Arc<Mutex<GlyphCache>>,
    is_default: bool,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub lines: Vec<String>,
    pub size: Size,
    pub line_height: f32,
}

impl Font {
    /// TTF parsing is intended for the small bundled Latin font or host tools, never full CJK on P4.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let inner =
            InnerFont::from_bytes(bytes, FontSettings::default()).map_err(|_| "font_parse")?;
        Ok(Self {
            inner: Arc::new(inner),
            cache: Arc::new(Mutex::new(GlyphCache::default())),
            is_default: false,
        })
    }
    pub fn default_font() -> &'static Font {
        DEFAULT_FONT.get_or_init(|| {
            let mut f = Self::from_bytes(DEFAULT_FONT_BYTES).expect("bundled Latin font");
            f.is_default = true;
            f
        })
    }
    pub fn inner(&self) -> &InnerFont {
        &self.inner
    }
    pub fn metrics(&self, ch: char, size: f32) -> Metrics {
        self.inner.metrics(ch, size)
    }
    pub fn cap_height(&self, size: f32) -> f32 {
        if self.is_default {
            return (size * 0.90).ceil();
        }
        let m = self.inner.metrics('H', size);
        (m.ymin + m.height as i32).max(1) as f32
    }
    pub fn line_height(&self, size: f32) -> f32 {
        (size * 1.35).ceil()
    }

    /// Resolve each character independently: user atlas -> UI subset -> legacy ASCII -> Latin -> box.
    /// Measurement and drawing call this same resolver, so missing CJK never collapses to zero width.
    pub fn glyph(&self, ch: char, size: f32) -> Glyph {
        let px = size.round().clamp(8.0, 128.0) as u16;
        if self.is_default {
            let active = ACTIVE_PACK
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            if let Some(g) = active.as_ref().and_then(|p| p.glyph(px, ch)) {
                return Glyph::from_pack(g);
            }
            if let Some(g) = ui_pack().and_then(|p| p.glyph(px, ch)) {
                return Glyph::from_pack(g);
            }
            if let Some(g) = get_baked_font(size).and_then(|f| f.get_glyph(ch)) {
                return Glyph {
                    width: g.width as usize,
                    height: g.height as usize,
                    xmin: g.xmin as i32,
                    ymin: g.ymin as i32,
                    advance: g.advance_width,
                    bitmap: Bitmap::Static(g.bitmap),
                };
            }
        }
        let key = (ch, (size * 10.0).round() as u32);
        if let Some(g) = self
            .cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entries
            .get(&key)
            .cloned()
        {
            return g;
        }
        let glyph = if self.inner.lookup_glyph_index(ch) != 0 {
            let (m, b) = self.inner.rasterize(ch, size);
            Glyph {
                width: m.width,
                height: m.height,
                xmin: m.xmin,
                ymin: m.ymin,
                advance: m.advance_width,
                bitmap: Bitmap::Owned(b.into()),
            }
        } else {
            let side = (size * 0.78).round().max(6.0) as usize;
            let mut b = vec![0; side * side];
            for y in 0..side {
                for x in 0..side {
                    if x == 0 || y == 0 || x == side - 1 || y == side - 1 {
                        b[y * side + x] = 180;
                    }
                }
            }
            Glyph {
                width: side,
                height: side,
                xmin: 1,
                ymin: 0,
                advance: size,
                bitmap: Bitmap::Owned(b.into()),
            }
        };
        let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        while cache.entries.len() >= CACHE_GLYPHS
            || cache.bytes + glyph.bitmap().len() > CACHE_BYTES
        {
            if let Some(old) = cache.order.pop_front() {
                if let Some(g) = cache.entries.remove(&old) {
                    cache.bytes -= g.bitmap().len();
                }
            } else {
                break;
            }
        }
        cache.bytes += glyph.bitmap().len();
        cache.order.push_back(key);
        cache.entries.insert(key, glyph.clone());
        glyph
    }
    pub fn advance(&self, ch: char, size: f32) -> f32 {
        if ch == '\u{200a}' {
            return (size * 0.068).round().max(2.0);
        }
        if ch == '\t' {
            return self.advance(' ', size) * 4.0;
        }
        if self.is_default {
            let px = size.round().clamp(8.0, 128.0) as u16;
            let active = ACTIVE_PACK
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            if let Some(m) = active.as_ref().and_then(|p| p.metrics(px, ch)) {
                return m.advance;
            }
            if let Some(m) = ui_pack().and_then(|p| p.metrics(px, ch)) {
                return m.advance;
            }
            if let Some(g) = get_baked_font(size).and_then(|p| p.get_glyph(ch)) {
                return g.advance_width;
            }
        }
        if self.inner.lookup_glyph_index(ch) != 0 {
            self.inner.metrics(ch, size).advance_width
        } else {
            size
        }
    }
    pub fn layout_text(&self, text: &str, size: f32, max_width: f32, wrap: bool) -> TextLayout {
        let mut lines = Vec::new();
        let mut line = String::new();
        let mut w = 0.0f32;
        let mut widest = 0.0f32;
        let limit = if max_width.is_finite() {
            max_width.max(1.0)
        } else {
            f32::INFINITY
        };
        for ch in text.chars() {
            if ch == '\r' {
                continue;
            }
            if ch == '\n' {
                widest = widest.max(w);
                lines.push(std::mem::take(&mut line));
                w = 0.0;
                continue;
            }
            let advance = self.advance(ch, size);
            if wrap && !line.is_empty() && w + advance > limit {
                widest = widest.max(w);
                lines.push(std::mem::take(&mut line));
                w = 0.0;
            }
            line.push(ch);
            w += advance;
        }
        widest = widest.max(w);
        lines.push(line);
        let line_height = self.line_height(size);
        TextLayout {
            size: Size::new(widest, line_height * lines.len() as f32),
            lines,
            line_height,
        }
    }
    pub fn measure_text(&self, text: &str, size: f32) -> Size {
        self.layout_text(text, size, f32::INFINITY, false).size
    }
    pub fn rasterize(&self, ch: char, size: f32) -> (Metrics, Vec<u8>) {
        self.inner.rasterize(ch, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chinese_wrap_and_missing_fallback_have_same_advance() {
        let f = Font::default_font();
        let s = "中文漢\n世界";
        let l = f.layout_text(s, 22.0, 44.0, true);
        assert!(l.lines.len() >= 3);
        assert!(f.advance('漢', 22.0) > 0.0);
        for line in &l.lines {
            assert!(f.measure_text(line, 22.0).width <= 44.1);
        }
        assert_eq!(l.size.height, l.lines.len() as f32 * f.line_height(22.0));
    }
    #[test]
    fn explicit_newlines_and_empty_lines_survive() {
        let l = Font::default_font().layout_text("a\r\n\nb\n", 18.0, 200.0, true);
        assert_eq!(l.lines, vec!["a", "", "b", ""]);
    }
}
