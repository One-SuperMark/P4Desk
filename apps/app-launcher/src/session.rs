//! Versioned, bounded reboot checkpoints. No credentials, USB commands or UI animations.
use crate::launcher_state::{
    ActiveApp, LauncherState, NotesView, APP_IDS, MAX_BACKGROUND_APPS, MAX_RECENT_APPS,
};
use crate::radio::SettingsSection;
use crate::timer::{Phase, TimerKind};
use calculator::CalcState;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tiny_flutter::ScrollController;

pub const MIN_CLOCK_MS: i64 = 946_684_800_000;
pub const MAX_CLOCK_MS: i64 = 4_102_444_800_000;
pub const MAX_SESSION_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistenceStatus {
    Pending,
    Saved,
    Failed,
    Recovered,
}
impl PersistenceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "等待自动保存",
            Self::Saved => "应用状态已保存",
            Self::Failed => "自动保存失败，请检查存储",
            Self::Recovered => "已恢复上次应用状态",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedTimer {
    pub kind: TimerKind,
    pub phase: Phase,
    pub remaining_ms: u64,
    pub countdown_ms: u64,
    pub completed_cycles: u32,
    pub finished: bool,
    pub was_running: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SavedApp {
    Clock,
    Timer,
    Notes {
        selected_id: Option<String>,
        offset: f32,
    },
    Calculator(Box<CalcState>),
    Mac,
    Settings,
    Planned(crate::planned_apps::PlannedApp),
}
impl SavedApp {
    fn id(&self) -> &'static str {
        match self {
            Self::Clock => "clock",
            Self::Timer => "timer",
            Self::Notes { .. } => "notes",
            Self::Calculator(_) => "calculator",
            Self::Mac => "mac",
            Self::Settings => "settings",
            Self::Planned(app) => app.id(),
        }
    }
    fn capture(app: &ActiveApp, state: &LauncherState) -> Option<Self> {
        Some(match app {
            ActiveApp::Clock => Self::Clock,
            ActiveApp::Timer => Self::Timer,
            ActiveApp::Notes(view) => {
                let v = view.lock().unwrap();
                Self::Notes {
                    selected_id: state.snapshot.notes.get(v.selected).map(|n| n.id.clone()),
                    offset: v.scroll.offset(),
                }
            }
            ActiveApp::Calculator(calc) => Self::Calculator(Box::new(calc.lock().unwrap().clone())),
            ActiveApp::MacControls => Self::Mac,
            ActiveApp::Settings => Self::Settings,
            ActiveApp::Planned(app) => Self::Planned(*app),
            ActiveApp::Launcher | ActiveApp::DisplaySetup => return None,
        })
    }
    fn restore(&self, state: &LauncherState) -> ActiveApp {
        match self {
            Self::Clock => ActiveApp::Clock,
            Self::Timer => ActiveApp::Timer,
            Self::Calculator(calc) => ActiveApp::Calculator(Arc::new(Mutex::new((**calc).clone()))),
            Self::Mac => ActiveApp::MacControls,
            Self::Settings => ActiveApp::Settings,
            Self::Planned(app) => ActiveApp::Planned(*app),
            Self::Notes {
                selected_id,
                offset,
            } => {
                let selected = selected_id
                    .as_ref()
                    .and_then(|id| state.snapshot.notes.iter().position(|n| &n.id == id));
                ActiveApp::Notes(Arc::new(Mutex::new(NotesView {
                    selected: selected.unwrap_or(0),
                    scroll: ScrollController::with_offset(if selected.is_some() {
                        *offset
                    } else {
                        0.0
                    }),
                    confirm_delete: false,
                })))
            }
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::Calculator(c) => c.valid_checkpoint(),
            Self::Notes {
                selected_id,
                offset,
            } => {
                selected_id.as_ref().is_none_or(|id| id.len() <= 128)
                    && offset.is_finite()
                    && (0.0..=1_000_000.0).contains(offset)
            }
            _ => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub version: u32,
    pub unix_ms: Option<i64>,
    pub clock_calibrated: bool,
    pub timer: SavedTimer,
    pub foreground: Option<SavedApp>,
    pub background: Vec<SavedApp>,
    pub recent: Vec<String>,
    pub settings_section: SettingsSection,
    pub mac_page: usize,
}
impl Session {
    pub fn capture(state: &LauncherState) -> Self {
        let timer = &state.timer;
        Self {
            version: 1,
            unix_ms: (MIN_CLOCK_MS..=MAX_CLOCK_MS)
                .contains(&state.unix_ms)
                .then_some(state.unix_ms),
            clock_calibrated: state.time_valid && !state.time_estimated,
            timer: SavedTimer {
                kind: timer.kind,
                phase: timer.phase,
                remaining_ms: timer.remaining_ms,
                countdown_ms: timer.countdown_ms,
                completed_cycles: timer.completed_cycles,
                finished: timer.finished,
                was_running: timer.is_running(),
            },
            foreground: SavedApp::capture(&state.active_app, state),
            background: state
                .background_app_ids()
                .filter_map(|id| state.running_apps.get(id))
                .filter_map(|app| SavedApp::capture(app, state))
                .collect(),
            recent: state.recent_app_ids().map(str::to_owned).collect(),
            settings_section: state.settings_view.section,
            mac_page: state.mac_page,
        }
    }
    pub fn valid(&self) -> bool {
        let t = &self.timer;
        let duration = if t.kind == TimerKind::Pomodoro {
            t.phase.duration_ms()
        } else {
            t.countdown_ms
        };
        let mut ids = Vec::new();
        self.version == 1
            && self
                .unix_ms
                .is_none_or(|ms| (MIN_CLOCK_MS..=MAX_CLOCK_MS).contains(&ms))
            && (1_000..=7_200_000).contains(&t.countdown_ms)
            && t.remaining_ms <= duration
            && (!t.finished || (t.remaining_ms == 0 && !t.was_running))
            && self.background.len() <= MAX_BACKGROUND_APPS
            && self.recent.len() <= MAX_RECENT_APPS
            && self.mac_page <= 1024
            && self
                .foreground
                .iter()
                .chain(self.background.iter())
                .all(|app| {
                    if !app.valid() || ids.contains(&app.id()) {
                        return false;
                    }
                    ids.push(app.id());
                    true
                })
            && self.recent.iter().enumerate().all(|(i, id)| {
                (APP_IDS.contains(&id.as_str()) || id == "display")
                    && !self.recent[..i].contains(id)
            })
    }
    /// Exclude naturally advancing clocks from interaction debouncing.
    pub fn same_interaction(&self, other: &Self) -> bool {
        let mut normalized = self.clone();
        normalized.unix_ms = other.unix_ms;
        if self.timer.was_running && other.timer.was_running {
            normalized.timer.remaining_ms = other.timer.remaining_ms;
        }
        normalized == *other
    }
    pub fn restore(&self, state: &mut LauncherState) -> bool {
        if !self.valid() {
            return false;
        }
        state.running_apps.clear();
        state.background_order.clear();
        for app in &self.background {
            state
                .running_apps
                .insert(app.id().into(), app.restore(state));
            state.background_order.push_back(app.id().into());
        }
        state.active_app = self
            .foreground
            .as_ref()
            .map(|app| app.restore(state))
            .unwrap_or(ActiveApp::Launcher);
        state.recent_apps = self
            .recent
            .iter()
            .filter_map(|id| {
                APP_IDS
                    .into_iter()
                    .chain(["display"])
                    .find(|known| *known == id)
            })
            .collect();
        state.settings_view.section = self.settings_section;
        state.mac_page = self
            .mac_page
            .min(state.snapshot.buttons.len().div_ceil(12).saturating_sub(1));
        let t = &mut state.timer;
        t.kind = self.timer.kind;
        t.phase = self.timer.phase;
        t.remaining_ms = self.timer.remaining_ms;
        t.countdown_ms = self.timer.countdown_ms;
        t.completed_cycles = self.timer.completed_cycles;
        t.finished = self.timer.finished;
        t.restore_paused();
        state.persistence_status = PersistenceStatus::Recovered;
        state.changed();
        true
    }
}
