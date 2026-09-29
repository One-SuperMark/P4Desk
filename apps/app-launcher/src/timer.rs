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
    LongBreak,
}
impl Phase {
    pub const ALL: [Self; 3] = [Self::Focus, Self::Break, Self::LongBreak];
    pub fn label(self) -> &'static str {
        match self {
            Self::Focus => "专注",
            Self::Break => "短休息",
            Self::LongBreak => "长休息",
        }
    }
    pub fn duration_ms(self) -> u64 {
        match self {
            Self::Focus => 25 * 60_000,
            Self::Break => 5 * 60_000,
            Self::LongBreak => 15 * 60_000,
        }
    }
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
    finished_at_ms: Option<u64>,
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
            finished_at_ms: None,
            countdown_ms: 5 * 60_000,
        }
    }
}
impl TimerService {
    pub fn is_running(&self) -> bool {
        self.deadline_ms.is_some()
    }
    pub fn finished_at_ms(&self) -> Option<u64> {
        self.finished_at_ms
    }
    pub fn tick(&mut self, now_ms: u64) -> bool {
        let before = self.remaining_ms.div_ceil(1000);
        if let Some(deadline) = self.deadline_ms {
            self.remaining_ms = deadline.saturating_sub(now_ms);
            if now_ms >= deadline {
                self.deadline_ms = None;
                self.remaining_ms = 0;
                self.finished = true;
                self.finished_at_ms = Some(deadline);
                if self.kind == TimerKind::Pomodoro && self.phase == Phase::Focus {
                    self.completed_cycles = self.completed_cycles.saturating_add(1);
                }
            }
        }
        before != self.remaining_ms.div_ceil(1000) || (self.finished && before > 0)
    }
    pub fn toggle(&mut self, now_ms: u64) {
        let was_running = self.is_running();
        self.tick(now_ms);
        if was_running {
            self.deadline_ms = None;
            return;
        }
        if self.remaining_ms == 0 {
            if self.kind == TimerKind::Pomodoro {
                self.phase = self.next_phase();
            }
            self.reset();
        }
        self.finished = false;
        self.finished_at_ms = None;
        self.deadline_ms = Some(now_ms.saturating_add(self.remaining_ms));
    }
    pub fn reset(&mut self) {
        self.deadline_ms = None;
        self.finished = false;
        self.finished_at_ms = None;
        self.remaining_ms = self.duration_ms();
    }
    pub fn duration_ms(&self) -> u64 {
        match self.kind {
            TimerKind::Pomodoro => self.phase.duration_ms(),
            TimerKind::Countdown => self.countdown_ms,
        }
    }
    /// Completed sessions select the next break; skipped focus is never counted.
    pub fn next_phase(&self) -> Phase {
        match self.phase {
            Phase::Focus
                if self.finished && self.completed_cycles > 0 && self.completed_cycles % 4 == 0 =>
            {
                Phase::LongBreak
            }
            Phase::Focus => Phase::Break,
            Phase::Break | Phase::LongBreak => Phase::Focus,
        }
    }
    pub fn set_phase(&mut self, phase: Phase) {
        if self.kind == TimerKind::Pomodoro && self.phase == phase {
            return;
        }
        self.kind = TimerKind::Pomodoro;
        self.phase = phase;
        self.reset();
    }
    /// Stop and prepare the next stage. The user starts it explicitly.
    pub fn next(&mut self, now_ms: u64) {
        self.tick(now_ms);
        if self.kind == TimerKind::Pomodoro {
            self.phase = self.next_phase();
        }
        self.reset();
    }
    pub fn set_countdown(&mut self, minutes: u64) {
        self.set_countdown_seconds(minutes.clamp(1, 120) * 60);
    }
    pub fn set_countdown_seconds(&mut self, seconds: u64) {
        self.kind = TimerKind::Countdown;
        self.countdown_ms = seconds.clamp(1, 7_200) * 1_000;
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
    #[test]
    fn four_completed_focus_sessions_prepare_long_break_and_wait_for_start() {
        let mut t = TimerService::default();
        let mut now = 0;
        for cycle in 1..=4 {
            t.toggle(now);
            now += 25 * 60_000;
            t.tick(now);
            assert_eq!(t.completed_cycles, cycle);
            assert!(t.finished && !t.is_running());
            assert_eq!(
                t.next_phase(),
                if cycle == 4 {
                    Phase::LongBreak
                } else {
                    Phase::Break
                }
            );
            // Late UI frames cannot start another interval or count the finish twice.
            t.tick(now + 50_000);
            assert_eq!(t.completed_cycles, cycle);
            t.toggle(now);
            assert_eq!(
                t.phase,
                if cycle == 4 {
                    Phase::LongBreak
                } else {
                    Phase::Break
                }
            );
            now += t.duration_ms();
            t.tick(now);
            assert_eq!(t.completed_cycles, cycle);
            t.next(now);
            assert_eq!(t.phase, Phase::Focus);
            assert!(!t.is_running());
        }
        t.next(now);
        assert_eq!(
            t.phase,
            Phase::Break,
            "skipping the next focus is not another completed fourth focus"
        );
        assert_eq!(t.completed_cycles, 4);
    }
    #[test]
    fn phase_selection_and_skip_stop_without_fabricating_completed_sessions() {
        let mut t = TimerService::default();
        t.toggle(0);
        t.tick(30_000);
        t.set_phase(Phase::Focus);
        assert!(t.is_running());
        assert_eq!(t.remaining_ms, 1_470_000);
        t.next(30_000);
        assert_eq!(t.phase, Phase::Break);
        assert!(!t.is_running());
        assert_eq!(t.completed_cycles, 0);
        t.set_phase(Phase::LongBreak);
        assert_eq!(t.remaining_ms, 900_000);
        t.set_countdown(120);
        assert_eq!(t.display(), "120:00");
        t.toggle(40_000);
        t.next(50_000);
        assert_eq!(t.remaining_ms, 120 * 60_000);
        assert!(!t.is_running());
    }
    #[test]
    fn pause_tap_at_deadline_finishes_without_starting_a_break() {
        let mut t = TimerService::default();
        t.toggle(0);
        t.toggle(25 * 60_000);
        assert_eq!(t.phase, Phase::Focus);
        assert!(t.finished && !t.is_running());
        assert_eq!(t.completed_cycles, 1);
    }
}
