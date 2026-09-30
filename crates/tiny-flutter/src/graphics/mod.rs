pub mod baked_font;
pub mod colloid_icons_generated;
pub mod numix_icons_generated;
pub mod svg_icons_generated;
pub mod ui_icons;
mod vector_cache;
pub use vector_cache::vector_cache_stats;
pub mod vector_icon;
pub mod baked_icons {
    pub use super::ui_icons::*;
}
pub mod canvas;
pub mod color;
pub mod font;
pub mod fontpack;
pub mod geometry;

pub use baked_font::{get_baked_font, BakedFont, BakedGlyph};
pub use canvas::Canvas;
pub use color::Color;
pub use font::Font;
pub use geometry::{EdgeInsets, Offset, Point, RRect, Radius, Rect, Size};
pub use ui_icons::*;
pub use vector_icon::{VectorCommand, VectorIcon, VectorLayer, VectorPaint, VectorShape};

pub mod whitesur_icons_generated;

pub mod folio_light_icons_generated;
