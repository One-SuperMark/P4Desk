//! Desktop icon SVG geometry; rasterized at the actual desktop/status-bar size.
use tiny_flutter::graphics::svg_icons_generated::*;
use tiny_flutter::VectorIcon;

pub type AppIconAsset = VectorIcon;
pub const DESKTOP_ICON_SIDE: u32 = 146;

pub fn get_app_icon_asset(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "clock" => Some(&DESKTOP_CLOCK),
        "timer" => Some(&DESKTOP_TIMER),
        "notes" => Some(&DESKTOP_NOTES),
        "calculator" => Some(&DESKTOP_CALCULATOR),
        "mac" => Some(&DESKTOP_MAC),
        "settings" => Some(&DESKTOP_SETTINGS),
        "display" => Some(&DESKTOP_DISPLAY),
        "screen" => Some(&DESKTOP_SCREEN),
        _ => None,
    }
}
