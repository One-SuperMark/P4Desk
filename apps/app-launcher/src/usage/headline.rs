//! Presentation-only, interruptible Token counter. The worker's totals remain
//! authoritative; interpolation never writes back into a Data snapshot.

pub const UPDATE_DURATION_MS: u64 = 1_000;
pub const PERIOD_DURATION_MS: u64 = 800;
pub const FRAME_INTERVAL_MS: u64 = 33;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeadlineSample {
    pub value: Option<u64>,
    pub from: Option<u64>,
    pub target: Option<u64>,
    pub active: bool,
    /// Decimal digit slots. A renderer can use fixed-width digit advances.
    pub width_digits: usize,
    /// Includes grouping separators, for a stable minimum text width.
    pub width_chars: usize,
}

#[derive(Default)]
pub struct HeadlineState {
    from: Option<u64>,
    target: Option<u64>,
    started_ms: Option<u64>,
    duration_ms: u64,
    last_frame: u64,
}

impl HeadlineState {
    /// Initial, unavailable and unrelated-scope values are shown directly.
    pub fn snap(&mut self, value: Option<u64>) {
        self.from = value;
        self.target = value;
        self.started_ms = None;
        self.duration_ms = 0;
        self.last_frame = 0;
    }

    /// Interrupting an animation starts at the value actually visible now.
    /// Durations are bounded to the two desktop timings to keep arithmetic
    /// predictable and avoid an accidental long-running render loop.
    pub fn update(&mut self, value: Option<u64>, now_ms: u64, duration_ms: u64) {
        if value == self.target {
            return;
        }
        let from = self.sample(now_ms).value;
        if from.is_none() || value.is_none() || from == value {
            self.snap(value);
            return;
        }
        self.from = from;
        self.target = value;
        self.started_ms = Some(now_ms);
        self.duration_ms = duration_ms.clamp(1, UPDATE_DURATION_MS);
        self.last_frame = 0;
    }

    /// Stop producing frames when the app, screen or headline is hidden.
    pub fn settle(&mut self) {
        self.snap(self.target);
    }

    pub fn sample(&self, now_ms: u64) -> HeadlineSample {
        let elapsed = self.started_ms.map(|start| now_ms.saturating_sub(start));
        let active = elapsed.is_some_and(|ms| ms < self.duration_ms);
        let value =
            if let (Some(elapsed), Some(from), Some(target)) = (elapsed, self.from, self.target) {
                if elapsed >= self.duration_ms {
                    Some(target)
                } else {
                    // easeOutQuart = 1 - (1 - t)^4. Integer rational evaluation
                    // preserves adjacent u64 values even above f64's exact range.
                    // duration <= 1,000 means delta * duration^4 fits in u128.
                    let duration = self.duration_ms as u128;
                    let remaining = (self.duration_ms - elapsed) as u128;
                    let denominator = duration.pow(4);
                    let numerator = denominator - remaining.pow(4);
                    let delta = from.abs_diff(target) as u128;
                    let step = ((delta * numerator + denominator / 2) / denominator) as u64;
                    Some(if target >= from {
                        from + step
                    } else {
                        from - step
                    })
                }
            } else {
                self.target
            };
        let width_digits = if active {
            digits(self.from).max(digits(self.target))
        } else {
            digits(value)
        };
        HeadlineSample {
            value,
            // The render object may use non-consuming sampling rather than
            // take_dirty. Once settled, do not retain the old value's wider
            // abbreviation reservation in that render path.
            from: if active { self.from } else { value },
            target: self.target,
            active,
            width_digits,
            width_chars: if width_digits == 0 {
                1 // unavailable em dash
            } else {
                width_digits + (width_digits - 1) / 3
            },
        }
    }

    /// Consume one current frame, never enqueue historical frames. The final
    /// exact target is repainted even after a blocked loop skipped the ending.
    pub fn take_dirty(&mut self, now_ms: u64) -> bool {
        let Some(start) = self.started_ms else {
            return false;
        };
        let elapsed = now_ms.saturating_sub(start);
        if elapsed >= self.duration_ms {
            self.settle();
            return true;
        }
        let frame = elapsed / FRAME_INTERVAL_MS;
        if frame == self.last_frame {
            return false;
        }
        self.last_frame = frame;
        true
    }
}

fn digits(value: Option<u64>) -> usize {
    value.map_or(0, |value| value.checked_ilog10().unwrap_or(0) as usize + 1)
}
