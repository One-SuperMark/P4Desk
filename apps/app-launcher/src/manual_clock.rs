#[derive(Debug, Clone)]
pub struct ManualClock {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}
impl Default for ManualClock {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 9,
            day: 28,
            hour: 12,
            minute: 0,
        }
    }
}
impl ManualClock {
    pub fn adjust(&mut self, index: usize, delta: i32) {
        match index {
            0 => self.year = (self.year + delta).clamp(2020, 2099),
            1 => self.month = (self.month as i32 - 1 + delta).rem_euclid(12) as u8 + 1,
            2 => {
                self.day = (self.day as i32 - 1 + delta)
                    .rem_euclid(days(self.year, self.month) as i32) as u8
                    + 1
            }
            3 => self.hour = (self.hour as i32 + delta).rem_euclid(24) as u8,
            4 => self.minute = (self.minute as i32 + delta).rem_euclid(60) as u8,
            _ => (),
        }
        self.day = self.day.min(days(self.year, self.month));
    }
    pub fn unix_ms(&self, timezone_minutes: i32) -> i64 {
        let y = self.year as i64 - if self.month <= 2 { 1 } else { 0 };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let mp = m + if m > 2 { -3 } else { 9 };
        let doy = (153 * mp + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let day = era * 146097 + doe - 719468;
        (day * 86400 + self.hour as i64 * 3600 + self.minute as i64 * 60
            - timezone_minutes as i64 * 60)
            * 1000
    }
    pub fn values(&self) -> [i32; 5] {
        [
            self.year,
            self.month as i32,
            self.day as i32,
            self.hour as i32,
            self.minute as i32,
        ]
    }
}
fn days(year: i32, month: u8) -> u8 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leap_day_and_timezone_roundtrip() {
        let mut c = ManualClock {
            year: 2024,
            month: 2,
            day: 29,
            hour: 9,
            minute: 35,
        };
        let (t, d) = crate::launcher_state::clock_strings(c.unix_ms(480), 480);
        assert_eq!(t, "09:35:00");
        assert!(d.starts_with("2024 年 02 月 29 日"));
        c.adjust(0, 1);
        assert_eq!(c.day, 28);
    }
}
