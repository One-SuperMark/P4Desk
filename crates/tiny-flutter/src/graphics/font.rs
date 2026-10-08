use crate::graphics::fontpack::{FontPack, PackGlyphRef, PackMetrics};
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
static FILE_FONT: OnceLock<Font> = OnceLock::new();
static DYNAMIC_FONT: OnceLock<Font> = OnceLock::new();
static UI_PACK: OnceLock<Option<Arc<FontPack>>> = OnceLock::new();
static ACTIVE_PACK: RwLock<Option<Arc<FontPack>>> = RwLock::new(None);
static ACTIVE_PACK_VERSION: AtomicUsize = AtomicUsize::new(0);
static FILE_PACK: RwLock<Option<Arc<FontPack>>> = RwLock::new(None);
static FILE_PACK_VERSION: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static FONT_PACK_TEST_LOCK: Mutex<()> = Mutex::new(());
const CACHE_BYTES: usize = 512 * 1024;
const CACHE_GLYPHS: usize = 512;

/// Metrics use fontdue's baseline convention: `ymin` locates the bitmap's bottom edge.
/// Typeface providers must return the same metrics for measurement and rasterization.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypefaceMetrics {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance: f32,
}
impl TypefaceMetrics {
    fn valid(self) -> bool {
        self.width <= 256
            && self.height <= 256
            && self.xmin.unsigned_abs() <= 4096
            && self.ymin.unsigned_abs() <= 4096
            && self.advance.is_finite()
            && (0.0..=4096.0).contains(&self.advance)
    }
    fn as_fontdue(self) -> Metrics {
        Metrics {
            width: self.width,
            height: self.height,
            xmin: self.xmin,
            ymin: self.ymin,
            advance_width: self.advance,
            bounds: fontdue::OutlineBounds {
                xmin: self.xmin as f32,
                ymin: self.ymin as f32,
                width: self.width as f32,
                height: self.height as f32,
            },
            ..Metrics::default()
        }
    }
}
impl From<PackMetrics> for TypefaceMetrics {
    fn from(value: PackMetrics) -> Self {
        Self {
            width: value.width as usize,
            height: value.height as usize,
            xmin: value.xmin as i32,
            ymin: value.ymin as i32,
            advance: value.advance,
        }
    }
}

/// On-demand typeface backend, for example a TF-backed FreeType HAL.
/// Calls run without the Rust glyph-cache lock. Implementations serialize their own IO.
pub trait TypefaceProvider: Send + Sync {
    fn metrics(&self, ch: char, size: u16) -> Option<TypefaceMetrics>;
    fn rasterize(&self, ch: char, size: u16) -> Option<Glyph>;
}

#[derive(Clone)]
struct TypefaceSnapshot {
    provider: Arc<dyn TypefaceProvider>,
    revision: usize,
}
static TYPEFACE_PROVIDER: RwLock<Option<TypefaceSnapshot>> = RwLock::new(None);
static TYPEFACE_VERSION: AtomicUsize = AtomicUsize::new(0);
// A worker can make an initially unavailable glyph ready without replacing the
// provider. Keep layout invalidation separate from provider/cache ownership.
static TYPEFACE_CONTENT_VERSION: AtomicUsize = AtomicUsize::new(0);
static TYPEFACE_CACHE: OnceLock<Mutex<TypefaceCache>> = OnceLock::new();

/// Replace the provider and invalidate layouts and its bounded cache. Previously returned
/// glyphs own their alpha masks, so finishing an old frame remains safe after replacement.
pub fn install_typeface_provider(provider: Option<Arc<dyn TypefaceProvider>>) {
    let mut current = TYPEFACE_PROVIDER.write().unwrap_or_else(|p| p.into_inner());
    let revision = TYPEFACE_VERSION
        .fetch_add(1, Ordering::AcqRel)
        .wrapping_add(1);
    *current = provider.map(|provider| TypefaceSnapshot { provider, revision });
    if let Some(cache) = TYPEFACE_CACHE.get() {
        *cache.lock().unwrap_or_else(|p| p.into_inner()) = TypefaceCache {
            revision,
            ..TypefaceCache::default()
        };
    }
}

/// Include this revision in caches containing text that may use the TF typeface.
pub fn typeface_revision() -> usize {
    TYPEFACE_VERSION
        .load(Ordering::Acquire)
        .wrapping_add(TYPEFACE_CONTENT_VERSION.load(Ordering::Acquire))
}

/// Publish newly available typeface metrics/masks after a worker finishes a batch.
/// Previously measured fallback text will remeasure. Already warmed glyphs and
/// metrics stay cached; this does not change the provider generation or ownership.
pub fn notify_typeface_glyphs_ready() {
    TYPEFACE_CONTENT_VERSION.fetch_add(1, Ordering::Release);
}

#[derive(Clone)]
struct TypefaceCacheEntry {
    metrics: TypefaceMetrics,
    glyph: Option<Glyph>,
}
#[derive(Default)]
struct TypefaceCache {
    revision: usize,
    entries: HashMap<(char, u16), TypefaceCacheEntry>,
    order: VecDeque<(char, u16)>,
    bytes: usize,
}
impl TypefaceCache {
    fn lookup(&self, revision: usize, key: (char, u16)) -> Option<TypefaceCacheEntry> {
        (self.revision == revision)
            .then(|| self.entries.get(&key).cloned())
            .flatten()
    }
    fn insert(
        &mut self,
        revision: usize,
        key: (char, u16),
        metrics: TypefaceMetrics,
        glyph: Option<Glyph>,
    ) {
        // An old IO operation can finish after replacement. Never let it repopulate the
        // new provider's cache or evict any of its entries.
        if TYPEFACE_VERSION.load(Ordering::Acquire) != revision {
            return;
        }
        if self.revision != revision {
            *self = Self {
                revision,
                ..Self::default()
            };
        }
        if let Some(existing) = self.entries.get(&key) {
            if existing.glyph.is_some() || glyph.is_none() {
                return;
            }
        }
        let extra_bytes = glyph.as_ref().map_or(0, |g| g.bitmap().len());
        while (self.entries.len() >= CACHE_GLYPHS && !self.entries.contains_key(&key))
            || self.bytes + extra_bytes > CACHE_BYTES
        {
            if let Some(old) = self.order.pop_front() {
                if let Some(entry) = self.entries.remove(&old) {
                    self.bytes -= entry.glyph.as_ref().map_or(0, |g| g.bitmap().len());
                }
            } else {
                break;
            }
        }
        if !self.entries.contains_key(&key) {
            self.order.push_back(key);
        }
        self.bytes += extra_bytes;
        self.entries
            .insert(key, TypefaceCacheEntry { metrics, glyph });
    }
}
fn typeface_cache() -> &'static Mutex<TypefaceCache> {
    TYPEFACE_CACHE.get_or_init(|| Mutex::new(TypefaceCache::default()))
}
fn typeface_metrics(ch: char, size: u16) -> Option<(TypefaceSnapshot, TypefaceMetrics)> {
    let snapshot = TYPEFACE_PROVIDER
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone()?;
    if let Some(entry) = typeface_cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .lookup(snapshot.revision, (ch, size))
    {
        return Some((snapshot, entry.metrics));
    }
    let metrics = snapshot.provider.metrics(ch, size).filter(|m| m.valid())?;
    typeface_cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(snapshot.revision, (ch, size), metrics, None);
    Some((snapshot, metrics))
}
fn typeface_glyph(
    snapshot: &TypefaceSnapshot,
    ch: char,
    size: u16,
    metrics: TypefaceMetrics,
) -> Option<Glyph> {
    if let Some(glyph) = typeface_cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .lookup(snapshot.revision, (ch, size))
        .and_then(|entry| entry.glyph)
    {
        return Some(glyph);
    }
    let glyph = snapshot.provider.rasterize(ch, size)?;
    if glyph.typeface_metrics() != metrics
        || !metrics.valid()
        || glyph.bitmap().len() != metrics.width * metrics.height
    {
        return None;
    }
    typeface_cache()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(snapshot.revision, (ch, size), metrics, Some(glyph.clone()));
    Some(glyph)
}

enum ExternalGlyph {
    Pack(Arc<FontPack>, TypefaceMetrics),
    Typeface(TypefaceSnapshot, TypefaceMetrics),
}
impl ExternalGlyph {
    fn metrics(&self) -> TypefaceMetrics {
        match self {
            Self::Pack(_, metrics) | Self::Typeface(_, metrics) => *metrics,
        }
    }
    fn glyph(&self, ch: char, px: u16) -> Option<Glyph> {
        match self {
            Self::Pack(pack, _) => pack.glyph(px, ch).map(Glyph::from_pack),
            Self::Typeface(snapshot, metrics) => typeface_glyph(snapshot, ch, px, *metrics),
        }
    }
}

/// Atomically replace the owned, file-backed font provider. Existing glyph references remain valid.
pub fn install_fontpack(pack: Option<Arc<FontPack>>) {
    let mut active = ACTIVE_PACK.write().unwrap_or_else(|p| p.into_inner());
    *active = pack;
    // Publish the revision while the provider write lock is still held. Text
    // layouts can reuse immutable metrics, but synced content must remeasure
    // after its atlas changes, even if the text and width stayed the same.
    ACTIVE_PACK_VERSION.fetch_add(1, Ordering::Release);
}

/// File names and previews have their own atlas; replacing it never changes note text or UI labels.
/// Owned glyph references remain valid while an older page finishes drawing.
pub fn install_file_fontpack(pack: Option<Arc<FontPack>>) {
    let mut active = FILE_PACK.write().unwrap_or_else(|p| p.into_inner());
    *active = pack;
    FILE_PACK_VERSION.fetch_add(1, Ordering::Release);
}

/// Include this revision in any raster cache that contains file text.
pub fn file_fontpack_revision() -> usize {
    FILE_PACK_VERSION.load(Ordering::Acquire)
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
    /// Construct an owned, tightly packed alpha8 mask. No borrowed HAL buffer crosses
    /// this boundary; callers may immediately reuse their decode/raster buffers.
    pub fn from_alpha8(metrics: TypefaceMetrics, bitmap: Vec<u8>) -> Result<Self, &'static str> {
        if !metrics.valid() || bitmap.len() != metrics.width * metrics.height {
            return Err("typeface_glyph_bounds");
        }
        Ok(Self {
            width: metrics.width,
            height: metrics.height,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
            advance: metrics.advance,
            bitmap: Bitmap::Owned(bitmap.into()),
        })
    }
    fn typeface_metrics(&self) -> TypefaceMetrics {
        TypefaceMetrics {
            width: self.width,
            height: self.height,
            xmin: self.xmin,
            ymin: self.ymin,
            advance: self.advance,
        }
    }
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
    use_file_pack: bool,
    use_dynamic_packs: bool,
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
            && self.use_file_pack == other.use_file_pack
            && self.use_dynamic_packs == other.use_dynamic_packs
    }
    pub(crate) fn layout_revision(&self) -> usize {
        if !self.is_default {
            return 0;
        }
        let mut revision = typeface_revision();
        if self.use_file_pack || self.use_dynamic_packs {
            revision = revision.wrapping_add(FILE_PACK_VERSION.load(Ordering::Acquire));
        }
        if self.use_active_pack || self.use_dynamic_packs {
            revision = revision.wrapping_add(ACTIVE_PACK_VERSION.load(Ordering::Acquire));
        }
        revision
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
            use_file_pack: false,
            use_dynamic_packs: false,
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
    /// Dynamic file text: file atlas → bundled UI subset → TF typeface → Latin/missing glyph.
    /// This face never falls back through the note/button atlas.
    pub fn file_font() -> &'static Font {
        FILE_FONT.get_or_init(|| {
            let mut font = Self::default_font().clone();
            font.use_file_pack = true;
            font
        })
    }
    /// Network names and other text not known when a synced atlas was generated.
    /// Resolution is note/button atlas → file atlas → TF typeface → UI subset → Latin/box.
    /// The typeface precedes the UI subset so a network name has consistent letterforms.
    pub fn dynamic_font() -> &'static Font {
        DYNAMIC_FONT.get_or_init(|| {
            let mut font = Self::default_font().clone();
            font.use_dynamic_packs = true;
            font
        })
    }
    fn external_glyph(&self, ch: char, px: u16) -> Option<ExternalGlyph> {
        if !self.is_default {
            return None;
        }
        if self.use_active_pack || self.use_dynamic_packs {
            let active = ACTIVE_PACK
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            if let Some(pack) = active {
                if let Some(metrics) = pack.metrics(px, ch) {
                    return Some(ExternalGlyph::Pack(pack, metrics.into()));
                }
            }
        }
        if self.use_file_pack || self.use_dynamic_packs {
            let file = FILE_PACK.read().unwrap_or_else(|p| p.into_inner()).clone();
            if let Some(pack) = file {
                if let Some(metrics) = pack.metrics(px, ch) {
                    return Some(ExternalGlyph::Pack(pack, metrics.into()));
                }
            }
        }
        if self.use_dynamic_packs {
            if let Some((provider, metrics)) = typeface_metrics(ch, px) {
                return Some(ExternalGlyph::Typeface(provider, metrics));
            }
        }
        if let Some(pack) = ui_pack() {
            if let Some(metrics) = pack.metrics(px, ch) {
                return Some(ExternalGlyph::Pack(pack.clone(), metrics.into()));
            }
        }
        if self.use_dynamic_packs {
            None
        } else {
            typeface_metrics(ch, px)
                .map(|(provider, metrics)| ExternalGlyph::Typeface(provider, metrics))
        }
    }
    pub fn inner(&self) -> &InnerFont {
        &self.inner
    }
    pub fn metrics(&self, ch: char, size: f32) -> Metrics {
        let px = size.round().clamp(8.0, 128.0) as u16;
        if let Some(glyph) = self.external_glyph(ch, px) {
            glyph.metrics().as_fontdue()
        } else if self.inner.lookup_glyph_index(ch) != 0 {
            self.inner.metrics(ch, size)
        } else {
            Self::missing_glyph(size, size)
                .typeface_metrics()
                .as_fontdue()
        }
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

    /// Resolve user text through its atlas, then the UI subset, TF typeface and Latin/box.
    /// System labels skip the user atlas, including older TF generations with another typeface.
    /// Measurement and drawing call this same resolver, so missing CJK never collapses to zero width.
    pub fn glyph(&self, ch: char, size: f32) -> Glyph {
        let px = size.round().clamp(8.0, 128.0) as u16;
        if let Some(source) = self.external_glyph(ch, px) {
            // A transient TF read/raster failure draws a box with the already measured
            // advance, and is never placed in the Latin cache. The next draw retries IO.
            return source
                .glyph(ch, px)
                .unwrap_or_else(|| Self::missing_glyph(size, source.metrics().advance));
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
            Self::missing_glyph(size, size)
        };
        self.cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert_or_get(key, glyph)
    }
    fn missing_glyph(size: f32, advance: f32) -> Glyph {
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
            advance,
            bitmap: Bitmap::Owned(b.into()),
        }
    }
    pub fn advance(&self, ch: char, size: f32) -> f32 {
        if ch == '\u{200a}' {
            return (size * 0.068).round().max(2.0);
        }
        if ch == '\t' {
            return self.advance(' ', size) * 4.0;
        }
        let px = size.round().clamp(8.0, 128.0) as u16;
        if let Some(source) = self.external_glyph(ch, px) {
            return source.metrics().advance;
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
    use std::sync::atomic::AtomicBool;

    struct MockTypeface {
        metrics: Mutex<HashMap<char, TypefaceMetrics>>,
        mask: u8,
        metrics_calls: AtomicUsize,
        raster_calls: AtomicUsize,
        raster_fails: AtomicBool,
    }
    impl MockTypeface {
        fn new(chars: &str, advance: f32, mask: u8) -> Arc<Self> {
            Arc::new(Self {
                metrics: Mutex::new(
                    chars
                        .chars()
                        .map(|ch| {
                            (
                                ch,
                                TypefaceMetrics {
                                    width: 2,
                                    height: 3,
                                    xmin: -1,
                                    ymin: 1,
                                    advance,
                                },
                            )
                        })
                        .collect(),
                ),
                mask,
                metrics_calls: AtomicUsize::new(0),
                raster_calls: AtomicUsize::new(0),
                raster_fails: AtomicBool::new(false),
            })
        }
    }
    impl TypefaceProvider for MockTypeface {
        fn metrics(&self, ch: char, _size: u16) -> Option<TypefaceMetrics> {
            // A slow IO callback must not prevent another renderer from using its cached masks.
            assert!(typeface_cache().try_lock().is_ok());
            assert!(TYPEFACE_PROVIDER.try_write().is_ok());
            self.metrics_calls.fetch_add(1, Ordering::Relaxed);
            self.metrics.lock().unwrap().get(&ch).copied()
        }
        fn rasterize(&self, ch: char, _size: u16) -> Option<Glyph> {
            assert!(typeface_cache().try_lock().is_ok());
            assert!(TYPEFACE_PROVIDER.try_write().is_ok());
            self.raster_calls.fetch_add(1, Ordering::Relaxed);
            if self.raster_fails.load(Ordering::Relaxed) {
                return None;
            }
            let metrics = self.metrics.lock().unwrap().get(&ch).copied()?;
            Glyph::from_alpha8(metrics, vec![self.mask; metrics.width * metrics.height]).ok()
        }
    }

    #[test]
    fn tf_typeface_measures_without_bitmap_and_draws_cjk_not_in_the_ui_subset() {
        use crate::graphics::{Canvas, Color, Point};
        use crate::tiny_gfx::Pixmap565;
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        install_typeface_provider(None);
        assert!(ui_pack().unwrap().metrics(22, '龘').is_none());
        assert!(ui_pack().unwrap().metrics(22, '邃').is_none());
        let provider = MockTypeface::new("龘邃", 27.0, 255);
        install_typeface_provider(Some(provider.clone()));
        let font = Font::dynamic_font();
        assert_eq!(font.measure_text("龘邃", 22.0).width, 54.0);
        assert_eq!(font.metrics('龘', 22.0).advance_width, 27.0);
        assert_eq!(provider.metrics_calls.load(Ordering::Relaxed), 2);
        assert_eq!(provider.raster_calls.load(Ordering::Relaxed), 0);
        let glyph = font.glyph('龘', 22.0);
        assert_eq!(glyph.advance, font.advance('龘', 22.0));
        assert_eq!(
            (glyph.width, glyph.height, glyph.xmin, glyph.ymin),
            (2, 3, -1, 1)
        );
        assert_eq!(glyph.bitmap(), [255; 6]);
        let mut pixels = Pixmap565::new(64, 40).unwrap();
        let mut canvas = Canvas::new(pixels.as_mut());
        canvas.draw_text("龘邃", font, 22.0, Point::new(4.0, 2.0), Color::WHITE);
        let lit = canvas
            .pixels_rgb565()
            .iter()
            .filter(|pixel| **pixel != 0)
            .count();
        assert_eq!(lit, 12);
        assert_eq!(provider.raster_calls.load(Ordering::Relaxed), 2);
        // The system face can fill an absent UI glyph from TF without changing existing labels.
        assert_eq!(Font::default_font().glyph('龘', 22.0).bitmap(), [255; 6]);
        assert_eq!(provider.raster_calls.load(Ordering::Relaxed), 2);
        install_typeface_provider(None);
    }

    #[test]
    fn dynamic_names_choose_typeface_before_ui_but_system_labels_keep_the_ui_face() {
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        install_typeface_provider(None);
        let system = Font::default_font();
        let previous = system.glyph('钟', 22.0);
        let tf_provider = MockTypeface::new("钟龘", 69.0, 222);
        install_typeface_provider(Some(tf_provider));
        assert_eq!(system.glyph('钟', 22.0).bitmap(), previous.bitmap());
        assert_eq!(system.advance('钟', 22.0), previous.advance);
        assert_eq!(Font::content_font().advance('钟', 22.0), previous.advance);
        assert_eq!(Font::file_font().advance('钟', 22.0), previous.advance);
        let dynamic = Font::dynamic_font();
        assert_eq!(dynamic.advance('钟', 22.0), 69.0);
        assert_eq!(dynamic.glyph('钟', 22.0).bitmap(), [222; 6]);
        install_file_fontpack(Some(provider('钟', 55.0, 211)));
        install_fontpack(Some(provider('钟', 41.0, 127)));
        assert_eq!(dynamic.advance('钟', 22.0), 41.0);
        assert_eq!(dynamic.glyph('钟', 22.0).bitmap(), [127; 6]);
        install_fontpack(None);
        assert_eq!(dynamic.advance('钟', 22.0), 55.0);
        assert_eq!(dynamic.glyph('钟', 22.0).bitmap(), [211; 6]);
        install_file_fontpack(None);
        assert_eq!(dynamic.advance('钟', 22.0), 69.0);
        assert!(!dynamic.same_layout_source(system));
        install_typeface_provider(None);
    }

    #[test]
    fn typeface_swap_invalidates_layout_and_cache_while_owned_glyphs_stay_valid() {
        use crate::rendering::BoxConstraints;
        use crate::widgets::{Text, Widget};
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        install_typeface_provider(Some(MockTypeface::new("龘邃", 27.0, 130)));
        let font = Font::dynamic_font();
        let first_revision = font.layout_revision();
        let old = font.glyph('龘', 22.0);
        let mut text = Text::new("龘邃")
            .font_size(22.0)
            .font(font.clone())
            .create_render_object();
        let bounds = BoxConstraints::loose(Size::new(400.0, 100.0));
        assert_eq!(text.layout(&bounds).width, 54.0);
        assert!(!typeface_cache().lock().unwrap().entries.is_empty());
        install_typeface_provider(Some(MockTypeface::new("龘邃", 19.0, 245)));
        assert_ne!(font.layout_revision(), first_revision);
        assert!(typeface_cache().lock().unwrap().entries.is_empty());
        assert_eq!(text.layout(&bounds).width, 38.0);
        assert_eq!(font.glyph('龘', 22.0).bitmap(), [245; 6]);
        assert_eq!(old.bitmap(), [130; 6]);
        install_typeface_provider(None);
    }

    #[test]
    fn async_glyph_ready_remeasures_fallback_text_without_clearing_warmed_glyphs() {
        use crate::rendering::BoxConstraints;
        use crate::widgets::{Text, Widget};
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        let provider = MockTypeface::new("邃", 27.0, 150);
        install_typeface_provider(Some(provider.clone()));
        let font = Font::dynamic_font();
        let old_glyph = font.glyph('邃', 22.0);
        let mut text = Text::new("龘龘")
            .font_size(22.0)
            .font(font.clone())
            .create_render_object();
        let bounds = BoxConstraints::loose(Size::new(400.0, 100.0));
        assert_eq!(text.layout(&bounds).width, 44.0);
        let before_layout_revision = font.layout_revision();
        let provider_revision = TYPEFACE_VERSION.load(Ordering::Acquire);
        // Simulate the resource worker resolving a previously pending character.
        provider.metrics.lock().unwrap().insert(
            '龘',
            TypefaceMetrics {
                width: 2,
                height: 3,
                xmin: -1,
                ymin: 1,
                advance: 31.0,
            },
        );
        assert_eq!(font.glyph('龘', 22.0).bitmap(), [150; 6]);
        let cached_counts = {
            let cache = typeface_cache().lock().unwrap();
            (cache.entries.len(), cache.bytes, cache.revision)
        };
        notify_typeface_glyphs_ready();
        assert_ne!(font.layout_revision(), before_layout_revision);
        assert_eq!(TYPEFACE_VERSION.load(Ordering::Acquire), provider_revision);
        {
            let cache = typeface_cache().lock().unwrap();
            assert_eq!(
                (cache.entries.len(), cache.bytes, cache.revision),
                cached_counts
            );
        }
        assert_eq!(text.layout(&bounds).width, 62.0);
        assert_eq!(font.glyph('邃', 22.0).bitmap(), old_glyph.bitmap());
        assert_eq!(font.glyph('龘', 22.0).bitmap(), [150; 6]);
        assert_eq!(provider.raster_calls.load(Ordering::Relaxed), 2);
        install_typeface_provider(None);
    }

    #[test]
    fn failed_typeface_reads_are_retried_and_never_poison_the_latin_cache() {
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        let provider = MockTypeface::new("", 25.0, 200);
        install_typeface_provider(Some(provider.clone()));
        let font = Font::dynamic_font();
        assert_eq!(font.advance('龘', 22.0), 22.0);
        let _missing = font.glyph('龘', 22.0);
        assert!(typeface_cache().lock().unwrap().entries.is_empty());
        let metrics = TypefaceMetrics {
            width: 2,
            height: 3,
            xmin: 0,
            ymin: 0,
            advance: 25.0,
        };
        provider.metrics.lock().unwrap().insert('龘', metrics);
        provider.raster_fails.store(true, Ordering::Relaxed);
        assert_eq!(font.advance('龘', 22.0), 25.0);
        let missing = font.glyph('龘', 22.0);
        assert_eq!(missing.advance, 25.0);
        assert!(typeface_cache().lock().unwrap().entries[&('龘', 22)]
            .glyph
            .is_none());
        provider.raster_fails.store(false, Ordering::Relaxed);
        assert_eq!(font.glyph('龘', 22.0).bitmap(), [200; 6]);
        assert_eq!(provider.raster_calls.load(Ordering::Relaxed), 2);
        install_typeface_provider(None);
    }

    #[test]
    fn typeface_alpha_masks_and_cache_are_bounded() {
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_typeface_provider(None);
        let metrics = TypefaceMetrics {
            width: 256,
            height: 256,
            xmin: -3,
            ymin: -4,
            advance: 256.0,
        };
        assert!(Glyph::from_alpha8(metrics, vec![0; 256 * 256]).is_ok());
        assert!(Glyph::from_alpha8(
            TypefaceMetrics {
                width: 257,
                ..metrics
            },
            vec![]
        )
        .is_err());
        assert!(Glyph::from_alpha8(
            TypefaceMetrics {
                height: 257,
                ..metrics
            },
            vec![]
        )
        .is_err());
        assert!(Glyph::from_alpha8(metrics, vec![0; 256 * 256 - 1]).is_err());
        assert!(Glyph::from_alpha8(
            TypefaceMetrics {
                advance: f32::NAN,
                ..metrics
            },
            vec![]
        )
        .is_err());
        assert!(Glyph::from_alpha8(
            TypefaceMetrics {
                xmin: i32::MIN,
                ..metrics
            },
            vec![]
        )
        .is_err());
        let mut cache = TypefaceCache::default();
        let revision = TYPEFACE_VERSION.load(Ordering::Acquire);
        for index in 0..12 {
            let ch = char::from_u32(0x4000 + index).unwrap();
            let glyph = Glyph::from_alpha8(metrics, vec![100; 256 * 256]).unwrap();
            cache.insert(revision, (ch, 22), metrics, Some(glyph));
            assert!(cache.bytes <= CACHE_BYTES);
        }
        assert_eq!(cache.bytes, CACHE_BYTES);
        assert_eq!(cache.entries.len(), 8);
        assert!(cache.lookup(revision, ('\u{4000}', 22)).is_none());
        let small = TypefaceMetrics {
            width: 1,
            height: 1,
            ..metrics
        };
        for index in 0..600 {
            cache.insert(
                revision,
                (char::from_u32(0x5000 + index).unwrap(), 22),
                small,
                None,
            );
        }
        assert_eq!(cache.entries.len(), CACHE_GLYPHS);
        let counts = (cache.entries.len(), cache.bytes);
        cache.insert(revision.wrapping_sub(1), ('龘', 22), small, None);
        assert_eq!((cache.entries.len(), cache.bytes), counts);
    }

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

    fn provider(character: char, advance: f32, mask: u8) -> Arc<FontPack> {
        Arc::new(
            FontPack::from_bytes(
                encode_fontpack(vec![PackGlyph {
                    character,
                    size: 22,
                    width: 2,
                    height: 3,
                    xmin: 0,
                    ymin: 0,
                    advance,
                    bitmap: vec![mask; 6],
                }])
                .unwrap(),
            )
            .unwrap(),
        )
    }
    #[test]
    fn file_and_note_providers_never_override_each_other_or_system_labels() {
        let _provider_guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(None);
        install_file_fontpack(None);
        let ui = Font::default_font();
        let note = Font::content_font();
        let file = Font::file_font();
        let ui_glyph = ui.glyph('钟', 22.0);
        let ui_advance = ui.advance('钟', 22.0);
        install_fontpack(Some(provider('钟', 41.0, 127)));
        install_file_fontpack(Some(provider('钟', 55.0, 211)));
        assert_eq!(note.advance('钟', 22.0), 41.0);
        assert_eq!(file.advance('钟', 22.0), 55.0);
        assert_eq!(note.glyph('钟', 22.0).bitmap(), [127; 6]);
        assert_eq!(file.glyph('钟', 22.0).bitmap(), [211; 6]);
        assert_eq!(ui.advance('钟', 22.0), ui_advance);
        assert_eq!(ui.glyph('钟', 22.0).bitmap(), ui_glyph.bitmap());
        assert_eq!(file.measure_text("钟钟", 22.0).width, 110.0);
        assert!(!file.same_layout_source(note));
        assert!(!file.same_layout_source(ui));
        assert!(file.same_layout_source(&file.clone()));
        let old_file = file.glyph('钟', 22.0);
        install_file_fontpack(None);
        assert_eq!(file.advance('钟', 22.0), ui_advance);
        assert_eq!(note.advance('钟', 22.0), 41.0);
        assert_eq!(old_file.bitmap(), [211; 6]);
        install_fontpack(None);
    }
    #[test]
    fn file_pack_layout_revision_changes_only_for_its_provider() {
        let _provider_guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        let ui = Font::default_font();
        let note = Font::content_font();
        let file = Font::file_font();
        let (ui_revision, note_revision, file_revision) = (
            ui.layout_revision(),
            note.layout_revision(),
            file.layout_revision(),
        );
        install_file_fontpack(Some(provider('钟', 55.0, 211)));
        assert_eq!(ui.layout_revision(), ui_revision);
        assert_eq!(note.layout_revision(), note_revision);
        assert_ne!(file.layout_revision(), file_revision);
        let file_revision = file.layout_revision();
        install_fontpack(Some(provider('钟', 41.0, 127)));
        assert_eq!(file.layout_revision(), file_revision);
        assert_ne!(note.layout_revision(), note_revision);
        install_fontpack(None);
        install_file_fontpack(None);
    }
    #[test]
    fn existing_file_text_layout_remeasures_after_file_atlas_swap_only() {
        use crate::rendering::BoxConstraints;
        use crate::widgets::{Text, Widget};
        let _provider_guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        install_fontpack(Some(provider('钟', 41.0, 127)));
        install_file_fontpack(Some(provider('钟', 55.0, 211)));
        let bounds = BoxConstraints::loose(Size::new(400.0, 100.0));
        let mut file = Text::new("钟钟")
            .font_size(22.0)
            .font(Font::file_font().clone())
            .create_render_object();
        let mut note = Text::new("钟钟")
            .font_size(22.0)
            .font(Font::content_font().clone())
            .create_render_object();
        let mut ui = Text::new("钟钟").font_size(22.0).create_render_object();
        let ui_size = ui.layout(&bounds);
        assert_eq!(file.layout(&bounds).width, 110.0);
        assert_eq!(note.layout(&bounds).width, 82.0);
        install_file_fontpack(Some(provider('钟', 17.0, 201)));
        assert_eq!(file.layout(&bounds).width, 34.0);
        assert_eq!(note.layout(&bounds).width, 82.0);
        assert_eq!(ui.layout(&bounds), ui_size);
        install_fontpack(Some(provider('钟', 13.0, 188)));
        assert_eq!(file.layout(&bounds).width, 34.0);
        assert_eq!(note.layout(&bounds).width, 26.0);
        install_fontpack(None);
        install_file_fontpack(None);
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
