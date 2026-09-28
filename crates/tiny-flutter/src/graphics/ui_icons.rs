//! Small monochrome masks authored for this port; no proprietary icon asset dependency.
//! Const evaluation keeps the original UiIcon API and stores alpha masks in flash.

#![allow(dead_code)]

/// A monochrome vector UI icon mask (8-bit alpha coverage, dynamically tinted at runtime).
#[derive(Clone, Copy)]
pub struct UiIcon {
    pub width: u16,
    pub height: u16,
    pub bitmap: &'static [u8],
}

/// Backwards-compatibility alias
pub type BakedIcon = UiIcon;

pub static ICON_BACKSPACE: UiIcon = UiIcon {
    width: 24,
    height: 24,
    bitmap: &simple_mask::<576>(24, 1),
};

pub static ICON_BACKWARD_FILL: UiIcon = UiIcon {
    width: 28,
    height: 28,
    bitmap: &simple_mask::<784>(28, 2),
};

pub static ICON_BATTERY_0PERCENT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 3),
};

pub static ICON_BATTERY_100PERCENT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 4),
};

pub static ICON_BATTERY_100PERCENT_BOLT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 5),
};

pub static ICON_BATTERY_25PERCENT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 6),
};

pub static ICON_BATTERY_50PERCENT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 7),
};

pub static ICON_BATTERY_75PERCENT: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 8),
};

pub static ICON_CAPSLOCK: UiIcon = UiIcon {
    width: 24,
    height: 24,
    bitmap: &simple_mask::<576>(24, 9),
};

pub static ICON_FORWARD_FILL: UiIcon = UiIcon {
    width: 28,
    height: 28,
    bitmap: &simple_mask::<784>(28, 10),
};

pub static ICON_PAUSE_CIRCLE: UiIcon = UiIcon {
    width: 48,
    height: 48,
    bitmap: &simple_mask::<2304>(48, 11),
};

pub static ICON_PLAY_CIRCLE: UiIcon = UiIcon {
    width: 48,
    height: 48,
    bitmap: &simple_mask::<2304>(48, 12),
};

pub static ICON_SHIFT: UiIcon = UiIcon {
    width: 24,
    height: 24,
    bitmap: &simple_mask::<576>(24, 13),
};

pub static ICON_WIFI_0: UiIcon = UiIcon {
    width: 32,
    height: 32,
    bitmap: &simple_mask::<1024>(32, 14),
};

pub static ICON_WIFI_1: UiIcon = UiIcon {
    width: 32,
    height: 32,
    bitmap: &simple_mask::<1024>(32, 15),
};

pub static ICON_WIFI_2: UiIcon = UiIcon {
    width: 32,
    height: 32,
    bitmap: &simple_mask::<1024>(32, 16),
};

pub static ICON_WIFI_3: UiIcon = UiIcon {
    width: 32,
    height: 32,
    bitmap: &simple_mask::<1024>(32, 17),
};

pub static ICON_APP_CALCULATOR: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 18),
};

pub static ICON_APP_COUNTER: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 19),
};

pub static ICON_APP_HELLO: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 20),
};

pub static ICON_APP_PALETTE: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 21),
};

pub static ICON_APP_SETTINGS: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 22),
};

pub static ICON_APP_STORAGE: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 23),
};

pub static ICON_APP_SYSTEM: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 24),
};

pub static ICON_APP_TERMINAL: UiIcon = UiIcon {
    width: 40,
    height: 40,
    bitmap: &simple_mask::<1600>(40, 25),
};

const fn simple_mask<const N: usize>(w: usize, kind: u8) -> [u8; N] {
    let mut out = [0; N];
    let mut i = 0;
    while i < N {
        let x = i % w;
        let y = i / w;
        let c = w as i32 / 2;
        let dx = x as i32 - c;
        let dy = y as i32 - c;
        let radius = w as i32 * 3 / 8;
        let ring = dx * dx + dy * dy;
        let border = ring <= radius * radius && ring >= (radius - 2) * (radius - 2);
        let mark = if kind % 3 == 0 {
            y > w / 3 && y < w * 2 / 3 && (x == w / 3 || x == w * 2 / 3)
        } else if kind % 3 == 1 {
            x > w / 3 && x < w * 2 / 3 && y.abs_diff(w / 2) <= x - w / 3
        } else {
            (x.abs_diff(w / 2) < 2 && y > w / 4 && y < w * 3 / 4)
                || (y.abs_diff(w / 2) < 2 && x > w / 4 && x < w * 3 / 4)
        };
        if border || mark {
            out[i] = 255;
        }
        i += 1;
    }
    out
}
