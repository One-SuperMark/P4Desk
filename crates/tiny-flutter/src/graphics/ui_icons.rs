//! SVG-backed UI icons, retaining the upstream named icon/widget API.
//! All sizes are rasterized from vectors, including the compatibility aliases.
use super::{svg_icons_generated as svg, VectorIcon};

#[derive(Clone, Copy)]
pub struct UiIcon {
    pub width: u16,
    pub height: u16,
    pub vector: &'static VectorIcon,
}
/// Upstream API name retained; the icon itself contains no baked bitmap.
pub type BakedIcon = UiIcon;
macro_rules! icon {
    ($name:ident, $size:literal, $source:ident) => {
        pub static $name: UiIcon = UiIcon {
            width: $size,
            height: $size,
            vector: &svg::$source,
        };
    };
}
icon!(ICON_BACKSPACE, 24, UI_BACKSPACE);
icon!(ICON_BACKWARD_FILL, 28, UI_BACKWARD_FILL);
icon!(ICON_FORWARD_FILL, 28, UI_FORWARD_FILL);
icon!(ICON_BATTERY_0PERCENT, 40, UI_BATTERY_0);
icon!(ICON_BATTERY_25PERCENT, 40, UI_BATTERY_25);
icon!(ICON_BATTERY_50PERCENT, 40, UI_BATTERY_50);
icon!(ICON_BATTERY_75PERCENT, 40, UI_BATTERY_75);
icon!(ICON_BATTERY_100PERCENT, 40, UI_BATTERY_100);
icon!(ICON_BATTERY_100PERCENT_BOLT, 40, UI_BATTERY_BOLT);
icon!(ICON_CAPSLOCK, 24, UI_CAPSLOCK);
icon!(ICON_SHIFT, 24, UI_SHIFT);
icon!(ICON_PAUSE_CIRCLE, 48, UI_PAUSE_CIRCLE);
icon!(ICON_PLAY_CIRCLE, 48, UI_PLAY_CIRCLE);
icon!(ICON_WIFI_0, 32, UI_WIFI_0);
icon!(ICON_WIFI_1, 32, UI_WIFI_1);
icon!(ICON_WIFI_2, 32, UI_WIFI_2);
icon!(ICON_WIFI_3, 32, UI_WIFI_3);
icon!(ICON_APP_CALCULATOR, 40, DESKTOP_CALCULATOR);
icon!(ICON_APP_COUNTER, 40, UI_COUNTER);
icon!(ICON_APP_HELLO, 40, UI_HELLO);
icon!(ICON_APP_PALETTE, 40, UI_PALETTE);
icon!(ICON_APP_SETTINGS, 40, DESKTOP_SETTINGS);
icon!(ICON_APP_STORAGE, 40, UI_STORAGE);
icon!(ICON_APP_SYSTEM, 40, UI_SYSTEM);
icon!(ICON_APP_TERMINAL, 40, UI_TERMINAL);
