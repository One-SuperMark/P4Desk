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
