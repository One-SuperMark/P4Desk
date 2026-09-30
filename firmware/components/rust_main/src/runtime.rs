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
    fn radio_snapshot(&self, _revision: u32) -> Option<app_launcher::radio::RadioSnapshot> {
        None
    }
    fn radio_command(&mut self, _command: &app_launcher::radio::RadioCommand) -> bool {
        false
    }
    fn connected(&self) -> bool;
    fn host_active(&self) -> bool;
    fn reset_reason(&self) -> u32 {
        0
    }
    fn battery_reading(&self) -> app_launcher::battery::BatteryReading {
        app_launcher::battery::BatteryReading::default()
    }
    fn sd_ready(&self) -> bool;
    fn sd_free_bytes(&self) -> u64;
    fn mode(&self) -> Mode;
    fn set_mode(&mut self, mode: Mode, session: u32) -> bool;
    fn direct_jpeg_rotation_degrees(&self) -> u16 {
        0
    }
    fn set_mode_with_jpeg_rotation(
        &mut self,
        mode: Mode,
        session: u32,
        jpeg_rotation_degrees: u16,
    ) -> bool {
        jpeg_rotation_degrees == 0 && self.set_mode(mode, session)
    }
    fn arm_display_transition(&mut self, _duration_ms: u32) -> bool {
        false
    }
    fn cancel_display_transition(&mut self) {}
    fn display_transition_pending(&self) -> bool {
        false
    }
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
    last_battery_poll_ms: Option<u64>,
    last_radio_poll_ms: Option<u64>,
    status: Option<(Mode, bool, bool, u64)>,
    session_writer: Option<crate::persistence::SessionWriter>,
    recovered_clock: Option<(i64, u64)>,
}
impl<H: Hal> DeviceRuntime<H> {
    pub fn new(mut hal: H, root: impl Into<PathBuf>, local_root: impl Into<PathBuf>) -> Self {
        let mut store = GenerationStore::new(root, local_root);
        let mut state = LauncherState::new();
        state.reset_reason = hal.reset_reason();
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
        let session_store = store.session_store();
        let mut recovered_clock = None;
        match session_store.load() {
            Ok(Some(session)) => {
                session.restore(&mut state);
                recovered_clock = session.unix_ms.map(|ms| (ms, hal.monotonic_ms()));
                crate::diagnostics::diagnostic!(
                    "p4desk_session: restore=ok apps={} recent={} clock={} timer=paused",
                    session.background.len() + usize::from(session.foreground.is_some()),
                    session.recent.len(),
                    session.unix_ms.is_some()
                );
            }
            Ok(None) => crate::diagnostics::diagnostic!("p4desk_session: restore=empty"),
            Err(_) => {
                state.persistence_status = app_launcher::session::PersistenceStatus::Failed;
                state.notice = "应用状态恢复失败，请检查存储".into();
                crate::diagnostics::diagnostic!("p4desk_session: restore=failed");
            }
        }
        let session_writer = match crate::persistence::SessionWriter::new(session_store) {
            Ok(writer) => Some(writer),
            Err(error) => {
                crate::diagnostics::diagnostic!("p4desk_session: thread_start_error={error}");
                None
            }
        };
        if session_writer.is_none() {
            state.persistence_status = app_launcher::session::PersistenceStatus::Failed;
        }
        crate::diagnostics::diagnostic!(
            "p4desk_session: writer={}",
            if session_writer.is_some() {
                "ready"
            } else {
                "unavailable"
            }
        );
        let was_connected = hal.connected();
        Self {
            hal,
            state: Arc::new(Mutex::new(state)),
            store,
            hello: false,
            was_connected,
            was_active: false,
            pending_activity: 0,
            last_battery_poll_ms: None,
            last_radio_poll_ms: None,
            status: None,
            session_writer,
            recovered_clock,
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
                        direct_jpeg_rotation_degrees: self.hal.direct_jpeg_rotation_degrees(),
                    },
                    id,
                );
            }
            HostMessage::Heartbeat { .. } => {
                self.hal.heartbeat();
                self.ack(id, "heartbeat", Ok(()), None);
            }
            HostMessage::SetMode {
                mode,
                session,
                jpeg_rotation_degrees,
                ..
            } => {
                let result = if self.store.pending_generation().is_some() {
                    Err("sync_busy")
                } else if (mode == Mode::Pad && jpeg_rotation_degrees != 0)
                    || (jpeg_rotation_degrees != 0
                        && jpeg_rotation_degrees != self.hal.direct_jpeg_rotation_degrees())
                {
                    Err("jpeg_rotation")
                } else if self
                    .hal
                    .set_mode_with_jpeg_rotation(mode, session, jpeg_rotation_degrees)
                {
                    self.hal.screen(true);
                    Ok(())
                } else {
                    Err("mode_rejected")
                };
                if mode == Mode::Pad && result.is_ok() {
                    self.hal.cancel_display_transition();
                    self.state
                        .lock()
                        .unwrap()
                        .fail_display_launch("操作未完成，请重试");
                }
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
        if self
            .last_radio_poll_ms
            .is_none_or(|last| now.saturating_sub(last) >= 250)
        {
            self.last_radio_poll_ms = Some(now);
            if let Some(next) = self.hal.radio_snapshot(state.radio.revision) {
                let desktop_radio_changed = next.wifi_status_changed(&state.radio)
                    || next.bt_on != state.radio.bt_on
                    || next.bt_ready != state.radio.bt_ready
                    || (state.status_panel_open
                        && state.status_panel_kind
                            == app_launcher::status_bar::StatusPanelKind::Wifi
                        && next.wifi_rssi_dbm != state.radio.wifi_rssi_dbm);
                state.radio = next;
                if matches!(state.active_app, app_launcher::ActiveApp::Settings)
                    || (desktop_radio_changed
                        && matches!(state.active_app, app_launcher::ActiveApp::Launcher))
                {
                    state.changed();
                }
            }
        }
        if self
            .last_battery_poll_ms
            .is_none_or(|last| now.saturating_sub(last) >= 2000)
        {
            self.last_battery_poll_ms = Some(now);
            if state.battery.update(self.hal.battery_reading()) {
                state.changed();
            }
        }
        if state.mode == Mode::Display && mode == Mode::Pad {
            state.fail_display_launch("操作未完成，请重试");
        }
        if state.usb_connected != connected
            || state.connected != (connected && active && self.hello)
            || state.sd_ready != sd_ready
            || state.mode != mode
        {
            state.usb_connected = connected;
            state.connected = connected && active && self.hello;
            state.sd_ready = sd_ready;
            state.mode = mode;
            state.changed();
        }
        if mode == Mode::Display && !self.hal.display_transition_pending() {
            state.complete_display_launch();
        }
        let live_time = self.hal.unix_ms();
        let live_valid = (MIN_UNIX_MS..=MAX_UNIX_MS).contains(&live_time);
        if live_valid {
            self.recovered_clock = None;
        }
        let estimated = !live_valid && self.recovered_clock.is_some();
        let unix_ms = if live_valid {
            live_time
        } else {
            self.recovered_clock
                .map(|(saved, start)| {
                    saved
                        .saturating_add(now.saturating_sub(start).min(i64::MAX as u64) as i64)
                        .min(MAX_UNIX_MS)
                })
                .unwrap_or(0)
        };
        if state.time_estimated != estimated {
            state.time_estimated = estimated;
            state.last_time_refresh();
            state.changed();
        }
        state.tick(now, unix_ms);
        if let Some(writer) = &mut self.session_writer {
            if writer.due(now) {
                let snapshot = app_launcher::session::Session::capture(&state);
                let animating = state.app_launch.frame(now).is_some()
                    || state.timer_completion.progress(now).is_some();
                if let Some(ok) = writer.sample(now, snapshot, animating) {
                    state.persistence_status = if ok {
                        app_launcher::session::PersistenceStatus::Saved
                    } else {
                        app_launcher::session::PersistenceStatus::Failed
                    };
                    state.changed();
                }
            }
        }
        let status = (
            mode,
            sd_ready,
            state.time_valid && !state.time_estimated,
            state.snapshot.generation,
        );
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
                UiCommand::Radio(command) => {
                    if self.hal.radio_command(&command) {
                        Ok(())
                    } else {
                        Err("radio_busy")
                    }
                }
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
                UiCommand::Appearance { light, glass } => {
                    let mut settings = self.state.lock().unwrap().settings.clone();
                    settings.icon_light_override = Some(settings.light_icons());
                    settings.light_appearance = light;
                    settings.glass_amount = glass;
                    match self.store.save_settings(&settings) {
                        Ok(()) => {
                            let mut state = self.state.lock().unwrap();
                            state.settings = settings;
                            state.desktop_backdrop.lock().unwrap().take();
                            state.app_launch.cancel();
                            state.changed();
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                UiCommand::IconTheme(theme) => {
                    let mut settings = self.state.lock().unwrap().settings.clone();
                    settings.icon_theme = theme;
                    match self.store.save_settings(&settings) {
                        Ok(()) => {
                            let mut state = self.state.lock().unwrap();
                            state.settings = settings;
                            state.desktop_backdrop.lock().unwrap().take();
                            state.app_launch.cancel();
                            state.changed();
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                UiCommand::IconLight(light) => {
                    let mut settings = self.state.lock().unwrap().settings.clone();
                    settings.icon_light_override = Some(light);
                    match self.store.save_settings(&settings) {
                        Ok(()) => {
                            let mut state = self.state.lock().unwrap();
                            state.settings = settings;
                            state.desktop_backdrop.lock().unwrap().take();
                            state.app_launch.cancel();
                            state.changed();
                            Ok(())
                        }
                        Err(e) => Err(e),
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
                UiCommand::StartDisplayTransition { duration_ms } => {
                    if !self.hal.host_active() {
                        Err("mac_offline")
                    } else if !self.hal.arm_display_transition(duration_ms) {
                        Err("mode_rejected")
                    } else if self.hal.send(
                        &DeviceMessage::RequestMode {
                            mode: Mode::Display,
                        },
                        0,
                    ) {
                        Ok(())
                    } else {
                        self.hal.cancel_display_transition();
                        Err("usb_send")
                    }
                }
                UiCommand::CancelDisplayTransition => {
                    self.hal.cancel_display_transition();
                    self.hal.set_mode(Mode::Pad, 0);
                    if self.hal.host_active() {
                        self.hal
                            .send(&DeviceMessage::RequestMode { mode: Mode::Pad }, 0);
                    }
                    Ok(())
                }
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
                state.fail_display_launch(if code == "mac_offline" {
                    "Mac 未连接，请连接 USB 和 Mac 应用"
                } else {
                    "操作未完成，请重试"
                });
                state.notice = match code {
                    "radio_busy" => "无线任务繁忙，请稍后重试",
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
        direct_rotation: u16,
        mode_ready: bool,
        mode_requests: Vec<(Mode, u32, u16)>,
        screen_calls: usize,
        transition_pending: bool,
        transition_arms: Vec<u32>,
        send_ready: bool,
        send_attempts: usize,
        messages: Vec<(u16, DeviceMessage)>,
        battery: app_launcher::battery::BatteryReading,
        radio: app_launcher::radio::RadioSnapshot,
    }
    impl Hal for Mock {
        fn radio_snapshot(&self, last: u32) -> Option<app_launcher::radio::RadioSnapshot> {
            (self.radio.revision != last).then_some(self.radio)
        }
        fn connected(&self) -> bool {
            self.connected
        }
        fn host_active(&self) -> bool {
            self.connected
        }
        fn battery_reading(&self) -> app_launcher::battery::BatteryReading {
            self.battery
        }
        fn reset_reason(&self) -> u32 {
            9
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
        fn set_mode(&mut self, m: Mode, session: u32) -> bool {
            self.mode_requests.push((m, session, 0));
            if !self.mode_ready {
                return false;
            }
            self.mode = m;
            true
        }
        fn direct_jpeg_rotation_degrees(&self) -> u16 {
            self.direct_rotation
        }
        fn set_mode_with_jpeg_rotation(&mut self, m: Mode, session: u32, degrees: u16) -> bool {
            if degrees == 0 {
                return self.set_mode(m, session);
            }
            self.mode_requests.push((m, session, degrees));
            if !self.mode_ready || m == Mode::Pad || degrees != self.direct_rotation {
                return false;
            }
            self.mode = m;
            true
        }
        fn arm_display_transition(&mut self, duration: u32) -> bool {
            self.transition_arms.push(duration);
            self.transition_pending = true;
            true
        }
        fn cancel_display_transition(&mut self) {
            self.transition_pending = false;
        }
        fn display_transition_pending(&self) -> bool {
            self.transition_pending
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
        fn screen(&mut self, _: bool) {
            self.screen_calls += 1;
        }
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
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "p4desk-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        DeviceRuntime::new(
            Mock {
                now: 0,
                wall: 0,
                connected: true,
                mode: Mode::Pad,
                direct_rotation: 0,
                mode_ready: true,
                mode_requests: vec![],
                screen_calls: 0,
                transition_pending: false,
                transition_arms: vec![],
                send_ready: true,
                send_attempts: 0,
                messages: vec![],
                battery: app_launcher::battery::BatteryReading::default(),
                radio: app_launcher::radio::RadioSnapshot::default(),
            },
            path.join("sd"),
            path.join("flash"),
        )
    }
    #[test]
    fn wifi_changes_refresh_desktop_without_waiting_for_clock_but_ble_does_not() {
        use app_launcher::radio::WifiIndicator;
        let mut r = runtime();
        r.tick();
        let rev = r.state.lock().unwrap().revision;
        r.hal.now = 250;
        r.hal.radio.revision = 1;
        r.hal.radio.backend = 2;
        r.hal.radio.wifi_on = 1;
        r.hal.radio.wifi_phase = 5;
        assert!(r.tick());
        assert!(r.state.lock().unwrap().revision > rev);
        assert_eq!(
            r.state.lock().unwrap().radio.wifi_indicator(),
            WifiIndicator::Connected
        );
        let rev = r.state.lock().unwrap().revision;
        r.hal.now = 500;
        r.hal.radio.revision = 2;
        r.hal.radio.ble_count = 3;
        assert!(!r.tick());
        assert_eq!(r.state.lock().unwrap().revision, rev);
        r.hal.now = 750;
        r.hal.radio.revision = 3;
        r.hal.radio.wifi_on = 0;
        r.hal.radio.wifi_phase = 0;
        assert!(r.tick());
        assert_eq!(
            r.state.lock().unwrap().radio.wifi_indicator(),
            WifiIndicator::Disabled
        );
    }
    #[test]
    fn wifi_signal_level_refreshes_connected_desktop_without_ble_noise() {
        let mut r = runtime();
        r.hal.radio.revision = 1;
        r.hal.radio.backend = 2;
        r.hal.radio.wifi_on = 1;
        r.hal.radio.wifi_phase = 5;
        r.hal.radio.wifi_rssi_valid = 1;
        r.hal.radio.wifi_rssi_dbm = -48;
        r.tick();
        assert_eq!(r.state.lock().unwrap().radio.wifi_signal_level(), Some(3));
        r.hal.now = 250;
        r.hal.radio.revision = 2;
        r.hal.radio.wifi_rssi_dbm = -74;
        assert!(r.tick());
        assert_eq!(r.state.lock().unwrap().radio.wifi_signal_level(), Some(1));
        r.hal.now = 500;
        r.hal.radio.revision = 3;
        r.hal.radio.wifi_rssi_dbm = -75;
        assert!(!r.tick(), "same bar level does not redraw the desktop");
        r.hal.now = 750;
        r.hal.radio.revision = 4;
        r.hal.radio.wifi_rssi_valid = 0;
        assert!(r.tick());
        assert_eq!(r.state.lock().unwrap().radio.wifi_signal_level(), None);
    }
    #[test]
    fn folio_bluetooth_state_refreshes_rail_without_waiting_for_clock() {
        let mut r = runtime();
        r.tick();
        r.hal.now = 250;
        r.hal.radio.revision = 1;
        r.hal.radio.bt_on = 1;
        assert!(r.tick());
        r.hal.now = 500;
        r.hal.radio.revision = 2;
        r.hal.radio.bt_ready = 1;
        assert!(r.tick());
        r.hal.now = 750;
        r.hal.radio.revision = 3;
        r.hal.radio.ble_count = 4;
        assert!(!r.tick(), "scan count is not shown in the rail");
    }
    #[test]
    fn battery_is_polled_offline_and_in_display_without_inventing_charging() {
        use app_launcher::battery::{BatteryReading, ChargeState};
        let mut r = runtime();
        r.hal.connected = false;
        assert_eq!(r.state.lock().unwrap().reset_reason, 9);
        r.hal.battery = BatteryReading {
            voltage_mv: Some(3900),
            charge: ChargeState::Unknown,
        };
        r.tick();
        assert_eq!(r.state.lock().unwrap().battery.percent, Some(70));
        assert_eq!(r.state.lock().unwrap().battery.charge, ChargeState::Unknown);
        r.hal.connected = true;
        r.hal.battery.voltage_mv = None;
        r.hal.now = 1000;
        r.tick();
        assert_eq!(
            r.state.lock().unwrap().battery.percent,
            Some(70),
            "not polled every UI loop"
        );
        assert_eq!(
            r.state.lock().unwrap().battery.charge,
            ChargeState::Unknown,
            "USB is not charging"
        );
        r.hal.now = 2000;
        r.hal.mode = Mode::Display;
        r.tick();
        assert_eq!(
            r.state.lock().unwrap().battery.percent,
            None,
            "failed sample must clear stale charge"
        );
        r.hal.battery.voltage_mv = Some(3800);
        r.hal.now = 4000;
        r.tick();
        assert_eq!(r.state.lock().unwrap().battery.percent, Some(50));
    }

    #[test]
    fn negotiated_caps_and_request_sequence_match() {
        let mut r = runtime();
        r.control(27, br#"{"op":"hello","request_id":27,"version":1}"#);
        assert!(matches!(
            r.hal.messages[0],
            (
                27,
                DeviceMessage::Caps {
                    request_id: 27,
                    direct_jpeg_rotation_degrees: 0,
                    ..
                }
            )
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

    fn assert_mode_ack(runtime: &DeviceRuntime<Mock>, id: u16, ok: bool, error: Option<&str>) {
        match runtime.hal.messages.last().unwrap() {
            (
                sequence,
                DeviceMessage::Ack {
                    request_id,
                    acknowledged,
                    ok: actual_ok,
                    error: actual_error,
                    ..
                },
            ) => {
                assert_eq!(*sequence, id);
                assert_eq!(*request_id, id);
                assert_eq!(acknowledged, "set_mode");
                assert_eq!(*actual_ok, ok);
                assert_eq!(actual_error.as_deref(), error);
            }
            other => panic!("expected set_mode ACK, received {other:?}"),
        }
    }

    #[test]
    fn legacy_mode_request_forwards_zero_rotation_even_when_direct_decode_is_available() {
        let mut r = runtime();
        r.hal.direct_rotation = 180;
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.control(
            2,
            br#"{"op":"set_mode","request_id":2,"mode":"display","session":41}"#,
        );
        assert_mode_ack(&r, 2, true, None);
        assert_eq!(r.hal.mode_requests, [(Mode::Display, 41, 0)]);
        assert_eq!(r.hal.mode, Mode::Display);
        r.control(
            3,
            br#"{"op":"set_mode","request_id":3,"mode":"pad","session":0}"#,
        );
        assert_mode_ack(&r, 3, true, None);
        assert_eq!(r.hal.mode_requests.last(), Some(&(Mode::Pad, 0, 0)));
        assert_eq!(r.hal.mode, Mode::Pad);
    }

    #[test]
    fn direct_jpeg_rotation_requires_advertised_capability_and_pad_keeps_zero() {
        let mut r = runtime();
        r.hal.direct_rotation = 180;
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        assert!(matches!(
            r.hal.messages[0],
            (
                1,
                DeviceMessage::Caps {
                    direct_jpeg_rotation_degrees: 180,
                    ..
                }
            )
        ));
        r.control(2, br#"{"op":"set_mode","request_id":2,"mode":"display","session":42,"jpeg_rotation_degrees":180}"#);
        assert_mode_ack(&r, 2, true, None);
        assert_eq!(r.hal.mode_requests, [(Mode::Display, 42, 180)]);
        r.tick();
        assert_eq!(r.state.lock().unwrap().mode, Mode::Display);

        for (index, (mode, degrees)) in
            [(Mode::Display, 90), (Mode::Display, 360), (Mode::Pad, 180)]
                .into_iter()
                .enumerate()
        {
            let id = index as u16 + 3;
            let request = HostMessage::SetMode {
                request_id: id,
                mode,
                session: 42,
                jpeg_rotation_degrees: degrees,
            };
            r.control(id, &serde_json::to_vec(&request).unwrap());
            assert_mode_ack(&r, id, false, Some("jpeg_rotation"));
            assert_eq!(r.hal.mode_requests, [(Mode::Display, 42, 180)]);
            assert_eq!(r.hal.mode, Mode::Display);
            assert_eq!(r.hal.screen_calls, 1);
            r.tick();
            assert_eq!(r.state.lock().unwrap().mode, Mode::Display);
        }
        r.control(6, br#"{"op":"set_mode","request_id":6,"mode":"pad","session":0,"jpeg_rotation_degrees":0}"#);
        assert_mode_ack(&r, 6, true, None);
        assert_eq!(r.hal.mode, Mode::Pad);
    }

    #[test]
    fn absent_capability_and_hal_rejection_do_not_change_mode() {
        let mut r = runtime();
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.control(2, br#"{"op":"set_mode","request_id":2,"mode":"display","session":42,"jpeg_rotation_degrees":180}"#);
        assert_mode_ack(&r, 2, false, Some("jpeg_rotation"));
        assert!(r.hal.mode_requests.is_empty());
        assert_eq!(r.hal.mode, Mode::Pad);

        r.hal.direct_rotation = 180;
        r.hal.mode_ready = false;
        r.control(3, br#"{"op":"hello","request_id":3,"version":1}"#);
        r.control(4, br#"{"op":"set_mode","request_id":4,"mode":"display","session":42,"jpeg_rotation_degrees":180}"#);
        assert_mode_ack(&r, 4, false, Some("mode_rejected"));
        assert_eq!(r.hal.mode_requests, [(Mode::Display, 42, 180)]);
        assert_eq!(r.hal.mode, Mode::Pad);
        assert_eq!(r.hal.screen_calls, 0);
    }

    // A pre-extension HAL inherits the zero-only defaults; it must never be
    // called for already-rotated JPEGs merely because the protocol has a field.
    struct LegacyHal(Mock);
    impl Hal for LegacyHal {
        fn connected(&self) -> bool {
            self.0.connected()
        }
        fn host_active(&self) -> bool {
            self.0.host_active()
        }
        fn sd_ready(&self) -> bool {
            self.0.sd_ready()
        }
        fn sd_free_bytes(&self) -> u64 {
            self.0.sd_free_bytes()
        }
        fn mode(&self) -> Mode {
            self.0.mode()
        }
        fn set_mode(&mut self, mode: Mode, session: u32) -> bool {
            self.0.set_mode(mode, session)
        }
        fn heartbeat(&mut self) {
            self.0.heartbeat();
        }
        fn unix_ms(&self) -> i64 {
            self.0.unix_ms()
        }
        fn set_time(&mut self, unix_ms: i64) {
            self.0.set_time(unix_ms);
        }
        fn monotonic_ms(&self) -> u64 {
            self.0.monotonic_ms()
        }
        fn brightness(&mut self, percent: u8) {
            self.0.brightness(percent);
        }
        fn screen(&mut self, on: bool) {
            self.0.screen(on);
        }
        fn media(&mut self, usage: u16) {
            self.0.media(usage);
        }
        fn send(&mut self, message: &DeviceMessage, sequence: u16) -> bool {
            self.0.send(message, sequence)
        }
    }

    #[test]
    fn legacy_hal_defaults_reject_rotated_input_without_calling_set_mode() {
        let mut legacy = LegacyHal(runtime().hal);
        assert_eq!(legacy.direct_jpeg_rotation_degrees(), 0);
        assert!(!legacy.set_mode_with_jpeg_rotation(Mode::Display, 42, 180));
        assert!(legacy.0.mode_requests.is_empty());
        assert_eq!(legacy.0.mode, Mode::Pad);
        assert!(legacy.set_mode_with_jpeg_rotation(Mode::Display, 42, 0));
        assert_eq!(legacy.0.mode_requests, [(Mode::Display, 42, 0)]);
        assert_eq!(legacy.0.mode, Mode::Display);
    }
    #[test]
    fn desktop_usb_status_tracks_enumeration_without_companion_handshake() {
        let mut r = runtime();
        assert!(r.tick());
        {
            let state = r.state.lock().unwrap();
            assert!(state.usb_connected);
            assert!(!state.connected);
        }
        // A change within the same clock second still rebuilds the status bar.
        r.hal.connected = false;
        assert!(r.tick());
        assert!(!r.state.lock().unwrap().usb_connected);
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

    fn launch_usb(r: &mut DeviceRuntime<Mock>) {
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.tick();
        r.hal.messages.clear();
        r.state.lock().unwrap().launch_app(
            "display",
            tiny_flutter::Rect::from_ltwh(563.38, 334.38, 137.24, 137.24),
        );
    }
    fn launch_tick(r: &mut DeviceRuntime<Mock>, now: u64) {
        r.hal.now = now;
        r.tick();
        r.state
            .lock()
            .unwrap()
            .take_launch_animation_dirty(tiny_flutter::Size::new(1024.0, 600.0));
    }
    #[test]
    fn desktop_usb_requests_at_full_cover_and_waits_for_lcd_reveal_completion() {
        let mut r = runtime();
        launch_usb(&mut r);
        for now in [0, 130, 260, 339] {
            launch_tick(&mut r, now);
            assert!(!r.process_commands());
        }
        launch_tick(&mut r, 340);
        assert!(r.process_commands());
        assert_eq!(r.hal.transition_arms, [600]);
        assert!(matches!(
            r.hal.messages.as_slice(),
            [(
                0,
                DeviceMessage::RequestMode {
                    mode: Mode::Display
                }
            )]
        ));
        for now in [940, 5000] {
            launch_tick(&mut r, now);
            assert!(!r.process_commands());
            assert_eq!(
                r.state
                    .lock()
                    .unwrap()
                    .app_launch
                    .frame(now)
                    .unwrap()
                    .elapsed_ms,
                340
            );
            assert_eq!(r.hal.mode, Mode::Pad);
        }
        r.control(
            2,
            br#"{"op":"set_mode","request_id":2,"mode":"display","session":99}"#,
        );
        r.tick();
        assert_eq!(r.hal.mode, Mode::Display);
        assert_eq!(r.state.lock().unwrap().active_app.id(), Some("display"));
        // HAL becomes not pending only after first valid JPEG + 600 ms reveal + LCD completion.
        r.hal.transition_pending = false;
        r.tick();
        assert!(matches!(
            r.state.lock().unwrap().active_app,
            app_launcher::ActiveApp::Launcher
        ));
        assert!(!r.process_commands());
    }
    #[test]
    fn host_failure_or_no_first_frame_preserves_intermediate_page_and_releases_hal() {
        for host_error in [true, false] {
            let mut r = runtime();
            launch_usb(&mut r);
            launch_tick(&mut r, 340);
            r.process_commands();
            r.control(
                2,
                br#"{"op":"set_mode","request_id":2,"mode":"display","session":99}"#,
            );
            r.tick();
            if host_error {
                r.control(
                    3,
                    br#"{"op":"set_mode","request_id":3,"mode":"pad","session":0}"#,
                );
                r.tick();
            } else {
                r.hal.now = 10340;
                r.tick(); // No successful JPEG ever arrived.
            }
            assert!(r.process_commands());
            assert!(!r.hal.transition_pending);
            assert_eq!(r.hal.mode, Mode::Pad);
            launch_tick(&mut r, 12000);
            let mut s = r.state.lock().unwrap();
            assert_eq!(s.active_app.id(), Some("display"));
            assert!(!s.notice.is_empty());
            assert!(s.app_launch.frame(12000).is_none());
            assert!(s.take_commands().is_empty());
        }
    }
    #[test]
    fn device_side_pad_return_before_first_frame_keeps_failure_page() {
        let mut r = runtime();
        launch_usb(&mut r);
        launch_tick(&mut r, 340);
        r.process_commands();
        r.control(
            2,
            br#"{"op":"set_mode","request_id":2,"mode":"display","session":99}"#,
        );
        r.tick();
        r.hal.mode = Mode::Pad; // Watchdog or three-finger exit, without a host command.
        r.hal.transition_pending = false;
        r.hal.now = 2000;
        r.tick();
        r.process_commands();
        launch_tick(&mut r, 2700);
        assert_eq!(r.state.lock().unwrap().active_app.id(), Some("display"));
        assert!(r.state.lock().unwrap().app_launch.frame(2700).is_none());
    }

    #[test]
    fn failed_start_request_reveals_intermediate_page_and_cancels_armed_transition() {
        let mut r = runtime();
        launch_usb(&mut r);
        r.hal.send_ready = false;
        launch_tick(&mut r, 340);
        r.process_commands();
        r.process_commands();
        assert!(!r.hal.transition_pending);
        launch_tick(&mut r, 940);
        assert_eq!(r.state.lock().unwrap().active_app.id(), Some("display"));
        assert!(r.state.lock().unwrap().notice.contains("操作未完成"));
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

    #[test]
    fn asynchronous_wifi_time_refreshes_ui_without_mac_or_changing_timer_and_timezone() {
        let mut r = runtime();
        r.hal.connected = false;
        {
            let mut s = r.state.lock().unwrap();
            s.open_app("settings");
            s.settings.timezone_minutes = -60;
            s.timer.set_countdown_seconds(10);
            s.timer.toggle(0);
        }
        r.tick();
        assert!(!r.state.lock().unwrap().time_valid);
        r.hal.now = 1000;
        r.hal.wall = 946684800000;
        r.hal.radio.revision = 1;
        r.hal.radio.wifi_on = 1;
        r.hal.radio.wifi_phase = 5;
        r.hal.radio.time_sync.phase = 2;
        r.hal.radio.time_sync.last_sync_unix_s = 946684800;
        assert!(r.tick());
        {
            let s = r.state.lock().unwrap();
            assert!(s.time_valid);
            assert_eq!(s.clock, "23:00:00");
            assert_eq!(s.settings.timezone_minutes, -60);
            assert_eq!(s.timer.remaining_ms, 9000);
            assert_eq!(s.radio.time_sync.phase, 2);
        }
        // A failed refresh keeps the running clock; a later backwards correction
        // still cannot extend the monotonic countdown.
        r.hal.radio.revision = 2;
        r.hal.radio.time_sync.phase = 3;
        r.hal.now = 3000;
        r.tick();
        assert!(r.state.lock().unwrap().time_valid);
        r.hal.wall -= 3600000;
        r.hal.now = 10000;
        r.tick();
        assert!(r.state.lock().unwrap().timer.finished);
        assert_eq!(r.state.lock().unwrap().timer.remaining_ms, 0);
    }
    #[test]
    fn reboot_uses_saved_clock_as_estimate_until_real_sync_without_resuming_timer() {
        let path = std::env::temp_dir().join(format!(
            "p4-reboot-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = GenerationStore::new(path.join("sd"), path.join("flash"));
        let mut state = LauncherState::new();
        state.tick(100, 1_790_800_000_000);
        state.open_app("timer");
        state.timer.set_countdown_seconds(10);
        state.timer.toggle(100);
        state.timer.tick(2600);
        store
            .session_store()
            .save(&app_launcher::session::Session::capture(&state))
            .unwrap();
        let mut hal = runtime().hal;
        hal.wall = 0;
        hal.now = 200;
        let mut r = DeviceRuntime::new(hal, path.join("sd"), path.join("flash"));
        r.tick();
        {
            let s = r.state.lock().unwrap();
            assert_eq!(s.unix_ms, 1_790_800_000_000);
            assert!(s.time_estimated && s.time_valid);
            assert_eq!(s.timer.remaining_ms, 7500);
            assert!(!s.timer.is_running());
            assert_eq!(s.active_app.id(), Some("timer"));
        }
        r.control(1, br#"{"op":"hello","request_id":1,"version":1}"#);
        r.hal.now = 1200;
        r.tick();
        assert_eq!(r.state.lock().unwrap().unix_ms, 1_790_800_001_000);
        assert!(r.hal.messages.iter().any(|(_, m)| matches!(
            m,
            DeviceMessage::Status {
                time_valid: false,
                ..
            }
        )));
        r.hal.wall = 1_790_900_000_000;
        r.hal.now = 1300;
        r.tick();
        {
            let s = r.state.lock().unwrap();
            assert!(!s.time_estimated);
            assert_eq!(s.unix_ms, r.hal.wall);
            assert_eq!(s.timer.remaining_ms, 7500);
            assert!(!s.timer.is_running());
        }
        assert!(r.hal.messages.iter().any(|(_, m)| matches!(
            m,
            DeviceMessage::Status {
                time_valid: true,
                ..
            }
        )));
        drop(r);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn appearance_is_durable_and_failure_keeps_the_previous_selection() {
        let mut r = runtime();
        r.state.lock().unwrap().queue(UiCommand::Appearance {
            light: true,
            glass: 25,
        });
        r.process_commands();
        assert!(r.state.lock().unwrap().settings.light_appearance);
        assert_eq!(r.store.load_settings().glass_amount, 25);
        assert!(r.store.load_settings().light_appearance);
        r.state.lock().unwrap().queue(UiCommand::Appearance {
            light: false,
            glass: 255,
        });
        r.process_commands();
        assert!(r.state.lock().unwrap().settings.light_appearance);
        assert_eq!(r.store.load_settings().glass_amount, 25);
    }

    #[test]
    fn icon_theme_is_durable_and_failed_write_keeps_current_theme() {
        use app_launcher::icon_theme::IconTheme;
        let mut r = runtime();
        for theme in IconTheme::ALL {
            r.state.lock().unwrap().queue(UiCommand::IconTheme(theme));
            r.process_commands();
            assert_eq!(r.store.load_settings().icon_theme, theme);
            assert_eq!(r.state.lock().unwrap().settings.icon_theme, theme);
        }
        let blocked =
            std::env::temp_dir().join(format!("p4desk-theme-blocked-{}", std::process::id()));
        std::fs::write(&blocked, b"not a directory").unwrap();
        r.store = GenerationStore::new(blocked.join("sd"), blocked.join("local"));
        r.state
            .lock()
            .unwrap()
            .queue(UiCommand::IconTheme(IconTheme::Folio));
        r.process_commands();
        assert_eq!(
            r.state.lock().unwrap().settings.icon_theme,
            IconTheme::WhiteSur
        );
        std::fs::remove_file(blocked).unwrap();
    }
    #[test]
    fn icon_palette_is_independent_durable_and_preserved_on_write_failure() {
        let mut r = runtime();
        for (appearance, icons) in [(true, false), (false, true)] {
            r.state.lock().unwrap().queue(UiCommand::IconLight(icons));
            r.process_commands();
            r.state.lock().unwrap().queue(UiCommand::Appearance {
                light: appearance,
                glass: 65,
            });
            r.process_commands();
            let settings = r.store.load_settings();
            assert_eq!(settings.light_appearance, appearance);
            assert_eq!(settings.light_icons(), icons);
            assert_eq!(settings.icon_light_override, Some(icons));
        }
        let blocked =
            std::env::temp_dir().join(format!("p4desk-palette-blocked-{}", std::process::id()));
        std::fs::write(&blocked, b"not a directory").unwrap();
        r.store = GenerationStore::new(blocked.join("sd"), blocked.join("local"));
        r.state.lock().unwrap().queue(UiCommand::IconLight(false));
        r.process_commands();
        assert!(r.state.lock().unwrap().settings.light_icons());
        std::fs::remove_file(blocked).unwrap();
    }
}
