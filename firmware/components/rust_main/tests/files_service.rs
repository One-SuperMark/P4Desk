use p4desk_protocol::DeviceMessage;
use rust_main::files::{Completion, HostJob, Service};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Card(PathBuf);
impl Card {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "p4desk-files-service-{id}-{}",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Card {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn completion(service: &Service, id: u16) -> (DeviceMessage, bool, bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for result in service.poll() {
            if let Completion::Host {
                id: found,
                epoch,
                message,
                clear_upload,
                changed,
            } = result
            {
                if found == id && epoch == service.epoch() {
                    return (message, clear_upload, changed);
                }
            }
        }
        assert!(Instant::now() < deadline, "worker completion timed out");
        thread::sleep(Duration::from_millis(2));
    }
}
fn ack(result: &(DeviceMessage, bool, bool), ok: bool, error: Option<&str>) {
    let DeviceMessage::Ack {
        ok: actual,
        error: actual_error,
        ..
    } = &result.0
    else {
        panic!("ACK required")
    };
    assert_eq!(*actual, ok);
    assert_eq!(actual_error.as_deref(), error);
}
fn begin(service: &Service, id: u16, path: &str, bytes: &[u8]) {
    service
        .host(
            id,
            HostJob::Begin {
                path: path.into(),
                length: bytes.len() as u64,
                sha256: hash(bytes),
            },
            true,
        )
        .unwrap();
    let result = completion(service, id);
    ack(&result, true, None);
    assert!(!result.1 && !result.2);
}

#[test]
fn acknowledged_commit_is_closed_persisted_verified_and_visible() {
    let card = Card::new();
    let service = Service::new(card.0.clone()).unwrap();
    let bytes = vec![17u8; 32_001];
    begin(&service, 1, "Downloads/example.bin", &bytes);
    assert!(!card.0.join("Downloads/example.bin").exists());
    service
        .host(
            2,
            HostJob::Chunk {
                offset: 0,
                bytes: bytes[..32_000].to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 2), true, None);
    assert!(!card.0.join("Downloads/example.bin").exists());
    service
        .host(
            3,
            HostJob::Chunk {
                offset: 32_000,
                bytes: bytes[32_000..].to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 3), true, None);
    service.host(4, HostJob::Commit, true).unwrap();
    let result = completion(&service, 4);
    ack(&result, true, None);
    assert!(result.1 && result.2);
    assert_eq!(
        fs::read(card.0.join("Downloads/example.bin")).unwrap(),
        bytes
    );
    assert_eq!(fs::read_dir(card.0.join("Downloads")).unwrap().count(), 1);
}

#[test]
fn bad_offset_and_digest_abort_without_publishing() {
    let card = Card::new();
    let service = Service::new(card.0.clone()).unwrap();
    begin(&service, 1, "Downloads/bad.bin", b"123");
    service
        .host(
            2,
            HostJob::Chunk {
                offset: 1,
                bytes: b"123".to_vec(),
            },
            true,
        )
        .unwrap();
    let result = completion(&service, 2);
    ack(&result, false, Some("offset"));
    assert!(result.1);
    service.host(3, HostJob::Commit, true).unwrap();
    ack(&completion(&service, 3), false, Some("offset"));
    begin(&service, 4, "Downloads/bad.bin", b"123");
    service
        .host(
            5,
            HostJob::Chunk {
                offset: 0,
                bytes: b"321".to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 5), true, None);
    service.host(6, HostJob::Commit, true).unwrap();
    let result = completion(&service, 6);
    ack(&result, false, Some("integrity"));
    assert!(result.1);
    assert!(!card.0.join("Downloads/bad.bin").exists());
    assert_eq!(fs::read_dir(card.0.join("Downloads")).unwrap().count(), 0);
}

#[test]
fn disconnect_changes_epoch_cleans_partial_and_allows_new_connection() {
    let card = Card::new();
    let service = Service::new(card.0.clone()).unwrap();
    begin(&service, 1, "Downloads/old.bin", b"123");
    service
        .host(
            2,
            HostJob::Chunk {
                offset: 0,
                bytes: b"1".to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 2), true, None);
    let before = service.epoch();
    service.disconnect();
    assert_ne!(service.epoch(), before);
    // Reset has priority over this new connection's BEGIN.
    begin(&service, 3, "Downloads/new.bin", b"456");
    assert!(!card.0.join("Downloads/old.bin").exists());
    service
        .host(
            4,
            HostJob::Chunk {
                offset: 0,
                bytes: b"456".to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 4), true, None);
    service.host(5, HostJob::Commit, true).unwrap();
    ack(&completion(&service, 5), true, None);
    assert_eq!(fs::read(card.0.join("Downloads/new.bin")).unwrap(), b"456");
    assert_eq!(fs::read_dir(card.0.join("Downloads")).unwrap().count(), 1);
}

#[test]
fn service_backpressure_is_bounded_when_replies_are_not_consumed() {
    let card = Card::new();
    let service = Service::new(card.0.clone()).unwrap();
    let mut accepted = 0;
    for id in 1..=100 {
        let result = service.host(
            id,
            HostJob::List {
                path: String::new(),
                offset: 0,
                limit: 32,
            },
            true,
        );
        if result.is_err() {
            assert_eq!(result, Err(app_launcher::files::FileError::Busy));
            break;
        }
        accepted += 1;
    }
    assert!(accepted > 0 && accepted <= 17);
}

#[test]
fn abort_invalidates_a_queued_commit_before_it_can_publish() {
    let card = Card::new();
    let service = Service::new(card.0.clone()).unwrap();
    begin(&service, 1, "Downloads/cancelled.bin", b"123");
    service
        .host(
            2,
            HostJob::Chunk {
                offset: 0,
                bytes: b"123".to_vec(),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 2), true, None);
    // Fill the completion queue to park the worker before queuing COMMIT and ABORT.
    for id in 10..18 {
        service
            .host(
                id,
                HostJob::List {
                    path: String::new(),
                    offset: 0,
                    limit: 32,
                },
                true,
            )
            .unwrap();
    }
    thread::sleep(Duration::from_millis(100));
    service.host(3, HostJob::Commit, true).unwrap();
    service.host(4, HostJob::Abort, true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut commit = None;
    let mut abort = None;
    while commit.is_none() || abort.is_none() {
        for result in service.poll() {
            if let Completion::Host {
                id,
                epoch,
                message,
                clear_upload,
                changed,
            } = result
            {
                if epoch != service.epoch() {
                    continue;
                }
                if id == 3 {
                    commit = Some((message, clear_upload, changed));
                } else if id == 4 {
                    abort = Some((message, clear_upload, changed));
                }
            }
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    ack(&commit.unwrap(), false, Some("cancelled"));
    ack(&abort.unwrap(), true, None);
    assert!(!card.0.join("Downloads/cancelled.bin").exists());
    assert_eq!(fs::read_dir(card.0.join("Downloads")).unwrap().count(), 0);
}

#[test]
fn font_header_bound_is_checked_before_index_allocation() {
    let card = Card::new();
    fs::create_dir(card.0.join("Fonts")).unwrap();
    let mut header = [0u8; 32];
    header[..4].copy_from_slice(b"P4F1");
    header[4..6].copy_from_slice(&1u16.to_le_bytes());
    header[6..8].copy_from_slice(&32u16.to_le_bytes());
    header[8..12].copy_from_slice(&16_385u32.to_le_bytes());
    header[12..16].copy_from_slice(&32u32.to_le_bytes());
    let length = 32 + 16_385 * 32;
    header[16..20].copy_from_slice(&(length as u32).to_le_bytes());
    header[20..24].copy_from_slice(&(length as u32).to_le_bytes());
    let mut bytes = vec![0u8; length];
    bytes[..32].copy_from_slice(&header);
    fs::write(card.0.join("Fonts/large.p4f"), &bytes).unwrap();
    let service = Service::new(card.0.clone()).unwrap();
    service
        .host(
            1,
            HostJob::FontInstall {
                path: "Fonts/large.p4f".into(),
                sha256: hash(&bytes),
            },
            true,
        )
        .unwrap();
    ack(&completion(&service, 1), false, Some("too_large"));
    assert!(!card.0.join("p4desk/files-font.json").exists());
}
