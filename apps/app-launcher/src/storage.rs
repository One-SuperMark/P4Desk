use p4desk_protocol::{Snapshot, MAX_CONTROL};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiny_flutter::graphics::fontpack::{FontPack, MAX_FONTPACK_BYTES};

pub const FONT_SIZES: [u16; 4] = [18, 22, 28, 36];
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSettings {
    pub brightness: u8,
    pub screen_on: bool,
    pub timezone_minutes: i32,
}
impl Default for LocalSettings {
    fn default() -> Self {
        Self {
            brightness: 75,
            screen_on: true,
            timezone_minutes: 480,
        }
    }
}
impl LocalSettings {
    pub fn validate(&self) -> bool {
        self.brightness <= 100 && (-840..=840).contains(&self.timezone_minutes)
    }
}
#[derive(Serialize, Deserialize)]
struct Manifest {
    generation: u64,
    font_length: u32,
    font_sha256: String,
}
#[derive(Default, Serialize, Deserialize)]
struct Tombstones {
    deleted_note_ids: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Journal {
    revision: u64,
    payload: String,
    sha256: String,
}
struct Pending {
    generation: u64,
    state: Snapshot,
    length: u32,
    sha256: String,
    offset: u32,
    path: PathBuf,
    file: Option<File>,
}
pub struct Committed {
    pub state: Snapshot,
    pub font: Arc<FontPack>,
}

/// All state transitions are prepared, checked, fsynced and renamed before becoming visible.
/// Pending directories have a dot prefix and are ignored by startup recovery.
pub struct GenerationStore {
    root: PathBuf,
    local_root: PathBuf,
    pending: Option<Pending>,
    pub active_generation: u64,
}
impl GenerationStore {
    pub fn new(root: impl Into<PathBuf>, local_root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            local_root: local_root.into(),
            pending: None,
            active_generation: 0,
        }
    }
    pub fn load_settings(&self) -> LocalSettings {
        read_journal::<LocalSettings>(&self.local_root, "settings")
            .ok()
            .flatten()
            .filter(LocalSettings::validate)
            .unwrap_or_default()
    }
    pub fn save_settings(&self, settings: &LocalSettings) -> Result<(), &'static str> {
        if !settings.validate() {
            return Err("settings_invalid");
        }
        write_journal(&self.local_root, "settings", settings)
    }
    fn deleted_ids(&self) -> Result<Vec<String>, &'static str> {
        let Some(value) = read_journal::<Tombstones>(&self.local_root, "tombstones")? else {
            return Ok(Vec::new());
        };
        let snapshot = Snapshot {
            deleted_note_ids: value.deleted_note_ids.clone(),
            ..Snapshot::default()
        };
        snapshot.validate()?;
        Ok(value.deleted_note_ids)
    }
    pub fn delete_note(&self, current: &Snapshot, id: &str) -> Result<Snapshot, &'static str> {
        if !current.notes.iter().any(|n| n.id == id) {
            return Err("note_not_found");
        }
        let mut next = current.clone();
        next.deleted_note_ids.extend(self.deleted_ids()?);
        next.deleted_note_ids.push(id.into());
        next.deleted_note_ids.sort();
        next.deleted_note_ids.dedup();
        next.apply_deletions();
        next.validate()?;
        write_journal(
            &self.local_root,
            "tombstones",
            &Tombstones {
                deleted_note_ids: next.deleted_note_ids.clone(),
            },
        )?;
        Ok(next)
    }
    pub fn begin(
        &mut self,
        generation: u64,
        mut state: Snapshot,
        length: u32,
        sha256: String,
    ) -> Result<(), &'static str> {
        if self.pending.is_some() {
            return Err("sync_busy");
        }
        if generation == 0 || generation <= self.active_generation || state.generation != generation
        {
            return Err("generation_invalid");
        }
        state.validate()?;
        let deleted = Snapshot {
            deleted_note_ids: self.deleted_ids()?,
            ..Snapshot::default()
        };
        state.merge_deletions(&deleted);
        if length as usize > MAX_FONTPACK_BYTES || length < 32 {
            return Err("font_length");
        }
        if !valid_sha(&sha256) {
            return Err("font_sha256");
        }
        if serde_json::to_vec(&state)
            .map_err(|_| "state_encode")?
            .len()
            > MAX_CONTROL
        {
            return Err("state_size");
        }
        let generations = self.root.join("generations");
        fs::create_dir_all(&generations).map_err(|_| "storage_create")?;
        let final_path = generations.join(generation.to_string());
        if final_path.exists() {
            return Err("generation_exists");
        }
        let path = generations.join(format!(".{generation}.pending"));
        // A stale transfer is never visible. Removing only this known numeric staging path is safe.
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|_| "staging_cleanup")?;
        }
        fs::create_dir(&path).map_err(|_| "storage_create")?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join("font.p4f"))
            .map_err(|_| "font_create")?;
        self.pending = Some(Pending {
            generation,
            state,
            length,
            sha256,
            offset: 0,
            path,
            file: Some(file),
        });
        Ok(())
    }
    pub fn chunk(&mut self, offset: u32, bytes: &[u8]) -> Result<(), &'static str> {
        let p = self.pending.as_mut().ok_or("sync_not_started")?;
        if bytes.is_empty()
            || bytes.len() > 32768
            || offset != p.offset
            || offset
                .checked_add(bytes.len() as u32)
                .is_none_or(|end| end > p.length)
        {
            return Err("chunk_offset");
        }
        p.file
            .as_mut()
            .ok_or("transfer_closed")?
            .write_all(bytes)
            .map_err(|_| "font_write")?;
        p.offset += bytes.len() as u32;
        Ok(())
    }
    pub fn pending_generation(&self) -> Option<u64> {
        self.pending.as_ref().map(|p| p.generation)
    }
    pub fn commit(&mut self, generation: u64) -> Result<Committed, &'static str> {
        let p = self.pending.as_mut().ok_or("sync_not_started")?;
        if p.generation != generation || p.offset != p.length {
            return Err("sync_incomplete");
        }
        if let Some(file) = &p.file {
            file.sync_all().map_err(|_| "font_fsync")?;
        }
        let font_path = p.path.join("font.p4f");
        if hash_file(&font_path)? != p.sha256.to_ascii_lowercase() {
            return Err("font_hash");
        }
        let font = FontPack::from_file(&font_path).map_err(|_| "font_invalid")?;
        if font.byte_length() != p.length as usize
            || !font.covers(&p.state.font_text(), &FONT_SIZES)
        {
            return Err("font_coverage");
        }
        drop(font);
        // Deletions made during a transfer also win over the incoming host snapshot.
        let deleted = Snapshot {
            deleted_note_ids: self.deleted_ids()?,
            ..Snapshot::default()
        };
        let p = self.pending.as_mut().unwrap();
        p.state.merge_deletions(&deleted);
        p.state.validate()?;
        write_sync(&p.path.join("state.json"), &p.state)?;
        write_sync(
            &p.path.join("complete.json"),
            &Manifest {
                generation,
                font_length: p.length,
                font_sha256: p.sha256.clone(),
            },
        )?;
        sync_directory(&p.path)?;
        // FAT can refuse directory rename while a file inside it is open.
        drop(p.file.take());
        let destination = self.root.join("generations").join(generation.to_string());
        fs::rename(&p.path, &destination).map_err(|_| "generation_rename")?;
        sync_directory(&self.root.join("generations"))?;
        let font =
            Arc::new(FontPack::from_file(destination.join("font.p4f")).map_err(|_| "font_open")?);
        let state = p.state.clone();
        self.pending = None;
        self.active_generation = generation;
        Ok(Committed { state, font })
    }
    pub fn abort(&mut self) -> Result<(), &'static str> {
        if let Some(p) = self.pending.take() {
            drop(p.file);
            if p.path.exists() {
                fs::remove_dir_all(p.path).map_err(|_| "staging_cleanup")?;
            }
        }
        Ok(())
    }
    pub fn restore(&mut self) -> Result<Option<Committed>, &'static str> {
        let generations = self.root.join("generations");
        if !generations.exists() {
            return Ok(None);
        }
        let mut ids: Vec<u64> = fs::read_dir(&generations)
            .map_err(|_| "storage_read")?
            .filter_map(|e| e.ok()?.file_name().to_str()?.parse().ok())
            .collect();
        ids.sort_unstable_by(|a, b| b.cmp(a));
        let deleted = self.deleted_ids()?;
        for id in ids {
            let path = generations.join(id.to_string());
            let validated = (|| {
                let m: Manifest = read_json(&path.join("complete.json"))?;
                if m.generation != id
                    || m.font_length as usize > MAX_FONTPACK_BYTES
                    || !valid_sha(&m.font_sha256)
                {
                    return Err("manifest_invalid");
                }
                let font_path = path.join("font.p4f");
                if hash_file(&font_path)? != m.font_sha256.to_ascii_lowercase() {
                    return Err("font_hash");
                }
                let font = FontPack::from_file(&font_path).map_err(|_| "font_invalid")?;
                let mut state: Snapshot = read_json(&path.join("state.json"))?;
                state.validate()?;
                if state.generation != id
                    || font.byte_length() != m.font_length as usize
                    || !font.covers(&state.font_text(), &FONT_SIZES)
                {
                    return Err("font_coverage");
                }
                state.merge_deletions(&Snapshot {
                    deleted_note_ids: deleted.clone(),
                    ..Snapshot::default()
                });
                Ok(Committed {
                    state,
                    font: Arc::new(font),
                })
            })();
            if let Ok(committed) = validated {
                self.active_generation = id;
                return Ok(Some(committed));
            }
        }
        Ok(None)
    }
}
fn valid_sha(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn hash_file(path: &Path) -> Result<String, &'static str> {
    let mut f = File::open(path).map_err(|_| "font_read")?;
    let mut h = Sha256::new();
    let mut b = [0u8; 8192];
    let mut total = 0usize;
    loop {
        let n = f.read(&mut b).map_err(|_| "font_read")?;
        if n == 0 {
            break;
        }
        total += n;
        if total > MAX_FONTPACK_BYTES {
            return Err("font_length");
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, &'static str> {
    let f = File::open(path).map_err(|_| "state_read")?;
    let size = f.metadata().map_err(|_| "state_read")?.len();
    if size > 512 * 1024 {
        return Err("state_size");
    }
    serde_json::from_reader(f).map_err(|_| "state_invalid")
}
fn write_sync<T: Serialize>(path: &Path, value: &T) -> Result<(), &'static str> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|_| "state_write")?;
    serde_json::to_writer(&mut f, value).map_err(|_| "state_encode")?;
    f.write_all(b"\n").map_err(|_| "state_write")?;
    f.sync_all().map_err(|_| "state_fsync")
}
fn valid_journal(root: &Path, name: &str, slot: usize) -> Option<Journal> {
    let j: Journal = read_json(&root.join(format!("{name}.{slot}.json"))).ok()?;
    if j.revision == 0
        || j.payload.len() > 512 * 1024
        || format!("{:x}", Sha256::digest(j.payload.as_bytes())) != j.sha256
    {
        return None;
    }
    Some(j)
}
fn latest_journal(root: &Path, name: &str) -> Option<(usize, Journal)> {
    [0, 1]
        .into_iter()
        .filter_map(|slot| valid_journal(root, name, slot).map(|j| (slot, j)))
        .max_by_key(|(_, j)| j.revision)
}
fn read_journal<T: serde::de::DeserializeOwned>(
    root: &Path,
    name: &str,
) -> Result<Option<T>, &'static str> {
    if let Some((_, j)) = latest_journal(root, name) {
        return serde_json::from_str(&j.payload)
            .map(Some)
            .map_err(|_| "local_state_invalid");
    }
    if [0, 1]
        .into_iter()
        .any(|slot| root.join(format!("{name}.{slot}.json")).exists())
    {
        return Err("local_state_invalid");
    }
    // Read compatibility with an early development build, without rewriting its file.
    let legacy = root.join(format!("{name}.json"));
    if legacy.exists() {
        return read_json(&legacy).map(Some);
    }
    Ok(None)
}
fn write_journal<T: Serialize>(root: &Path, name: &str, value: &T) -> Result<(), &'static str> {
    #[cfg(not(target_os = "espidf"))]
    fs::create_dir_all(root).map_err(|_| "storage_create")?;
    // SPIFFS has no mkdir and its rename cannot replace an existing target. Use two slots.
    // The latest complete slot is never removed; a power loss leaves it available at startup.
    let latest = latest_journal(root, name);
    let revision = latest
        .as_ref()
        .map(|(_, j)| j.revision)
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("journal_revision")?;
    let slot = latest.map(|(slot, _)| 1 - slot).unwrap_or(0);
    let target = root.join(format!("{name}.{slot}.json"));
    if target.exists() {
        fs::remove_file(&target).map_err(|_| "state_write")?;
    }
    let payload = serde_json::to_string(value).map_err(|_| "state_encode")?;
    let sha256 = format!("{:x}", Sha256::digest(payload.as_bytes()));
    let temporary = root.join(format!(".{name}.{slot}.tmp"));
    write_sync(
        &temporary,
        &Journal {
            revision,
            payload,
            sha256,
        },
    )?;
    fs::rename(&temporary, &target).map_err(|_| "state_rename")?;
    sync_directory(root)
}
fn sync_directory(root: &Path) -> Result<(), &'static str> {
    #[cfg(not(target_os = "espidf"))]
    {
        File::open(root)
            .and_then(|f| f.sync_all())
            .map_err(|_| "directory_fsync")?;
    }
    // ESP-IDF VFS/FAT and SPIFFS do not implement fsync on directories. Files above are fsynced.
    #[cfg(target_os = "espidf")]
    {
        let _ = root;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use p4desk_protocol::Note;
    use tiny_flutter::graphics::fontpack::{encode_fontpack, PackGlyph};
    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "p4desk-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn state(generation: u64) -> Snapshot {
        Snapshot {
            generation,
            notes: vec![Note {
                id: "note_1".into(),
                title: "a".into(),
                body: "b".into(),
                updated_ms: 0,
            }],
            ..Snapshot::default()
        }
    }
    fn pack() -> Vec<u8> {
        encode_fontpack(
            FONT_SIZES
                .into_iter()
                .flat_map(|size| {
                    ['a', 'b'].map(move |ch| PackGlyph {
                        character: ch,
                        size,
                        width: 1,
                        height: 1,
                        xmin: 0,
                        ymin: 0,
                        advance: 1.0,
                        bitmap: vec![255],
                    })
                })
                .collect(),
        )
        .unwrap()
    }
    #[test]
    fn complete_generation_active_immediately_and_recovers_previous_valid() {
        let p = temp("restore");
        let mut store = GenerationStore::new(p.join("sd"), p.join("flash"));
        let bytes = pack();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        for id in [1, 2] {
            store
                .begin(id, state(id), bytes.len() as u32, hash.clone())
                .unwrap();
            store.chunk(0, &bytes).unwrap();
            let c = store.commit(id).unwrap();
            assert!(c.font.has_glyph(22, 'a'));
            assert_eq!(c.state.generation, id);
        }
        fs::write(p.join("sd/generations/2/font.p4f"), b"broken").unwrap();
        let mut reboot = GenerationStore::new(p.join("sd"), p.join("flash"));
        assert_eq!(reboot.restore().unwrap().unwrap().state.generation, 1);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn failed_write_does_not_delete_and_tombstone_survives_host_replay() {
        let p = temp("delete");
        let bad = p.join("not_directory");
        fs::write(&bad, b"file").unwrap();
        let s = state(1);
        let store = GenerationStore::new(p.join("sd"), bad);
        assert!(store.delete_note(&s, "note_1").is_err());
        assert_eq!(s.notes.len(), 1);
        let mut store = GenerationStore::new(p.join("sd"), p.join("flash"));
        let next = store.delete_note(&s, "note_1").unwrap();
        assert!(next.notes.is_empty());
        let bytes = pack();
        store
            .begin(
                2,
                state(2),
                bytes.len() as u32,
                format!("{:x}", Sha256::digest(&bytes)),
            )
            .unwrap();
        store.chunk(0, &bytes).unwrap();
        assert!(store.commit(2).unwrap().state.notes.is_empty());
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn spiffs_journal_multiple_saves_and_interrupted_update_recovers() {
        let p = temp("journal");
        let store = GenerationStore::new(p.join("sd"), p.join("flash"));
        for brightness in [25, 50, 75, 100] {
            let settings = LocalSettings {
                brightness,
                ..LocalSettings::default()
            };
            store.save_settings(&settings).unwrap();
            assert_eq!(store.load_settings().brightness, brightness);
        }
        // Simulate a power loss after deleting the old slot but before completing its replacement.
        let (latest_slot, _) = latest_journal(&p.join("flash"), "settings").unwrap();
        let inactive = 1 - latest_slot;
        fs::remove_file(p.join("flash").join(format!("settings.{inactive}.json"))).unwrap();
        fs::write(
            p.join("flash").join(format!(".settings.{inactive}.tmp")),
            b"interrupted",
        )
        .unwrap();
        assert_eq!(store.load_settings().brightness, 100);
        // A corrupt newer target is ignored; the previous complete slot is still selected.
        fs::write(
            p.join("flash").join(format!("settings.{inactive}.json")),
            b"corrupt",
        )
        .unwrap();
        assert_eq!(store.load_settings().brightness, 100);
        let s = state(1);
        let deleted = store.delete_note(&s, "note_1").unwrap();
        write_journal(
            &p.join("flash"),
            "tombstones",
            &Tombstones {
                deleted_note_ids: deleted.deleted_note_ids.clone(),
            },
        )
        .unwrap();
        assert_eq!(store.deleted_ids().unwrap(), vec!["note_1"]);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn hash_and_coverage_fail_without_activating_generation() {
        let p = temp("coverage");
        let mut store = GenerationStore::new(p.join("sd"), p.join("flash"));
        let b = pack();
        store
            .begin(1, state(1), b.len() as u32, "0".repeat(64))
            .unwrap();
        assert!(store.chunk(1, &b).is_err());
        store.chunk(0, &b).unwrap();
        assert_eq!(store.commit(1).err(), Some("font_hash"));
        assert_eq!(store.active_generation, 0);
        store.abort().unwrap();
        fs::remove_dir_all(p).unwrap();
    }
}
