//! Voltage-based estimate for the 7B's single-cell 4.2 V lithium battery.
//! Real charging and the user's Type-C host connection indication remain distinct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChargeState {
    #[default]
    Unknown,
    Charging,
    /// User-requested indication while the native Type-C port sees host SOF.
    /// This is a UI convention, not measured charge current or STAT telemetry.
    PluggedInAssumed,
    Discharging,
    Full,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BatteryReading {
    pub voltage_mv: Option<u16>,
    pub charge: ChargeState,
}
#[derive(Clone, Debug, Default)]
pub struct BatteryState {
    pub voltage_mv: Option<u16>,
    pub percent: Option<u8>,
    pub charge: ChargeState,
    filtered_mv: Option<u32>,
}
impl BatteryState {
    pub fn update(&mut self, reading: BatteryReading) -> bool {
        let before = (self.voltage_mv, self.percent, self.charge);
        self.charge = reading.charge;
        // Missing/calibration-failed samples clear stale values; do not invent 0%.
        match reading.voltage_mv.filter(|mv| (2500..=4500).contains(mv)) {
            Some(mv) => {
                let filtered = self.filtered_mv.map_or(u32::from(mv) * 8, |old| {
                    (old * 3 + u32::from(mv) * 8 + 2) / 4
                });
                self.filtered_mv = Some(filtered);
                self.voltage_mv = Some(((filtered + 4) / 8) as u16);
                let estimate = estimate_percent(self.voltage_mv.unwrap());
                // A 1% dead band avoids a twitching label at a table boundary.
                if self.percent.is_none_or(|old| old.abs_diff(estimate) >= 2)
                    || estimate == 0
                    || estimate == 100
                {
                    self.percent = Some(estimate);
                }
            }
            None => {
                self.filtered_mv = None;
                self.voltage_mv = None;
                self.percent = None;
            }
        }
        before != (self.voltage_mv, self.percent, self.charge)
    }
    pub fn percent_label(&self) -> String {
        self.percent
            .map_or_else(|| "--%".into(), |p| format!("{p}%"))
    }
    pub fn voltage_label(&self) -> String {
        self.voltage_mv.map_or_else(
            || "无法读取".into(),
            |mv| format!("{}.{:02} V", mv / 1000, (mv % 1000) / 10),
        )
    }
    pub fn charge_label(&self) -> &'static str {
        match self.charge {
            ChargeState::Unknown => "无法读取",
            ChargeState::Charging => "正在充电",
            ChargeState::PluggedInAssumed => "插电充电",
            ChargeState::Discharging => "电池供电",
            ChargeState::Full => "充电完成",
        }
    }
}
/// Generic, deliberately approximate single-cell lithium curve. Under load or
/// charge the terminal voltage is not open-circuit SOC; no runtime is predicted.
pub fn estimate_percent(mv: u16) -> u8 {
    const CURVE: [(u16, u8); 12] = [
        (3200, 0),
        (3400, 5),
        (3500, 10),
        (3600, 20),
        (3700, 30),
        (3750, 40),
        (3800, 50),
        (3850, 60),
        (3900, 70),
        (4000, 80),
        (4100, 90),
        (4200, 100),
    ];
    if mv <= CURVE[0].0 {
        return 0;
    }
    for pair in CURVE.windows(2) {
        let [(lo, a), (hi, b)] = [pair[0], pair[1]];
        if mv <= hi {
            return a
                + ((u32::from(mv - lo) * u32::from(b - a) + u32::from(hi - lo) / 2)
                    / u32::from(hi - lo)) as u8;
        }
    }
    100
}
