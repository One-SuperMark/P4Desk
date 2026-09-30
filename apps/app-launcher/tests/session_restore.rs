use app_launcher::{session::Session, ActiveApp, GenerationStore, LauncherState};
use calculator::{CalcMode, UnaryOp};
use p4desk_protocol::Note;
use std::fs;

fn root() -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "p4-session-test-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn reboot(state: &LauncherState) -> LauncherState {
    let saved = Session::capture(state);
    assert!(saved.valid());
    let bytes = serde_json::to_vec(&saved).unwrap();
    let saved: Session = serde_json::from_slice(&bytes).unwrap();
    let mut next = LauncherState::new();
    next.snapshot = state.snapshot.clone();
    assert!(saved.restore(&mut next));
    next
}
#[test]
fn timers_restore_remaining_paused_without_finishing_or_counting_downtime() {
    let mut s = LauncherState::new();
    s.timer.set_countdown_seconds(10);
    s.timer.toggle(100);
    s.timer.tick(3600);
    let mut restored = reboot(&s);
    assert_eq!(restored.timer.remaining_ms, 6500);
    assert!(!restored.timer.is_running());
    restored.tick(99_000_000, 0);
    assert_eq!(restored.timer.remaining_ms, 6500);
    restored.timer.toggle(99_000_000);
    restored.timer.tick(99_006_500);
    assert!(restored.timer.finished);
    let mut completed = reboot(&restored);
    assert!(completed.timer.finished && completed.timer.finished_at_ms().is_none());
    completed.tick(1, 0);
    assert!(completed.timer_completion.progress(1).is_none());
}
#[test]
fn unfinished_scientific_expression_and_memory_continue_after_reboot() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    if let ActiveApp::Calculator(calc) = &s.active_app {
        let mut c = calc.lock().unwrap();
        c.set_mode(CalcMode::Scientific);
        c.memory = 42.5;
        c.input_digit('2');
        c.input_operator('+');
        c.function(UnaryOp::Sqrt);
        c.open_parenthesis();
        c.input_digit('9');
    }
    let restored = reboot(&s);
    let ActiveApp::Calculator(calc) = restored.active_app else {
        panic!("calculator lost")
    };
    let mut c = calc.lock().unwrap();
    assert_eq!(c.mode, CalcMode::Scientific);
    assert_eq!(c.memory, 42.5);
    c.close_parenthesis();
    c.calculate();
    assert_eq!(c.value().unwrap(), 5.0);
    c.calculate();
    assert_eq!(c.value().unwrap(), 8.0);
}
#[test]
fn programmer_keeps_all_64_bits_and_radix() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    if let ActiveApp::Calculator(calc) = &s.active_app {
        let mut c = calc.lock().unwrap();
        c.mode = CalcMode::Programmer;
        c.programmer.set_value(u64::MAX);
        c.programmer.set_radix(16);
    }
    let restored = reboot(&s);
    let ActiveApp::Calculator(calc) = restored.active_app else {
        panic!()
    };
    assert_eq!(calc.lock().unwrap().primary_display(), "FFFFFFFFFFFFFFFF");
}
#[test]
fn closed_app_history_survives_but_closed_instance_does_not_return() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    s.kill_active_app();
    s.open_app("clock");
    s.open_app("timer");
    s.open_app("notes");
    s.open_app("settings");
    s.open_app("mac");
    let restored = reboot(&s);
    assert_eq!(restored.active_app.id(), Some("mac"));
    assert_eq!(
        restored.background_app_ids().collect::<Vec<_>>(),
        vec!["clock", "timer", "notes", "settings"]
    );
    assert_eq!(
        restored.recent_app_ids().collect::<Vec<_>>(),
        s.recent_app_ids().collect::<Vec<_>>()
    );
    assert!(!restored.running_apps.contains_key("calculator"));
    let mut s = LauncherState::new();
    s.open_app("calculator");
    s.kill_active_app();
    let restored = reboot(&s);
    assert_eq!(
        restored.recent_app_ids().collect::<Vec<_>>(),
        vec!["calculator"]
    );
    assert!(restored.running_apps.is_empty());
}
#[test]
fn notes_restore_by_stable_id_and_drop_deleted_selection() {
    let mut s = LauncherState::new();
    s.snapshot.notes = vec![
        Note {
            id: "a".into(),
            title: "".into(),
            body: "".into(),
            updated_ms: 0,
        },
        Note {
            id: "b".into(),
            title: "".into(),
            body: "".into(),
            updated_ms: 0,
        },
    ];
    s.open_app("notes");
    if let ActiveApp::Notes(v) = &s.active_app {
        let mut v = v.lock().unwrap();
        v.selected = 1;
        v.scroll.set_offset(80.0);
        v.confirm_delete = true;
    }
    let saved = Session::capture(&s);
    s.snapshot.notes.reverse();
    saved.restore(&mut s);
    if let ActiveApp::Notes(v) = &s.active_app {
        let v = v.lock().unwrap();
        assert_eq!(v.selected, 0);
        assert_eq!(v.scroll.offset(), 80.0);
        assert!(!v.confirm_delete);
    }
    s.snapshot.notes.clear();
    saved.restore(&mut s);
    if let ActiveApp::Notes(v) = &s.active_app {
        assert_eq!(v.lock().unwrap().scroll.offset(), 0.0);
    }
}
#[test]
fn usb_display_never_restarts_a_host_session_on_reboot() {
    let mut s = LauncherState::new();
    s.open_app("clock");
    s.open_app("display");
    let mut restored = reboot(&s);
    assert!(matches!(restored.active_app, ActiveApp::Launcher));
    assert!(restored.take_commands().is_empty());
    assert_eq!(
        restored.recent_app_ids().collect::<Vec<_>>(),
        vec!["clock", "display"]
    );
}
#[test]
fn journal_recovers_interrupted_write_and_rejects_semantically_invalid_newer_slot() {
    let root = root();
    let store = GenerationStore::new(root.join("sd"), root.join("flash")).session_store();
    let mut s = LauncherState::new();
    s.open_app("clock");
    let first = Session::capture(&s);
    store.save(&first).unwrap();
    fs::write(root.join("flash/.session.1.tmp"), b"{incomplete").unwrap();
    assert_eq!(store.load().unwrap().unwrap(), first);
    s.open_app("timer");
    store.save(&Session::capture(&s)).unwrap();
    fs::write(root.join("flash/session.1.json"), b"{incomplete").unwrap();
    assert_eq!(store.load().unwrap().unwrap(), first);
    use sha2::{Digest, Sha256};
    let mut invalid = first.clone();
    invalid.version = 99;
    let payload = serde_json::to_string(&invalid).unwrap();
    fs::write(root.join("flash/session.1.json"), serde_json::to_vec(&serde_json::json!({
        "revision":100, "sha256":format!("{:x}", Sha256::digest(payload.as_bytes())), "payload":payload
    })).unwrap()).unwrap();
    assert_eq!(store.load().unwrap().unwrap(), first);
    store.save(&Session::capture(&s)).unwrap();
    fs::write(root.join("flash/session.1.json"), b"bad").unwrap();
    assert_eq!(
        store.load().unwrap().unwrap(),
        first,
        "old usable slot was preserved"
    );
    fs::write(root.join("flash/session.0.json"), b"bad").unwrap();
    assert!(store.load().is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn storage_unavailable_reports_failure_and_bad_radix_cannot_reach_calculator() {
    let root = root();
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("flash"), b"blocked").unwrap();
    let store = GenerationStore::new(root.join("sd"), root.join("flash")).session_store();
    assert!(store
        .save(&Session::capture(&LauncherState::new()))
        .is_err());
    let mut s = LauncherState::new();
    s.open_app("calculator");
    if let ActiveApp::Calculator(c) = &s.active_app {
        c.lock().unwrap().programmer.radix = 90;
    }
    assert!(!Session::capture(&s).valid());
    fs::remove_dir_all(root).unwrap();
}
