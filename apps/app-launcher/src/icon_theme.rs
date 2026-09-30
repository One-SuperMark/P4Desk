//! Icon selection is independent from the light/dark appearance.
use serde::{Deserialize, Serialize};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconTheme {
    Folio,
    Numix,
    WhiteSur,
    #[default]
    #[serde(other)]
    Colloid,
}
impl IconTheme {
    pub const ALL: [Self; 4] = [Self::Folio, Self::Numix, Self::Colloid, Self::WhiteSur];
    pub const fn index(self) -> u8 {
        match self {
            Self::Folio => 0,
            Self::Numix => 1,
            Self::Colloid => 2,
            Self::WhiteSur => 3,
        }
    }
    pub fn from_index(value: u8) -> Self {
        Self::ALL.get(value as usize).copied().unwrap_or_default()
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Folio => "Folio",
            Self::Numix => "Numix Circle",
            Self::Colloid => "Colloid",
            Self::WhiteSur => "WhiteSur",
        }
    }
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Folio => "原版圆形",
            Self::Numix => "柔和彩色",
            Self::Colloid => "圆角方形",
            Self::WhiteSur => "精致立体",
        }
    }
}
thread_local! { static CURRENT: Cell<IconTheme> = const { Cell::new(IconTheme::Colloid) }; }
pub fn configure(theme: IconTheme) {
    CURRENT.with(|v| v.set(theme));
}
pub fn current() -> IconTheme {
    CURRENT.with(Cell::get)
}

thread_local! { static LIGHT: Cell<Option<bool>> = const { Cell::new(None) }; }
pub fn configure_light(value: Option<bool>) {
    LIGHT.with(|v| v.set(value));
}
pub fn is_light() -> bool {
    LIGHT
        .with(Cell::get)
        .unwrap_or_else(tiny_flutter::theme::Folio::is_light)
}
/// Page rasterization includes both icon color and appearance-dependent labels.
pub fn raster_key() -> u64 {
    current().index() as u64 * 4
        + u64::from(is_light()) * 2
        + u64::from(tiny_flutter::theme::Folio::is_light())
}
