use crate::storage::LocalSettings;
use crate::timer::TimerService;
use calculator::CalcState;
use p4desk_protocol::{Mode, Snapshot};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tiny_flutter::{PageController, ScrollController};

pub const APP_IDS: [&str; 6] = ["clock", "timer", "notes", "calculator", "mac", "settings"];
#[derive(Clone)]
pub enum ActiveApp {
    Launcher,
    Clock,
    Timer,
    Notes(Arc<Mutex<NotesView>>),
    Calculator(Arc<Mutex<CalcState>>),
    MacControls,
    Settings,
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
    DeleteNote(String),
    Action(String),
    Media(u16),
    Brightness(u8),
    Screen(bool),
    RequestTimeSync,
    SetTime(i64, i32),
    RequestMode(Mode),
}

pub struct LauncherState {
    pub active_app: ActiveApp,
    pub running_apps: HashMap<String, ActiveApp>,
    pub page_controller: PageController,
    pub snapshot: Snapshot,
    pub timer: TimerService,
    pub settings: LocalSettings,
    pub connected: bool,
    pub sd_ready: bool,
    pub time_valid: bool,
    pub mode: Mode,
    pub unix_ms: i64,
    pub monotonic_ms: u64,
    pub clock: String,
    pub date: String,
    pub notice: String,
    pub mac_page: usize,
    pub revision: u64,
    pub manual_time_open: bool,
    pub manual_clock: crate::manual_clock::ManualClock,
    commands: VecDeque<UiCommand>,
    last_second: u64,
}
impl Default for LauncherState {
    fn default() -> Self {
        Self::new()
    }
}
impl LauncherState {
    pub fn new() -> Self {
        Self {
            active_app: ActiveApp::Launcher,
            running_apps: HashMap::new(),
            page_controller: PageController::new(),
            snapshot: Snapshot::default(),
            timer: TimerService::default(),
            settings: LocalSettings::default(),
            connected: false,
            sd_ready: false,
            time_valid: false,
            mode: Mode::Pad,
            unix_ms: 0,
            monotonic_ms: 0,
            clock: "--:--:--".into(),
            date: "等待 Mac 校时".into(),
            notice: String::new(),
            mac_page: 0,
            revision: 1,
            manual_time_open: false,
            manual_clock: crate::manual_clock::ManualClock::default(),
            commands: VecDeque::new(),
            last_second: u64::MAX,
        }
    }
    pub fn open_app(&mut self, id: &str) {
        self.background_active_app();
        self.active_app = self.running_apps.remove(id).unwrap_or_else(|| match id {
            "clock" => ActiveApp::Clock,
            "timer" => ActiveApp::Timer,
            "notes" => ActiveApp::Notes(Arc::new(Mutex::new(NotesView::default()))),
            "calculator" => ActiveApp::Calculator(Arc::new(Mutex::new(CalcState::new()))),
            "mac" => ActiveApp::MacControls,
            "settings" => ActiveApp::Settings,
            _ => ActiveApp::Launcher,
        });
        self.changed();
    }
    pub fn background_active_app(&mut self) {
        if let Some(id) = self.active_app.id() {
            self.running_apps.insert(id.into(), self.active_app.clone());
        }
        self.active_app = ActiveApp::Launcher;
        self.changed();
    }
    pub fn kill_active_app(&mut self) {
        if let Some(id) = self.active_app.id() {
            self.running_apps.remove(id);
        }
        self.active_app = ActiveApp::Launcher;
        self.changed();
    }
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
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
        self.snapshot = snapshot;
        self.changed();
    }
    pub fn last_time_refresh(&mut self) {
        self.last_second = u64::MAX;
        self.changed();
    }
    pub fn tick(&mut self, monotonic_ms: u64, unix_ms: i64) -> bool {
        self.monotonic_ms = monotonic_ms;
        self.unix_ms = unix_ms;
        let changed = self.timer.tick(monotonic_ms);
        let second = monotonic_ms / 1000;
        if second != self.last_second || changed {
            self.last_second = second;
            self.time_valid = unix_ms > 0;
            if self.time_valid {
                (self.clock, self.date) = clock_strings(unix_ms, self.settings.timezone_minutes);
            } else {
                self.clock = "--:--:--".into();
                self.date = "等待 Mac 校时".into();
            }
            self.changed();
            return true;
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
