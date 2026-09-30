//! Selectable Folio / Numix / Colloid vector themes; each pack retains its license.
use crate::icon_theme::{self, IconTheme};
use tiny_flutter::graphics::{
    colloid_icons_generated::*, folio_light_icons_generated::*, numix_icons_generated::*,
    svg_icons_generated::*, whitesur_icons_generated::*,
};
use tiny_flutter::{Color, VectorIcon};
pub type AppIconAsset = VectorIcon;
pub const DESKTOP_ICON_SIDE: u32 = 146;
fn choose(dark: &'static VectorIcon, light: &'static VectorIcon) -> &'static VectorIcon {
    if icon_theme::is_light() {
        light
    } else {
        dark
    }
}
pub fn get_app_icon_asset(id: &str) -> Option<&'static AppIconAsset> {
    get_app_icon_for(icon_theme::current(), id)
}
pub fn get_app_icon_for(theme: IconTheme, id: &str) -> Option<&'static AppIconAsset> {
    match theme {
        IconTheme::Folio => folio_app(id),
        IconTheme::Numix => numix_app(id),
        IconTheme::Colloid => colloid_app(id),
        IconTheme::WhiteSur => whitesur_app(id),
    }
}
fn colloid_app(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "clock" => Some(choose(&COLLOID_DARK_CLOCK, &COLLOID_LIGHT_CLOCK)),
        "timer" => Some(choose(&COLLOID_DARK_TIMER, &COLLOID_LIGHT_TIMER)),
        "notes" => Some(choose(&COLLOID_DARK_NOTES, &COLLOID_LIGHT_NOTES)),
        "calculator" => Some(choose(&COLLOID_DARK_CALCULATOR, &COLLOID_LIGHT_CALCULATOR)),
        "mac" => Some(choose(&COLLOID_DARK_MAC, &COLLOID_LIGHT_MAC)),
        "settings" => Some(choose(&COLLOID_DARK_SETTINGS, &COLLOID_LIGHT_SETTINGS)),
        "display" => Some(choose(&COLLOID_DARK_DISPLAY, &COLLOID_LIGHT_DISPLAY)),
        "screen" => Some(choose(&COLLOID_DARK_SCREEN, &COLLOID_LIGHT_SCREEN)),
        "file-manager" => Some(choose(
            &COLLOID_DARK_FILE_MANAGER,
            &COLLOID_LIGHT_FILE_MANAGER,
        )),
        "office-viewer" => Some(choose(
            &COLLOID_DARK_OFFICE_VIEWER,
            &COLLOID_LIGHT_OFFICE_VIEWER,
        )),
        "sub2api-monitor" => Some(choose(
            &COLLOID_DARK_SUB2API_MONITOR,
            &COLLOID_LIGHT_SUB2API_MONITOR,
        )),
        _ => None,
    }
}
fn whitesur_app(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "clock" => Some(choose(&WHITESUR_DARK_CLOCK, &WHITESUR_LIGHT_CLOCK)),
        "timer" => Some(choose(&WHITESUR_DARK_TIMER, &WHITESUR_LIGHT_TIMER)),
        "notes" => Some(choose(&WHITESUR_DARK_NOTES, &WHITESUR_LIGHT_NOTES)),
        "calculator" => Some(choose(
            &WHITESUR_DARK_CALCULATOR,
            &WHITESUR_LIGHT_CALCULATOR,
        )),
        "mac" => Some(choose(&WHITESUR_DARK_MAC, &WHITESUR_LIGHT_MAC)),
        "settings" => Some(choose(&WHITESUR_DARK_SETTINGS, &WHITESUR_LIGHT_SETTINGS)),
        "display" => Some(choose(&WHITESUR_DARK_DISPLAY, &WHITESUR_LIGHT_DISPLAY)),
        "screen" => Some(choose(&WHITESUR_DARK_SCREEN, &WHITESUR_LIGHT_SCREEN)),
        "file-manager" => Some(choose(
            &WHITESUR_DARK_FILE_MANAGER,
            &WHITESUR_LIGHT_FILE_MANAGER,
        )),
        "office-viewer" => Some(choose(
            &WHITESUR_DARK_OFFICE_VIEWER,
            &WHITESUR_LIGHT_OFFICE_VIEWER,
        )),
        "sub2api-monitor" => Some(choose(
            &WHITESUR_DARK_SUB2API_MONITOR,
            &WHITESUR_LIGHT_SUB2API_MONITOR,
        )),
        _ => None,
    }
}
fn numix_app(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "clock" => Some(choose(&NUMIX_DARK_CLOCK, &NUMIX_LIGHT_CLOCK)),
        "timer" => Some(choose(&NUMIX_DARK_TIMER, &NUMIX_LIGHT_TIMER)),
        "notes" => Some(choose(&NUMIX_DARK_NOTES, &NUMIX_LIGHT_NOTES)),
        "calculator" => Some(choose(&NUMIX_DARK_CALCULATOR, &NUMIX_LIGHT_CALCULATOR)),
        "mac" => Some(choose(&NUMIX_DARK_MAC, &NUMIX_LIGHT_MAC)),
        "settings" => Some(choose(&NUMIX_DARK_SETTINGS, &NUMIX_LIGHT_SETTINGS)),
        "display" => Some(choose(&NUMIX_DARK_DISPLAY, &NUMIX_LIGHT_DISPLAY)),
        "screen" => Some(choose(&NUMIX_DARK_SCREEN, &NUMIX_LIGHT_SCREEN)),
        "file-manager" => Some(choose(&NUMIX_DARK_FILE_MANAGER, &NUMIX_LIGHT_FILE_MANAGER)),
        "office-viewer" => Some(choose(
            &NUMIX_DARK_OFFICE_VIEWER,
            &NUMIX_LIGHT_OFFICE_VIEWER,
        )),
        "sub2api-monitor" => Some(choose(
            &NUMIX_DARK_SUB2API_MONITOR,
            &NUMIX_LIGHT_SUB2API_MONITOR,
        )),
        _ => None,
    }
}
pub fn get_settings_icon(id: &str) -> Option<&'static VectorIcon> {
    match icon_theme::current() {
        IconTheme::Folio => None,
        IconTheme::Numix => numix_setting(id),
        IconTheme::Colloid => colloid_setting(id),
        IconTheme::WhiteSur => whitesur_setting(id),
    }
}
fn colloid_setting(id: &str) -> Option<&'static VectorIcon> {
    match id {
        "wifi" => Some(choose(&COLLOID_DARK_WIFI, &COLLOID_LIGHT_WIFI)),
        "bluetooth" => Some(choose(&COLLOID_DARK_BLUETOOTH, &COLLOID_LIGHT_BLUETOOTH)),
        "display-settings" => Some(choose(
            &COLLOID_DARK_DISPLAY_SETTINGS,
            &COLLOID_LIGHT_DISPLAY_SETTINGS,
        )),
        "date-time" => Some(choose(&COLLOID_DARK_DATE_TIME, &COLLOID_LIGHT_DATE_TIME)),
        "storage" => Some(choose(&COLLOID_DARK_STORAGE, &COLLOID_LIGHT_STORAGE)),
        "about" => Some(choose(&COLLOID_DARK_ABOUT, &COLLOID_LIGHT_ABOUT)),
        "battery" => Some(choose(&COLLOID_DARK_BATTERY, &COLLOID_LIGHT_BATTERY)),
        "usb" => Some(choose(&COLLOID_DARK_USB, &COLLOID_LIGHT_USB)),
        "brightness" => Some(choose(&COLLOID_DARK_BRIGHTNESS, &COLLOID_LIGHT_BRIGHTNESS)),
        "audio" => Some(choose(&COLLOID_DARK_AUDIO, &COLLOID_LIGHT_AUDIO)),
        _ => None,
    }
}
fn whitesur_setting(id: &str) -> Option<&'static VectorIcon> {
    match id {
        "wifi" => Some(choose(&WHITESUR_DARK_WIFI, &WHITESUR_LIGHT_WIFI)),
        "bluetooth" => Some(choose(&WHITESUR_DARK_BLUETOOTH, &WHITESUR_LIGHT_BLUETOOTH)),
        "display-settings" => Some(choose(
            &WHITESUR_DARK_DISPLAY_SETTINGS,
            &WHITESUR_LIGHT_DISPLAY_SETTINGS,
        )),
        "date-time" => Some(choose(&WHITESUR_DARK_DATE_TIME, &WHITESUR_LIGHT_DATE_TIME)),
        "storage" => Some(choose(&WHITESUR_DARK_STORAGE, &WHITESUR_LIGHT_STORAGE)),
        "about" => Some(choose(&WHITESUR_DARK_ABOUT, &WHITESUR_LIGHT_ABOUT)),
        "battery" => Some(choose(&WHITESUR_DARK_BATTERY, &WHITESUR_LIGHT_BATTERY)),
        "usb" => Some(choose(&WHITESUR_DARK_USB, &WHITESUR_LIGHT_USB)),
        "brightness" => Some(choose(
            &WHITESUR_DARK_BRIGHTNESS,
            &WHITESUR_LIGHT_BRIGHTNESS,
        )),
        "audio" => Some(choose(&WHITESUR_DARK_AUDIO, &WHITESUR_LIGHT_AUDIO)),
        _ => None,
    }
}
fn numix_setting(id: &str) -> Option<&'static VectorIcon> {
    match id {
        "wifi" => Some(choose(&NUMIX_DARK_WIFI, &NUMIX_LIGHT_WIFI)),
        "bluetooth" => Some(choose(&NUMIX_DARK_BLUETOOTH, &NUMIX_LIGHT_BLUETOOTH)),
        "display-settings" => Some(choose(
            &NUMIX_DARK_DISPLAY_SETTINGS,
            &NUMIX_LIGHT_DISPLAY_SETTINGS,
        )),
        "date-time" => Some(choose(&NUMIX_DARK_DATE_TIME, &NUMIX_LIGHT_DATE_TIME)),
        "storage" => Some(choose(&NUMIX_DARK_STORAGE, &NUMIX_LIGHT_STORAGE)),
        "about" => Some(choose(&NUMIX_DARK_ABOUT, &NUMIX_LIGHT_ABOUT)),
        "battery" => Some(choose(&NUMIX_DARK_BATTERY, &NUMIX_LIGHT_BATTERY)),
        "usb" => Some(choose(&NUMIX_DARK_USB, &NUMIX_LIGHT_USB)),
        "brightness" => Some(choose(&NUMIX_DARK_BRIGHTNESS, &NUMIX_LIGHT_BRIGHTNESS)),
        "audio" => Some(choose(&NUMIX_DARK_AUDIO, &NUMIX_LIGHT_AUDIO)),
        _ => None,
    }
}
pub fn is_live_clock(icon: &VectorIcon) -> bool {
    [
        &DESKTOP_CLOCK,
        &FOLIO_LIGHT_CLOCK,
        &NUMIX_DARK_CLOCK,
        &NUMIX_LIGHT_CLOCK,
        &COLLOID_DARK_CLOCK,
        &COLLOID_LIGHT_CLOCK,
        &WHITESUR_DARK_CLOCK,
        &WHITESUR_LIGHT_CLOCK,
    ]
    .iter()
    .any(|candidate| std::ptr::eq(icon, *candidate))
}

/// Theme color sampled from the selected icon tile, reused by launch transitions.
pub fn launch_color(id: &str) -> Color {
    launch_color_for(id, icon_theme::is_light())
}
pub fn launch_color_for(id: &str, light_appearance: bool) -> Color {
    launch_color_for_theme(id, light_appearance, icon_theme::current())
}
pub fn launch_color_for_theme(id: &str, light_appearance: bool, theme: IconTheme) -> Color {
    if theme == IconTheme::Folio {
        return folio_color(id, light_appearance);
    }
    if theme == IconTheme::Numix {
        return numix_color(id, light_appearance);
    }
    if theme == IconTheme::WhiteSur {
        return whitesur_color(id, light_appearance);
    }
    let (dark, light) = match id {
        "clock" => (0x2c4147, 0xf0f6f5),
        "timer" => (0x482e39, 0xfce7e8),
        "notes" => (0x2b384e, 0xf4f7fe),
        "calculator" => (0x294465, 0x447ccc),
        "mac" => (0x31394b, 0xe5e9f2),
        "settings" => (0x2d3747, 0xe9edf5),
        "display" => (0x253c59, 0xe1edff),
        "screen" => (0x482936, 0xfbe3e6),
        "file-manager" => (0x27374d, 0xedf4ff),
        "office-viewer" => (0x303346, 0xf0f0fa),
        "sub2api-monitor" => (0x3e766e, 0x83b8b0),
        _ => (0x305f52, 0xd6e8dd),
    };
    Color::from_hex(if light_appearance { light } else { dark })
}

fn folio_app(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "calculator" => Some(choose(&DESKTOP_CALCULATOR, &FOLIO_LIGHT_CALCULATOR)),
        "clock" => Some(choose(&DESKTOP_CLOCK, &FOLIO_LIGHT_CLOCK)),
        "display" => Some(choose(&DESKTOP_DISPLAY, &FOLIO_LIGHT_DISPLAY)),
        "file-manager" => Some(choose(&DESKTOP_FILE_MANAGER, &FOLIO_LIGHT_FILE_MANAGER)),
        "mac" => Some(choose(&DESKTOP_MAC, &FOLIO_LIGHT_MAC)),
        "notes" => Some(choose(&DESKTOP_NOTES, &FOLIO_LIGHT_NOTES)),
        "office-viewer" => Some(choose(&DESKTOP_OFFICE_VIEWER, &FOLIO_LIGHT_OFFICE_VIEWER)),
        "screen" => Some(choose(&DESKTOP_SCREEN, &FOLIO_LIGHT_SCREEN)),
        "settings" => Some(choose(&DESKTOP_SETTINGS, &FOLIO_LIGHT_SETTINGS)),
        "sub2api-monitor" => Some(choose(
            &DESKTOP_SUB2API_MONITOR,
            &FOLIO_LIGHT_SUB2API_MONITOR,
        )),
        "timer" => Some(choose(&DESKTOP_TIMER, &FOLIO_LIGHT_TIMER)),
        _ => None,
    }
}
fn folio_color(id: &str, light: bool) -> Color {
    if light {
        return Color::from_hex(match id {
            "clock" => 0xd0e2e8,
            "timer" => 0xf3d9d0,
            "notes" => 0xf4e4bd,
            "calculator" => 0xd3e9df,
            "mac" => 0xd9e4f4,
            "settings" => 0xdde7eb,
            "display" => 0xd1eaeb,
            "screen" => 0xe4dcef,
            "file-manager" => 0xe4ddf0,
            "office-viewer" => 0xf1dce6,
            "sub2api-monitor" => 0xcde9e2,
            _ => 0xd6e8dd,
        });
    }
    Color::from_hex(match id {
        "calculator" => 0x689f8b,
        "clock" => 0x304b5c,
        "display" => 0x448b94,
        "file-manager" => 0x8c80ae,
        "mac" => 0x657fa8,
        "notes" => 0xcfaa61,
        "office-viewer" => 0xba8098,
        "screen" => 0x8d80a4,
        "settings" => 0x788a96,
        "sub2api-monitor" => 0x589e8e,
        "timer" => 0xd07969,
        _ => 0x305f52,
    })
}
fn numix_color(id: &str, light: bool) -> Color {
    let (dark, pale) = match id {
        "clock" => (0x2a554a, 0xd0e9e1),
        "timer" => (0x5d3a44, 0xf2dadd),
        "notes" => (0x574c30, 0xf2e5c2),
        "calculator" => (0x28586a, 0x9fcfd7),
        "mac" => (0x384860, 0xdce3ed),
        "settings" => (0x3b4d43, 0xe3eae3),
        "display" => (0x374a64, 0xe0e7f1),
        "screen" => (0x62353d, 0xf1d3d2),
        "file-manager" => (0x4d405c, 0xe3dbee),
        "office-viewer" => (0x5d3a4c, 0xf0d7e2),
        "sub2api-monitor" => (0x255449, 0xcee9df),
        _ => (0x305f52, 0xd6e8dd),
    };
    Color::from_hex(if light { pale } else { dark })
}

fn whitesur_color(id: &str, light: bool) -> Color {
    let (dark, pale) = match id {
        "calculator" => (0x3b6cb4, 0x246fd8),
        "clock" => (0x3a5850, 0xeafaf5),
        "display" => (0x354e6e, 0xecf4fc),
        "file-manager" => (0x2d3e56, 0xffffff),
        "mac" => (0x2d394e, 0x3f3f3f),
        "notes" => (0x30405a, 0xf9f8f6),
        "office-viewer" => (0x3b485e, 0xf4f5f6),
        "screen" => (0x98445b, 0xdc3d46),
        "settings" => (0x3e4e60, 0xb4babc),
        "sub2api-monitor" => (0x4a7c7d, 0xa4aeb1),
        "timer" => (0x000000, 0x000000),
        _ => (0x305f52, 0xd6e8dd),
    };
    Color::from_hex(if light { pale } else { dark })
}
