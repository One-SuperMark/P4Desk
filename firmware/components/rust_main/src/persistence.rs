//! Coalesced checkpoints; the display loop never waits for flash IO.
use app_launcher::{session::Session, storage::SessionStore};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};

pub struct SessionWriter {
    sender: Option<SyncSender<Session>>,
    worker: Option<std::thread::JoinHandle<()>>,
    result: Receiver<bool>,
    observed: Option<Session>,
    saved: Option<Session>,
    in_flight: Option<Session>,
    dirty_since: u64,
    last_change: u64,
    last_saved: u64,
    retry_after: u64,
    next_sample: u64,
}
impl SessionWriter {
    pub fn new(store: SessionStore) -> std::io::Result<Self> {
        let (sender, receive) = mpsc::sync_channel::<Session>(1);
        let (send_result, result) = mpsc::sync_channel(1);
        #[cfg(target_os = "espidf")]
        let previous_config = unsafe {
            let mut previous = esp_idf_sys::esp_pthread_get_default_config();
            // IDF clears the output on ESP_ERR_NOT_FOUND. Reapply defaults;
            // otherwise the first pthread gets stack_size=0 and EINVAL.
            if esp_idf_sys::esp_pthread_get_cfg(&mut previous) != 0 {
                previous = esp_idf_sys::esp_pthread_get_default_config();
            }
            let mut config = previous;
            config.prio = 1;
            config.inherit_cfg = false;
            // Flash IO uses an internal stack; avoid borrowing scarce DMA memory
            // for a large default Rust thread stack.
            config.stack_alloc_caps =
                esp_idf_sys::MALLOC_CAP_INTERNAL | esp_idf_sys::MALLOC_CAP_8BIT;
            let error = esp_idf_sys::esp_pthread_set_cfg(&config);
            if error != 0 {
                crate::diagnostics::diagnostic!("p4desk_session: thread_config_error={error}");
                return Err(std::io::Error::other("checkpoint thread config"));
            }
            previous
        };
        let stack_size = if cfg!(target_os = "espidf") {
            8192
        } else {
            256 * 1024
        };
        let spawned = std::thread::Builder::new()
            .name("p4-checkpoint".into())
            .stack_size(stack_size)
            .spawn(move || {
                while let Ok(session) = receive.recv() {
                    let started = std::time::Instant::now();
                    let ok = store.save(&session).is_ok();
                    #[cfg(target_os = "espidf")]
                    let stack_free = unsafe { esp_idf_sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };
                    #[cfg(not(target_os = "espidf"))]
                    let stack_free = 0;
                    // Counts and outcome only; never log persisted inputs or contents.
                    crate::diagnostics::diagnostic!(
                        "p4desk_session: save={} apps={} recent={} clock={} elapsed_ms={} stack_free={}",
                        if ok { "ok" } else { "failed" },
                        session.background.len() + usize::from(session.foreground.is_some()),
                        session.recent.len(),
                        session.unix_ms.is_some(), started.elapsed().as_millis(), stack_free
                    );
                    if send_result.send(ok).is_err() {
                        break;
                    }
                }
            });
        #[cfg(target_os = "espidf")]
        unsafe {
            esp_idf_sys::esp_pthread_set_cfg(&previous_config);
        }
        let worker = spawned?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
            result,
            observed: None,
            saved: None,
            in_flight: None,
            dirty_since: 0,
            last_change: 0,
            last_saved: 0,
            retry_after: 0,
            next_sample: 0,
        })
    }
    pub fn due(&self, now: u64) -> bool {
        now >= self.next_sample
    }
    /// Return a completion only after fsync + slot switch; failures retry after 5 s.
    pub fn sample(&mut self, now: u64, session: Session, animating: bool) -> Option<bool> {
        self.next_sample = now.saturating_add(500);
        let completed = match self.result.try_recv() {
            Ok(ok) => {
                if ok {
                    self.saved = self.in_flight.take();
                    self.last_saved = now;
                } else {
                    self.in_flight = None;
                    self.retry_after = now.saturating_add(5000);
                }
                Some(ok)
            }
            Err(TryRecvError::Disconnected) => {
                self.in_flight = None;
                Some(false)
            }
            Err(TryRecvError::Empty) => None,
        };
        let dirty = self
            .saved
            .as_ref()
            .is_none_or(|s| !s.same_interaction(&session));
        if self
            .observed
            .as_ref()
            .is_none_or(|s| !s.same_interaction(&session))
        {
            if self.observed.as_ref().is_none_or(|s| {
                self.saved
                    .as_ref()
                    .is_some_and(|saved| s.same_interaction(saved))
            }) {
                self.dirty_since = now;
            }
            self.last_change = now;
        }
        let interaction_due = dirty
            && (now.saturating_sub(self.last_change) >= 1000
                || now.saturating_sub(self.dirty_since) >= 5000);
        let clock_became_available = self
            .saved
            .as_ref()
            .is_some_and(|s| s.unix_ms.is_none() && session.unix_ms.is_some());
        let checkpoint_due = self.saved.is_some()
            && (session.unix_ms.is_some() || session.timer.was_running)
            && now.saturating_sub(self.last_saved)
                >= if session.timer.was_running {
                    10_000
                } else {
                    60_000
                };
        self.observed = Some(session.clone());
        if !animating
            && self.in_flight.is_none()
            && now >= self.retry_after
            && (interaction_due || checkpoint_due || clock_became_available)
        {
            if self
                .sender
                .as_ref()
                .unwrap()
                .try_send(session.clone())
                .is_ok()
            {
                self.in_flight = Some(session);
            } else {
                self.retry_after = now.saturating_add(5000);
                return Some(false);
            }
        }
        completed
    }
}
impl Drop for SessionWriter {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_launcher::{GenerationStore, LauncherState};
    fn setup() -> (std::path::PathBuf, SessionStore, SessionWriter) {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "p4-writer-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = GenerationStore::new(path.join("sd"), path.join("flash")).session_store();
        let writer = SessionWriter::new(store.clone()).unwrap();
        (path, store, writer)
    }
    fn finish(writer: &mut SessionWriter, now: u64, s: &Session) {
        for _ in 0..200 {
            if let Some(ok) = writer.sample(now, s.clone(), false) {
                assert!(ok);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("writer did not complete");
    }
    #[test]
    fn debounce_checkpoints_and_animation_deferral_do_not_write_every_second() {
        let (path, store, mut writer) = setup();
        let mut s = LauncherState::new();
        s.tick(0, 1_790_800_000_000);
        let session = Session::capture(&s);
        writer.sample(0, session.clone(), false);
        writer.sample(1000, session.clone(), true);
        assert!(writer.in_flight.is_none(), "defer during launch");
        writer.sample(1500, session.clone(), false);
        finish(&mut writer, 2000, &session);
        assert_eq!(store.load().unwrap().unwrap(), session);
        for now in (2500..62_000).step_by(500) {
            s.tick(now, 1_790_800_000_000 + now as i64);
            writer.sample(now, Session::capture(&s), false);
            assert!(writer.in_flight.is_none());
        }
        let next = Session::capture(&s);
        writer.sample(62_000, next.clone(), false);
        assert!(writer.in_flight.is_some());
        finish(&mut writer, 62_500, &next);
        drop(writer);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn running_timer_checkpoints_and_pause_are_durable() {
        let (path, store, mut writer) = setup();
        let mut s = LauncherState::new();
        s.timer.toggle(0);
        writer.sample(0, Session::capture(&s), false);
        s.timer.tick(1000);
        let first = Session::capture(&s);
        writer.sample(1000, first.clone(), false);
        finish(&mut writer, 1500, &first);
        s.timer.tick(11_500);
        let next = Session::capture(&s);
        writer.sample(11_500, next.clone(), false);
        finish(&mut writer, 12_000, &next);
        s.timer.toggle(12_500);
        let paused = Session::capture(&s);
        writer.sample(12_500, paused.clone(), false);
        writer.sample(13_500, paused.clone(), false);
        finish(&mut writer, 14_000, &paused);
        let saved = store.load().unwrap().unwrap();
        assert_eq!(saved.timer.remaining_ms, 1_487_500);
        assert!(!saved.timer.was_running);
        drop(writer);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn write_failure_keeps_pending_state_and_retries() {
        let (path, store, mut writer) = setup();
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("flash"), b"blocked").unwrap();
        let s = Session::capture(&LauncherState::new());
        writer.sample(0, s.clone(), false);
        writer.sample(1000, s.clone(), false);
        let mut result = None;
        for _ in 0..200 {
            result = writer.sample(1500, s.clone(), false);
            if result.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(result, Some(false));
        assert!(writer.saved.is_none());
        std::fs::remove_file(path.join("flash")).unwrap();
        writer.sample(6500, s.clone(), false);
        finish(&mut writer, 7000, &s);
        assert_eq!(store.load().unwrap().unwrap(), s);
        drop(writer);
        std::fs::remove_dir_all(path).unwrap();
    }
}
