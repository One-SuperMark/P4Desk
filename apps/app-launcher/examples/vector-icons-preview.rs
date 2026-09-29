//! Render the full SVG catalog with the production vector rasterizer.
use app_launcher::headless::HeadlessBackend;
use std::path::PathBuf;
use tiny_flutter::graphics::svg_icons_generated::ALL_VECTOR_ICONS;
use tiny_flutter::prelude::*;
struct Catalog;
impl CustomPainter for Catalog {
    fn paint(&self, canvas: &mut Canvas, _size: Size) {
        canvas.clear(Color::from_hex(0x171a1d));
        let font = Font::default_font();
        for (index, (name, icon)) in ALL_VECTOR_ICONS.iter().enumerate() {
            let x = (index % 6) as f32 * 170.0;
            let y = (index / 6) as f32 * 150.0;
            let tint = if *name == "UI_CHECK_CIRCLE" {
                Color::from_hex(0x963e46)
            } else {
                Color::WHITE
            };
            icon.paint(
                canvas,
                Rect::from_ltwh(x + 41.0, y + 13.0, 88.0, 88.0),
                tint,
            );
            let label = name
                .strip_prefix("DESKTOP_")
                .or_else(|| name.strip_prefix("UI_"))
                .unwrap_or(name);
            let width = font.measure_text(label, 14.0).width;
            canvas.draw_text(
                label,
                &font,
                14.0,
                Point::new(x + (170.0 - width) * 0.5, y + 113.0),
                Color::from_hex(0xc3c8ce),
            );
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/vector-icons.png".into()),
    );
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let height = ALL_VECTOR_ICONS.len().div_ceil(6) * 150;
    let size = Size::new(1020.0, height as f32);
    let mut backend = HeadlessBackend::new(1020, height);
    App::new(CustomPaint::new(Catalog).size(size), size).step(&mut backend);
    #[cfg(feature = "screenshots")]
    backend.screenshot(&out)?;
    Ok(())
}
