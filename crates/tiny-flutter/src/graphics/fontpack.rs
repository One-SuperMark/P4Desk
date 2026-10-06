//! P4F1: immutable, explicitly little-endian glyph atlases shared by host and device.
//! The file backend reads alpha masks on demand and retains a bounded cache.
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub const MAX_FONTPACK_BYTES: usize = 8 * 1024 * 1024;
pub const HEADER_BYTES: usize = 32;
pub const RECORD_BYTES: usize = 32;
const CACHE_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug)]
pub struct PackGlyph {
    pub character: char,
    pub size: u16,
    pub width: u16,
    pub height: u16,
    pub xmin: i16,
    pub ymin: i16,
    pub advance: f32,
    pub bitmap: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    character: char,
    size: u16,
    width: u16,
    height: u16,
    xmin: i16,
    ymin: i16,
    advance: f32,
    offset: usize,
    length: usize,
}

#[derive(Clone)]
pub struct PackGlyphRef {
    pub character: char,
    pub size: u16,
    pub width: u16,
    pub height: u16,
    pub xmin: i16,
    pub ymin: i16,
    pub advance: f32,
    data: Arc<[u8]>,
    offset: usize,
    length: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct PackMetrics {
    pub width: u16,
    pub height: u16,
    pub xmin: i16,
    pub ymin: i16,
    pub advance: f32,
}
impl PackGlyphRef {
    pub fn bitmap(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.length]
    }
}

enum Source {
    Memory(Arc<[u8]>),
    File(Mutex<File>),
}
#[derive(Default)]
struct BitmapCache {
    entries: HashMap<(u16, char), Arc<[u8]>>,
    order: VecDeque<(u16, char)>,
    bytes: usize,
}
impl BitmapCache {
    fn insert_or_get(&mut self, key: (u16, char), data: Arc<[u8]>) -> Arc<[u8]> {
        // File reads run outside the cache lock, so recheck after the read.
        if let Some(existing) = self.entries.get(&key) {
            return existing.clone();
        }
        if data.len() > CACHE_BYTES {
            return data;
        }
        while self.bytes + data.len() > CACHE_BYTES || self.entries.len() >= 512 {
            if let Some(old) = self.order.pop_front() {
                if let Some(bytes) = self.entries.remove(&old) {
                    self.bytes -= bytes.len();
                }
            } else {
                break;
            }
        }
        self.bytes += data.len();
        self.order.push_back(key);
        self.entries.insert(key, data.clone());
        data
    }
}

pub struct FontPack {
    source: Source,
    records: Vec<Record>,
    length: usize,
    cache: Mutex<BitmapCache>,
}
impl std::fmt::Debug for FontPack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontPack")
            .field("glyph_count", &self.records.len())
            .field("bytes", &self.length)
            .finish()
    }
}

fn u16_at(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([b[p], b[p + 1]])
}
fn u32_at(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]])
}
fn parse_header(bytes: &[u8], actual_length: usize) -> Result<(usize, usize), &'static str> {
    if bytes.len() < HEADER_BYTES || &bytes[..4] != b"P4F1" {
        return Err("invalid font header");
    }
    if u16_at(bytes, 4) != 1 || u16_at(bytes, 6) as usize != RECORD_BYTES {
        return Err("unsupported font version");
    }
    if actual_length > MAX_FONTPACK_BYTES || u32_at(bytes, 20) as usize != actual_length {
        return Err("invalid font length");
    }
    if u32_at(bytes, 12) as usize != HEADER_BYTES
        || u32_at(bytes, 24) != 0
        || u32_at(bytes, 28) != 0
    {
        return Err("invalid font layout");
    }
    let count = u32_at(bytes, 8) as usize;
    let end = HEADER_BYTES
        .checked_add(
            count
                .checked_mul(RECORD_BYTES)
                .ok_or("font table overflow")?,
        )
        .ok_or("font table overflow")?;
    if end > actual_length || u32_at(bytes, 16) as usize != end {
        return Err("invalid font table");
    }
    Ok((count, end))
}
fn parse_records(
    table: &[u8],
    count: usize,
    pool: usize,
    length: usize,
) -> Result<Vec<Record>, &'static str> {
    if table.len() != count * RECORD_BYTES {
        return Err("truncated font table");
    }
    let mut records = Vec::with_capacity(count);
    let mut previous = None;
    for b in table.chunks_exact(RECORD_BYTES) {
        let character = char::from_u32(u32_at(b, 0)).ok_or("invalid Unicode codepoint")?;
        let size = u16_at(b, 4);
        let width = u16_at(b, 6);
        let height = u16_at(b, 8);
        let advance = f32::from_le_bytes(b[16..20].try_into().unwrap());
        let offset = u32_at(b, 20) as usize;
        let bitmap_length = u32_at(b, 24) as usize;
        let key = (size, character as u32);
        if !(8..=128).contains(&size)
            || width > 256
            || height > 256
            || !advance.is_finite()
            || !(0.0..=512.0).contains(&advance)
        {
            return Err("invalid glyph metrics");
        }
        if u16_at(b, 14) != 0 || u32_at(b, 28) != 0 || previous.is_some_and(|p| p >= key) {
            return Err("unordered glyph table");
        }
        if bitmap_length != width as usize * height as usize
            || offset < pool
            || offset
                .checked_add(bitmap_length)
                .is_none_or(|end| end > length)
        {
            return Err("invalid glyph bitmap");
        }
        records.push(Record {
            character,
            size,
            width,
            height,
            xmin: u16_at(b, 10) as i16,
            ymin: u16_at(b, 12) as i16,
            advance,
            offset,
            length: bitmap_length,
        });
        previous = Some(key);
    }
    Ok(records)
}

impl FontPack {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        Self::from_shared(bytes.into())
    }
    pub fn from_shared(bytes: Arc<[u8]>) -> Result<Self, &'static str> {
        let (count, pool) = parse_header(&bytes, bytes.len())?;
        let records = parse_records(&bytes[HEADER_BYTES..pool], count, pool, bytes.len())?;
        let length = bytes.len();
        Ok(Self {
            source: Source::Memory(bytes),
            records,
            length,
            cache: Mutex::new(BitmapCache::default()),
        })
    }
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, &'static str> {
        let mut file = File::open(path).map_err(|_| "font open failed")?;
        let length: usize = file
            .metadata()
            .map_err(|_| "font stat failed")?
            .len()
            .try_into()
            .map_err(|_| "font too large")?;
        let mut header = [0; HEADER_BYTES];
        file.read_exact(&mut header)
            .map_err(|_| "font header read failed")?;
        let (count, pool) = parse_header(&header, length)?;
        let mut table = vec![0; pool - HEADER_BYTES];
        file.read_exact(&mut table)
            .map_err(|_| "font table read failed")?;
        let records = parse_records(&table, count, pool, length)?;
        Ok(Self {
            source: Source::File(Mutex::new(file)),
            records,
            length,
            cache: Mutex::new(BitmapCache::default()),
        })
    }
    pub fn glyph_count(&self) -> usize {
        self.records.len()
    }
    /// Reading layout metrics never reads an alpha bitmap from TF.
    pub fn metrics(&self, size: u16, character: char) -> Option<PackMetrics> {
        self.record(size, character).map(|r| PackMetrics {
            width: r.width,
            height: r.height,
            xmin: r.xmin,
            ymin: r.ymin,
            advance: r.advance,
        })
    }
    pub fn byte_length(&self) -> usize {
        self.length
    }
    pub fn has_glyph(&self, size: u16, character: char) -> bool {
        self.record(size, character).is_some()
    }
    pub fn covers(&self, text: &str, sizes: &[u16]) -> bool {
        text.chars()
            .filter(|c| !c.is_control())
            .all(|c| sizes.iter().all(|s| self.has_glyph(*s, c)))
    }
    fn record(&self, size: u16, character: char) -> Option<Record> {
        self.records
            .binary_search_by_key(&(size, character as u32), |r| (r.size, r.character as u32))
            .ok()
            .map(|i| self.records[i])
    }
    pub fn glyph(&self, size: u16, character: char) -> Option<PackGlyphRef> {
        let r = self.record(size, character)?;
        let (data, offset) = match &self.source {
            Source::Memory(data) => (data.clone(), r.offset),
            Source::File(file) => {
                let key = (size, character);
                let cached = self.cache.lock().ok()?.entries.get(&key).cloned();
                let data = if let Some(data) = cached {
                    data
                } else {
                    let mut data = vec![0; r.length];
                    {
                        let mut f = file.lock().ok()?;
                        f.seek(SeekFrom::Start(r.offset as u64)).ok()?;
                        f.read_exact(&mut data).ok()?;
                    }
                    let data: Arc<[u8]> = data.into();
                    self.cache.lock().ok()?.insert_or_get(key, data)
                };
                (data, 0)
            }
        };
        Some(PackGlyphRef {
            character: r.character,
            size: r.size,
            width: r.width,
            height: r.height,
            xmin: r.xmin,
            ymin: r.ymin,
            advance: r.advance,
            data,
            offset,
            length: r.length,
        })
    }
}

pub fn encode_fontpack(mut glyphs: Vec<PackGlyph>) -> Result<Vec<u8>, &'static str> {
    glyphs.sort_by_key(|g| (g.size, g.character as u32));
    let pool = HEADER_BYTES
        .checked_add(
            glyphs
                .len()
                .checked_mul(RECORD_BYTES)
                .ok_or("font table overflow")?,
        )
        .ok_or("font table overflow")?;
    let length = glyphs.iter().try_fold(pool, |sum, g| {
        sum.checked_add(g.bitmap.len()).ok_or("font size overflow")
    })?;
    if length > MAX_FONTPACK_BYTES {
        return Err("font package exceeds 8 MiB");
    }
    let mut bytes = vec![0u8; length];
    bytes[..4].copy_from_slice(b"P4F1");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&(RECORD_BYTES as u16).to_le_bytes());
    bytes[8..12].copy_from_slice(&(glyphs.len() as u32).to_le_bytes());
    bytes[12..16].copy_from_slice(&(HEADER_BYTES as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&(pool as u32).to_le_bytes());
    bytes[20..24].copy_from_slice(&(length as u32).to_le_bytes());
    let mut offset = pool;
    for (i, g) in glyphs.iter().enumerate() {
        let b = &mut bytes[HEADER_BYTES + i * RECORD_BYTES..HEADER_BYTES + (i + 1) * RECORD_BYTES];
        b[..4].copy_from_slice(&(g.character as u32).to_le_bytes());
        b[4..6].copy_from_slice(&g.size.to_le_bytes());
        b[6..8].copy_from_slice(&g.width.to_le_bytes());
        b[8..10].copy_from_slice(&g.height.to_le_bytes());
        b[10..12].copy_from_slice(&g.xmin.to_le_bytes());
        b[12..14].copy_from_slice(&g.ymin.to_le_bytes());
        b[16..20].copy_from_slice(&g.advance.to_le_bytes());
        b[20..24].copy_from_slice(&(offset as u32).to_le_bytes());
        b[24..28].copy_from_slice(&(g.bitmap.len() as u32).to_le_bytes());
        bytes[offset..offset + g.bitmap.len()].copy_from_slice(&g.bitmap);
        offset += g.bitmap.len();
    }
    FontPack::from_shared(Arc::from(bytes.as_slice()))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn competing_file_reads_do_not_duplicate_cache_bytes_or_eviction_order() {
        let mut cache = BitmapCache::default();
        let first = cache.insert_or_get((22, '中'), Arc::from([1u8, 2, 3, 4]));
        let competing = cache.insert_or_get((22, '中'), Arc::from([9u8, 8, 7, 6]));
        assert!(Arc::ptr_eq(&first, &competing));
        assert_eq!(cache.bytes, 4);
        assert_eq!(cache.order.len(), 1);
        assert_eq!(cache.entries.len(), 1);
        let oversized = cache.insert_or_get((128, '文'), vec![0; CACHE_BYTES + 1].into());
        assert_eq!(oversized.len(), CACHE_BYTES + 1);
        assert_eq!(cache.bytes, 4);
    }
    fn glyph() -> PackGlyph {
        PackGlyph {
            character: '中',
            size: 22,
            width: 2,
            height: 2,
            xmin: 0,
            ymin: -1,
            advance: 22.0,
            bitmap: vec![0, 64, 128, 255],
        }
    }
    #[test]
    fn round_trip_and_owned_lifetime() {
        let bytes = encode_fontpack(vec![glyph()]).unwrap();
        let pack = FontPack::from_bytes(bytes).unwrap();
        let mask = pack.glyph(22, '中').unwrap();
        drop(pack);
        assert_eq!(mask.bitmap(), &[0, 64, 128, 255]);
    }
    #[test]
    fn corrupt_offset_is_rejected() {
        let mut bytes = encode_fontpack(vec![glyph()]).unwrap();
        bytes[52..56].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(FontPack::from_bytes(bytes).is_err());
    }
    #[test]
    fn duplicate_codepoints_are_rejected() {
        assert!(encode_fontpack(vec![glyph(), glyph()]).is_err());
    }
    #[test]
    fn surrogate_is_rejected() {
        let mut b = encode_fontpack(vec![glyph()]).unwrap();
        b[32..36].copy_from_slice(&0xd800u32.to_le_bytes());
        assert!(FontPack::from_bytes(b).is_err());
    }
    #[test]
    fn coverage_checks_every_size() {
        let p = FontPack::from_bytes(encode_fontpack(vec![glyph()]).unwrap()).unwrap();
        assert!(p.covers("中\n", &[22]));
        assert!(!p.covers("中文", &[22]));
        assert!(!p.covers("中", &[18, 22]));
    }
    #[test]
    fn file_metrics_do_not_read_alpha_and_missing_file_data_is_safe() {
        let path = std::env::temp_dir().join(format!(
            "p4desk-metrics-{}-{}.p4f",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, encode_fontpack(vec![glyph()]).unwrap()).unwrap();
        let pack = FontPack::from_file(&path).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(64)
            .unwrap();
        assert_eq!(pack.metrics(22, '中').unwrap().advance, 22.0);
        assert!(pack.cache.lock().unwrap().entries.is_empty());
        assert!(pack.glyph(22, '中').is_none());
        drop(pack);
        std::fs::remove_file(path).unwrap();
    }
}
