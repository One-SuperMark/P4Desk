use app_launcher::{GenerationStore, LauncherState, UiCommand};
use app_launcher::launcher_state::{SystemError, SystemNotice};
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
    fn usage_transport(&mut self) -> Option<Box<dyn app_launcher::usage::api::Transport + Send>> { None }
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
    usage: Option<crate::usage::Service>,
    files: Option<crate::files::Service>,
    file_upload_active: bool,
    file_upload_activity: u64,
    file_upload_length: u64,
    file_verifying: bool,
    file_replies: std::collections::VecDeque<(u16, u32, DeviceMessage)>,
}

fn battery_ui_changed(
    state: &LauncherState,
    before: (Option<u8>, app_launcher::battery::ChargeState, Option<u16>),
) -> bool {
    use app_launcher::{status_bar::StatusPanelKind, ActiveApp};
    if state.mode != Mode::Pad || !state.settings.screen_on {
        return false;
    }
    let launching = state.app_launch.frame(state.monotonic_ms).is_some();
    if !matches!(state.active_app, ActiveApp::Launcher) && !launching {
        return false;
    }
    // The passive rail paints percentage only. Voltage continues to be
    // sampled in every application, but its ADC/filter noise must not rebuild
    // pages that have no battery UI. During a desktop launch retain a
    // conservative refresh for the visible cached backdrop.
    let percent_changed = before.0 != state.battery.percent;
    if launching {
        return percent_changed || before.1 != state.battery.charge;
    }
    if !state.status_panel_open {
        return percent_changed;
    }
    match state.status_panel_kind {
        StatusPanelKind::Device => {
            percent_changed
                || before.1 != state.battery.charge
                || before.2.map(|mv| mv / 10) != state.battery.voltage_mv.map(|mv| mv / 10)
        }
        StatusPanelKind::Control => percent_changed || before.1 != state.battery.charge,
        StatusPanelKind::Wifi => percent_changed,
    }
}
fn publish_persistence_status(
    state: &mut LauncherState,
    next: app_launcher::session::PersistenceStatus,
) {
    let changed = state.persistence_status != next;
    state.persistence_status = next;
    if changed && next == app_launcher::session::PersistenceStatus::Failed {
        state.show_error(SystemError::SaveFailed);
        return;
    }
    if changed
        && state.mode == Mode::Pad
        && state.settings.screen_on
        && matches!(state.active_app, app_launcher::ActiveApp::Settings)
        && state.settings_view.section == app_launcher::radio::SettingsSection::Storage
    {
        state.changed();
    }
}

impl<H: Hal> DeviceRuntime<H> {
    pub fn new(mut hal: H, root: impl Into<PathBuf>, local_root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let files_root = root.parent().unwrap_or(&root).to_path_buf();
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
                Err(_) => { state.show_error(SystemError::ResourceRecoveryFailed); },
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
                state.show_error(SystemError::SessionRecoveryFailed);
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
            state.show_error(SystemError::SaveFailed);
        }
        crate::diagnostics::diagnostic!(
            "p4desk_session: writer={}",
            if session_writer.is_some() {
                "ready"
            } else {
                "unavailable"
            }
        );
        let monitor_store = store.monitor_store();
        match monitor_store.load_config() {
            Ok(config) => state.usage.config = config,
            Err(_) => state.usage.status = "监控配置读取失败，请重新配置".into(),
        }
        let usage = hal.usage_transport().and_then(|transport| crate::usage::Service::new(transport, monitor_store).ok());
        let files = crate::files::Service::new(files_root).ok();
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
            usage,
            files,
            file_upload_active: false,
            file_upload_activity: 0,
            file_upload_length: 0,
            file_verifying: false,
            file_replies: std::collections::VecDeque::new(),
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
            HostMessage::FileList { path, offset, limit, .. } => {
                self.file_job(id, crate::files::HostJob::List { path, offset, limit });
            }
            HostMessage::FileMkdir { path, .. } => {
                self.file_job(id, crate::files::HostJob::Mkdir { path });
            }
            HostMessage::FileUploadBegin { path, length, sha256, .. } => {
                if self.file_upload_active { self.ack(id, "file_upload_begin", Err("busy"), None); }
                else if self.store.pending_generation().is_some() { self.ack(id, "file_upload_begin", Err("sync_busy"), None); }
                else if self.hal.sd_free_bytes() < length.saturating_add(128 * 1024) { self.ack(id, "file_upload_begin", Err("no_space"), None); }
                else if self.file_job(id, crate::files::HostJob::Begin { path, length, sha256 }) {
                    self.file_upload_active = true;
                    self.file_upload_length = length;
                    self.file_verifying = false;
                    self.file_upload_activity = self.hal.monotonic_ms();
                }
            }
            HostMessage::FileUploadCommit { .. } => {
                if self.file_job(id, crate::files::HostJob::Commit) { self.file_verifying = true; }
            }
            HostMessage::FileUploadAbort { .. } => { self.file_job(id, crate::files::HostJob::Abort); }
            HostMessage::FileFontInstall { path, sha256, .. } => { self.file_job(id, crate::files::HostJob::FontInstall { path, sha256 }); }
            HostMessage::MonitorGetStatus { .. } => {
                let state = self.state.lock().unwrap();
                self.hal.send(&DeviceMessage::MonitorStatus { request_id: id,
                    configured: state.usage.config.is_some(), busy: state.usage.busy,
                    configuration_result: state.usage.configuration_result, message: state.usage.status.clone() }, id);
            }
            HostMessage::MonitorConfigure { site, key, .. } => {
                let result = if self.usage.is_none() { Err("monitor_unavailable") }
                    else if self.state.lock().unwrap().usage.busy { Err("monitor_busy") }
                    else { app_launcher::usage::Config::new(&site, &key.0).map(|c| {
                        let mut state = self.state.lock().unwrap();
                        self.usage.as_mut().unwrap().command(&mut state, app_launcher::usage::Command::Save(c));
                    }).map_err(|_|"monitor_config_invalid") };
                // ACK means queued. Poll monitor_get_status for validated persistence.
                self.ack(id, "monitor_configure", result, None);
            }
            HostMessage::MonitorForget { .. } => {
                let result = if let Some(service) = &mut self.usage {
                    let mut state = self.state.lock().unwrap();
                    service.command(&mut state, app_launcher::usage::Command::Forget); Ok(())
                } else { Err("monitor_unavailable") };
                self.ack(id, "monitor_forget", result, None);
            }
            HostMessage::Hello { version, .. } => {
                if version != PROTOCOL_VERSION {
                    self.hello = false;
                    self.abort_files();
                    self.ack(id, "hello", Err("version"), None);
                    return;
                }
                // A new companion handshake is a fresh file session, even if the
                // physical USB configuration stayed mounted between processes.
                self.abort_files();
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
                        file_transfer: self.files.is_some(),
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
                let result = if self.file_upload_active {
                    Err("files_busy")
                } else if self.store.pending_generation().is_some() {
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
                let result = if self.file_upload_active {
                    Err("files_busy")
                } else if !self.hal.sd_ready() {
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
                        state.show_notice(SystemNotice::NotesSynced);
                        Ok(())
                    }
                    Err(code) => {
                        let mut state = self.state.lock().unwrap();
                        state.show_error(SystemError::SyncFailed);
                        Err(code)
                    }
                };
                let errno = if result.is_ok() { 0 } else { std::io::Error::last_os_error().raw_os_error().unwrap_or(0) };
                crate::diagnostics::diagnostic!("p4desk_sync: commit ok={} code={} errno={}", result.is_ok(), result.err().unwrap_or("none"), errno);
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
        if sequence > 1023 { return; }
        if self.file_upload_active {
            if bytes.len() < 5 || bytes.len() > p4desk_protocol::MAX_RESOURCE {
                self.ack(sequence, "file_chunk", Err("resource_length"), None);
                self.abort_files();
            } else {
                self.file_job(sequence, crate::files::HostJob::Chunk {
                    offset: u32::from_le_bytes(bytes[..4].try_into().unwrap()), bytes: bytes[4..].to_vec(),
                });
            }
            return;
        }
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
    fn file_job(&mut self, id: u16, job: crate::files::HostJob) -> bool {
        let op = job.op();
        let abort = matches!(job, crate::files::HostJob::Abort);
        let result = if !self.hello { Err("hello_required") }
            else if !abort && self.store.pending_generation().is_some() { Err("sync_busy") }
            else if !abort && !self.hal.sd_ready() { Err("storage_unavailable") }
            else if !abort && (self.hal.mode() != Mode::Pad || self.hal.display_transition_pending()
                || self.state.lock().unwrap().display_launch_waiting()) { Err("display_busy") }
            else if let Some(files) = &self.files { files.host(id, job, self.hal.sd_ready()).map_err(|e| e.code()) }
            else { Err("files_unavailable") };
        if let Err(code) = result { self.ack(id, op, Err(code), None); false }
        else { self.file_upload_activity = self.hal.monotonic_ms(); true }
    }
    fn abort_files(&mut self) {
        self.file_upload_active = false;
        self.file_verifying = false;
        self.file_replies.clear();
        if let Some(files) = &self.files { files.disconnect(); }
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
            self.abort_files();
        }
        if self.was_active && !active {
            self.hello = false;
            let _ = self.store.abort();
            self.abort_files();
        }
        self.was_active = active;
        self.was_connected = connected;
        let file_timeout = if self.file_verifying { 30_000 + self.file_upload_length.div_ceil(512 * 1024) * 1000 } else { 15_000 };
        if self.file_upload_active && (!sd_ready || now.saturating_sub(self.file_upload_activity) > file_timeout) {
            self.abort_files();
        }
        if self.store.pending_generation().is_some()
            && (!sd_ready || now.saturating_sub(self.pending_activity) > 15_000)
        {
            let _ = self.store.abort();
        }
        let mut state = self.state.lock().unwrap();
        let before = state.revision;
        if let Some(files) = &self.files {
            for completion in files.poll() {
                match completion {
                    crate::files::Completion::Ui(response) => {
                        let refresh = matches!(response.result, Ok(app_launcher::files::Outcome::Changed { .. }));
                        if state.apply_files_response(response) {
                            if refresh {
                                let request = state.files.refresh_request_silent();
                                let revision = request.revision;
                                if let Err(error) = files.ui(request, sd_ready) {
                                    state.apply_files_response(app_launcher::files::Response { revision, result: Err(error) });
                                }
                            }
                            if matches!(state.active_app, app_launcher::ActiveApp::Files) { state.changed(); }
                        }
                    }
                    crate::files::Completion::Host { id, epoch, message, clear_upload, changed } => {
                        if epoch == files.epoch() && self.hello {
                            if clear_upload { self.file_upload_active = false; self.file_verifying = false; }
                            if self.file_replies.len() < 16 { self.file_replies.push_back((id, epoch, message)); }
                            if changed && matches!(state.active_app, app_launcher::ActiveApp::Files) && !state.files.busy {
                                let request = state.files.refresh_request_silent();
                                let revision = request.revision;
                                if let Err(error) = files.ui(request, sd_ready) { state.apply_files_response(app_launcher::files::Response { revision, result: Err(error) }); }
                                state.changed();
                            }
                        }
                    }
                }
            }
            // A persisted foreground entry does not call open_app(), so initialize its directory here.
            if matches!(state.active_app, app_launcher::ActiveApp::Files)
                && state.files.listing.is_none() && !state.files.busy && state.files.error.is_none() {
                let request = state.files.refresh_request_silent();
                let revision = request.revision;
                if let Err(error) = files.ui(request, sd_ready) { state.apply_files_response(app_launcher::files::Response { revision, result: Err(error) }); }
                state.changed();
            }
        }
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
            let battery_before = (state.battery.percent, state.battery.charge, state.battery.voltage_mv);
            state.battery.update(self.hal.battery_reading());
            if battery_ui_changed(&state, battery_before) {
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
        if let Some(usage) = &mut self.usage { usage.poll(&mut state); }
        if let Some(writer) = &mut self.session_writer {
            if writer.due(now) {
                let snapshot = app_launcher::session::Session::capture(&state);
                let animating = state.app_launch.frame(now).is_some()
                    || state.timer_completion.progress(now).is_some();
                if let Some(ok) = writer.sample(now, snapshot, animating) {
                    let status = if ok {
                        app_launcher::session::PersistenceStatus::Saved
                    } else {
                        app_launcher::session::PersistenceStatus::Failed
                    };
                    publish_persistence_status(&mut state, status);
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
        while let Some((id, epoch, message)) = self.file_replies.front() {
            if !self.hello || self.files.as_ref().is_none_or(|f| f.epoch() != *epoch) { self.file_replies.pop_front(); continue; }
            if !self.hal.send(message, *id) { break; }
            self.file_replies.pop_front();
        }
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
                UiCommand::DismissError { id } => {
                    self.state.lock().unwrap().dismiss_dialog(id);
                    Ok(())
                }
                UiCommand::Files(request) => {
                    let revision = request.revision;
                    let result = if let Some(files) = &self.files { files.ui(request, self.hal.sd_ready()) }
                        else { Err(app_launcher::files::FileError::StorageUnavailable) };
                    if let Err(error) = result {
                        let mut state = self.state.lock().unwrap();
                        state.apply_files_response(app_launcher::files::Response { revision, result: Err(error) }); state.changed();
                    }
                    Ok(())
                }
                UiCommand::Usage(command) => {
                    let mut state = self.state.lock().unwrap();
                    if let Some(usage) = &mut self.usage { usage.command(&mut state, command); }
                    else { state.usage.status.clear(); state.show_error(SystemError::MonitorUnavailable); }
                    Ok(())
                }
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
                    if self.file_upload_active {
                        Err("files_busy")
                    } else if !self.hal.host_active() {
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
                    if mode == Mode::Display && self.file_upload_active {
                        Err("files_busy")
                    } else if !self.hal.host_active() {
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
                let error = match code {
                    "radio_busy" => SystemError::WirelessBusy,
                    "mac_offline" => SystemError::MacDisconnected,
                    "note_not_found" => SystemError::NoteNotFound,
                    "storage_create" | "state_write" | "state_fsync" | "state_rename" => {
                        SystemError::SaveFailed
                    }
                    _ => SystemError::OperationFailed,
                };
                state.show_error(error);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn committed_sync_opens_one_dismissible_notice_without_a_persistent_banner() {
        use app_launcher::launcher_state::DialogContent;
        use sha2::{Digest, Sha256};
        use tiny_flutter::graphics::fontpack::{encode_fontpack, PackGlyph};
        let mut r = runtime();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("p4desk-sync-notice-{}-{nonce}", std::process::id()));
        r.store = GenerationStore::new(path.join("sd"), path.join("flash"));
        r.hal.sd = true;
        {
            let mut state = r.state.lock().unwrap();
            state.open_app("clock");
            state.take_commands();
        }
        let font = encode_fontpack([18, 22, 28, 36].into_iter().map(|size| PackGlyph {
            character: 'a', size, width: 1, height: 1, xmin: 0, ymin: 0,
            advance: 1.0, bitmap: vec![255],
        }).collect()).unwrap();
        let snapshot = p4desk_protocol::Snapshot { generation: 1, ..Default::default() };
        file_request(&mut r, HostMessage::Hello { request_id: 10, version: 1 });
        file_request(&mut r, HostMessage::SyncBegin {
            request_id: 11, generation: 1, state: snapshot.clone(),
            font_length: font.len() as u32, font_sha256: format!("{:x}", Sha256::digest(&font)),
        });
        let mut chunk = 0u32.to_le_bytes().to_vec();
        chunk.extend_from_slice(&font);
        r.resource(12, &chunk);
        file_request(&mut r, HostMessage::SyncCommit { request_id: 13, generation: 1 });
        assert!(matches!(r.hal.messages.iter().find(|(id, _)| *id == 13),
            Some((_, DeviceMessage::Ack { ok: true, .. }))));
        let mut state = r.state.lock().unwrap();
        let dialog = state.active_dialog().unwrap();
        assert_eq!(dialog.content, DialogContent::Notice(SystemNotice::NotesSynced));
        assert!(state.error_dialog.is_none());
        assert!(state.notice.is_empty());
        assert_eq!(state.snapshot, snapshot);
        assert!(matches!(state.active_app, app_launcher::ActiveApp::Clock));
        state.queue(UiCommand::DismissError { id: dialog.id });
        drop(state);
        r.process_commands();
        r.hal.now = 10_000;
        r.tick();
        let state = r.state.lock().unwrap();
        assert!(state.active_dialog().is_none());
        assert!(state.notice.is_empty());
        assert_eq!(state.snapshot, snapshot);
        drop(state);
        let mut recovery = GenerationStore::new(path.join("sd"), path.join("flash"));
        assert_eq!(recovery.restore().unwrap().unwrap().state, snapshot);
        drop(r);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn sync_failure_opens_dismissible_error_and_preserves_snapshot() {
        let mut r = runtime();
        r.state.lock().unwrap().snapshot.notes.push(p4desk_protocol::Note {
            id: "retained".into(), title: "synthetic".into(), body: "fixture".into(), updated_ms: 0,
        });
        let before = r.state.lock().unwrap().snapshot.clone();
        r.control(0, br#"{"op":"hello","request_id":0,"version":1}"#);
        r.control(1, br#"{"op":"sync_commit","request_id":1,"generation":1}"#);
        let mut state = r.state.lock().unwrap();
        assert_eq!(state.snapshot, before);
        assert!(state.notice.is_empty());
        let dialog = state.error_dialog.unwrap();
        assert_eq!(dialog.error, SystemError::SyncFailed);
        state.queue(UiCommand::DismissError { id: dialog.id });
        drop(state);
        r.process_commands();
        assert!(r.state.lock().unwrap().error_dialog.is_none());
        assert_eq!(r.state.lock().unwrap().snapshot, before);
        assert!(matches!(r.hal.messages.iter().find(|(id, _)| *id == 1).map(|(_, message)| message),
            Some(DeviceMessage::Ack { ok: false, error: Some(code), .. }) if code == "sync_not_started"));
    }
    fn file_request(r: &mut DeviceRuntime<Mock>, request: HostMessage) {
        r.control(request.request_id(), &serde_json::to_vec(&request).unwrap());
    }
    fn wait_file_reply(r: &mut DeviceRuntime<Mock>, id: u16) -> DeviceMessage {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            r.tick();
            if let Some(index) = r.hal.messages.iter().position(|(sequence, _)| *sequence == id) { return r.hal.messages.remove(index).1; }
            assert!(std::time::Instant::now() < deadline, "file reply timed out");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[test]
    fn usb_file_ack_tracks_persisted_completion_and_display_sync_exclusion() {
        use sha2::{Digest, Sha256};
        let mut r = runtime();
        let path = std::env::temp_dir().join(format!("p4desk-file-runtime-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        r.files = Some(crate::files::Service::new(path.clone()).unwrap()); r.hal.sd = true;
        file_request(&mut r, HostMessage::Hello { request_id: 10, version: 1 });
        assert!(matches!(wait_file_reply(&mut r, 10), DeviceMessage::Caps { file_transfer: true, .. }));
        let payload = b"file runtime fixture";
        file_request(&mut r, HostMessage::FileUploadBegin { request_id: 11, path: "Downloads/runtime.txt".into(), length: payload.len() as u64, sha256: format!("{:x}", Sha256::digest(payload)) });
        assert!(matches!(wait_file_reply(&mut r, 11), DeviceMessage::Ack { ok: true, .. }));
        file_request(&mut r, HostMessage::SetMode { request_id: 12, mode: Mode::Display, session: 1, jpeg_rotation_degrees: 0 });
        assert!(matches!(wait_file_reply(&mut r, 12), DeviceMessage::Ack { ok: false, error: Some(e), .. } if e == "files_busy"));
        file_request(&mut r, HostMessage::SyncBegin { request_id: 13, generation: 1, state: Default::default(), font_length: 10, font_sha256: String::new() });
        assert!(matches!(wait_file_reply(&mut r, 13), DeviceMessage::Ack { ok: false, error: Some(e), .. } if e == "files_busy"));
        let mut chunk = 0u32.to_le_bytes().to_vec(); chunk.extend_from_slice(payload);
        r.resource(14, &chunk);
        assert!(matches!(wait_file_reply(&mut r, 14), DeviceMessage::Ack { acknowledged, ok: true, .. } if acknowledged == "file_chunk"));
        assert!(!path.join("Downloads/runtime.txt").exists());
        file_request(&mut r, HostMessage::FileUploadCommit { request_id: 15 });
        assert!(matches!(wait_file_reply(&mut r, 15), DeviceMessage::Ack { ok: true, .. }));
        assert_eq!(std::fs::read(path.join("Downloads/runtime.txt")).unwrap(), payload);
        assert!(!r.file_upload_active);
        file_request(&mut r, HostMessage::FileList { request_id: 16, path: "Downloads".into(), offset: 0, limit: 32 });
        assert!(matches!(wait_file_reply(&mut r, 16), DeviceMessage::FileListing { total: 1, .. }));
        drop(r); std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn unavailable_file_storage_is_explicit_and_no_ui_filesystem_work_runs() {
        let mut r = runtime();
        file_request(&mut r, HostMessage::Hello { request_id: 10, version: 1 });
        wait_file_reply(&mut r, 10);
        file_request(&mut r, HostMessage::FileList { request_id: 11, path: String::new(), offset: 0, limit: 32 });
        assert!(matches!(wait_file_reply(&mut r, 11), DeviceMessage::Ack { ok: false, error: Some(e), .. } if e == "storage_unavailable"));
        r.state.lock().unwrap().open_app("file-manager");
        r.process_commands();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while r.state.lock().unwrap().files.busy {
            r.tick(); assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(r.state.lock().unwrap().files.error, Some(app_launcher::files::FileError::StorageUnavailable));
    }
    struct Mock {
        sd: bool,
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
            self.sd
        }
        fn sd_free_bytes(&self) -> u64 {
            if self.sd { 1024 * 1024 * 1024 } else { 0 }
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
                sd: false,
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

    fn telemetry_runtime(app: &str) -> DeviceRuntime<Mock> {
        let mut r = runtime();
        r.session_writer = None;
        r.hal.now = 10_000;
        r.hal.wall = 1_791_287_400_000;
        r.hal.battery = app_launcher::battery::BatteryReading {
            voltage_mv: Some(3900),
            charge: app_launcher::battery::ChargeState::Unknown,
        };
        r.state.lock().unwrap().open_app(app);
        r.tick();
        r
    }
    /// Advance unrelated wall-clock UI first, then measure the actual runtime
    /// poll/result path independently of ordinary per-second clock revisions.
    fn isolated_runtime_tick(r: &mut DeviceRuntime<Mock>, now: u64) -> bool {
        r.hal.wall += now.saturating_sub(r.hal.now) as i64;
        r.hal.now = now;
        r.state.lock().unwrap().tick(r.hal.now, r.hal.wall);
        r.tick()
    }
    #[test]
    fn background_battery_voltage_and_charge_updates_do_not_rebuild_application_pages() {
        use app_launcher::battery::ChargeState;
        for app in [
            "sub2api-monitor",
            "clock",
            "timer",
            "settings",
            "mac",
            "notes",
            "calculator",
            "file-manager",
            "office-viewer",
        ] {
            let mut r = telemetry_runtime(app);
            r.hal.battery.voltage_mv = Some(3908);
            assert!(
                !isolated_runtime_tick(&mut r, 12_000),
                "hidden battery invalidated {app}"
            );
            let s = r.state.lock().unwrap();
            assert_eq!(
                s.battery.voltage_mv,
                Some(3902),
                "telemetry must still advance"
            );
            assert_eq!(s.battery.percent, Some(70));
            drop(s);
            r.hal.battery.charge = ChargeState::PluggedInAssumed;
            assert!(
                !isolated_runtime_tick(&mut r, 14_000),
                "hidden charging state invalidated {app}"
            );
            assert_eq!(
                r.state.lock().unwrap().battery.charge,
                ChargeState::PluggedInAssumed
            );
        }
    }
    #[test]
    fn desktop_battery_refreshes_visible_values_and_device_detail_voltage() {
        use app_launcher::{battery::ChargeState, status_bar::StatusPanelKind, ActiveApp};
        let mut r = telemetry_runtime("sub2api-monitor");
        r.state.lock().unwrap().active_app = ActiveApp::Launcher;
        r.hal.battery.voltage_mv = Some(3908);
        assert!(
            !isolated_runtime_tick(&mut r, 12_000),
            "rail does not show raw ADC voltage"
        );
        {
            let mut s = r.state.lock().unwrap();
            s.status_panel_open = true;
            s.status_panel_kind = StatusPanelKind::Device;
        }
        r.hal.battery.voltage_mv = Some(3940);
        assert!(
            isolated_runtime_tick(&mut r, 14_000),
            "visible 0.01V detail must update"
        );
        assert_eq!(
            r.state.lock().unwrap().battery.percent,
            Some(70),
            "test isolates voltage from percentage"
        );
        r.state.lock().unwrap().status_panel_kind = StatusPanelKind::Control;
        r.hal.battery.voltage_mv = Some(3910);
        assert!(
            !isolated_runtime_tick(&mut r, 16_000),
            "control center does not show voltage"
        );
        r.hal.battery.charge = ChargeState::PluggedInAssumed;
        assert!(
            isolated_runtime_tick(&mut r, 18_000),
            "control center charge label must update"
        );
        r.state.lock().unwrap().status_panel_open = false;
        r.hal.battery.voltage_mv = Some(4200);
        assert!(
            isolated_runtime_tick(&mut r, 20_000),
            "rail percentage must update"
        );
        r.state.lock().unwrap().settings.screen_on = false;
        r.hal.battery.voltage_mv = None;
        assert!(
            !isolated_runtime_tick(&mut r, 22_000),
            "screen-off telemetry must not rebuild"
        );
        assert_eq!(r.state.lock().unwrap().battery.percent, None);
    }
    #[test]
    fn launch_backdrop_conservatively_keeps_visible_battery_updates() {
        use app_launcher::battery::ChargeState;
        let mut r = telemetry_runtime("sub2api-monitor");
        r.state.lock().unwrap().app_launch.start(
            "sub2api-monitor",
            tiny_flutter::Rect::from_ltwh(100., 100., 100., 100.),
            10_000,
        );
        r.last_battery_poll_ms = None;
        r.hal.battery.voltage_mv = Some(4200);
        assert!(
            isolated_runtime_tick(&mut r, 10_010),
            "launch still includes a desktop backdrop"
        );
        r.state.lock().unwrap().app_launch.cancel();
        r.last_battery_poll_ms = None;
        r.hal.battery.charge = ChargeState::PluggedInAssumed;
        assert!(
            !isolated_runtime_tick(&mut r, 10_020),
            "settled application hides battery UI"
        );
    }
    #[test]
    fn checkpoint_success_only_updates_visible_storage_but_failure_opens_error() {
        use app_launcher::{radio::SettingsSection, session::PersistenceStatus};
        for (app, storage, fail) in [
            ("sub2api-monitor", false, false),
            ("sub2api-monitor", false, true),
            ("settings", true, false),
            ("settings", true, true),
            ("settings", false, true),
        ] {
            let mut r = telemetry_runtime(app);
            if storage {
                r.state.lock().unwrap().settings_view.section = SettingsSection::Storage;
            }
            let blocked = std::env::temp_dir().join(format!(
                "p4desk-blocked-session-{}-{}-{}",
                std::process::id(),
                app,
                storage
            ));
            let store = if fail {
                let _ = std::fs::remove_file(&blocked);
                std::fs::write(&blocked, b"synthetic").unwrap();
                GenerationStore::new(blocked.join("sd"), &blocked).session_store()
            } else {
                r.store.session_store()
            };
            r.session_writer = Some(crate::persistence::SessionWriter::new(store.clone()).unwrap());
            r.state.lock().unwrap().persistence_status = PersistenceStatus::Pending;
            assert!(!isolated_runtime_tick(&mut r, 11_000));
            assert!(!isolated_runtime_tick(&mut r, 12_000));
            let expected = if fail {
                PersistenceStatus::Failed
            } else {
                PersistenceStatus::Saved
            };
            let mut completed = false;
            for i in 0..100 {
                std::thread::sleep(std::time::Duration::from_millis(2));
                let changed = isolated_runtime_tick(&mut r, 12_500 + i * 500);
                if r.state.lock().unwrap().persistence_status == expected {
                    assert_eq!(
                        changed, storage || fail,
                        "success updates visible Storage; a failure must show its alert"
                    );
                    assert_eq!(r.state.lock().unwrap().error_dialog.map(|d| d.error), fail.then_some(SystemError::SaveFailed));
                    completed = true;
                    break;
                }
                assert!(!changed);
            }
            assert!(
                completed,
                "real checkpoint worker did not acknowledge its result"
            );
            if !fail {
                assert!(
                    store.load().unwrap().is_some(),
                    "status must represent a durable checkpoint"
                );
            }
            let revision = r.state.lock().unwrap().revision;
            publish_persistence_status(&mut r.state.lock().unwrap(), expected);
            assert_eq!(
                r.state.lock().unwrap().revision,
                revision,
                "same status must not invalidate again"
            );
            r.session_writer = None;
            if fail {
                let _ = std::fs::remove_file(blocked);
            }
        }
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
        let mut state = r.state.lock().unwrap();
        // This is the failed-creation page state, never a global status banner.
        assert_eq!(state.notice, "操作未完成，请重试");
        assert_eq!(state.error_dialog.unwrap().error, SystemError::OperationFailed);
        let id = state.active_dialog().unwrap().id;
        assert!(state.dismiss_dialog(id));
        assert_eq!(state.active_app.id(), Some("display"));
        assert_eq!(state.notice, "操作未完成，请重试");
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
                if let Some(dialog) = state.error_dialog { state.dismiss_error(dialog.id); }
                state.queue(command);
                state.revision
            };
            assert!(r.process_commands());
            let state = r.state.lock().unwrap();
            assert!(state.notice.is_empty());
            assert_eq!(state.error_dialog.unwrap().error, SystemError::OperationFailed);
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
