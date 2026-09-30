//! Position-based page selection stays in PageView. This only shapes settling.
use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) struct Settle {
    pub started: Instant,
    from: f32,
    duration: f32,
    tangent: f32,
}
impl Settle {
    pub fn new(from: f32, velocity: f32, width: f32) -> Self {
        let fraction = (from.abs() / (width * 0.5).max(1.0)).clamp(0.0, 1.0);
        let duration = 0.14 + 0.07 * fraction;
        // Preserve motion toward the chosen page, while keeping the curve
        // monotonic. Opposing motion brakes before returning; no overshoot.
        let tangent = if from.abs() > 0.25 {
            (-velocity * duration / from).clamp(0.0, 2.5)
        } else {
            0.0
        };
        Self {
            started: Instant::now(),
            from,
            duration,
            tangent,
        }
    }
    pub fn position(&self, elapsed: f32) -> f32 {
        if elapsed >= self.duration {
            return 0.0;
        }
        let t = (elapsed / self.duration).max(0.0);
        let r = 1.0 - t;
        self.from * r * r * (1.0 + (2.0 - self.tangent) * t)
    }
}

#[derive(Default)]
pub(super) struct DragVelocity {
    samples: [Option<(Instant, f32)>; 6],
    len: usize,
}
impl DragVelocity {
    pub fn record(&mut self, now: Instant, position: f32) {
        if self.len > 0
            && now
                .duration_since(self.samples[self.len - 1].unwrap().0)
                .as_secs_f32()
                < 0.004
        {
            self.samples[self.len - 1] = Some((now, position));
            return;
        }
        if self.len == self.samples.len() {
            self.samples.copy_within(1.., 0);
            self.len -= 1;
        }
        self.samples[self.len] = Some((now, position));
        self.len += 1;
    }
    pub fn velocity(&self, now: Instant) -> f32 {
        if self.len < 2 {
            return 0.0;
        }
        let (last_time, last) = self.samples[self.len - 1].unwrap();
        if now.duration_since(last_time).as_secs_f32() > 0.08 {
            return 0.0;
        }
        for sample in self.samples[..self.len - 1].iter().flatten() {
            let age = now.duration_since(sample.0).as_secs_f32();
            let dt = last_time.duration_since(sample.0).as_secs_f32();
            if age <= 0.10 && dt >= 0.004 {
                return ((last - sample.1) / dt).clamp(-5000.0, 5000.0);
            }
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn settle_is_continuous_monotonic_and_finishes_without_a_tail() {
        for from in [-430.0, -80.0, 80.0, 430.0] {
            for velocity in [-5000.0, -700.0, 0.0, 700.0, 5000.0] {
                let s = Settle::new(from, velocity, 864.0);
                assert_eq!(s.position(0.0), from);
                let mut previous = from.abs();
                for ms in 0..=240 {
                    let x = s.position(ms as f32 / 1000.0);
                    assert!(x.abs() <= previous + 0.001);
                    assert!(x == 0.0 || x.signum() == from.signum());
                    previous = x.abs();
                }
                assert_eq!(s.position(0.22), 0.0);
            }
        }
        let s = Settle::new(300.0, -700.0, 864.0);
        assert!(((s.position(0.0001) - 300.0) / 0.0001 + 700.0).abs() < 5.0);
        let rest = Settle::new(300.0, 0.0, 864.0);
        assert!((rest.position(0.0001) - 300.0).abs() < 0.01);
    }
    #[test]
    fn coalesced_samples_and_holds_do_not_invent_release_velocity() {
        let start = Instant::now();
        let mut v = DragVelocity::default();
        for (ms, x) in [(0, 0.0), (20, -20.0), (40, -40.0), (41, -42.0)] {
            v.record(start + Duration::from_millis(ms), x);
        }
        assert!((v.velocity(start + Duration::from_millis(42)) + 1024.39).abs() < 1.0);
        assert_eq!(v.velocity(start + Duration::from_millis(200)), 0.0);
    }
}
