use app_launcher::{GenerationStore, LauncherState, UiCommand};
use p4desk_protocol::{
    DeviceMessage, HostMessage, Mode, MAX_CONTROL, MAX_JPEG, PROTOCOL_VERSION, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::font::install_fontpack;

// UTC 2000-01-01 through 2100-01-01, inclusive; keep aligned with p4desk_time_set.
const MIN_UNIX_MS: i64 = 946_684_800_000;
const MAX_UNIX_MS: i64 = 4_102_444_800_000;

pub trait Hal {
    fn connected(&self) -> bool;
    fn host_active(&self) -> bool;
    fn sd_ready(&self) -> bool;
    fn sd_free_bytes(&self) -> u64;
    fn mode(&self) -> Mode;
    fn set_mode(&mut self, mode: Mode, session: u32) -> bool;
    fn heartbeat(&mut self);
    fn unix_ms(&self) -> i64;
    fn set_time(&mut self, unix_ms: i64);
    fn monotonic_ms(&self) -> u64;
    fn brightness(&mut self, percent: u8);
    fn screen(&mut self, on: bool);
    fn media(&mut self, usage: u16);
    fn send(&mut self, message: &DeviceMessage, sequence: u16) -> bool;
}

pub struct DeviceRuntime<H: Hal> {
    pub hal: H,
    pub state: Arc<Mutex<LauncherState>>,
    pub store: GenerationStore,
    hello: bool,
    was_connected: bool,
    was_active: bool,
    pending_activity: u64,
    status: Option<(Mode, bool, bool, u64)>,
}
impl<H: Hal> DeviceRuntime<H> {
    pub fn new(mut hal: H, root: impl Into<PathBuf>, local_root: impl Into<PathBuf>) -> Self {
        let mut store = GenerationStore::new(root, local_root);
        let mut state = LauncherState::new();
        state.settings = store.load_settings();
        state.settings.screen_on = true;
        hal.brightness(state.settings.brightness);
        if hal.sd_ready() {
            match store.restore() {
                Ok(Some(c)) => {
                    install_fontpack(Some(c.font));
                    state.apply_snapshot(c.state);
                }
                Ok(None) => (),
                Err(_) => state.notice = "资源恢复失败，基本工具仍可用".into(),
            }
        }
        let was_connected = hal.connected();
        Self {
            hal,
            state: Arc::new(Mutex::new(state)),
            store,
            hello: false,
            was_connected,
            was_active: false,
            pending_activity: 0,
            status: None,
        }
    }
    fn ack(
        &mut self,
        id: u16,
        op: &str,
        result: Result<(), &'static str>,
        generation: Option<u64>,
    ) {
        let error = result.err();
        self.hal.send(
            &DeviceMessage::Ack {
                request_id: id,
                acknowledged: op.into(),
                ok: error.is_none(),
                error: error.map(str::to_owned),
                generation,
            },
            id,
        );
    }
    pub fn control(&mut self, sequence: u16, bytes: &[u8]) {
        if bytes.len() > MAX_CONTROL || sequence > 1023 {
            return;
        }
        let Ok(message) = serde_json::from_slice::<HostMessage>(bytes) else {
            self.ack(sequence, "control", Err("json_invalid"), None);
            return;
        };
        let id = message.request_id();
        if id != sequence || id > 1023 {
            self.ack(sequence, "control", Err("request_id"), None);
            return;
        }
        if !self.hello && !matches!(message, HostMessage::Hello { .. }) {
            self.ack(id, "control", Err("hello_required"), None);
            return;
        }
        match message {
            HostMessage::Hello { version, .. } => {
                if version != PROTOCOL_VERSION {
                    self.hello = false;
                    self.ack(id, "hello", Err("version"), None);
                    return;
                }
                self.hello = true;
                self.hal.heartbeat();
                self.hal.send(
                    &DeviceMessage::Caps {
                        request_id: id,
                        version: PROTOCOL_VERSION,
                        width: SCREEN_WIDTH,
                        height: SCREEN_HEIGHT,
                        max_jpeg: MAX_JPEG as u32,
                        max_control: MAX_CONTROL as u32,
                        sd_ready: self.hal.sd_ready(),
                        mode: self.hal.mode(),
                    },
                    id,
                );
            }
            HostMessage::Heartbeat { .. } => {
                self.hal.heartbeat();
                self.ack(id, "heartbeat", Ok(()), None);
            }
            HostMessage::SetMode { mode, session, .. } => {
                let result = if self.store.pending_generation().is_some() {
                    Err("sync_busy")
                } else if self.hal.set_mode(mode, session) {
                    self.hal.screen(true);
                    Ok(())
                } else {
                    Err("mode_rejected")
                };
                self.ack(id, "set_mode", result, None);
            }
            HostMessage::TimeSync {
                unix_ms,
                timezone_minutes,
                ..
            } => {
                let result = self.set_time(unix_ms, timezone_minutes);
                self.ack(id, "time_sync", result, None);
            }
            HostMessage::GetState { .. } => {
                let state = self.state.lock().unwrap().snapshot.clone();
                self.hal.send(
                    &DeviceMessage::State {
                        request_id: id,
                        state,
                    },
                    id,
                );
            }
            HostMessage::SyncBegin {
                generation,
                state,
                font_length,
                font_sha256,
                ..
            } => {
                let result = if !self.hal.sd_ready() {
                    Err("sd_unavailable")
                } else if self.hal.sd_free_bytes() < font_length as u64 + 128 * 1024 {
                    Err("sd_space")
                } else {
                    self.store
                        .begin(generation, state, font_length, font_sha256)
                };
                if result.is_ok() {
                    self.pending_activity = self.hal.monotonic_ms();
                }
                self.ack(id, "sync_begin", result, Some(generation));
            }
            HostMessage::SyncCommit { generation, .. } => {
                let result = match self.store.commit(generation) {
                    Ok(c) => {
                        install_fontpack(Some(c.font));
                        let mut state = self.state.lock().unwrap();
                        state.apply_snapshot(c.state);
                        state.notice = "便签与字体已同步".into();
                        Ok(())
                    }
                    Err(code) => {
                        let mut state = self.state.lock().unwrap();
                        state.notice = "同步失败，保留已有便签和字体".into();
                        state.changed();
                        Err(code)
                    }
                };
                self.ack(id, "sync_commit", result, Some(generation));
                if result.is_ok() {
                    let state = self.state.lock().unwrap().snapshot.clone();
                    self.hal.send(
                        &DeviceMessage::State {
                            request_id: 0,
                            state,
                        },
                        0,
                    );
                }
            }
            HostMessage::SyncAbort { .. } => {
                let result = self.store.abort();
                self.ack(id, "sync_abort", result, None);
            }
        }
    }
    pub fn resource(&mut self, sequence: u16, bytes: &[u8]) {
        let result = if !self.hello {
            Err("hello_required")
        } else if !self.hal.sd_ready() {
            Err("sd_unavailable")
        } else if bytes.len() < 5 || bytes.len() > 32772 {
            Err("resource_length")
        } else {
            self.store.chunk(
                u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                &bytes[4..],
            )
        };
        if result.is_ok() {
            self.pending_activity = self.hal.monotonic_ms();
        }
        if result == Err("font_write") {
            let _ = self.store.abort();
        }
        self.ack(
            sequence,
            "resource",
            result,
            self.store.pending_generation(),
        );
    }
    fn set_time(&mut self, unix_ms: i64, timezone_minutes: i32) -> Result<(), &'static str> {
        if !(MIN_UNIX_MS..=MAX_UNIX_MS).contains(&unix_ms)
            || !(-840..=840).contains(&timezone_minutes)
        {
            return Err("time_invalid");
        }
        let mut settings = self.state.lock().unwrap().settings.clone();
        settings.timezone_minutes = timezone_minutes;
        self.store.save_settings(&settings)?;
        self.hal.set_time(unix_ms);
        let mut state = self.state.lock().unwrap();
        state.settings = settings;
        state.last_time_refresh();
        Ok(())
    }
    pub fn tick(&mut self) -> bool {
        let connected = self.hal.connected();
        let sd_ready = self.hal.sd_ready();
        let active = self.hal.host_active();
        let mode = self.hal.mode();
        let now = self.hal.monotonic_ms();
        if self.was_connected && !connected {
            self.hello = false;
            let _ = self.store.abort();
        }
        if self.was_active && !active {
            self.hello = false;
            let _ = self.store.abort();
        }
        self.was_active = active;
        self.was_connected = connected;
        if self.store.pending_generation().is_some()
            && (!sd_ready || now.saturating_sub(self.pending_activity) > 15_000)
        {
            let _ = self.store.abort();
        }
        let mut state = self.state.lock().unwrap();
        let before = state.revision;
        if state.connected != (connected && active && self.hello)
            || state.sd_ready != sd_ready
            || state.mode != mode
        {
            state.connected = connected && active && self.hello;
            state.sd_ready = sd_ready;
            state.mode = mode;
            state.changed();
        }
        state.tick(now, self.hal.unix_ms());
        let status = (mode, sd_ready, state.time_valid, state.snapshot.generation);
        let changed = state.revision != before;
        drop(state);
        if connected && self.hello && self.status != Some(status) {
            // A full USB queue did not accept this status; leave the previous value so
            // the next tick retries, including a gesture-driven return from Display.
            if self.hal.send(
                &DeviceMessage::Status {
                    mode,
                    sd_ready,
                    time_valid: status.2,
                    generation: status.3,
                },
                0,
            ) {
                self.status = Some(status);
            }
        }
        changed
    }
    pub fn process_commands(&mut self) -> bool {
        let commands = self.state.lock().unwrap().take_commands();
        let changed = !commands.is_empty();
        for command in commands {
            let result = match command {
                UiCommand::DeleteNote(id) => {
                    let current = self.state.lock().unwrap().snapshot.clone();
                    match self.store.delete_note(&current, &id) {
                        Ok(next) => {
                            self.state.lock().unwrap().apply_snapshot(next.clone());
                            self.hal.send(
                                &DeviceMessage::State {
                                    request_id: 0,
                                    state: next,
                                },
                                0,
                            );
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                UiCommand::Action(id) => {
                    let state = self.state.lock().unwrap();
                    let present = state.snapshot.buttons.iter().any(|b| b.id == id);
                    drop(state);
                    if !present {
                        Err("action_unknown")
                    } else if !self.hal.host_active() {
                        Err("mac_offline")
                    } else if self.hal.send(&DeviceMessage::Action { action_id: id }, 0) {
                        Ok(())
                    } else {
                        Err("usb_send")
                    }
                }
                UiCommand::Media(usage) => {
                    if self.hal.connected() {
                        self.hal.media(usage);
                        Ok(())
                    } else {
                        Err("mac_offline")
                    }
                }
                UiCommand::Brightness(value) => {
                    let mut settings = self.state.lock().unwrap().settings.clone();
                    settings.brightness = value;
                    match self.store.save_settings(&settings) {
                        Ok(()) => {
                            self.hal.brightness(value);
                            self.state.lock().unwrap().settings = settings;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                UiCommand::Screen(on) => {
                    self.hal.screen(on);
                    self.state.lock().unwrap().settings.screen_on = on;
                    Ok(())
                }
                UiCommand::RequestTimeSync => {
                    if !self.hal.host_active() {
                        Err("mac_offline")
                    } else if self.hal.send(&DeviceMessage::RequestTimeSync, 0) {
                        Ok(())
                    } else {
                        Err("usb_send")
                    }
                }
                UiCommand::SetTime(ms, tz) => self.set_time(ms, tz),
                UiCommand::RequestMode(mode) => {
                    if !self.hal.host_active() {
                        Err("mac_offline")
                    } else if self.hal.send(&DeviceMessage::RequestMode { mode }, 0) {
                        Ok(())
                    } else {
                        Err("usb_send")
                    }
                }
            };
            if let Err(code) = result {
                let mut state = self.state.lock().unwrap();
                state.notice = match code {
                    "mac_offline" => "Mac 未连接，请连接 USB 和 Mac 应用",
                    "note_not_found" => "便签已不存在",
                    "storage_create" | "state_write" | "state_fsync" | "state_rename" => {
                        "保存失败，原数据已保留"
                    }
                    _ => "操作未完成，请重试",
                }
                .into();
                state.changed();
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock {
        now: u64,
        wall: i64,
        connected: bool,
        mode: Mode,
        send_ready: bool,
        send_attempts: usize,
        messages: Vec<(u16, DeviceMessage)>,
    }
    impl Hal for Mock {
        fn connected(&self) -> bool {
            self.connected
        }
        fn host_active(&self) -> bool {
            self.connected
        }
        fn sd_ready(&self) -> bool {
            false
        }
        fn sd_free_bytes(&self) -> u64 {
            0
        }
        fn mode(&self) -> Mode {
            self.mode
        }
        fn set_mode(&mut self, m: Mode, _: u32) -> bool {
            self.mode = m;
            true
        }
        fn heartbeat(&mut self) {}
        fn unix_ms(&self) -> i64 {
            self.wall
        }
        fn set_time(&mut self, t: i64) {
            self.wall = t;
        }
        fn monotonic_ms(&self) -> u64 {
            self.now
        }
        fn brightness(&mut self, _: u8) {}
        fn screen(&mut self, _: bool) {}
        fn media(&mut self, _: u16) {}
        fn send(&mut self, m: &DeviceMessage, s: u16) -> bool {
            self.send_attempts += 1;
            if !self.send_ready {
                return false;
            }
            self.messages.push((s, m.clone()));
            true
        }
    }
    fn runtime() -> DeviceRuntime<Mock> {
        let path = std::env::temp_dir().join(format!("p4desk-runtime-{}", std::process::id()));
        DeviceRuntime::new(
            Mock {
                now: 0,
                wall: 0,
                connected: true,
                mode: Mode::Pad,
                send_ready: true,
                send_attempts: 0,
                messages: vec![],
            },
            path.join("sd"),
            path.join("flash"),
        )
    }
    #[test]
    fn negotiated_caps_and_request_sequence_match() {
        let mut r = runtime();
        r.control(27, br#"{"op":"hello","request_id":27,"version":1}"#);
        assert!(matches!(
            r.hal.messages[0],
            (27, DeviceMessage::Caps { request_id: 27, .. })
        ));
        r.control(3, br#"{"op":"get_state","request_id":3}"#);
        assert!(matches!(
            r.hal.messages.last().unwrap(),
            (3, DeviceMessage::State { request_id: 3, .. })
        ));
        r.control(4, br#"{"op":"heartbeat","request_id":5}"#);
        assert!(matches!(
            r.hal.messages.last().unwrap(),
            (4, DeviceMessage::Ack { ok: false, .. })
        ));
    }
    #[test]
    fn timer_continues_in_display_and_disconnect_returns_ui_state() {
        let mut r = runtime();
        r.state.lock().unwrap().timer.toggle(0);
        r.hal.mode = Mode::Display;
        r.hal.now = 90_000;
        r.tick();
        assert_eq!(r.state.lock().unwrap().timer.remaining_ms, 1_410_000);
        r.hal.connected = false;
        r.hal.mode = Mode::Pad;
        r.hal.now = 100_000;
        r.tick();
        assert_eq!(r.state.lock().unwrap().mode, Mode::Pad);
        assert_eq!(r.state.lock().unwrap().timer.remaining_ms, 1_400_000);
    }
    #[test]
    fn sync_without_sd_is_explicit_error_and_keeps_state() {
        let mut r = runtime();
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.control(2,br#"{"op":"sync_begin","request_id":2,"generation":1,"state":{"generation":1,"notes":[],"buttons":[],"deleted_note_ids":[]},"font_length":32,"font_sha256":"0000000000000000000000000000000000000000000000000000000000000000"}"#);
        assert!(
            matches!(r.hal.messages.last().unwrap(),(2,DeviceMessage::Ack{ok:false,error:Some(e),..}) if e=="sd_unavailable")
        );
        assert_eq!(r.state.lock().unwrap().snapshot.generation, 0);
    }

    #[test]
    fn status_retries_after_usb_queue_full_and_stops_after_success() {
        let mut r = runtime();
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.hal.mode = Mode::Display;
        r.tick();
        assert_eq!(r.status.unwrap().0, Mode::Display);
        r.hal.messages.clear();

        // The C gesture returns to Pad while the priority queue is full.
        r.hal.mode = Mode::Pad;
        r.hal.send_ready = false;
        let attempts = r.hal.send_attempts;
        r.tick();
        assert_eq!(r.hal.send_attempts, attempts + 1);
        assert!(r.hal.messages.is_empty());
        assert_eq!(r.status.unwrap().0, Mode::Display);
        assert_eq!(r.state.lock().unwrap().mode, Mode::Pad);

        r.hal.send_ready = true;
        r.tick();
        assert_eq!(r.hal.send_attempts, attempts + 2);
        assert!(matches!(
            r.hal.messages.as_slice(),
            [(
                0,
                DeviceMessage::Status {
                    mode: Mode::Pad,
                    ..
                }
            )]
        ));
        assert_eq!(r.status.unwrap().0, Mode::Pad);
        r.tick();
        assert_eq!(r.hal.send_attempts, attempts + 2);
        assert_eq!(r.hal.messages.len(), 1);
    }

    #[test]
    fn ui_requests_show_failure_when_usb_queue_is_full() {
        let mut r = runtime();
        r.state
            .lock()
            .unwrap()
            .snapshot
            .buttons
            .push(p4desk_protocol::Button {
                id: "launch".into(),
                label: "启动".into(),
                action: p4desk_protocol::Action::Application {
                    bundle_path: "/Applications/Example.app".into(),
                },
            });
        r.hal.send_ready = false;
        for command in [
            UiCommand::RequestMode(Mode::Display),
            UiCommand::Action("launch".into()),
            UiCommand::RequestTimeSync,
        ] {
            let revision = {
                let mut state = r.state.lock().unwrap();
                state.notice.clear();
                state.queue(command);
                state.revision
            };
            assert!(r.process_commands());
            let state = r.state.lock().unwrap();
            assert_eq!(state.notice, "操作未完成，请重试");
            assert!(state.revision > revision);
            assert_eq!(state.mode, Mode::Pad);
        }
        assert_eq!(r.hal.send_attempts, 3);
        assert!(r.hal.messages.is_empty());
    }

    #[test]
    fn time_sync_acks_only_hal_supported_epoch_boundaries() {
        let mut r = runtime();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "p4desk-time-boundaries-{}-{nonce}",
            std::process::id()
        ));
        r.store = GenerationStore::new(root.join("sd"), root.join("flash"));
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);

        for (index, (unix_ms, timezone_minutes, expected_ok)) in [
            (1, 480, false),
            (MIN_UNIX_MS - 1, 480, false),
            (MIN_UNIX_MS, 840, true),
            (MAX_UNIX_MS, -840, true),
            (MAX_UNIX_MS + 1, 480, false),
        ]
        .into_iter()
        .enumerate()
        {
            let request_id = index as u16 + 2;
            let before_wall = r.hal.wall;
            let before_timezone = r.store.load_settings().timezone_minutes;
            let message = HostMessage::TimeSync {
                request_id,
                unix_ms,
                timezone_minutes,
            };
            r.control(request_id, &serde_json::to_vec(&message).unwrap());
            match r.hal.messages.last().unwrap() {
                (
                    sequence,
                    DeviceMessage::Ack {
                        request_id: id,
                        acknowledged,
                        ok,
                        error,
                        ..
                    },
                ) => {
                    assert_eq!(*sequence, request_id);
                    assert_eq!(*id, request_id);
                    assert_eq!(acknowledged, "time_sync");
                    assert_eq!(*ok, expected_ok, "unix_ms={unix_ms}");
                    assert_eq!(
                        error.as_deref(),
                        if expected_ok {
                            None
                        } else {
                            Some("time_invalid")
                        }
                    );
                }
                other => panic!("expected time_sync ack, received {other:?}"),
            }
            if expected_ok {
                assert_eq!(r.hal.wall, unix_ms);
                assert_eq!(r.store.load_settings().timezone_minutes, timezone_minutes);
            } else {
                assert_eq!(r.hal.wall, before_wall);
                assert_eq!(r.store.load_settings().timezone_minutes, before_timezone);
            }
        }
        drop(r);
        std::fs::remove_dir_all(root).unwrap();
    }
}
