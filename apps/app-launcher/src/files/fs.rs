use super::{
    file_kind, parent_directory, Command, DirectoryListing, FileEntry, FileError, FileKind,
    Outcome, Request, Response, SortOrder, TrashEntry,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const TRASH: &str = ".p4desk-trash";
const TEMP_PREFIX: &str = ".p4desk-filepart-";
const MAX_UPLOAD: u64 = 256 * 1024 * 1024;
static NONCE: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub directory_entries: usize,
    pub directory_scan: usize,
    pub recursive_entries: usize,
    pub recursive_depth: usize,
    pub copy_bytes: u64,
    pub text_bytes: usize,
    pub image_file_bytes: u64,
    pub image_pixels: usize,
    pub preview_width: u32,
    pub preview_height: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            directory_entries: 256,
            directory_scan: 4096,
            recursive_entries: 1024,
            recursive_depth: 12,
            copy_bytes: 64 * 1024 * 1024,
            text_bytes: 32 * 1024,
            image_file_bytes: 4 * 1024 * 1024,
            image_pixels: 1024 * 1024,
            preview_width: 640,
            preview_height: 400,
        }
    }
}

/// Owned by one worker. Filesystem operations must run outside the UI state lock.
pub struct FileEngine {
    root: PathBuf,
    limits: Limits,
    upload: Option<Upload>,
    cleaned: bool,
}
impl FileEngine {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_limits(root, Limits::default())
    }
    pub fn with_limits(root: impl Into<PathBuf>, limits: Limits) -> Self {
        Self {
            root: root.into(),
            limits,
            upload: None,
            cleaned: false,
        }
    }
    pub fn execute(&mut self, request: Request) -> Response {
        let result = self.execute_command(request.command);
        Response {
            revision: request.revision,
            result,
        }
    }
    pub fn ensure_user_directories(&mut self) -> Result<(), FileError> {
        self.ready()?;
        for name in ["Files", "Documents", "Pictures", "Downloads", "Fonts"] {
            let path = self.resolve(name, false)?;
            match fs::create_dir(&path) {
                Ok(()) => (),
                Err(error)
                    if error.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() =>
                {
                    ()
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
    pub fn readable_user_file(&mut self, path: &str) -> Result<PathBuf, FileError> {
        self.ready()?;
        require_unprotected(path)?;
        let resolved = self.resolve(path, true)?;
        if !fs::symlink_metadata(&resolved)?.is_file() {
            return Err(FileError::Unsupported);
        }
        Ok(resolved)
    }
    fn execute_command(&mut self, command: Command) -> Result<Outcome, FileError> {
        self.ready()?;
        if self.upload.is_some()
            && !matches!(
                command,
                Command::List { .. } | Command::Preview { .. } | Command::ListTrash
            )
        {
            return Err(FileError::Busy);
        }
        match command {
            Command::List {
                directory,
                query,
                sort,
            } => Ok(Outcome::Listing(self.list(&directory, &query, sort)?)),
            Command::Preview { path } => {
                require_unprotected(&path)?;
                let resolved = self.resolve(&path, true)?;
                Ok(Outcome::Preview(super::preview::read_preview(
                    &resolved,
                    self.limits,
                )?))
            }
            Command::CreateFolder { directory, name } => {
                let path = join_name(&directory, &name)?;
                require_unprotected(&path)?;
                let target = self.resolve(&path, false)?;
                self.ensure_absent(&target)?;
                fs::create_dir(&target)?;
                Ok(Outcome::Changed { directory })
            }
            Command::Rename { path, name } => {
                require_unprotected(&path)?;
                let directory = parent_directory(&path);
                let target_relative = join_name(&directory, &name)?;
                require_unprotected(&target_relative)?;
                let source = self.resolve(&path, true)?;
                let target = self.resolve(&target_relative, false)?;
                self.ensure_absent(&target)?;
                fs::rename(source, target)?;
                Ok(Outcome::Changed { directory })
            }
            Command::Copy {
                source,
                destination,
            } => {
                self.copy(&source, &destination)?;
                Ok(Outcome::Changed {
                    directory: destination,
                })
            }
            Command::Move {
                source,
                destination,
            } => {
                let (resolved, target) = self.transfer_paths(&source, &destination)?;
                self.inspect_tree(&resolved, 0, &mut Budget::default())?;
                fs::rename(resolved, target)?;
                Ok(Outcome::Changed {
                    directory: destination,
                })
            }
            Command::Trash { path } => {
                self.trash(&path)?;
                Ok(Outcome::Changed {
                    directory: parent_directory(&path),
                })
            }
            Command::ListTrash => Ok(Outcome::TrashListing(self.list_trash()?)),
            Command::Restore { id } => {
                let directory = self.restore(&id)?;
                Ok(Outcome::Changed { directory })
            }
            Command::DeletePermanently { id } => {
                self.delete_permanently(&id)?;
                Ok(Outcome::TrashListing(self.list_trash()?))
            }
        }
    }

    fn ready(&mut self) -> Result<(), FileError> {
        if !self.root.is_dir() {
            return Err(FileError::StorageUnavailable);
        }
        let canonical = fs::canonicalize(&self.root).map_err(|_| FileError::StorageUnavailable)?;
        self.root = canonical;
        if !self.cleaned {
            self.cleanup_markers(&self.root, 0, &mut 0usize);
            self.cleaned = true;
        }
        Ok(())
    }

    /// Reject symlinks at every component instead of following a link out of the card.
    fn resolve(&self, relative: &str, must_exist: bool) -> Result<PathBuf, FileError> {
        validate_relative(relative)?;
        let mut path = self.root.clone();
        let parts: Vec<_> = Path::new(relative).components().collect();
        for (index, component) in parts.iter().enumerate() {
            path.push(component.as_os_str());
            match fs::symlink_metadata(&path) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        return Err(FileError::InvalidPath);
                    }
                    if index + 1 < parts.len() && !metadata.is_dir() {
                        return Err(FileError::NotDirectory);
                    }
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        && !must_exist
                        && index + 1 == parts.len() =>
                {
                    ()
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(path)
    }
    fn ensure_absent(&self, target: &Path) -> Result<(), FileError> {
        // FatFs is case insensitive even when the host test filesystem is not.
        if let Some(parent) = target.parent() {
            let name = target
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(FileError::InvalidName)?;
            for (index, item) in fs::read_dir(parent)?.enumerate() {
                if index >= self.limits.directory_scan {
                    return Err(FileError::TooManyEntries);
                }
                if item?
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .flat_map(char::to_lowercase)
                    .eq(name.chars().flat_map(char::to_lowercase))
                {
                    return Err(FileError::AlreadyExists);
                }
            }
        }
        match fs::symlink_metadata(target) {
            Ok(_) => Err(FileError::AlreadyExists),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
    fn list(
        &self,
        directory: &str,
        query: &str,
        sort: SortOrder,
    ) -> Result<DirectoryListing, FileError> {
        let resolved = self.resolve(directory, true)?;
        if !resolved.is_dir() {
            return Err(FileError::NotDirectory);
        }
        if query.len() > 256 {
            return Err(FileError::InvalidName);
        }
        let query = query.to_lowercase();
        let mut entries = Vec::with_capacity(self.limits.directory_entries.min(32));
        let mut scanned = 0usize;
        let mut truncated = false;
        for item in fs::read_dir(resolved)? {
            if scanned >= self.limits.directory_scan {
                truncated = true;
                break;
            }
            scanned += 1;
            let item = item?;
            let name = match item.file_name().into_string() {
                Ok(name) => name,
                Err(_) => {
                    truncated = true;
                    continue;
                }
            };
            if is_internal_name(&name) || name == "." || name == ".." {
                continue;
            }
            if !query.is_empty() && !name.to_lowercase().contains(&query) {
                continue;
            }
            let metadata = fs::symlink_metadata(item.path())?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            let path = match join_name(directory, &name) {
                Ok(path) => path,
                Err(_) => {
                    // Existing card entries outside the supported path contract must
                    // not prevent browsing the remaining, safely addressable entries.
                    truncated = true;
                    continue;
                }
            };
            let entry = FileEntry {
                path: path.clone(),
                name: name.clone(),
                kind: file_kind(&name, metadata.is_dir()),
                size: metadata.len(),
                modified_seconds: metadata.modified().ok().and_then(epoch_seconds),
                read_only: is_protected(&path) || metadata.permissions().readonly(),
            };
            // Keep only the first sorted N entries: large directories never build an unbounded vector.
            let position = entries
                .binary_search_by(|candidate| compare_entries(candidate, &entry, sort))
                .unwrap_or_else(|at| at);
            if entries.len() < self.limits.directory_entries {
                entries.insert(position, entry);
            } else {
                truncated = true;
                if position < entries.len() {
                    entries.insert(position, entry);
                    entries.pop();
                }
            }
        }
        Ok(DirectoryListing {
            directory: directory.to_owned(),
            entries,
            truncated,
            scanned,
        })
    }

    fn transfer_paths(
        &self,
        source: &str,
        destination: &str,
    ) -> Result<(PathBuf, PathBuf), FileError> {
        require_unprotected(source)?;
        validate_relative(destination)?;
        if is_protected(destination) {
            return Err(FileError::Protected);
        }
        if path_is_within_case_insensitive(Path::new(destination), Path::new(source)) {
            return Err(FileError::InvalidPath);
        }
        let from = self.resolve(source, true)?;
        let folder = self.resolve(destination, true)?;
        if !folder.is_dir() {
            return Err(FileError::NotDirectory);
        }
        let name = from
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(FileError::InvalidPath)?;
        let to = self.resolve(&join_name(destination, name)?, false)?;
        if to.starts_with(&from) || from == to {
            return Err(FileError::InvalidPath);
        }
        self.ensure_absent(&to)?;
        Ok((from, to))
    }

    fn inspect_tree(
        &self,
        source: &Path,
        depth: usize,
        budget: &mut Budget,
    ) -> Result<(), FileError> {
        if depth == 0 {
            budget.discovered = 1;
        }
        if depth > self.limits.recursive_depth {
            return Err(FileError::TooManyEntries);
        }
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            return Err(FileError::InvalidPath);
        }
        budget.entries += 1;
        if budget.entries > self.limits.recursive_entries {
            return Err(FileError::TooManyEntries);
        }
        if metadata.is_file() {
            budget.bytes = budget
                .bytes
                .checked_add(metadata.len())
                .ok_or(FileError::TooLarge)?;
            if budget.enforce_bytes && budget.bytes > self.limits.copy_bytes {
                return Err(FileError::TooLarge);
            }
        } else if metadata.is_dir() {
            let children = collect_children(
                source,
                self.limits
                    .recursive_entries
                    .saturating_sub(budget.discovered),
                false,
            )?;
            budget.discovered += children.len();
            for child in children {
                if is_internal_name(
                    &child
                        .file_name()
                        .ok_or(FileError::InvalidPath)?
                        .to_string_lossy(),
                ) {
                    return Err(FileError::Protected);
                }
                self.inspect_tree(&child, depth + 1, budget)?;
            }
        } else {
            return Err(FileError::Unsupported);
        }
        Ok(())
    }

    fn copy(&self, source: &str, destination: &str) -> Result<(), FileError> {
        let (source, target) = self.transfer_paths(source, destination)?;
        self.inspect_tree(
            &source,
            0,
            &mut Budget {
                enforce_bytes: true,
                ..Budget::default()
            },
        )?;
        let mut target_owned = false;
        let result = self.copy_tree(
            &source,
            &target,
            0,
            &mut Budget::default(),
            &mut target_owned,
        );
        if result.is_err() && target_owned {
            remove_owned_tree(&target, self.limits);
        }
        result
    }
    fn copy_tree(
        &self,
        source: &Path,
        target: &Path,
        depth: usize,
        budget: &mut Budget,
        target_owned: &mut bool,
    ) -> Result<(), FileError> {
        if depth == 0 {
            budget.discovered = 1;
        }
        if depth > self.limits.recursive_depth {
            return Err(FileError::TooManyEntries);
        }
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            return Err(FileError::InvalidPath);
        }
        budget.entries += 1;
        if budget.entries > self.limits.recursive_entries {
            return Err(FileError::TooManyEntries);
        }
        if metadata.is_dir() {
            fs::create_dir(target)?;
            *target_owned = true;
            let children = collect_children(
                source,
                self.limits
                    .recursive_entries
                    .saturating_sub(budget.discovered),
                false,
            )?;
            budget.discovered += children.len();
            for child in children {
                let name = child.file_name().ok_or(FileError::InvalidPath)?;
                if is_internal_name(&name.to_string_lossy()) {
                    return Err(FileError::Protected);
                }
                self.copy_tree(&child, &target.join(name), depth + 1, budget, &mut false)?;
            }
            return Ok(());
        }
        if !metadata.is_file() {
            return Err(FileError::Unsupported);
        }
        let mut input = File::open(source)?;
        let (mut temporary, mut output) =
            TempFile::create(target.parent().ok_or(FileError::InvalidPath)?)?;
        let mut buffer = vec![0u8; 8192];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            budget.bytes = budget
                .bytes
                .checked_add(count as u64)
                .ok_or(FileError::TooLarge)?;
            if budget.bytes > self.limits.copy_bytes {
                return Err(FileError::TooLarge);
            }
            output.write_all(&buffer[..count])?;
            std::thread::yield_now();
        }
        output.sync_all()?;
        drop(output);
        self.ensure_absent(target)?;
        temporary.commit(target)?;
        *target_owned = true;
        Ok(())
    }

    fn trash_root(&self, create: bool) -> Result<PathBuf, FileError> {
        let path = self.root.join(TRASH);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => Ok(path),
            Ok(_) => Err(FileError::InvalidPath),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => {
                fs::create_dir(&path)?;
                Ok(path)
            }
            Err(error) => Err(error.into()),
        }
    }
    fn trash(&self, relative: &str) -> Result<(), FileError> {
        require_unprotected(relative)?;
        let source = self.resolve(relative, true)?;
        self.inspect_tree(&source, 0, &mut Budget::default())?;
        let metadata = fs::metadata(&source)?;
        let root = self.trash_root(true)?;
        let folder = reserve_unique_directory(&root)?;
        let manifest = TrashManifest {
            version: 1,
            original_path: relative.to_owned(),
            directory: metadata.is_dir(),
            size: metadata.len(),
            deleted_seconds: now_seconds(),
        };
        let result = (|| {
            let encoded = serde_json::to_vec(&manifest).map_err(|_| FileError::Io)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(folder.join("entry.json"))?;
            file.write_all(&encoded)?;
            file.sync_all()?;
            drop(file);
            fs::rename(source, folder.join("data"))?;
            Ok(())
        })();
        if result.is_err() {
            remove_owned_tree(&folder, self.limits);
        }
        result
    }
    fn read_trash(&self, id: &str) -> Result<(PathBuf, TrashManifest), FileError> {
        if !valid_id(id) {
            return Err(FileError::InvalidPath);
        }
        let folder = self.trash_root(false)?.join(id);
        for path in [&folder, &folder.join("entry.json"), &folder.join("data")] {
            if fs::symlink_metadata(path)?.file_type().is_symlink() {
                return Err(FileError::InvalidPath);
            }
        }
        let mut data = Vec::new();
        File::open(folder.join("entry.json"))?
            .take(4097)
            .read_to_end(&mut data)?;
        if data.len() > 4096 {
            return Err(FileError::InvalidPath);
        }
        let manifest: TrashManifest =
            serde_json::from_slice(&data).map_err(|_| FileError::InvalidPath)?;
        if manifest.version != 1 {
            return Err(FileError::InvalidPath);
        }
        validate_relative(&manifest.original_path)?;
        require_unprotected(&manifest.original_path)?;
        Ok((folder, manifest))
    }
    fn list_trash(&self) -> Result<Vec<TrashEntry>, FileError> {
        let root = match self.trash_root(false) {
            Ok(root) => root,
            Err(FileError::NotFound) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut entries = Vec::new();
        for item in fs::read_dir(root)?.take(self.limits.directory_scan) {
            let item = item?;
            let Some(id) = item.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Ok((_, manifest)) = self.read_trash(&id) else {
                continue;
            };
            let name = manifest
                .original_path
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_owned();
            entries.push(TrashEntry {
                id,
                original_path: manifest.original_path,
                kind: file_kind(&name, manifest.directory),
                name,
                size: manifest.size,
                deleted_seconds: manifest.deleted_seconds,
            });
            if entries.len() >= self.limits.directory_entries {
                break;
            }
        }
        entries.sort_by(|a, b| {
            b.deleted_seconds
                .cmp(&a.deleted_seconds)
                .then(a.id.cmp(&b.id))
        });
        Ok(entries)
    }
    fn restore(&self, id: &str) -> Result<String, FileError> {
        let (folder, manifest) = self.read_trash(id)?;
        let target = self.resolve(&manifest.original_path, false)?;
        self.ensure_absent(&target)?;
        self.inspect_tree(&folder.join("data"), 0, &mut Budget::default())?;
        fs::rename(folder.join("data"), target)?;
        let _ = fs::remove_file(folder.join("entry.json"));
        let _ = fs::remove_dir(folder);
        Ok(parent_directory(&manifest.original_path))
    }
    fn delete_permanently(&self, id: &str) -> Result<(), FileError> {
        let (folder, _) = self.read_trash(id)?;
        self.inspect_tree(&folder.join("data"), 0, &mut Budget::default())?;
        remove_tree_checked(&folder.join("data"), self.limits)?;
        fs::remove_file(folder.join("entry.json"))?;
        fs::remove_dir(folder)?;
        Ok(())
    }

    pub fn begin_upload(&mut self, path: &str, length: u64, sha256: &str) -> Result<(), FileError> {
        self.ready()?;
        if self.upload.is_some() {
            return Err(FileError::Busy);
        }
        require_unprotected(path)?;
        if length > MAX_UPLOAD {
            return Err(FileError::TooLarge);
        }
        let expected = parse_sha256(sha256).ok_or(FileError::InvalidPath)?;
        let target = self.resolve(path, false)?;
        self.ensure_absent(&target)?;
        let (temporary, output) = TempFile::create(target.parent().ok_or(FileError::InvalidPath)?)?;
        self.upload = Some(Upload {
            target,
            relative: path.to_owned(),
            temporary,
            output: Some(output),
            length,
            received: 0,
            expected,
            digest: Sha256::new(),
        });
        Ok(())
    }
    pub fn upload_chunk(&mut self, offset: u32, bytes: &[u8]) -> Result<(), FileError> {
        let upload = self.upload.as_mut().ok_or(FileError::InvalidPath)?;
        if bytes.len() > 32 * 1024
            || bytes.is_empty()
            || upload.received != u64::from(offset)
            || upload
                .received
                .checked_add(bytes.len() as u64)
                .is_none_or(|end| end > upload.length)
        {
            return Err(FileError::Offset);
        }
        upload
            .output
            .as_mut()
            .ok_or(FileError::InvalidPath)?
            .write_all(bytes)?;
        upload.digest.update(bytes);
        upload.received += bytes.len() as u64;
        Ok(())
    }
    pub fn commit_upload(&mut self) -> Result<String, FileError> {
        self.commit_upload_if(|| true)
    }
    /// A disconnect/timeout token can cancel a long persisted-file verification.
    /// No completed destination becomes visible after cancellation is observed.
    pub fn commit_upload_if(
        &mut self,
        mut still_active: impl FnMut() -> bool,
    ) -> Result<String, FileError> {
        let mut upload = self.upload.take().ok_or(FileError::InvalidPath)?;
        if !still_active() {
            return Err(FileError::Cancelled);
        }
        if upload.received != upload.length
            || <[u8; 32]>::from(upload.digest.clone().finalize()) != upload.expected
        {
            return Err(FileError::Integrity);
        }
        let output = upload.output.take().ok_or(FileError::InvalidPath)?;
        output.sync_all()?;
        drop(output);
        // Verify persisted bytes rather than acknowledging only the transport's digest.
        let mut persisted = Sha256::new();
        let mut input = File::open(&upload.temporary.part)?;
        let mut buffer = vec![0u8; 8192];
        let mut persisted_length = 0u64;
        loop {
            if !still_active() {
                return Err(FileError::Cancelled);
            }
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            persisted_length = persisted_length
                .checked_add(count as u64)
                .ok_or(FileError::Integrity)?;
            if persisted_length > upload.length {
                return Err(FileError::Integrity);
            }
            persisted.update(&buffer[..count]);
            std::thread::yield_now();
        }
        drop(input);
        if persisted_length != upload.length
            || <[u8; 32]>::from(persisted.finalize()) != upload.expected
        {
            return Err(FileError::Integrity);
        }
        self.ensure_absent(&upload.target)?;
        // Revalidate the target parent after the transfer, before exposing a complete file.
        if self.resolve(&upload.relative, false)? != upload.target {
            return Err(FileError::InvalidPath);
        }
        if !still_active() {
            return Err(FileError::Cancelled);
        }
        upload.temporary.commit(&upload.target)?;
        Ok(parent_directory(&upload.relative))
    }
    pub fn abort_upload(&mut self) {
        self.upload = None;
    }

    fn cleanup_markers(&self, folder: &Path, depth: usize, visited: &mut usize) {
        if depth > self.limits.recursive_depth || *visited >= self.limits.directory_scan {
            return;
        }
        let Ok(paths) = collect_children(
            folder,
            self.limits.directory_scan.saturating_sub(*visited),
            true,
        ) else {
            return;
        };
        *visited += paths.len();
        for path in paths {
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            let Some(name) = path.file_name() else {
                continue;
            };
            let Some(name) = name.to_str() else {
                continue;
            };
            if meta.is_dir() {
                if !is_internal_name(name) && !name.eq_ignore_ascii_case("p4desk") {
                    self.cleanup_markers(&path, depth + 1, visited);
                }
            } else if name.starts_with(TEMP_PREFIX) && name.ends_with(".json") {
                let id = &name[TEMP_PREFIX.len()..name.len() - 5];
                if !valid_id(id) || meta.len() > 128 {
                    continue;
                }
                let Ok(bytes) = fs::read(&path) else {
                    continue;
                };
                let Ok(marker) = serde_json::from_slice::<TempMarker>(&bytes) else {
                    continue;
                };
                if marker.version != 1 || marker.id != id {
                    continue;
                }
                let part = folder.join(format!("{TEMP_PREFIX}{id}.part"));
                if let Ok(meta) = fs::symlink_metadata(&part) {
                    if meta.is_file() && !meta.file_type().is_symlink() {
                        let _ = fs::remove_file(part);
                    }
                }
                let _ = fs::remove_file(&path);
            }
        }
    }
}

#[derive(Default)]
struct Budget {
    bytes: u64,
    entries: usize,
    discovered: usize,
    enforce_bytes: bool,
}
#[derive(Serialize, Deserialize)]
struct TrashManifest {
    version: u8,
    original_path: String,
    directory: bool,
    size: u64,
    deleted_seconds: u64,
}
#[derive(Serialize, Deserialize)]
struct TempMarker {
    version: u8,
    id: String,
}
struct Upload {
    target: PathBuf,
    relative: String,
    temporary: TempFile,
    output: Option<File>,
    length: u64,
    received: u64,
    expected: [u8; 32],
    digest: Sha256,
}
impl Drop for Upload {
    fn drop(&mut self) {
        self.output = None;
    }
}

struct TempFile {
    part: PathBuf,
    marker: PathBuf,
    committed: bool,
    part_created: bool,
}
impl TempFile {
    fn create(folder: &Path) -> Result<(Self, File), FileError> {
        for _ in 0..16 {
            match Self::create_once(folder) {
                Err(FileError::AlreadyExists) => continue,
                result => return result,
            }
        }
        Err(FileError::AlreadyExists)
    }
    fn create_once(folder: &Path) -> Result<(Self, File), FileError> {
        let id = unique_id();
        let marker = folder.join(format!("{TEMP_PREFIX}{id}.json"));
        let part = folder.join(format!("{TEMP_PREFIX}{id}.part"));
        let mut marker_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&marker)?;
        let mut guard = Self {
            part,
            marker,
            committed: false,
            part_created: false,
        };
        let encoded =
            serde_json::to_vec(&TempMarker { version: 1, id }).map_err(|_| FileError::Io)?;
        let marker_result = marker_file
            .write_all(&encoded)
            .and_then(|_| marker_file.sync_all());
        // FatFs rejects unlinking an open file when FS_LOCK is enabled. Close the
        // marker before propagating write/sync errors so the guard can remove it.
        drop(marker_file);
        marker_result?;
        let output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&guard.part)?;
        guard.part_created = true;
        guard.committed = false;
        Ok((guard, output))
    }
    fn commit(&mut self, destination: &Path) -> Result<(), FileError> {
        fs::rename(&self.part, destination)?;
        self.committed = true;
        let _ = fs::remove_file(&self.marker);
        Ok(())
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.committed && self.part_created {
            let _ = fs::remove_file(&self.part);
        }
        let _ = fs::remove_file(&self.marker);
    }
}

fn compare_entries(a: &FileEntry, b: &FileEntry, sort: SortOrder) -> std::cmp::Ordering {
    (a.kind != FileKind::Directory)
        .cmp(&(b.kind != FileKind::Directory))
        .then_with(|| match sort {
            SortOrder::Name => a
                .name
                .chars()
                .flat_map(char::to_lowercase)
                .cmp(b.name.chars().flat_map(char::to_lowercase)),
            SortOrder::Modified => b.modified_seconds.cmp(&a.modified_seconds),
            SortOrder::Size => b.size.cmp(&a.size),
        })
        .then(a.name.cmp(&b.name))
}
fn validate_relative(path: &str) -> Result<(), FileError> {
    if path.len() > 512 || path.contains('\0') || path.contains('\\') || path.starts_with('/') {
        return Err(FileError::InvalidPath);
    }
    for component in Path::new(path).components() {
        match component {
            Component::Normal(name)
                if name.to_str().is_some_and(|name| !is_internal_name(name)) =>
            {
                ()
            }
            _ => return Err(FileError::InvalidPath),
        }
    }
    // Path::components normalizes embedded '.'; reject it and repeated separators explicitly.
    if !path.is_empty()
        && path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.len() > 255
                || part.ends_with('.')
                || part.ends_with(' ')
                || part
                    .chars()
                    .any(|c| c.is_control() || ":*?\"<>|".contains(c))
        })
    {
        return Err(FileError::InvalidPath);
    }
    Ok(())
}
fn join_name(directory: &str, name: &str) -> Result<String, FileError> {
    validate_relative(directory)?;
    if name.is_empty()
        || name.len() > 255
        || name.ends_with(' ')
        || name.ends_with('.')
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
        || name == "."
        || name == ".."
        || is_internal_name(name)
    {
        return Err(FileError::InvalidName);
    }
    let result = if directory.is_empty() {
        name.to_owned()
    } else {
        format!("{directory}/{name}")
    };
    validate_relative(&result)?;
    Ok(result)
}
fn is_internal_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(TRASH) || name.to_ascii_lowercase().starts_with(TEMP_PREFIX)
}
fn is_protected(path: &str) -> bool {
    path.split('/')
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("p4desk"))
}
fn path_is_within_case_insensitive(candidate: &Path, parent: &Path) -> bool {
    let mut components = candidate.components();
    parent.components().all(|expected| {
        components.next().is_some_and(|actual| {
            expected
                .as_os_str()
                .to_string_lossy()
                .chars()
                .flat_map(char::to_uppercase)
                .eq(actual.as_os_str().to_string_lossy().chars().flat_map(char::to_uppercase))
        })
    })
}
fn require_unprotected(path: &str) -> Result<(), FileError> {
    validate_relative(path)?;
    if path.is_empty() || is_protected(path) {
        Err(FileError::Protected)
    } else {
        Ok(())
    }
}
fn epoch_seconds(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}
fn now_seconds() -> u64 {
    epoch_seconds(SystemTime::now()).unwrap_or(0)
}
fn unique_id() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    format!(
        "{time:016x}{:016x}",
        u64::from(NONCE.fetch_add(1, Ordering::Relaxed))
    )
}
fn reserve_unique_directory(root: &Path) -> Result<PathBuf, FileError> {
    for _ in 0..16 {
        let folder = root.join(unique_id());
        match fs::create_dir(&folder) {
            Ok(()) => return Ok(folder),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(FileError::AlreadyExists)
}
fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}
/// Convert entries to owned paths and close ReadDir before callers recurse.
/// DirEntry itself can retain its parent's directory handle on the host.
fn collect_children(
    directory: &Path,
    maximum: usize,
    allow_truncation: bool,
) -> Result<Vec<PathBuf>, FileError> {
    let mut paths = Vec::with_capacity(maximum.min(32));
    for item in fs::read_dir(directory)? {
        if paths.len() >= maximum {
            if allow_truncation {
                break;
            }
            return Err(FileError::TooManyEntries);
        }
        paths.push(item?.path());
    }
    Ok(paths)
}
fn remove_tree_checked(path: &Path, limits: Limits) -> Result<(), FileError> {
    remove_tree_inner(
        path,
        limits,
        0,
        &mut Budget {
            discovered: 1,
            ..Budget::default()
        },
    )
}
fn remove_tree_inner(
    path: &Path,
    limits: Limits,
    depth: usize,
    budget: &mut Budget,
) -> Result<(), FileError> {
    if depth > limits.recursive_depth {
        return Err(FileError::TooManyEntries);
    }
    budget.entries += 1;
    if budget.entries > limits.recursive_entries {
        return Err(FileError::TooManyEntries);
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(FileError::InvalidPath);
    }
    if metadata.is_dir() {
        let children = collect_children(
            path,
            limits.recursive_entries.saturating_sub(budget.discovered),
            false,
        )?;
        budget.discovered += children.len();
        for child in children {
            remove_tree_inner(&child, limits, depth + 1, budget)?;
        }
        fs::remove_dir(path)?;
    } else if metadata.is_file() {
        fs::remove_file(path)?;
    } else {
        return Err(FileError::Unsupported);
    }
    Ok(())
}
fn remove_owned_tree(path: &Path, limits: Limits) {
    let _ = remove_tree_checked(path, limits);
}
fn parse_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut result = [0u8; 32];
    for (index, value) in value.as_bytes().chunks_exact(2).enumerate() {
        fn hex(value: u8) -> Option<u8> {
            match value {
                b'0'..=b'9' => Some(value - b'0'),
                b'a'..=b'f' => Some(value - b'a' + 10),
                b'A'..=b'F' => Some(value - b'A' + 10),
                _ => None,
            }
        }
        result[index] = hex(value[0])? * 16 + hex(value[1])?;
    }
    Some(result)
}
