use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerKind {
    Pomodoro,
    Countdown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Focus,
    Break,
}

/// Deadlines use the hardware monotonic clock and do not depend on UI frames or time_sync.
#[derive(Debug, Clone)]
pub struct TimerService {
    pub kind: TimerKind,
    pub phase: Phase,
    pub remaining_ms: u64,
    pub completed_cycles: u32,
    pub finished: bool,
    deadline_ms: Option<u64>,
    pub countdown_ms: u64,
}
impl Default for TimerService {
    fn default() -> Self {
        Self {
            kind: TimerKind::Pomodoro,
            phase: Phase::Focus,
            remaining_ms: 25 * 60_000,
            completed_cycles: 0,
            finished: false,
            deadline_ms: None,
            countdown_ms: 5 * 60_000,
        }
    }
}
impl TimerService {
    pub fn is_running(&self) -> bool {
        self.deadline_ms.is_some()
    }
    pub fn tick(&mut self, now_ms: u64) -> bool {
        let before = self.remaining_ms.div_ceil(1000);
        if let Some(deadline) = self.deadline_ms {
            self.remaining_ms = deadline.saturating_sub(now_ms);
            if now_ms >= deadline {
                self.deadline_ms = None;
                self.remaining_ms = 0;
                self.finished = true;
                if self.kind == TimerKind::Pomodoro && self.phase == Phase::Focus {
                    self.completed_cycles = self.completed_cycles.saturating_add(1);
                }
            }
        }
        before != self.remaining_ms.div_ceil(1000) || (self.finished && before > 0)
    }
    pub fn toggle(&mut self, now_ms: u64) {
        self.tick(now_ms);
        if self.deadline_ms.take().is_some() {
            return;
        }
        if self.remaining_ms == 0 {
            if self.kind == TimerKind::Pomodoro {
                self.phase = if self.phase == Phase::Focus {
                    Phase::Break
                } else {
                    Phase::Focus
                };
            }
            self.reset();
        }
        self.finished = false;
        self.deadline_ms = Some(now_ms.saturating_add(self.remaining_ms));
    }
    pub fn reset(&mut self) {
        self.deadline_ms = None;
        self.finished = false;
        self.remaining_ms = match (self.kind, self.phase) {
            (TimerKind::Pomodoro, Phase::Focus) => 25 * 60_000,
            (TimerKind::Pomodoro, Phase::Break) => 5 * 60_000,
            _ => self.countdown_ms,
        };
    }
    pub fn set_countdown(&mut self, minutes: u64) {
        self.kind = TimerKind::Countdown;
        self.countdown_ms = minutes.clamp(1, 120) * 60_000;
        self.reset();
    }
    pub fn set_pomodoro(&mut self) {
        self.kind = TimerKind::Pomodoro;
        self.phase = Phase::Focus;
        self.reset();
    }
    pub fn display(&self) -> String {
        let seconds = self.remaining_ms.div_ceil(1000);
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continues_without_ui_frames_and_ignores_wall_clock() {
        let mut t = TimerService::default();
        t.toggle(100);
        t.tick(1_000_100);
        assert_eq!(t.remaining_ms, 500_000);
        t.tick(1_500_100);
        assert!(t.finished);
        assert_eq!(t.completed_cycles, 1);
        t.tick(2_000_000);
        assert_eq!(t.completed_cycles, 1);
    }
    #[test]
    fn pause_resume_and_break() {
        let mut t = TimerService::default();
        t.toggle(100);
        t.toggle(30_100);
        assert_eq!(t.remaining_ms, 1_470_000);
        t.tick(100_000);
        assert_eq!(t.remaining_ms, 1_470_000);
        t.toggle(100_000);
        t.tick(1_570_000);
        t.toggle(1_570_000);
        assert_eq!(t.phase, Phase::Break);
        assert_eq!(t.remaining_ms, 300_000);
    }
    #[test]
    fn countdown_finishes_once() {
        let mut t = TimerService::default();
        t.set_countdown(1);
        t.toggle(100);
        assert!(t.tick(60_100));
        assert_eq!(t.display(), "00:00");
        assert_eq!(t.completed_cycles, 0);
    }
}
