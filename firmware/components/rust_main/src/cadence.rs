//! Keep a 16 ms loop budget, including work, with at least one scheduler tick.
pub(crate) fn idle_delay_ms(elapsed_us: i64) -> u32 {
    let remaining = 16_000i64.saturating_sub(elapsed_us.max(0)).max(0);
    ((remaining + 999) / 1000).max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_consumes_the_loop_budget_and_long_frames_still_yield() {
        for (work, wait) in [
            (-1, 16),
            (0, 16),
            (999, 16),
            (1_000, 15),
            (4_001, 12),
            (15_001, 1),
            (16_000, 1),
            (30_000, 1),
            (i64::MAX, 1),
        ] {
            assert_eq!(idle_delay_ms(work), wait);
        }
    }
}

/// Consume movement through the newest sample before painting, retaining frame
/// boundaries after Down and terminal events for feedback and app transitions.
#[derive(Default)]
pub(crate) struct TouchBatch {
    count: u8,
    boundary: bool,
}
impl TouchBatch {
    pub fn ready(&self) -> bool {
        !self.boundary && self.count < 8
    }
    pub fn consumed(&mut self, kind: u32) {
        self.count += 1;
        self.boundary = kind != 2;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
#[cfg(test)]
mod touch_tests {
    use super::TouchBatch;
    #[test]
    fn moves_share_a_frame_but_down_and_release_keep_their_boundaries() {
        let mut b = TouchBatch::default();
        assert!(b.ready());
        b.consumed(1);
        assert!(!b.ready());
        b.reset();
        for kind in [2, 2, 3] {
            assert!(b.ready());
            b.consumed(kind);
        }
        assert!(!b.ready());
        b.reset();
        for _ in 0..8 {
            assert!(b.ready());
            b.consumed(2);
        }
        assert!(!b.ready());
        b.reset();
        b.consumed(4);
        assert!(!b.ready());
    }
}
