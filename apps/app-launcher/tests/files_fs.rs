use app_launcher::files::{
    Command, FileEngine, FileError, FileKind, Limits, Outcome, Preview, Request, SortOrder,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Card(PathBuf);
impl Card {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "p4desk-files-test-{id}-{}",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn engine(&self) -> FileEngine {
        FileEngine::new(&self.0)
    }
    fn write(&self, path: &str, bytes: &[u8]) {
        fs::write(self.0.join(path), bytes).unwrap();
    }
    fn folder(&self, path: &str) {
        fs::create_dir_all(self.0.join(path)).unwrap();
    }
}
impl Drop for Card {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(engine: &mut FileEngine, command: Command) -> Result<Outcome, FileError> {
    engine
        .execute(Request {
            revision: 9,
            command,
        })
        .result
}
fn list(
    engine: &mut FileEngine,
    directory: &str,
    query: &str,
    sort: SortOrder,
) -> app_launcher::files::DirectoryListing {
    match run(
        engine,
        Command::List {
            directory: directory.to_owned(),
            query: query.to_owned(),
            sort,
        },
    )
    .unwrap()
    {
        Outcome::Listing(listing) => listing,
        _ => panic!("listing required"),
    }
}
fn preview(engine: &mut FileEngine, path: &str) -> Result<Preview, FileError> {
    match run(
        engine,
        Command::Preview {
            path: path.to_owned(),
        },
    )? {
        Outcome::Preview(preview) => Ok(preview),
        _ => panic!("preview required"),
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn root_bounded_sorted_search_and_metadata() {
    let card = Card::new();
    card.folder("z-folder");
    card.write("b.txt", b"second");
    card.write("a.txt", b"a");
    card.folder("p4desk");
    card.write("p4desk/internal.json", b"private fixture");
    let mut engine = card.engine();
    let all = list(&mut engine, "", "", SortOrder::Name);
    assert_eq!(
        all.entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        vec!["p4desk", "z-folder", "a.txt", "b.txt"]
    );
    assert!(all.entries[0].read_only);
    assert_eq!(all.entries[2].kind, FileKind::Text);
    assert_eq!(all.entries[2].size, 1);
    assert!(all.entries[2].modified_seconds.is_some());
    assert_eq!(
        list(&mut engine, "", "B.T", SortOrder::Name).entries.len(),
        1
    );
    assert_eq!(
        list(&mut engine, "", "", SortOrder::Size).entries[2].name,
        "b.txt"
    );
    let mut limits = Limits::default();
    limits.directory_entries = 2;
    limits.directory_scan = 4;
    let mut engine = FileEngine::with_limits(&card.0, limits);
    let bounded = list(&mut engine, "", "", SortOrder::Name);
    assert!(bounded.truncated);
    assert_eq!(bounded.entries.len(), 2);
    assert!(bounded.scanned <= 4);
}

#[test]
fn listing_preserves_upload_name_contract_and_skips_unaddressable_entries() {
    let card = Card::new();
    card.folder("Downloads");
    let mut engine = card.engine();
    let bytes = b"temporary fixture";
    engine
        .begin_upload("Downloads/ leading.txt", bytes.len() as u64, &digest(bytes))
        .unwrap();
    engine.upload_chunk(0, bytes).unwrap();
    engine.commit_upload().unwrap();
    // A host fixture can contain a name that cannot be safely addressed on FAT.
    card.write("Downloads/invalid:name.txt", b"unchanged fixture");
    let listing = list(&mut engine, "Downloads", "", SortOrder::Name);
    assert!(listing.truncated);
    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].path, "Downloads/ leading.txt");
    assert!(matches!(
        preview(&mut engine, "Downloads/ leading.txt"),
        Ok(Preview::Text { .. })
    ));
    assert_eq!(
        fs::read(card.0.join("Downloads/invalid:name.txt")).unwrap(),
        b"unchanged fixture"
    );
}

#[test]
fn directory_transfer_rejects_case_insensitive_own_descendant() {
    let card = Card::new();
    card.folder("Files/Album/Sub");
    card.write("Files/Album/keep.txt", b"unchanged fixture");
    let mut engine = card.engine();
    for command in [
        Command::Copy { source: "Files/Album".into(), destination: "files/ALBUM/Sub".into() },
        Command::Move { source: "Files/Album".into(), destination: "FILES/album/Sub".into() },
    ] {
        assert_eq!(run(&mut engine, command), Err(FileError::InvalidPath));
    }
    assert_eq!(fs::read(card.0.join("Files/Album/keep.txt")).unwrap(), b"unchanged fixture");
}

#[test]
fn directories_are_created_without_overwriting_users() {
    let card = Card::new();
    card.folder("Documents");
    card.write("Documents/keep.txt", b"keep");
    let mut engine = card.engine();
    engine.ensure_user_directories().unwrap();
    engine.ensure_user_directories().unwrap();
    for name in ["Files", "Documents", "Pictures", "Downloads"] {
        assert!(card.0.join(name).is_dir());
    }
    assert_eq!(
        fs::read(card.0.join("Documents/keep.txt")).unwrap(),
        b"keep"
    );
    let other = Card::new();
    other.write("Pictures", b"user file");
    assert_eq!(
        other.engine().ensure_user_directories(),
        Err(FileError::AlreadyExists)
    );
}

#[test]
fn no_absolute_parent_hidden_fat_invalid_or_symlink_paths() {
    let card = Card::new();
    card.folder("allowed");
    let mut engine = card.engine();
    for directory in [
        "/tmp",
        "..",
        "allowed/../",
        "allowed/.",
        "allowed//x",
        ".p4desk-trash",
        ".P4DESK-filepart-abc",
        "a\\b",
        "a:b",
        "a?b",
        "tail.",
        "tail ",
    ] {
        assert!(run(
            &mut engine,
            Command::List {
                directory: directory.to_owned(),
                query: String::new(),
                sort: SortOrder::Name
            }
        )
        .is_err());
    }
    #[cfg(unix)]
    {
        let outside = Card::new();
        outside.write("keep.txt", b"outside");
        std::os::unix::fs::symlink(&outside.0, card.0.join("link")).unwrap();
        assert_eq!(
            preview(&mut engine, "link/keep.txt"),
            Err(FileError::InvalidPath)
        );
        assert!(!list(&mut engine, "", "", SortOrder::Name)
            .entries
            .iter()
            .any(|entry| entry.name == "link"));
        assert_eq!(fs::read(outside.0.join("keep.txt")).unwrap(), b"outside");
    }
}

#[test]
fn protected_tree_cannot_be_modified_previewed_or_copied() {
    let card = Card::new();
    card.folder("P4Desk");
    card.folder("target");
    card.write("P4Desk/config.json", b"private fixture");
    let mut engine = card.engine();
    assert!(list(&mut engine, "P4Desk", "", SortOrder::Name).entries[0].read_only);
    assert_eq!(
        preview(&mut engine, "P4Desk/config.json"),
        Err(FileError::Protected)
    );
    for command in [
        Command::Rename {
            path: "P4Desk/config.json".into(),
            name: "changed".into(),
        },
        Command::Copy {
            source: "P4Desk".into(),
            destination: "target".into(),
        },
        Command::Move {
            source: "P4Desk/config.json".into(),
            destination: "target".into(),
        },
        Command::Trash {
            path: "P4Desk/config.json".into(),
        },
        Command::CreateFolder {
            directory: "P4Desk".into(),
            name: "new".into(),
        },
    ] {
        assert_eq!(run(&mut engine, command), Err(FileError::Protected));
    }
    assert_eq!(
        engine.begin_upload("p4desk/new.txt", 0, &digest(b"")),
        Err(FileError::Protected)
    );
}

#[test]
fn create_rename_copy_move_no_overwrite_and_recursive_safety() {
    let card = Card::new();
    card.folder("source/nested");
    card.folder("target");
    card.write("source/nested/read.txt", b"payload");
    let mut engine = card.engine();
    run(
        &mut engine,
        Command::CreateFolder {
            directory: String::new(),
            name: "new".into(),
        },
    )
    .unwrap();
    run(
        &mut engine,
        Command::Rename {
            path: "new".into(),
            name: "renamed".into(),
        },
    )
    .unwrap();
    assert_eq!(
        run(
            &mut engine,
            Command::Rename {
                path: "renamed".into(),
                name: "target".into()
            }
        ),
        Err(FileError::AlreadyExists)
    );
    assert_eq!(
        run(
            &mut engine,
            Command::Rename {
                path: "renamed".into(),
                name: "TARGET".into()
            }
        ),
        Err(FileError::AlreadyExists)
    );
    run(
        &mut engine,
        Command::Copy {
            source: "source".into(),
            destination: "target".into(),
        },
    )
    .unwrap();
    assert_eq!(
        fs::read(card.0.join("target/source/nested/read.txt")).unwrap(),
        b"payload"
    );
    assert_eq!(
        run(
            &mut engine,
            Command::Copy {
                source: "source".into(),
                destination: "target".into()
            }
        ),
        Err(FileError::AlreadyExists)
    );
    assert_eq!(
        run(
            &mut engine,
            Command::Move {
                source: "source".into(),
                destination: "source/nested".into()
            }
        ),
        Err(FileError::InvalidPath)
    );
    run(
        &mut engine,
        Command::Move {
            source: "source/nested/read.txt".into(),
            destination: String::new(),
        },
    )
    .unwrap();
    assert!(!card.0.join("source/nested/read.txt").exists());
    assert_eq!(fs::read(card.0.join("read.txt")).unwrap(), b"payload");
    let mut limits = Limits::default();
    limits.copy_bytes = 2;
    let mut bounded = FileEngine::with_limits(&card.0, limits);
    assert_eq!(
        run(
            &mut bounded,
            Command::Copy {
                source: "read.txt".into(),
                destination: "renamed".into()
            }
        ),
        Err(FileError::TooLarge)
    );
    assert!(!card.0.join("renamed/read.txt").exists());
    assert!(card.0.join("read.txt").exists());
}

#[test]
fn trash_is_hidden_persistent_restore_is_no_overwrite_and_delete_explicit() {
    let card = Card::new();
    card.write("note.txt", b"safe data");
    let mut engine = card.engine();
    run(
        &mut engine,
        Command::Trash {
            path: "note.txt".into(),
        },
    )
    .unwrap();
    assert!(!card.0.join("note.txt").exists());
    assert!(list(&mut engine, "", "", SortOrder::Name)
        .entries
        .is_empty());
    drop(engine);
    let mut engine = card.engine();
    let entries = match run(&mut engine, Command::ListTrash).unwrap() {
        Outcome::TrashListing(entries) => entries,
        _ => panic!("trash required"),
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].original_path, "note.txt");
    card.write("note.txt", b"new data");
    assert_eq!(
        run(
            &mut engine,
            Command::Restore {
                id: entries[0].id.clone()
            }
        ),
        Err(FileError::AlreadyExists)
    );
    assert_eq!(fs::read(card.0.join("note.txt")).unwrap(), b"new data");
    fs::remove_file(card.0.join("note.txt")).unwrap();
    run(
        &mut engine,
        Command::Restore {
            id: entries[0].id.clone(),
        },
    )
    .unwrap();
    assert_eq!(fs::read(card.0.join("note.txt")).unwrap(), b"safe data");
    run(
        &mut engine,
        Command::Trash {
            path: "note.txt".into(),
        },
    )
    .unwrap();
    let entries = match run(&mut engine, Command::ListTrash).unwrap() {
        Outcome::TrashListing(entries) => entries,
        _ => panic!("trash required"),
    };
    run(
        &mut engine,
        Command::DeletePermanently {
            id: entries[0].id.clone(),
        },
    )
    .unwrap();
    assert!(
        matches!(run(&mut engine, Command::ListTrash).unwrap(), Outcome::TrashListing(entries) if entries.is_empty())
    );
    assert_eq!(
        run(
            &mut engine,
            Command::DeletePermanently {
                id: "../note.txt".into()
            }
        ),
        Err(FileError::InvalidPath)
    );
}

#[test]
fn text_preview_utf8_bom_binary_rejection_and_bounded_multibyte_end() {
    let card = Card::new();
    card.write("note.txt", "\u{feff}你好 P4".as_bytes());
    card.write("binary.txt", &[0, 1]);
    card.write("bad.txt", &[0xff]);
    card.write("large.txt", "你好世界".as_bytes());
    let mut engine = card.engine();
    assert!(
        matches!(preview(&mut engine, "note.txt").unwrap(), Preview::Text { text, truncated: false } if &*text == "你好 P4")
    );
    assert_eq!(
        preview(&mut engine, "binary.txt"),
        Err(FileError::InvalidText)
    );
    assert_eq!(preview(&mut engine, "bad.txt"), Err(FileError::InvalidText));
    let mut limits = Limits::default();
    limits.text_bytes = 8;
    let mut engine = FileEngine::with_limits(&card.0, limits);
    assert!(
        matches!(preview(&mut engine, "large.txt").unwrap(), Preview::Text { text, truncated: true } if &*text == "你好")
    );
}

#[test]
fn png_rgba_palette_bmp_top_down_and_jpeg_rgb565_previews() {
    let card = Card::new();
    {
        let file = fs::File::create(card.0.join("image.png")).unwrap();
        let mut encoder = png::Encoder::new(file, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 0, 0, 255, 0, 0, 255, 255])
            .unwrap();
    }
    {
        let file = fs::File::create(card.0.join("palette.png")).unwrap();
        let mut encoder = png::Encoder::new(file, 2, 1);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(vec![255, 0, 0, 0, 0, 255]);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0, 1])
            .unwrap();
    }
    write_bmp(&card.0.join("image.bmp"), false);
    write_bmp(&card.0.join("top.bmp"), true);
    let jpeg = hex(JPEG_HEX);
    card.write("image.jpg", &jpeg);
    let mut engine = card.engine();
    for name in [
        "image.png",
        "palette.png",
        "image.bmp",
        "top.bmp",
        "image.jpg",
    ] {
        let Preview::Image {
            width,
            height,
            pixels,
            original_width,
            original_height,
        } = preview(&mut engine, name).unwrap()
        else {
            panic!("image required")
        };
        assert_eq!(
            (width, height, original_width, original_height),
            (2, 1, 2, 1)
        );
        assert!(pixels[0] & 0xf800 >= 0xf000);
        assert!(pixels[0] & 0x001f <= 1);
        assert!(pixels[1] & 0x001f >= 30);
        assert!(pixels[1] & 0xf800 <= 0x0800);
    }
    card.write("invalid.png", b"damaged");
    assert_eq!(
        preview(&mut engine, "invalid.png"),
        Err(FileError::InvalidImage)
    );
    let mut limits = Limits::default();
    limits.image_pixels = 1;
    assert_eq!(
        preview(&mut FileEngine::with_limits(&card.0, limits), "image.png"),
        Err(FileError::TooLarge)
    );
}

#[test]
fn upload_streaming_offset_digest_abort_drop_and_no_overwrite() {
    let card = Card::new();
    card.folder("Downloads");
    let data = vec![42u8; 70_003];
    let mut engine = card.engine();
    engine
        .begin_upload("Downloads/new.bin", data.len() as u64, &digest(&data))
        .unwrap();
    assert!(!card.0.join("Downloads/new.bin").exists());
    assert_eq!(
        engine.begin_upload("Downloads/other.bin", 0, &digest(b"")),
        Err(FileError::Busy)
    );
    assert_eq!(engine.upload_chunk(1, &data[..32]), Err(FileError::Offset));
    assert_eq!(
        run(
            &mut engine,
            Command::CreateFolder {
                directory: "Downloads".into(),
                name: "folder".into()
            }
        ),
        Err(FileError::Busy)
    );
    let mut offset = 0u32;
    for chunk in data.chunks(32 * 1024) {
        engine.upload_chunk(offset, chunk).unwrap();
        offset += chunk.len() as u32;
    }
    assert_eq!(engine.commit_upload().unwrap(), "Downloads");
    assert_eq!(fs::read(card.0.join("Downloads/new.bin")).unwrap(), data);
    assert_eq!(
        engine.begin_upload("Downloads/new.bin", 0, &digest(b"")),
        Err(FileError::AlreadyExists)
    );
    engine
        .begin_upload("Downloads/bad.bin", 1, &digest(b"wrong"))
        .unwrap();
    engine.upload_chunk(0, b"x").unwrap();
    assert_eq!(engine.commit_upload(), Err(FileError::Integrity));
    assert!(!card.0.join("Downloads/bad.bin").exists());
    engine
        .begin_upload("Downloads/incomplete.bin", 3, &digest(b"123"))
        .unwrap();
    engine.upload_chunk(0, b"1").unwrap();
    assert_eq!(engine.commit_upload(), Err(FileError::Integrity));
    engine
        .begin_upload("Downloads/abort.bin", 3, &digest(b"123"))
        .unwrap();
    engine.abort_upload();
    engine
        .begin_upload("Downloads/drop.bin", 3, &digest(b"123"))
        .unwrap();
    drop(engine);
    let names = fs::read_dir(card.0.join("Downloads"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 1);
    assert_eq!(
        card.engine()
            .begin_upload("big.bin", 256 * 1024 * 1024 + 1, &digest(b"")),
        Err(FileError::TooLarge)
    );
}

#[test]
fn upload_preserves_late_conflict_and_cleanup_requires_valid_marker() {
    let card = Card::new();
    let id = "00112233445566778899001122334455";
    card.write(&format!(".p4desk-filepart-{id}.part"), b"abandoned");
    card.write(
        &format!(".p4desk-filepart-{id}.json"),
        format!("{{\"version\":1,\"id\":\"{id}\"}}").as_bytes(),
    );
    card.write(
        ".p4desk-filepart-ffffffffffffffffffffffffffffffff.part",
        b"no marker",
    );
    let mut engine = card.engine();
    assert!(list(&mut engine, "", "", SortOrder::Name)
        .entries
        .is_empty());
    assert!(!card.0.join(format!(".p4desk-filepart-{id}.part")).exists());
    assert!(card
        .0
        .join(".p4desk-filepart-ffffffffffffffffffffffffffffffff.part")
        .exists());
    engine.begin_upload("new.bin", 3, &digest(b"123")).unwrap();
    engine.upload_chunk(0, b"123").unwrap();
    card.write("new.bin", b"someone else");
    assert_eq!(engine.commit_upload(), Err(FileError::AlreadyExists));
    assert_eq!(fs::read(card.0.join("new.bin")).unwrap(), b"someone else");
}

#[test]
fn upload_commit_detects_corruption_of_persisted_bytes() {
    let card = Card::new();
    let mut engine = card.engine();
    engine.begin_upload("new.bin", 3, &digest(b"123")).unwrap();
    engine.upload_chunk(0, b"123").unwrap();
    let part = fs::read_dir(&card.0)
        .unwrap()
        .map(|item| item.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "part")
        })
        .unwrap();
    fs::write(part, b"321").unwrap();
    assert_eq!(engine.commit_upload(), Err(FileError::Integrity));
    assert!(!card.0.join("new.bin").exists());
    assert_eq!(fs::read_dir(&card.0).unwrap().count(), 0);
    engine.begin_upload("empty.bin", 0, &digest(b"")).unwrap();
    assert_eq!(engine.commit_upload().unwrap(), "");
    assert_eq!(fs::metadata(card.0.join("empty.bin")).unwrap().len(), 0);
}

#[test]
fn readable_user_file_rejects_system_directories_and_escape() {
    let card = Card::new();
    card.folder("Fonts");
    card.folder("p4desk");
    card.write("Fonts/example.p4f", b"fixture");
    card.write("p4desk/example.p4f", b"private fixture");
    let mut engine = card.engine();
    assert!(engine
        .readable_user_file("Fonts/example.p4f")
        .unwrap()
        .is_file());
    assert_eq!(
        engine.readable_user_file("p4desk/example.p4f"),
        Err(FileError::Protected)
    );
    assert_eq!(
        engine.readable_user_file("Fonts"),
        Err(FileError::Unsupported)
    );
    assert_eq!(
        engine.readable_user_file("../example.p4f"),
        Err(FileError::InvalidPath)
    );
}

#[test]
fn upload_commit_cancellation_cleans_partial_and_keeps_destination_absent() {
    let card = Card::new();
    let data = vec![9u8; 20_000];
    let mut engine = card.engine();
    engine
        .begin_upload("new.bin", data.len() as u64, &digest(&data))
        .unwrap();
    engine.upload_chunk(0, &data).unwrap();
    let mut checks = 0;
    assert_eq!(
        engine.commit_upload_if(|| {
            checks += 1;
            checks < 4
        }),
        Err(FileError::Cancelled)
    );
    assert!(!card.0.join("new.bin").exists());
    assert_eq!(fs::read_dir(&card.0).unwrap().count(), 0);
}

#[test]
fn deep_copy_trash_restore_delete_and_marker_cleanup_are_bounded() {
    let card = Card::new();
    card.folder("target");
    let mut deepest = "source".to_owned();
    for level in 0..11 {
        deepest.push_str(&format!("/d{level}"));
    }
    card.folder(&deepest);
    card.write(&format!("{deepest}/leaf.txt"), b"deep fixture");
    let id = "00112233445566778899001122334455";
    card.write(
        &format!("{deepest}/.p4desk-filepart-{id}.part"),
        b"interrupted fixture",
    );
    card.write(
        &format!("{deepest}/.p4desk-filepart-{id}.json"),
        format!("{{\"version\":1,\"id\":\"{id}\"}}").as_bytes(),
    );
    let mut engine = card.engine();
    run(
        &mut engine,
        Command::Copy {
            source: "source".into(),
            destination: "target".into(),
        },
    )
    .unwrap();
    assert_eq!(
        fs::read(card.0.join(format!("target/{deepest}/leaf.txt"))).unwrap(),
        b"deep fixture"
    );
    assert!(!card
        .0
        .join(format!("{deepest}/.p4desk-filepart-{id}.part"))
        .exists());
    run(
        &mut engine,
        Command::Trash {
            path: "source".into(),
        },
    )
    .unwrap();
    let entries = match run(&mut engine, Command::ListTrash).unwrap() {
        Outcome::TrashListing(entries) => entries,
        _ => panic!("trash required"),
    };
    run(
        &mut engine,
        Command::Restore {
            id: entries[0].id.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        fs::read(card.0.join(format!("{deepest}/leaf.txt"))).unwrap(),
        b"deep fixture"
    );
    run(
        &mut engine,
        Command::Trash {
            path: "source".into(),
        },
    )
    .unwrap();
    let entries = match run(&mut engine, Command::ListTrash).unwrap() {
        Outcome::TrashListing(entries) => entries,
        _ => panic!("trash required"),
    };
    run(
        &mut engine,
        Command::DeletePermanently {
            id: entries[0].id.clone(),
        },
    )
    .unwrap();
    assert!(
        matches!(run(&mut engine, Command::ListTrash).unwrap(), Outcome::TrashListing(entries) if entries.is_empty())
    );
    let mut limits = Limits::default();
    limits.recursive_entries = 5;
    card.folder("bounded");
    let mut bounded = FileEngine::with_limits(&card.0, limits);
    assert_eq!(
        run(
            &mut bounded,
            Command::Copy {
                source: "target".into(),
                destination: "bounded".into()
            }
        ),
        Err(FileError::TooManyEntries)
    );
    assert!(card.0.join(format!("target/{deepest}/leaf.txt")).exists());
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn deep_operations_preserve_twelve_handle_limit() {
    const FLAG: &str = "P4DESK_FILES_FD_LIMIT_CHILD";
    if std::env::var_os(FLAG).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "deep_operations_preserve_twelve_handle_limit",
                "--test-threads=1",
            ])
            .env(FLAG, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated descriptor test failed: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }
    // Constrain only the isolated child. Keep the parent test process's limits unchanged.
    #[repr(C)]
    struct RLimit {
        current: u64,
        maximum: u64,
    }
    unsafe extern "C" {
        fn getrlimit(resource: i32, limit: *mut RLimit) -> i32;
        fn setrlimit(resource: i32, limit: *const RLimit) -> i32;
    }
    #[cfg(target_os = "macos")]
    const RESOURCE: i32 = 8;
    #[cfg(target_os = "linux")]
    const RESOURCE: i32 = 7;
    let mut limit = RLimit {
        current: 0,
        maximum: 0,
    };
    assert_eq!(unsafe { getrlimit(RESOURCE, &mut limit) }, 0);
    limit.current = 12;
    assert_eq!(unsafe { setrlimit(RESOURCE, &limit) }, 0);
    let card = Card::new();
    card.folder("target");
    let mut deepest = "source".to_owned();
    for level in 0..11 {
        deepest.push_str(&format!("/d{level}"));
    }
    card.folder(&deepest);
    card.write(&format!("{deepest}/leaf.txt"), b"deep fixture");
    let mut held = Vec::new();
    for index in 0..6 {
        let name = format!("held{index}.bin");
        card.write(&name, b"fixture");
        held.push(fs::File::open(card.0.join(name)).unwrap());
    }
    let mut engine = card.engine();
    run(
        &mut engine,
        Command::Copy {
            source: "source".into(),
            destination: "target".into(),
        },
    )
    .unwrap();
    run(
        &mut engine,
        Command::Trash {
            path: "source".into(),
        },
    )
    .unwrap();
    let entries = match run(&mut engine, Command::ListTrash).unwrap() {
        Outcome::TrashListing(entries) => entries,
        _ => panic!("trash required"),
    };
    run(
        &mut engine,
        Command::DeletePermanently {
            id: entries[0].id.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        fs::read(card.0.join(format!("target/{deepest}/leaf.txt"))).unwrap(),
        b"deep fixture"
    );
    drop(held);
}

fn write_bmp(path: &Path, top_down: bool) {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&62u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&(if top_down { -1i32 } else { 1 }).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&8u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    bytes.extend_from_slice(&[0, 0, 255, 255, 0, 0, 0, 0]);
    fs::write(path, bytes).unwrap();
}
fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|value| u8::from_str_radix(std::str::from_utf8(value).unwrap(), 16).unwrap())
        .collect()
}
// Self-generated 2×1 red/blue JPEG, no external image or private metadata.
const JPEG_HEX: &str = "ffd8ffe000104a46494600010100004800480000ffe1004c4578696600004d4d002a00000008000187690004000000010000001a000000000003a00100030000000100010000a00200040000000100000002a0030004000000010000000100000000ffed003850686f746f73686f7020332e30003842494d04040000000000003842494d0425000000000010d41d8cd98f00b204e9800998ecf8427effc00011080001000203011100021101031101ffc4001f0000010501010101010100000000000000000102030405060708090a0bffc400b5100002010303020403050504040000017d01020300041105122131410613516107227114328191a1082342b1c11552d1f02433627282090a161718191a25262728292a3435363738393a434445464748494a535455565758595a636465666768696a737475767778797a838485868788898a92939495969798999aa2a3a4a5a6a7a8a9aab2b3b4b5b6b7b8b9bac2c3c4c5c6c7c8c9cad2d3d4d5d6d7d8d9dae1e2e3e4e5e6e7e8e9eaf1f2f3f4f5f6f7f8f9faffc4001f0100030101010101010101010000000000000102030405060708090a0bffc400b51100020102040403040705040400010277000102031104052131061241510761711322328108144291a1b1c109233352f0156272d10a162434e125f11718191a262728292a35363738393a434445464748494a535455565758595a636465666768696a737475767778797a82838485868788898a92939495969798999aa2a3a4a5a6a7a8a9aab2b3b4b5b6b7b8b9bac2c3c4c5c6c7c8c9cad2d3d4d5d6d7d8d9dae2e3e4e5e6e7e8e9eaf2f3f4f5f6f7f8f9faffdb00430001010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101ffdb00430101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101ffdd00040001ffda000c03010002110311003f00fe45be20ff00c8fbe37ffb1bfc4bff00a79bdaff00b4efa09ffca10fd0dffed157e8f3ff00ae8f840f2fe99bff002983f4aeff00b494f1d3ff005e87149fffd9";
