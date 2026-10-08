//! One bounded storage worker owns the TF file engine. The UI and USB loop never perform file I/O.
use app_launcher::files::{Command, FileEngine, FileError, FileKind, Outcome, Request, Response, SortOrder};
use p4desk_protocol::{DeviceMessage, RemoteFileEntry};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;

const QUEUE_LIMIT: usize = 8;

pub enum HostJob {
    List { path: String, offset: u32, limit: u16 },
    Mkdir { path: String },
    Begin { path: String, length: u64, sha256: String },
    Chunk { offset: u32, bytes: Vec<u8> },
    Commit,
    Abort,
    FontInstall { path: String, sha256: String },
}
impl HostJob {
    pub fn op(&self) -> &'static str {
        match self { Self::List { .. } => "file_list", Self::Mkdir { .. } => "file_mkdir",
            Self::Begin { .. } => "file_upload_begin", Self::Chunk { .. } => "file_chunk",
            Self::Commit => "file_upload_commit", Self::Abort => "file_upload_abort",
            Self::FontInstall { .. } => "file_font_install" }
    }
}
enum Job {
    Ui(Request, bool),
    Host { id: u16, epoch: u32, token: u32, job: HostJob, sd_ready: bool },
    Reset,
}
pub enum Completion {
    Ui(Response),
    Host { id: u16, epoch: u32, message: DeviceMessage, clear_upload: bool, changed: bool },
}
#[derive(Default)]
struct Mailbox { jobs: VecDeque<Job>, done: VecDeque<Completion>, stop: bool }
pub struct Service {
    mailbox: Arc<(Mutex<Mailbox>, Condvar)>,
    epoch: Arc<AtomicU32>,
    upload_token: Arc<AtomicU32>,
    thread: Option<JoinHandle<()>>,
}
impl Service {
    pub fn new(root: PathBuf) -> std::io::Result<Self> {
        let mailbox = Arc::new((Mutex::new(Mailbox::default()), Condvar::new()));
        let epoch = Arc::new(AtomicU32::new(0));
        let upload_token = Arc::new(AtomicU32::new(0));
        let worker_box = mailbox.clone(); let worker_epoch = epoch.clone();
        let worker_token = upload_token.clone();
        #[cfg(target_os = "espidf")]
        let previous_config = unsafe {
            let mut previous = esp_idf_sys::esp_pthread_get_default_config();
            if esp_idf_sys::esp_pthread_get_cfg(&mut previous) != 0 { previous = esp_idf_sys::esp_pthread_get_default_config(); }
            let mut config = previous;
            config.prio = 1; config.inherit_cfg = false;
            config.stack_alloc_caps = esp_idf_sys::MALLOC_CAP_INTERNAL | esp_idf_sys::MALLOC_CAP_8BIT;
            if esp_idf_sys::esp_pthread_set_cfg(&config) != 0 { return Err(std::io::Error::other("files thread config")); }
            previous
        };
        let spawned = std::thread::Builder::new().name("p4desk-files".into())
            .stack_size(if cfg!(target_os = "espidf") { 20 * 1024 } else { 256 * 1024 })
            .spawn(move || worker(root, worker_box, worker_epoch, worker_token));
        #[cfg(target_os = "espidf")]
        unsafe { esp_idf_sys::esp_pthread_set_cfg(&previous_config); }
        let thread = spawned?;
        Ok(Self { mailbox, epoch, upload_token, thread: Some(thread) })
    }
    fn submit(&self, job: Job) -> Result<(), FileError> {
        let (lock, wake) = &*self.mailbox;
        let mut box_ = lock.lock().unwrap();
        if box_.jobs.len() >= QUEUE_LIMIT || box_.stop { return Err(FileError::Busy); }
        box_.jobs.push_back(job); wake.notify_one(); Ok(())
    }
    pub fn ui(&self, request: Request, sd_ready: bool) -> Result<(), FileError> { self.submit(Job::Ui(request, sd_ready)) }
    pub fn host(&self, id: u16, job: HostJob, sd_ready: bool) -> Result<(), FileError> {
        if matches!(job, HostJob::Abort) { self.upload_token.fetch_add(1, Ordering::AcqRel); }
        self.submit(Job::Host { id, epoch: self.epoch(), token: self.upload_token.load(Ordering::Acquire), job, sd_ready })
    }
    pub fn epoch(&self) -> u32 { self.epoch.load(Ordering::Acquire) }
    pub fn disconnect(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
        let (lock, wake) = &*self.mailbox;
        let mut box_ = lock.lock().unwrap();
        box_.jobs.retain(|job| matches!(job, Job::Ui(..)));
        box_.done.retain(|done| matches!(done, Completion::Ui(_)));
        // Reset has priority over the next connection's begin. Mutations from the UI remain serialized.
        box_.jobs.push_front(Job::Reset); wake.notify_one();
    }
    pub fn poll(&self) -> Vec<Completion> {
        let (lock, wake) = &*self.mailbox;
        let out = lock.lock().unwrap().done.drain(..).collect();
        wake.notify_one(); out
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
        self.upload_token.fetch_add(1, Ordering::AcqRel);
        let (lock, wake) = &*self.mailbox;
        lock.lock().unwrap().stop = true; wake.notify_one();
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}
fn worker(root: PathBuf, mailbox: Arc<(Mutex<Mailbox>, Condvar)>, epoch: Arc<AtomicU32>, upload_token: Arc<AtomicU32>) {
    let font_pointer = root.join("p4desk/files-font.json");
    let mut engine = FileEngine::new(root);
    let mut initialized = false;
    let mut uploading = false;
    let mut active_epoch = epoch.load(Ordering::Acquire);
    loop {
        let job = {
            let (lock, wake) = &*mailbox;
            let mut box_ = lock.lock().unwrap();
            while !box_.stop && (box_.jobs.is_empty() || box_.done.len() >= QUEUE_LIMIT) { box_ = wake.wait(box_).unwrap(); }
            if box_.stop { break; }
            box_.jobs.pop_front().unwrap()
        };
        let next_epoch = epoch.load(Ordering::Acquire);
        let measured = !matches!(&job, Job::Host { job: HostJob::Chunk { .. }, .. });
        let started = std::time::Instant::now();
        if active_epoch != next_epoch { engine.abort_upload(); uploading = false; active_epoch = next_epoch; }
        if matches!(job, Job::Reset) { engine.abort_upload(); uploading = false; continue; }
        let ready = match &job { Job::Ui(_, ready) => *ready, Job::Host { sd_ready, .. } => *sd_ready, Job::Reset => false };
        if ready && !initialized {
            initialized = match engine.ensure_user_directories() {
                Ok(()) => true,
                Err(error) => {
                    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    crate::diagnostics::diagnostic!("p4desk_files: init code={} errno={}", error.code(), errno);
                    false
                }
            };
            if initialized {
                let restored = restore_file_font(&mut engine, &font_pointer, || true).is_some();
                crate::diagnostics::diagnostic!("p4desk_files: font_restore={}", restored);
            }
        }
        let completion = match job {
            Job::Ui(request, _) => {
                let response = if !ready { Response { revision: request.revision, result: Err(FileError::StorageUnavailable) } }
                    else if uploading { Response { revision: request.revision, result: Err(FileError::Busy) } }
                    else { engine.execute(request) };
                Completion::Ui(response)
            }
            Job::Host { id, epoch: job_epoch, token, job, .. } => {
                if job_epoch != epoch.load(Ordering::Acquire) { continue; }
                let op = job.op();
                let begin = matches!(job, HostJob::Begin { .. });
                let finish = matches!(job, HostJob::Commit | HostJob::Abort);
                let chunk = matches!(job, HostJob::Chunk { .. });
                let mut changed = false;
                let result = if !ready && !matches!(job, HostJob::Abort) { Err(FileError::StorageUnavailable) }
                    else if (begin || chunk || matches!(job, HostJob::Commit)) && token != upload_token.load(Ordering::Acquire) { Err(FileError::Cancelled) }
                    else {
                    match job {
                        HostJob::List { path, offset, limit } => {
                            if limit == 0 || limit > 32 { Err(FileError::TooManyEntries) } else {
                                engine.execute(Request { revision: 0, command: Command::List { directory: path.clone(), query: String::new(), sort: SortOrder::Name } }).result.and_then(|out| {
                                    let Outcome::Listing(list) = out else { return Err(FileError::Io); };
                                    let total = list.entries.len() as u32;
                                    let entries = list.entries.into_iter().skip(offset as usize).take(limit as usize).map(|entry| RemoteFileEntry {
                                        name: entry.name, path: entry.path, directory: entry.kind == FileKind::Directory,
                                        size: entry.size, modified_seconds: entry.modified_seconds.and_then(|n| i64::try_from(n).ok()), read_only: entry.read_only,
                                    }).collect();
                                    Ok(Some(DeviceMessage::FileListing { request_id: id, path: path.clone(), entries, total,
                                        truncated: list.truncated || offset.saturating_add(limit as u32) < total,
                                        read_only: path.split('/').next().is_some_and(|p| p.eq_ignore_ascii_case("p4desk")) }))
                                })
                            }
                        }
                        HostJob::Mkdir { path } => {
                            if uploading { Err(FileError::Busy) } else {
                                let directory = app_launcher::files::parent_directory(&path);
                                let name = path.rsplit('/').next().unwrap_or("").to_owned();
                                engine.execute(Request { revision: 0, command: Command::CreateFolder { directory, name } }).result.map(|_| { changed = true; None })
                            }
                        }
                        HostJob::Begin { path, length, sha256 } => {
                            if uploading { Err(FileError::Busy) } else { engine.begin_upload(&path, length, &sha256).map(|_| { uploading = true; None }) }
                        }
                        HostJob::Chunk { offset, bytes } => {
                            if !uploading { Err(FileError::Offset) } else { engine.upload_chunk(offset, &bytes).map(|_| None) }
                        }
                        HostJob::Commit => {
                            if !uploading { Err(FileError::Offset) } else { engine.commit_upload_if(|| epoch.load(Ordering::Acquire) == job_epoch && upload_token.load(Ordering::Acquire) == token).map(|_| { changed = true; None }) }
                        }
                        HostJob::Abort => { engine.abort_upload(); Ok(None) }
                        HostJob::FontInstall { path, sha256 } => {
                            if uploading { Err(FileError::Busy) }
                            else { load_font(&mut engine, &path, &sha256, Some(&font_pointer), || epoch.load(Ordering::Acquire) == job_epoch && upload_token.load(Ordering::Acquire) == token).map(|_| { changed = true; None }) }
                        }
                    }
                };
                let failed = result.is_err();
                let clear_upload = finish || ((begin || chunk) && failed);
                if clear_upload { engine.abort_upload(); uploading = false; }
                let message = match result { Ok(Some(message)) => message, result => DeviceMessage::Ack {
                    request_id: id, acknowledged: op.into(), ok: !failed,
                    error: result.err().map(|e| e.code().to_owned()), generation: None,
                } };
                Completion::Host { id, epoch: job_epoch, message, clear_upload, changed }
            }
            Job::Reset => unreachable!(),
        };
        let (lock, wake) = &*mailbox;
        if measured {
            let ok = match &completion { Completion::Ui(response) => response.result.is_ok(),
                Completion::Host { message: DeviceMessage::Ack { ok, .. }, .. } => *ok, _ => true };
            let code = match &completion {
                Completion::Ui(Response { result: Err(error), .. }) => error.code(),
                Completion::Host { message: DeviceMessage::Ack { error: Some(error), .. }, .. } => error.as_str(),
                _ => "none",
            };
            let errno = if ok { 0 } else { std::io::Error::last_os_error().raw_os_error().unwrap_or(0) };
            #[cfg(target_os = "espidf")]
            let stack_free = unsafe { esp_idf_sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
            #[cfg(not(target_os = "espidf"))]
            let stack_free = 0;
            crate::diagnostics::diagnostic!("p4desk_files: done ok={} code={} errno={} elapsed_ms={} stack_free={}", ok, code, errno, started.elapsed().as_millis(), stack_free);
        }
        lock.lock().unwrap().done.push_back(completion); wake.notify_one();
    }
    engine.abort_upload();
}

fn load_font(engine: &mut FileEngine, relative: &str, expected: &str, pointer: Option<&std::path::Path>, mut current: impl FnMut() -> bool) -> Result<(), FileError> {
    use tiny_flutter::graphics::font::install_file_fontpack;
    let pack = read_font_pack(engine, relative, expected, &mut current)?;
    if let Some(pointer) = pointer {
        persist_file_font(engine, pointer, relative, expected, &mut current)?;
    }
    if !current() { return Err(FileError::Cancelled); }
    install_file_fontpack(Some(Arc::new(pack))); Ok(())
}

fn read_font_pack(engine: &mut FileEngine, relative: &str, expected: &str, current: &mut impl FnMut() -> bool) -> Result<tiny_flutter::graphics::fontpack::FontPack, FileError> {
    use std::io::Read;
    use sha2::{Digest, Sha256};
    use tiny_flutter::graphics::fontpack::{FontPack, MAX_FONTPACK_BYTES, HEADER_BYTES, RECORD_BYTES};
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(FileError::Integrity); }
    let path = engine.readable_user_file(relative)?;
    let mut input = std::fs::File::open(&path)?;
    let length = input.metadata()?.len();
    if length > MAX_FONTPACK_BYTES as u64 { return Err(FileError::TooLarge); }
    let mut header = [0u8; HEADER_BYTES]; input.read_exact(&mut header)?;
    let count = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    if count > 16_384 { return Err(FileError::TooLarge); }
    let pool = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;
    if &header[..4] != b"P4F1" || count > 16_384 || pool != HEADER_BYTES + count * RECORD_BYTES
        || pool as u64 > length || u32::from_le_bytes(header[20..24].try_into().unwrap()) as u64 != length {
        return Err(FileError::InvalidText);
    }
    use std::io::{Seek, SeekFrom}; input.seek(SeekFrom::Start(0))?;
    let mut digest = Sha256::new(); let mut buffer = vec![0; 8192];
    loop { if !current() { return Err(FileError::Cancelled); } let n = input.read(&mut buffer)?; if n == 0 { break; } digest.update(&buffer[..n]); }
    if format!("{:x}", digest.finalize()) != expected.to_ascii_lowercase() { return Err(FileError::Integrity); }
    drop(input); drop(buffer);
    // The file source loads only its bounded index and glyph cache, not the bitmap pool.
    let pack = FontPack::from_file(&path).map_err(|_| FileError::InvalidText)?;
    if pack.glyph_count() > 16_384 { return Err(FileError::TooLarge); }
    if !current() { return Err(FileError::Cancelled); }
    Ok(pack)
}

const MAX_FONT_POINTER_BYTES: usize = 2048;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileFontPointer {
    version: u8,
    generation: u64,
    path: String,
    sha256: String,
    checksum: String,
}
impl FileFontPointer {
    fn checksum(&self) -> Result<String, FileError> {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(&("P4DeskFilesFontV1", self.generation, &self.path, &self.sha256))
            .map_err(|_| FileError::Io)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
    fn valid(&self) -> bool {
        self.version == 1 && self.generation != 0 && !self.path.is_empty()
            && self.path.len() <= 512 && valid_hash(&self.sha256) && valid_hash(&self.checksum)
            && self.checksum().is_ok_and(|hash| hash == self.checksum)
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFileFontPointer { path: String, sha256: String }

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn pointer_parent_is_safe(pointer: &std::path::Path) -> Result<(), FileError> {
    let metadata = std::fs::symlink_metadata(pointer.parent().ok_or(FileError::InvalidPath)?)?;
    if metadata.file_type().is_symlink() { return Err(FileError::InvalidPath); }
    if !metadata.is_dir() { return Err(FileError::NotDirectory); }
    Ok(())
}
fn bounded_pointer_bytes(path: &std::path::Path) -> Result<Vec<u8>, FileError> {
    use std::io::Read;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() { return Err(FileError::InvalidPath); }
    if metadata.len() > MAX_FONT_POINTER_BYTES as u64 { return Err(FileError::TooLarge); }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take((MAX_FONT_POINTER_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FONT_POINTER_BYTES { return Err(FileError::TooLarge); }
    Ok(bytes)
}
fn read_font_pointer(path: &std::path::Path) -> Option<FileFontPointer> {
    let record: FileFontPointer = serde_json::from_slice(&bounded_pointer_bytes(path).ok()?).ok()?;
    record.valid().then_some(record)
}
fn font_pointer_slots(pointer: &std::path::Path) -> Vec<(usize, FileFontPointer)> {
    if pointer_parent_is_safe(pointer).is_err() { return Vec::new(); }
    let mut records: Vec<_> = [0, 1].into_iter().filter_map(|slot| {
        read_font_pointer(&pointer.with_extension(format!("{slot}.json"))).map(|record| (slot, record))
    }).collect();
    records.sort_by(|a, b| b.1.generation.cmp(&a.1.generation));
    records
}

/// Prefer the newest complete pointer whose referenced font also passes bounded
/// structure and full-file SHA validation; a damaged newest font falls back.
fn restore_file_font(engine: &mut FileEngine, pointer: &std::path::Path, mut current: impl FnMut() -> bool) -> Option<String> {
    for (_, record) in font_pointer_slots(pointer) {
        if !current() { return None; }
        if load_font(engine, &record.path, &record.sha256, None, &mut current).is_ok() {
            return Some(record.path);
        }
    }
    if pointer_parent_is_safe(pointer).is_err() || !current() { return None; }
    let legacy: LegacyFileFontPointer = serde_json::from_slice(&bounded_pointer_bytes(pointer).ok()?).ok()?;
    if legacy.path.is_empty() || legacy.path.len() > 512 || !valid_hash(&legacy.sha256) { return None; }
    load_font(engine, &legacy.path, &legacy.sha256, None, &mut current).ok()?;
    Some(legacy.path)
}

struct FontPointerTemporary(PathBuf);
impl Drop for FontPointerTemporary {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); }
}
fn remove_owned_pointer(path: &std::path::Path) -> Result<(), FileError> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => Ok(std::fs::remove_file(path)?),
        Ok(_) => Err(FileError::InvalidPath),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
fn persist_file_font(engine: &mut FileEngine, pointer: &std::path::Path, relative: &str, expected: &str, current: &mut impl FnMut() -> bool) -> Result<(), FileError> {
    use std::io::Write;
    let parent = pointer.parent().ok_or(FileError::InvalidPath)?;
    match std::fs::create_dir(parent) {
        Ok(()) => (),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(error) => return Err(error.into()),
    }
    pointer_parent_is_safe(pointer)?;
    let records = font_pointer_slots(pointer);
    let generation = records.first().map(|(_, r)| r.generation).unwrap_or(0)
        .checked_add(1).ok_or(FileError::TooLarge)?;
    // Choose the opposite slot from a verified usable font, not merely the
    // highest JSON generation: that font may have been removed or damaged.
    let latest_usable = records.iter().find_map(|(slot, record)| {
        read_font_pack(engine, &record.path, &record.sha256, current).ok().map(|_| *slot)
    });
    let slot = latest_usable.map(|slot| 1 - slot).unwrap_or(0);
    let mut record = FileFontPointer { version: 1, generation, path: relative.to_owned(),
        sha256: expected.to_ascii_lowercase(), checksum: String::new() };
    record.checksum = record.checksum()?;
    let encoded = serde_json::to_vec(&record).map_err(|_| FileError::Io)?;
    if encoded.len() > MAX_FONT_POINTER_BYTES { return Err(FileError::TooLarge); }
    if !current() { return Err(FileError::Cancelled); }
    let temporary = pointer.with_extension(format!("{slot}.tmp"));
    remove_owned_pointer(&temporary)?;
    let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    let guard = FontPointerTemporary(temporary);
    let written = output.write_all(&encoded).and_then(|_| output.sync_all());
    drop(output); // FatFs forbids deleting or renaming an open file with FS_LOCK.
    written?;
    let persisted = read_font_pointer(&guard.0).ok_or(FileError::Integrity)?;
    if persisted.generation != generation || persisted.path != relative || persisted.sha256 != record.sha256 {
        return Err(FileError::Integrity);
    }
    if !current() { return Err(FileError::Cancelled); }
    let target = pointer.with_extension(format!("{slot}.json"));
    remove_owned_pointer(&target)?;
    if !current() { return Err(FileError::Cancelled); }
    std::fs::rename(&guard.0, target)?;
    Ok(())
}

#[cfg(test)]
mod font_pointer_tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use tiny_flutter::graphics::fontpack::{encode_fontpack, PackGlyph};

    struct Card(PathBuf);
    impl Card {
        fn new() -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(1);
            let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let root = std::env::temp_dir().join(format!("p4desk-font-pointer-{time}-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
            std::fs::create_dir(&root).unwrap();
            std::fs::create_dir(root.join("Fonts")).unwrap();
            Self(root)
        }
        fn font(&self, name: &str, character: char) -> String {
            let bytes = encode_fontpack(vec![PackGlyph { character, size: 22, width: 1, height: 1,
                xmin: 0, ymin: 0, advance: 12.0, bitmap: vec![200] }]).unwrap();
            std::fs::write(self.0.join(name), &bytes).unwrap();
            format!("{:x}", Sha256::digest(bytes))
        }
    }
    impl Drop for Card {
        fn drop(&mut self) {
            tiny_flutter::graphics::font::install_file_fontpack(None);
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn consecutive_font_installs_replace_old_slot_and_restore_verified_fallback() {
        let card = Card::new();
        let a = card.font("Fonts/a.p4f", '甲');
        let b = card.font("Fonts/b.p4f", '乙');
        let c = card.font("Fonts/c.p4f", '丙');
        let d = card.font("Fonts/d.p4f", '丁');
        let pointer = card.0.join("p4desk/files-font.json");
        let slot0 = pointer.with_extension("0.json");
        let slot1 = pointer.with_extension("1.json");
        let mut engine = FileEngine::new(&card.0);
        load_font(&mut engine, "Fonts/a.p4f", &a, Some(&pointer), || true).unwrap();
        assert_eq!(read_font_pointer(&slot0).unwrap().generation, 1);
        load_font(&mut engine, "Fonts/b.p4f", &b, Some(&pointer), || true).unwrap();
        assert_eq!(read_font_pointer(&slot1).unwrap().generation, 2);
        assert_eq!(read_font_pointer(&slot0).unwrap().path, "Fonts/a.p4f");
        load_font(&mut engine, "Fonts/c.p4f", &c, Some(&pointer), || true).unwrap();
        assert_eq!(read_font_pointer(&slot0).unwrap().generation, 3);
        assert_eq!(read_font_pointer(&slot1).unwrap().path, "Fonts/b.p4f");
        let good_slot0 = std::fs::read(&slot0).unwrap();
        let mut corrupt: serde_json::Value = serde_json::from_slice(&good_slot0).unwrap();
        corrupt["generation"] = serde_json::json!(999);
        std::fs::write(&slot0, serde_json::to_vec(&corrupt).unwrap()).unwrap();
        assert!(read_font_pointer(&slot0).is_none());
        assert_eq!(restore_file_font(&mut engine, &pointer, || true).as_deref(), Some("Fonts/b.p4f"));
        std::fs::write(&slot0, &good_slot0).unwrap();
        // A valid newest pointer can still reference a corrupted font. Both boot
        // recovery and the next write preserve the other usable font generation.
        std::fs::write(card.0.join("Fonts/c.p4f"), b"damaged fixture").unwrap();
        assert_eq!(restore_file_font(&mut engine, &pointer, || true).as_deref(), Some("Fonts/b.p4f"));
        load_font(&mut engine, "Fonts/d.p4f", &d, Some(&pointer), || true).unwrap();
        assert_eq!(read_font_pointer(&slot0).unwrap().generation, 4);
        assert_eq!(read_font_pointer(&slot1).unwrap().path, "Fonts/b.p4f");
        std::fs::write(&slot0, vec![b' '; MAX_FONT_POINTER_BYTES + 1]).unwrap();
        assert_eq!(restore_file_font(&mut engine, &pointer, || true).as_deref(), Some("Fonts/b.p4f"));
        std::fs::remove_file(&slot0).unwrap();
        std::fs::remove_file(&slot1).unwrap();
        std::fs::write(&pointer, serde_json::to_vec(&serde_json::json!({"path":"Fonts/a.p4f", "sha256":a})).unwrap()).unwrap();
        assert_eq!(restore_file_font(&mut engine, &pointer, || true).as_deref(), Some("Fonts/a.p4f"));
        assert!(!pointer.with_extension("0.tmp").exists());
        assert!(!pointer.with_extension("1.tmp").exists());
    }
}
