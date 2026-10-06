use app_launcher::{
    usage::{
        api::{Error, Transport},
        Command, Config, Page, Period,
    },
    LauncherState,
};
use rust_main::usage::Service;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};

#[derive(Default)]
struct Watch {
    paths: Mutex<Vec<String>>,
    tokens: AtomicU64,
    block: AtomicBool,
    started: AtomicBool,
    auth: AtomicBool,
}
struct Fake(Arc<Watch>);
impl Transport for Fake {
    fn request(&mut self, _: &Config, path: &str, _: Option<&str>) -> Result<Vec<u8>, Error> {
        self.0.paths.lock().unwrap().push(path.into());
        if path == "/admin/dashboard/stats" && self.0.block.load(Ordering::Acquire) {
            self.0.started.store(true, Ordering::Release);
            while self.0.block.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        if self.0.auth.load(Ordering::Acquire) {
            return Err(Error::Authentication);
        }
        let n = self.0.tokens.load(Ordering::Acquire);
        let data = if path == "/admin/dashboard/stats" {
            json!({"today_tokens":n,"total_tokens":900,"today_actual_cost":1,"total_actual_cost":9,"today_requests":2,"total_requests":20})
        } else if path.starts_with("/admin/accounts?") {
            json!({"items":[],"total":0})
        } else if path.starts_with("/admin/dashboard/trend?") {
            json!({"trend":[]})
        } else if path.starts_with("/admin/dashboard/models?") {
            json!({"models":[]})
        } else {
            json!({"users":[]})
        };
        Ok(serde_json::to_vec(&json!({"code":0,"data":data})).unwrap())
    }
}
fn fixture(name: &str) -> (Service, LauncherState, Arc<Watch>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("p4-live-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("sd")).unwrap();
    let store =
        app_launcher::GenerationStore::new(root.join("sd"), root.join("flash")).monitor_store();
    let w = Arc::new(Watch::default());
    w.tokens.store(10, Ordering::Release);
    let service = Service::new(Box::new(Fake(w.clone())), store).unwrap();
    let mut s = LauncherState::new();
    s.open_app("sub2api-monitor");
    s.tick(100, 1_791_261_000_000);
    s.radio.backend = 2;
    s.radio.wifi_on = 1;
    s.radio.wifi_phase = 5;
    s.usage.config = Some(Config::new("test.example", "test-key").unwrap());
    (service, s, w, root)
}
fn wait(f: impl Fn() -> bool) {
    for _ in 0..500 {
        if f() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("worker timeout");
}
fn complete(service: &mut Service, s: &mut LauncherState, stamp: i64) {
    for _ in 0..500 {
        service.poll(s);
        if s.usage.data.as_ref().is_some_and(|d| d.sampled_ms == stamp) && !s.usage.busy {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("sample timeout");
}
fn advance(s: &mut LauncherState, mono: u64) {
    let delta = mono.saturating_sub(s.monotonic_ms);
    s.monotonic_ms = mono;
    s.unix_ms += delta as i64;
}
fn journals(root: &std::path::Path) -> Vec<(String, Vec<u8>)> {
    let mut files = std::fs::read_dir(root.join("sd"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("monitor-cache.")
        })
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().to_string(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}
#[test]
fn fast_deadline_uses_request_start_not_completion_and_never_writes_tf() {
    let (mut service, mut s, w, root) = fixture("anchor");
    service.poll(&mut s);
    wait(|| w.paths.lock().unwrap().len() >= 5);
    advance(&mut s, 2100);
    let stamp = 1_791_261_000_000;
    complete(&mut service, &mut s, stamp);
    let files = journals(&root);
    assert!(!files.is_empty());
    let count = w.paths.lock().unwrap().len();
    advance(&mut s, 5099);
    service.poll(&mut s);
    assert_eq!(w.paths.lock().unwrap().len(), count);
    advance(&mut s, 5100);
    w.tokens.store(20, Ordering::Release);
    service.poll(&mut s);
    assert!(
        !s.usage.busy,
        "fast sampler must not flash the Refresh button"
    );
    let stamp = s.unix_ms;
    advance(&mut s, 5600);
    complete(&mut service, &mut s, stamp);
    assert_eq!(
        s.usage
            .data
            .as_ref()
            .unwrap()
            .totals
            .as_ref()
            .unwrap()
            .tokens,
        20
    );
    assert_eq!(w.paths.lock().unwrap().len(), count + 2);
    assert_eq!(
        journals(&root),
        files,
        "five-second RAM update must not journal TF"
    );
    advance(&mut s, 10100);
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    assert_eq!(w.paths.lock().unwrap().len(), count + 4);
    advance(&mut s, 60100);
    service.poll(&mut s);
    assert!(
        s.usage.busy,
        "full 60 second batch takes priority over fast slot"
    );
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    assert_ne!(journals(&root), files, "full dashboard still persists");
    drop(service);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn slow_fast_request_skips_elapsed_slots_without_pending_backlog() {
    let (mut service, mut s, w, root) = fixture("skip");
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    w.block.store(true, Ordering::Release);
    advance(&mut s, 5100);
    service.poll(&mut s);
    wait(|| w.started.load(Ordering::Acquire));
    let count = w.paths.lock().unwrap().len();
    advance(&mut s, 20100);
    for _ in 0..20 {
        service.poll(&mut s);
    }
    assert_eq!(w.paths.lock().unwrap().len(), count);
    assert!(!s.usage.busy);
    w.block.store(false, Ordering::Release);
    advance(&mut s, 20200);
    let stamp = 1_791_261_005_000;
    complete(&mut service, &mut s, stamp);
    let count = w.paths.lock().unwrap().len();
    for _ in 0..20 {
        service.poll(&mut s);
    }
    assert_eq!(w.paths.lock().unwrap().len(), count);
    advance(&mut s, 25099);
    service.poll(&mut s);
    assert_eq!(w.paths.lock().unwrap().len(), count);
    advance(&mut s, 25100);
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    assert_eq!(w.paths.lock().unwrap().len(), count + 2);
    drop(service);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn manual_refresh_during_fast_work_runs_one_full_batch_next() {
    let (mut service, mut s, w, root) = fixture("manual");
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    w.block.store(true, Ordering::Release);
    advance(&mut s, 5100);
    service.poll(&mut s);
    wait(|| w.started.load(Ordering::Acquire));
    service.command(&mut s, Command::Refresh);
    service.poll(&mut s);
    assert!(s.usage.refresh);
    assert!(!s.usage.busy);
    w.block.store(false, Ordering::Release);
    advance(&mut s, 5200);
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    let paths = w.paths.lock().unwrap().clone();
    assert_eq!(
        paths
            .iter()
            .filter(|p| p.starts_with("/admin/accounts?"))
            .count(),
        2
    );
    assert_eq!(
        paths
            .iter()
            .filter(|p| p.starts_with("/admin/dashboard/stats"))
            .count(),
        3
    );
    drop(service);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn month_total_and_scoped_pages_have_no_five_second_day_overwrite() {
    for (name, page, period) in [
        ("month", Page::Overview, Period::Month),
        ("total", Page::Overview, Period::Total),
        ("models", Page::Models, Period::Day),
    ] {
        let (mut service, mut s, w, root) = fixture(name);
        s.usage.navigate(page, period, None);
        let stamp = s.unix_ms;
        complete(&mut service, &mut s, stamp);
        let original = s.usage.data.clone().unwrap();
        let count = w.paths.lock().unwrap().len();
        advance(&mut s, 25100);
        for _ in 0..10 {
            service.poll(&mut s);
        }
        assert_eq!(w.paths.lock().unwrap().len(), count);
        assert!(Arc::ptr_eq(&original, s.usage.data.as_ref().unwrap()));
        assert_eq!(s.usage.scope.as_ref().unwrap().period, period);
        assert_eq!(s.usage.scope.as_ref().unwrap().page, page);
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
}
#[test]
fn fast_auth_failure_keeps_old_snapshot_and_backoff() {
    let (mut service, mut s, w, root) = fixture("failure");
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    let original = s.usage.data.clone().unwrap();
    w.auth.store(true, Ordering::Release);
    advance(&mut s, 5100);
    service.poll(&mut s);
    for _ in 0..500 {
        service.poll(&mut s);
        if s.usage.status == Error::Authentication.message() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(s.usage.status, Error::Authentication.message());
    assert!(s.usage.stale);
    assert!(Arc::ptr_eq(&original, s.usage.data.as_ref().unwrap()));
    let count = w.paths.lock().unwrap().len();
    advance(&mut s, 10100);
    service.poll(&mut s);
    assert_eq!(w.paths.lock().unwrap().len(), count);
    w.auth.store(false, Ordering::Release);
    let deadline = s.usage.next_refresh_ms;
    advance(&mut s, deadline);
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    assert!(!s.usage.stale);
    drop(service);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn canceled_fast_scope_cannot_enter_new_period_or_ram_cache() {
    let (mut service, mut s, w, root) = fixture("cancel");
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    w.block.store(true, Ordering::Release);
    advance(&mut s, 5100);
    service.poll(&mut s);
    wait(|| w.started.load(Ordering::Acquire));
    s.usage.navigate(Page::Models, Period::Month, None);
    service.poll(&mut s);
    w.block.store(false, Ordering::Release);
    let stamp = s.unix_ms;
    complete(&mut service, &mut s, stamp);
    assert_eq!(s.usage.scope.as_ref().unwrap().page, Page::Models);
    assert_eq!(s.usage.scope.as_ref().unwrap().period, Period::Month);
    assert_eq!(s.usage.cached_page_count(), 2);
    drop(service);
    let _ = std::fs::remove_dir_all(root);
}
