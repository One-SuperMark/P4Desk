use crate::app_launch::{
    AppLaunchState, DesktopBackdropCache, APP_LAUNCH_DURATION_MS, REVEAL_START_MS,
};
use crate::flip_clock::FlipClockState;
use crate::storage::LocalSettings;
use crate::timer::TimerService;
use crate::timer_completion::TimerCompletionState;
use calculator::CalcState;
use p4desk_protocol::{Mode, Snapshot};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tiny_flutter::{PageController, Rect, ScrollController, Size};

pub const APP_IDS: [&str; 9] = [
    "clock",
    "timer",
    "notes",
    "calculator",
    "mac",
    "settings",
    "file-manager",
    "office-viewer",
    "sub2api-monitor",
];
pub const MAX_BACKGROUND_APPS: usize = 4;
pub const MAX_RECENT_APPS: usize = 4;
#[derive(Clone)]
pub enum ActiveApp {
    Launcher,
    Clock,
    Timer,
    Notes(Arc<Mutex<NotesView>>),
    Calculator(Arc<Mutex<CalcState>>),
    MacControls,
    Settings,
    Usage,
    Planned(crate::planned_apps::PlannedApp),
    /// Temporary handoff page, never stored as a background app.
    DisplaySetup,
}
impl ActiveApp {
    pub fn id(&self) -> Option<&'static str> {
        match self {
            Self::Launcher => None,
            Self::Clock => Some("clock"),
            Self::Timer => Some("timer"),
            Self::Notes(_) => Some("notes"),
            Self::Calculator(_) => Some("calculator"),
            Self::MacControls => Some("mac"),
            Self::Settings => Some("settings"),
            Self::Usage => Some("sub2api-monitor"),
            Self::Planned(app) => Some(app.id()),
            Self::DisplaySetup => Some("display"),
        }
    }
}
pub struct NotesView {
    pub selected: usize,
    pub scroll: ScrollController,
    pub confirm_delete: bool,
}
impl Default for NotesView {
    fn default() -> Self {
        Self {
            selected: 0,
            scroll: ScrollController::new(),
            confirm_delete: false,
        }
    }
}
#[derive(Debug, Clone)]
pub enum UiCommand {
    Usage(crate::usage::Command),
    Radio(crate::radio::RadioCommand),
    DeleteNote(String),
    Action(String),
    Media(u16),
    Brightness(u8),
    Appearance { light: bool, glass: u8 },
    IconTheme(crate::icon_theme::IconTheme),
    IconLight(bool),
    Screen(bool),
    RequestTimeSync,
    SetTime(i64, i32),
    RequestMode(Mode),
    StartDisplayTransition { duration_ms: u32 },
    CancelDisplayTransition,
}

pub struct LauncherState {
    pub current_page: usize,
    pub total_pages: usize,
    pub active_app: ActiveApp,
    pub running_apps: HashMap<String, ActiveApp>,
    pub(crate) background_order: VecDeque<String>,
    pub(crate) recent_apps: VecDeque<&'static str>,
    pub page_controller: PageController,
    pub snapshot: Snapshot,
    pub timer: TimerService,
    pub timer_completion: TimerCompletionState,
    pub app_launch: AppLaunchState,
    pub desktop_backdrop: DesktopBackdropCache,
    pub(crate) control_center_backdrop: crate::control_center_glass::Backdrop,
    pub settings: LocalSettings,
    pub radio: crate::radio::RadioSnapshot,
    pub settings_view: crate::radio::SettingsView,
    pub usage: crate::usage::State,
    pub usb_connected: bool,
    pub connected: bool,
    pub sd_ready: bool,
    pub battery: crate::battery::BatteryState,
    pub status_panel_open: bool,
    pub status_panel_kind: crate::status_bar::StatusPanelKind,
    pub reset_reason: u32,
    pub time_valid: bool,
    /// A recovered wall clock cannot account for time spent without power.
    pub time_estimated: bool,
    pub persistence_status: crate::session::PersistenceStatus,
    pub mode: Mode,
    pub unix_ms: i64,
    pub monotonic_ms: u64,
    pub clock: String,
    pub flip_clock: FlipClockState,
    pub date: String,
    pub notice: String,
    pub mac_page: usize,
    pub revision: u64,
    pub manual_time_open: bool,
    pub manual_clock: crate::manual_clock::ManualClock,
    commands: VecDeque<UiCommand>,
    display_request_pending: bool,
    display_started_connected: bool,
    display_wait_since: Option<u64>,
    last_second: u64,
    last_unix_second: Option<i64>,
}
impl Default for LauncherState {
    fn default() -> Self {
        Self::new()
    }
}
impl LauncherState {
    pub fn new() -> Self {
        Self {
            current_page: 0,
            total_pages: 1,
            active_app: ActiveApp::Launcher,
            running_apps: HashMap::new(),
            background_order: VecDeque::new(),
            recent_apps: VecDeque::new(),
            page_controller: PageController::new(),
            snapshot: Snapshot::default(),
            timer: TimerService::default(),
            timer_completion: TimerCompletionState::default(),
            app_launch: AppLaunchState::default(),
            desktop_backdrop: Arc::new(Mutex::new(None)),
            control_center_backdrop: Arc::new(Mutex::new(None)),
            settings: LocalSettings::default(),
            radio: crate::radio::RadioSnapshot::default(),
            settings_view: crate::radio::SettingsView::default(),
            usage: crate::usage::State::default(),
            usb_connected: false,
            connected: false,
            sd_ready: false,
            battery: crate::battery::BatteryState::default(),
            status_panel_open: false,
            status_panel_kind: crate::status_bar::StatusPanelKind::Device,
            reset_reason: 0,
            time_valid: false,
            time_estimated: false,
            persistence_status: crate::session::PersistenceStatus::Pending,
            mode: Mode::Pad,
            unix_ms: 0,
            monotonic_ms: 0,
            clock: "--:--:--".into(),
            flip_clock: FlipClockState::default(),
            date: "等待 Mac 校时".into(),
            notice: String::new(),
            mac_page: 0,
            revision: 1,
            manual_time_open: false,
            manual_clock: crate::manual_clock::ManualClock::default(),
            commands: VecDeque::new(),
            display_request_pending: false,
            display_started_connected: false,
            display_wait_since: None,
            last_second: u64::MAX,
            last_unix_second: None,
        }
    }
    pub fn open_app(&mut self, id: &str) {
        self.status_panel_open = false;
        if self.active_app.id() == Some(id) {
            self.changed();
            return;
        }
        // Reserve the requested instance before hiding the foreground app.
        // Otherwise a full queue could evict the very app being resumed.
        let restored = self.running_apps.remove(id);
        self.background_order.retain(|key| key != id);
        self.background_active_app();
        self.active_app = restored.unwrap_or_else(|| match id {
            "clock" => ActiveApp::Clock,
            "timer" => ActiveApp::Timer,
            "notes" => ActiveApp::Notes(Arc::new(Mutex::new(NotesView::default()))),
            "calculator" => ActiveApp::Calculator(Arc::new(Mutex::new(CalcState::new()))),
            "mac" => ActiveApp::MacControls,
            "settings" => ActiveApp::Settings,
            "file-manager" => ActiveApp::Planned(crate::planned_apps::PlannedApp::Files),
            "office-viewer" => ActiveApp::Planned(crate::planned_apps::PlannedApp::Office),
            "sub2api-monitor" => ActiveApp::Usage,
            "display" => ActiveApp::DisplaySetup,
            _ => ActiveApp::Launcher,
        });
        if let Some(id) = self.active_app.id() {
            // History keeps identifiers only. Closing an application must still
            // release its instance while leaving a shortcut to reopen it.
            self.recent_apps.retain(|previous| *previous != id);
            self.recent_apps.push_back(id);
            while self.recent_apps.len() > MAX_RECENT_APPS {
                self.recent_apps.pop_front();
            }
        }
        self.flip_clock.snap(&self.clock);
        self.changed();
    }
    /// Oldest hidden application first; resuming and hiding again moves it last.
    pub fn background_app_ids(&self) -> impl Iterator<Item = &str> {
        self.background_order.iter().map(String::as_str)
    }
    /// Recent distinct apps, oldest first. Independent of live background instances.
    pub fn recent_app_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.recent_apps.iter().copied()
    }
    pub fn background_active_app(&mut self) {
        if matches!(self.active_app, ActiveApp::DisplaySetup) {
            self.commands.retain(|c| {
                !matches!(
                    c,
                    UiCommand::RequestMode(Mode::Display)
                        | UiCommand::StartDisplayTransition { .. }
                )
            });
            if self.display_wait_since.take().is_some() && self.mode == Mode::Pad {
                self.commands.push_back(UiCommand::CancelDisplayTransition);
            }
            if self.notice == "等待 Mac 应用" {
                self.notice.clear();
            }
        }
        self.settings_view.join = None;
        self.app_launch.cancel();
        self.display_request_pending = false;
        self.display_started_connected = false;
        self.desktop_backdrop.lock().unwrap().take();
        self.timer_completion.cancel();
        if let Some(id) = self.active_app.id().filter(|id| *id != "display") {
            self.background_order.retain(|key| key != id);
            self.running_apps.insert(id.into(), self.active_app.clone());
            self.background_order.push_back(id.into());
            while self.background_order.len() > MAX_BACKGROUND_APPS {
                if let Some(oldest) = self.background_order.pop_front() {
                    self.running_apps.remove(&oldest);
                    if oldest == "settings" {
                        self.manual_time_open = false;
                    }
                }
            }
        }
        self.active_app = ActiveApp::Launcher;
        self.flip_clock.snap(&self.clock);
        self.changed();
    }
    /// The original BackListener semantics, with the settings editor's nested route.
    pub fn back_active_app(&mut self) {
        if self.status_panel_open {
            self.status_panel_open = false;
            self.changed();
            return;
        }
        if matches!(self.active_app, ActiveApp::Settings) && self.manual_time_open {
            self.manual_time_open = false;
            self.changed();
        } else {
            self.background_active_app();
        }
    }
    pub fn kill_active_app(&mut self) {
        self.status_panel_open = false;
        if matches!(self.active_app, ActiveApp::DisplaySetup) {
            self.commands.retain(|c| {
                !matches!(
                    c,
                    UiCommand::RequestMode(Mode::Display)
                        | UiCommand::StartDisplayTransition { .. }
                )
            });
            if self.display_wait_since.take().is_some() && self.mode == Mode::Pad {
                self.commands.push_back(UiCommand::CancelDisplayTransition);
            }
            if self.notice == "等待 Mac 应用" {
                self.notice.clear();
            }
        }
        self.settings_view.join = None;
        self.app_launch.cancel();
        self.display_request_pending = false;
        self.display_started_connected = false;
        self.desktop_backdrop.lock().unwrap().take();
        self.timer_completion.cancel();
        if let Some(id) = self.active_app.id() {
            self.running_apps.remove(id);
            self.background_order.retain(|key| key != id);
        }
        if matches!(self.active_app, ActiveApp::Settings) {
            self.manual_time_open = false;
        }
        self.active_app = ActiveApp::Launcher;
        self.flip_clock.snap(&self.clock);
        self.changed();
    }
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    /// Desktop launches animate; direct opens and status-bar resumes retain
    /// their existing routing and preserved app instances.
    pub fn launch_app(&mut self, id: &str, source: Rect) {
        let desktop_clock = self
            .time_valid
            .then(|| crate::live_clock_icon::ClockTime::parse(&self.clock))
            .flatten();
        let desktop_dock = crate::folio_desktop::recent_apps(self);
        let backdrop = self.desktop_backdrop.lock().unwrap().take();
        self.open_app(id);
        if source.width > 0.0
            && source.height > 0.0
            && self.mode == Mode::Pad
            && self.settings.screen_on
        {
            if let Some(id) = self.active_app.id() {
                *self.desktop_backdrop.lock().unwrap() = backdrop;
                self.app_launch.start(id, source, self.monotonic_ms);
                self.app_launch.set_desktop_dock(desktop_dock);
                self.app_launch.set_desktop_clock(desktop_clock);
                if id == "display" {
                    self.notice.clear();
                    self.display_request_pending = true;
                    self.display_started_connected = self.connected;
                    if self.connected {
                        self.app_launch.hold_for_display();
                    }
                }
            }
        }
    }
    pub fn take_launch_animation_dirty(&mut self, size: Size) -> Option<Rect> {
        if self.mode != Mode::Pad && matches!(self.active_app, ActiveApp::DisplaySetup) {
            return None;
        }
        if self.mode != Mode::Pad || !self.settings.screen_on {
            self.app_launch.cancel();
            self.display_request_pending = false;
            self.desktop_backdrop.lock().unwrap().take();
            return None;
        }
        if self.active_app.id().is_none() {
            self.app_launch.cancel();
            // The firmware polls this every loop, including idle desktop loops.
            // Keep the prepared normal backdrop ready for the next icon tap.
            return None;
        }
        if self.display_request_pending
            && self.display_started_connected
            && self.connected
            && self
                .app_launch
                .frame(self.monotonic_ms)
                .is_some_and(|f| f.elapsed_ms >= REVEAL_START_MS)
        {
            self.display_request_pending = false;
            self.display_wait_since = Some(self.monotonic_ms);
            self.notice = "等待 Mac 应用".into();
            // The fully opaque bridge no longer needs the desktop allocation.
            self.desktop_backdrop.lock().unwrap().take();
            self.queue(UiCommand::StartDisplayTransition {
                duration_ms: (APP_LAUNCH_DURATION_MS - REVEAL_START_MS) as u32,
            });
        }
        let dirty = self.app_launch.take_dirty(self.monotonic_ms, size);
        if dirty.is_some() && self.app_launch.frame(self.monotonic_ms).is_none() {
            self.desktop_backdrop.lock().unwrap().take();
            if self.display_request_pending && matches!(self.active_app, ActiveApp::DisplaySetup) {
                self.display_request_pending = false;
                self.fail_display_launch("Mac 未连接，请连接 USB 和 Mac 应用");
            }
        }
        dirty
    }
    pub fn fail_display_launch(&mut self, notice: &str) {
        if matches!(self.active_app, ActiveApp::DisplaySetup) {
            self.app_launch
                .resume_after_display_failure(self.monotonic_ms);
            self.display_request_pending = false;
            self.display_started_connected = false;
            if self.display_wait_since.take().is_some() {
                self.commands
                    .retain(|c| !matches!(c, UiCommand::StartDisplayTransition { .. }));
                self.queue(UiCommand::CancelDisplayTransition);
            }
            self.notice = notice.into();
            self.changed();
        }
    }
    pub fn complete_display_launch(&mut self) {
        if matches!(self.active_app, ActiveApp::DisplaySetup) {
            self.display_wait_since = None;
            self.background_active_app();
        }
    }
    pub fn request_display_mode(&mut self) {
        if !matches!(self.active_app, ActiveApp::DisplaySetup)
            || self.app_launch.frame(self.monotonic_ms).is_some()
        {
            return;
        }
        if self.connected {
            self.display_wait_since = Some(self.monotonic_ms);
            self.display_started_connected = true;
            self.notice = "等待 Mac 应用".into();
            self.queue(UiCommand::RequestMode(Mode::Display));
        } else {
            self.notice = "Mac 未连接，请连接 USB 和 Mac 应用".into();
            self.changed();
        }
    }
    pub fn queue(&mut self, c: UiCommand) {
        if self.commands.len() < 64 {
            self.commands.push_back(c);
        }
        self.changed();
    }
    pub fn take_commands(&mut self) -> Vec<UiCommand> {
        self.commands.drain(..).collect()
    }
    pub fn apply_snapshot(&mut self, mut snapshot: Snapshot) {
        snapshot.apply_deletions();
        self.mac_page = self
            .mac_page
            .min(snapshot.buttons.len().div_ceil(12).saturating_sub(1));
        self.snapshot = snapshot;
        self.changed();
    }
    pub fn last_time_refresh(&mut self) {
        self.last_second = u64::MAX;
        self.last_unix_second = None;
        self.flip_clock.snap(&self.clock);
        self.changed();
    }
    fn clock_visible(&self) -> bool {
        matches!(self.active_app, ActiveApp::Clock)
            && self.app_launch.frame(self.monotonic_ms).is_none()
            && self.mode == Mode::Pad
            && self.settings.screen_on
    }
    fn timer_visible(&self) -> bool {
        matches!(self.active_app, ActiveApp::Timer)
            && self.app_launch.frame(self.monotonic_ms).is_none()
            && self.mode == Mode::Pad
            && self.settings.screen_on
    }
    pub fn usage_headline_visible(&self) -> bool {
        matches!(self.active_app, ActiveApp::Usage)
            && self.mode == Mode::Pad
            && self.settings.screen_on
            && !self.status_panel_open
            && self.usage.page != crate::usage::Page::Connection
            && self.app_launch.frame(self.monotonic_ms).is_none()
    }
    pub fn sync_usage_headline(&mut self) {
        let visible = self.usage_headline_visible();
        self.usage.sync_headline(self.monotonic_ms, visible);
    }
    pub fn take_timer_animation_dirty(&mut self, size: Size) -> Option<Rect> {
        if !self.timer_visible() || !self.timer.finished {
            self.timer_completion.cancel();
            return None;
        }
        self.timer_completion.take_dirty(self.monotonic_ms, size)
    }
    pub fn take_clock_animation_dirty(&mut self, size: Size) -> Option<Rect> {
        if !self.clock_visible() {
            self.flip_clock.snap(&self.clock);
            return None;
        }
        self.flip_clock.take_dirty(
            self.monotonic_ms,
            Size::new((size.width - 48.0).max(1.0), (size.height - 94.0).max(1.0)),
        )
    }
    pub fn tick(&mut self, monotonic_ms: u64, unix_ms: i64) -> bool {
        self.monotonic_ms = monotonic_ms;
        self.unix_ms = unix_ms;
        self.usage
            .set_clock_context(unix_ms, self.settings.timezone_minutes);
        if self.mode != Mode::Pad || !self.settings.screen_on {
            self.status_panel_open = false;
        }
        if self
            .display_wait_since
            .is_some_and(|start| monotonic_ms.saturating_sub(start) >= 10_000)
        {
            self.fail_display_launch("操作未完成，请重试");
        }
        if matches!(self.active_app, ActiveApp::DisplaySetup) {
            if !self.settings.screen_on {
                self.background_active_app();
            } else if self.display_started_connected && !self.connected {
                self.fail_display_launch("Mac 未连接，请连接 USB 和 Mac 应用");
            }
        }
        if (self.mode != Mode::Pad && !matches!(self.active_app, ActiveApp::DisplaySetup))
            || !self.settings.screen_on
        {
            self.app_launch.cancel();
            self.display_request_pending = false;
            self.desktop_backdrop.lock().unwrap().take();
        }
        // Display mode skips Pad drawing entirely, so cancellation belongs in
        // tick rather than relying on a hidden page consuming animation dirty.
        if !self.clock_visible() {
            self.flip_clock.snap(&self.clock);
        }
        self.sync_usage_headline();
        let previous_finish = self.timer.finished_at_ms();
        let changed = self.timer.tick(monotonic_ms);
        if !self.timer_visible() || !self.timer.finished {
            self.timer_completion.cancel();
        } else if self.timer.finished_at_ms() != previous_finish {
            if let Some(deadline) = self.timer.finished_at_ms() {
                self.timer_completion.start(deadline, monotonic_ms);
            }
        }
        let second = monotonic_ms / 1000;
        let unix_second = (unix_ms > 0).then(|| unix_ms.div_euclid(1000));
        if second != self.last_second || unix_second != self.last_unix_second || changed {
            // The monitor shows minute-resolution reset times. Keep wall time
            // and background deadlines current without rebuilding its entire
            // dashboard every second. Real UI/network events still call
            // changed(), while launches retain their original frame cadence.
            let idle_monitor = matches!(self.active_app, ActiveApp::Usage)
                && self.mode == Mode::Pad
                && self.settings.screen_on
                && !self.status_panel_open
                && self.app_launch.frame(monotonic_ms).is_none()
                && self.timer_completion.progress(monotonic_ms).is_none();
            let minute_changed = match (self.last_unix_second, unix_second) {
                (Some(before), Some(after)) => before.div_euclid(60) != after.div_euclid(60),
                _ => self.last_second / 60 != second / 60,
            };
            let clock_reset = self.last_second == u64::MAX
                || match (self.last_unix_second, unix_second) {
                    (Some(before), Some(after)) => after != before && after != before + 1,
                    (None, None) => false,
                    _ => true,
                };
            let timer_finished = self.timer.finished_at_ms() != previous_finish;
            let animate = self.clock_visible()
                && self.time_valid
                && self
                    .last_unix_second
                    .zip(unix_second)
                    .is_some_and(|(before, after)| after == before + 1);
            self.last_second = second;
            self.last_unix_second = unix_second;
            self.time_valid = unix_ms > 0;
            if self.time_valid {
                (self.clock, self.date) = clock_strings(unix_ms, self.settings.timezone_minutes);
            } else {
                self.clock = "--:--:--".into();
                self.date = "等待 Mac 校时".into();
            }
            self.flip_clock.update(&self.clock, monotonic_ms, animate);
            if !idle_monitor || minute_changed || clock_reset || timer_finished {
                self.changed();
                return true;
            }
        }
        false
    }
}

/// Gregorian civil date calculation, with explicit timezone supplied by Mac (no RTC/timezone DB).
pub fn clock_strings(unix_ms: i64, offset_minutes: i32) -> (String, String) {
    let secs = unix_ms.div_euclid(1000) + (offset_minutes as i64) * 60;
    let day = secs.div_euclid(86400);
    let s = secs.rem_euclid(86400);
    let z = day + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    y += if m <= 2 { 1 } else { 0 };
    let weekday = [
        "星期日",
        "星期一",
        "星期二",
        "星期三",
        "星期四",
        "星期五",
        "星期六",
    ][(day + 4).rem_euclid(7) as usize];
    (
        format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60),
        format!("{y} 年 {m:02} 月 {d:02} 日  {weekday}"),
    )
}
