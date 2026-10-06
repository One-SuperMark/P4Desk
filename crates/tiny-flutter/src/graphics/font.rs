use crate::graphics::fontpack::{FontPack, PackGlyphRef};
use crate::graphics::geometry::Size;
use fontdue::{Font as InnerFont, FontSettings, Metrics};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

static DEFAULT_FONT_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/source/HarmonyOS_Sans_Regular.ttf");
static UI_FONT_BYTES: &[u8] = include_bytes!("../../../../assets/generated/ui.p4f");
static DEFAULT_FONT: OnceLock<Font> = OnceLock::new();
static CONTENT_FONT: OnceLock<Font> = OnceLock::new();
static UI_PACK: OnceLock<Option<Arc<FontPack>>> = OnceLock::new();
static ACTIVE_PACK: RwLock<Option<Arc<FontPack>>> = RwLock::new(None);
static ACTIVE_PACK_VERSION: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static FONT_PACK_TEST_LOCK: Mutex<()> = Mutex::new(());
const CACHE_BYTES: usize = 512 * 1024;
const CACHE_GLYPHS: usize = 512;

/// Atomically replace the owned, file-backed font provider. Existing glyph references remain valid.
pub fn install_fontpack(pack: Option<Arc<FontPack>>) {
    let mut active = ACTIVE_PACK.write().unwrap_or_else(|p| p.into_inner());
    *active = pack;
    // Publish the revision while the provider write lock is still held. Text
    // layouts can reuse immutable metrics, but synced content must remeasure
    // after its atlas changes, even if the text and width stayed the same.
    ACTIVE_PACK_VERSION.fetch_add(1, Ordering::Release);
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
impl GlyphCache {
    fn insert_or_get(&mut self, key: (char, u32), glyph: Glyph) -> Glyph {
        // Rasterization runs outside the lock. Another renderer may have
        // installed this glyph since the first lookup; charge it only once.
        if let Some(existing) = self.entries.get(&key) {
            return existing.clone();
        }
        let bytes = glyph.bitmap().len();
        if bytes > CACHE_BYTES {
            return glyph;
        }
        while self.entries.len() >= CACHE_GLYPHS || self.bytes + bytes > CACHE_BYTES {
            if let Some(old) = self.order.pop_front() {
                if let Some(g) = self.entries.remove(&old) {
                    self.bytes -= g.bitmap().len();
                }
            } else {
                break;
            }
        }
        self.bytes += bytes;
        self.order.push_back(key);
        self.entries.insert(key, glyph.clone());
        glyph
    }
}

#[derive(Clone)]
pub struct Font {
    inner: Arc<InnerFont>,
    cache: Arc<Mutex<GlyphCache>>,
    is_default: bool,
    use_active_pack: bool,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub lines: Vec<String>,
    pub size: Size,
    pub line_height: f32,
}

impl Font {
    pub(crate) fn same_layout_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
            && self.is_default == other.is_default
            && self.use_active_pack == other.use_active_pack
    }
    pub(crate) fn layout_revision(&self) -> usize {
        if self.is_default && self.use_active_pack {
            ACTIVE_PACK_VERSION.load(Ordering::Acquire)
        } else {
            0
        }
    }
    /// TTF parsing is intended for the small bundled Latin font or host tools, never full CJK on P4.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let inner =
            InnerFont::from_bytes(bytes, FontSettings::default()).map_err(|_| "font_parse")?;
        Ok(Self {
            inner: Arc::new(inner),
            cache: Arc::new(Mutex::new(GlyphCache::default())),
            is_default: false,
            use_active_pack: false,
        })
    }
    pub fn default_font() -> &'static Font {
        DEFAULT_FONT.get_or_init(|| {
            let mut f = Self::from_bytes(DEFAULT_FONT_BYTES).expect("bundled Latin font");
            f.is_default = true;
            f
        })
    }
    /// User text uses its synced atlas, while system labels keep the bundled UI face.
    /// Both faces share the same bounded Latin fallback cache.
    pub fn content_font() -> &'static Font {
        CONTENT_FONT.get_or_init(|| {
            let mut f = Self::default_font().clone();
            f.use_active_pack = true;
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

    /// Resolve user text through its atlas, then the UI subset, Latin face and missing-glyph box.
    /// System labels skip the user atlas, including older TF generations with another typeface.
    /// Measurement and drawing call this same resolver, so missing CJK never collapses to zero width.
    pub fn glyph(&self, ch: char, size: f32) -> Glyph {
        let px = size.round().clamp(8.0, 128.0) as u16;
        if self.is_default {
            let active = self
                .use_active_pack
                .then(|| {
                    ACTIVE_PACK
                        .read()
                        .unwrap_or_else(|p| p.into_inner())
                        .clone()
                })
                .flatten();
            if let Some(g) = active.as_ref().and_then(|p| p.glyph(px, ch)) {
                return Glyph::from_pack(g);
            }
            if let Some(g) = ui_pack().and_then(|p| p.glyph(px, ch)) {
                return Glyph::from_pack(g);
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
        self.cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert_or_get(key, glyph)
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
            let active = self
                .use_active_pack
                .then(|| {
                    ACTIVE_PACK
                        .read()
                        .unwrap_or_else(|p| p.into_inner())
                        .clone()
                })
                .flatten();
            if let Some(m) = active.as_ref().and_then(|p| p.metrics(px, ch)) {
                return m.advance;
            }
            if let Some(m) = ui_pack().and_then(|p| p.metrics(px, ch)) {
                return m.advance;
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
    use crate::graphics::fontpack::{encode_fontpack, PackGlyph};

    fn owned_glyph(bytes: usize, value: u8) -> Glyph {
        Glyph {
            width: bytes,
            height: 1,
            xmin: 0,
            ymin: 0,
            advance: bytes as f32,
            bitmap: Bitmap::Owned(vec![value; bytes].into()),
        }
    }

    #[test]
    fn competing_insertions_charge_one_glyph_and_oversized_masks_are_not_retained() {
        let mut cache = GlyphCache::default();
        let first = cache.insert_or_get(('x', 220), owned_glyph(40, 70));
        let competing = cache.insert_or_get(('x', 220), owned_glyph(40, 90));
        assert_eq!(cache.bytes, 40);
        assert_eq!(cache.order.len(), 1);
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(first.bitmap(), competing.bitmap());
        let large = cache.insert_or_get(('y', 9000), owned_glyph(CACHE_BYTES + 1, 127));
        assert_eq!(large.bitmap().len(), CACHE_BYTES + 1);
        assert_eq!(cache.bytes, 40);
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn old_synced_atlas_cannot_override_ui_but_custom_content_font_still_works() {
        let _provider_guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        let system = Font::default_font();
        let baseline = system.glyph('钟', 22.0);
        let baseline_pixels = baseline.bitmap().to_vec();
        let baseline_advance = system.advance('钟', 22.0);
        let bytes = encode_fontpack(vec![PackGlyph {
            character: '钟',
            size: 22,
            width: 2,
            height: 3,
            xmin: 0,
            ymin: 0,
            advance: 41.0,
            bitmap: vec![127; 6],
        }])
        .unwrap();
        install_fontpack(Some(Arc::new(FontPack::from_bytes(bytes).unwrap())));
        assert_eq!(system.glyph('钟', 22.0).bitmap(), baseline_pixels);
        assert_eq!(system.advance('钟', 22.0), baseline_advance);
        let content = Font::content_font();
        assert_eq!(content.glyph('钟', 22.0).bitmap(), [127; 6]);
        assert_eq!(content.advance('钟', 22.0), 41.0);
        assert_eq!(content.measure_text("钟钟", 22.0).width, 82.0);
        assert_eq!(
            content.glyph('计', 22.0).bitmap(),
            system.glyph('计', 22.0).bitmap()
        );
        install_fontpack(None);
        assert_eq!(content.glyph('钟', 22.0).bitmap(), baseline_pixels);
        assert_eq!(content.advance('钟', 22.0), baseline_advance);
    }

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
