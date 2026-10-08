//! Small static vector details for the usage monitor.
//!
//! Provider marks are original geometric identifiers, not official logos.
//! Painters use only their widget bounds; they have no timer or redraw loop.
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

/// Stable identity tint, independent of the current rank or process hash seed.
pub(super) fn identity_color(key: &str) -> Color {
    let hash = key.bytes().fold(0x811c_9dc5u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });
    let (dark, light) = match hash % 8 {
        0 => (0x83b8ef, 0x246aa7),
        1 => (0x81c8ac, 0x277653),
        2 => (0xbea6e6, 0x7955a8),
        3 => (0xe6b081, 0x9b602d),
        4 => (0x88c9d3, 0x287985),
        5 => (0xe1a4bd, 0x9c4c70),
        6 => (0xb5c489, 0x657a30),
        _ => (0xa8afe3, 0x606baf),
    };
    Folio::pick(Color::from_hex(dark), Color::from_hex(light))
}

fn bounded_size(size: f32) -> f32 {
    if size.is_finite() {
        size.clamp(0.0, 128.0)
    } else {
        0.0
    }
}

#[derive(Clone, Copy)]
enum ProviderKind {
    Network,
    Rays,
    Grid,
}

struct ProviderMark {
    kind: ProviderKind,
    color: Color,
}

fn line(canvas: &mut Canvas, points: &[(f32, f32)], color: Color, width: f32, scale: f32) {
    let mut path = tiny_gfx::PathBuilder::new();
    for (index, &(x, y)) in points.iter().enumerate() {
        if index == 0 {
            path.move_to(x * scale, y * scale);
        } else {
            path.line_to(x * scale, y * scale);
        }
    }
    if let Some(path) = path.finish() {
        canvas.stroke_path(
            &path,
            &tiny_gfx::Paint::new(color.to_gfx()),
            &tiny_gfx::Stroke {
                width: width * scale,
                line_cap: tiny_gfx::LineCap::Round,
                line_join: tiny_gfx::LineJoin::Round,
                ..Default::default()
            },
        );
    }
}

impl CustomPainter for ProviderMark {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let side = bounded_size(size.width.min(size.height));
        if side <= 0.0 {
            return;
        }
        canvas.save();
        canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, side, side));
        canvas.paint_circle(
            Point::new(side / 2.0, side / 2.0),
            side / 2.0,
            &tiny_gfx::Paint::new(
                self.color
                    .with_opacity(if Folio::is_light() { 0.11 } else { 0.16 })
                    .to_gfx(),
            ),
            None,
        );
        // Keep the CustomPaint widget's translation intact. Scaling the canvas
        // also scales that translation in tiny_gfx; scale local geometry instead.
        let scale = side / 24.0;
        match self.kind {
            ProviderKind::Network => {
                // Three connected nodes form a compact network identifier.
                line(
                    canvas,
                    &[(12.0, 6.0), (6.8, 16.0), (17.2, 16.0), (12.0, 6.0)],
                    self.color,
                    1.5,
                    scale,
                );
                for (x, y) in [(12.0, 6.0), (6.8, 16.0), (17.2, 16.0)] {
                    canvas.paint_circle(
                        Point::new(x * scale, y * scale),
                        2.0 * scale,
                        &tiny_gfx::Paint::new(self.color.to_gfx()),
                        None,
                    );
                }
            }
            ProviderKind::Rays => {
                // Three offset rays have a distinct, warm provider silhouette.
                for points in [
                    [(6.5, 16.0), (10.0, 8.0)],
                    [(11.0, 16.0), (14.5, 8.0)],
                    [(15.5, 16.0), (18.0, 10.0)],
                ] {
                    line(canvas, &points, self.color, 2.0, scale);
                }
            }
            ProviderKind::Grid => {
                for (x, y) in [(6.0, 6.0), (13.0, 6.0), (6.0, 13.0), (13.0, 13.0)] {
                    canvas.draw_rrect_aa(
                        RRect::from_rect_and_radius(
                            Rect::from_ltwh(x * scale, y * scale, 5.0 * scale, 5.0 * scale),
                            Radius::circular(1.6 * scale),
                        ),
                        self.color,
                    );
                }
            }
        }
        canvas.restore();
    }
}

pub(super) fn provider(platform: &str, size: f32) -> CustomPaint {
    let (kind, color) = if platform.eq_ignore_ascii_case("openai") {
        (ProviderKind::Network, Folio::green())
    } else if platform.eq_ignore_ascii_case("anthropic") {
        (
            ProviderKind::Rays,
            Folio::pick(Color::from_hex(0xe8b494), Color::from_hex(0x9a6240)),
        )
    } else {
        (ProviderKind::Grid, identity_color(platform))
    };
    let side = bounded_size(size);
    CustomPaint::new(ProviderMark { kind, color }).size(Size::new(side, side))
}

/// A plain initial badge for a name that is already masked by the data layer.
pub(super) fn avatar(label: &str, size: f32, color: Color) -> Container {
    avatar_with_font(label, size, color, Font::default_font())
}

pub(super) fn user_avatar(label: &str, size: f32, color: Color) -> Container {
    avatar_with_font(label, size, color, Font::dynamic_font())
}

fn avatar_with_font(label: &str, size: f32, color: Color, font: &Font) -> Container {
    let side = bounded_size(size);
    let initial = label
        .chars()
        .find(|ch| ch.is_alphanumeric())
        .and_then(|ch| ch.to_uppercase().next())
        .unwrap_or('?');
    Container::new()
        .width(side)
        .height(side)
        .border_radius(side * 0.5)
        .color(color.with_opacity(if Folio::is_light() { 0.12 } else { 0.18 }))
        .child(Center::new(
            Text::new(initial.to_string())
                .font(font.clone())
                .font_size(side * 0.44)
                .color(color),
        ))
}

struct CompositionBar {
    input: u64,
    cache: u64,
    output: u64,
}

impl CustomPainter for CompositionBar {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let width = size.width;
        let height = size.height.min(8.0);
        if !width.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }
        let shape = RRect::from_rect_and_radius(
            Rect::from_ltwh(0.0, 0.0, width, height),
            Radius::circular(height * 0.5),
        );
        // Sum after conversion: three valid u64 counters may exceed u64::MAX.
        let sum = self.input as f64 + self.cache as f64 + self.output as f64;
        if sum == 0.0 {
            canvas.draw_rrect_aa(shape, Folio::raised());
            return;
        }
        let mut left = 0.0;
        let mut accumulated = 0.0;
        for (value, color) in [
            (self.input, Folio::blue()),
            (self.cache, Folio::green()),
            (self.output, Folio::orange()),
        ] {
            accumulated += value as f64;
            // Pixel-aligned shared boundaries avoid gaps between clipped spans.
            let right = if accumulated >= sum {
                width
            } else {
                (width as f64 * accumulated / sum).round() as f32
            };
            let right = right.clamp(left, width);
            if right > left {
                canvas.save();
                canvas.clip_rect(Rect::from_ltwh(left, 0.0, right - left, height));
                // Clip the complete rounded shape, preserving smooth outer ends.
                canvas.draw_rrect_aa(shape, color);
                canvas.restore();
            }
            left = right;
        }
    }
}

pub(super) fn composition_bar(input: u64, cache: u64, output: u64, width: f32) -> CustomPaint {
    let width = if width.is_finite() {
        width.clamp(0.0, 1024.0)
    } else {
        0.0
    };
    CustomPaint::new(CompositionBar {
        input,
        cache,
        output,
    })
    .size(Size::new(width, 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_flutter::tiny_gfx::Pixmap565;

    const BACKGROUND: Color = Color::from_hex(0x263547);

    struct RestoreAppearance(bool, u8);
    impl RestoreAppearance {
        fn capture() -> Self {
            Self(Folio::is_light(), Folio::glass_amount())
        }
    }
    impl Drop for RestoreAppearance {
        fn drop(&mut self) {
            Folio::configure(self.0, self.1);
        }
    }

    fn render(widget: CustomPaint, size: Size, offset: Offset) -> Pixmap565 {
        let mut render = widget.create_render_object();
        assert_eq!(render.layout(&BoxConstraints::tight(size)), size);
        let mut pixels = Pixmap565::new(200, 120).unwrap();
        pixels.fill(BACKGROUND.to_rgb565());
        render.paint(&mut Canvas::new(pixels.as_mut()), offset);
        pixels
    }

    fn assert_outside_unchanged(pixels: &Pixmap565, size: Size, offset: Offset) {
        for y in 0..pixels.height() as usize {
            for x in 0..pixels.width() as usize {
                if (x as f32) < offset.dx
                    || (x as f32) >= offset.dx + size.width
                    || (y as f32) < offset.dy
                    || (y as f32) >= offset.dy + size.height
                {
                    assert_eq!(
                        pixels.data()[y * pixels.width() as usize + x],
                        BACKGROUND.to_rgb565(),
                        "drawing escaped bounds at ({x}, {y})"
                    );
                }
            }
        }
    }

    #[test]
    fn provider_foreground_survives_size_and_nonzero_widget_offset() {
        let _restore = RestoreAppearance::capture();
        for light in [false, true] {
            Folio::configure(light, 0);
            for side in [24.0, 34.0, 48.0] {
                let size = Size::new(side, side);
                for offset in [Offset::new(17.0, 11.0), Offset::new(71.0, 43.0)] {
                    for platform in ["openai", "anthropic", "other"] {
                        let widget = provider(platform, side);
                        let pixels = render(widget, size, offset);
                        assert_outside_unchanged(&pixels, size, offset);
                        let color = match platform {
                            "openai" => Folio::green(),
                            "anthropic" => {
                                Folio::pick(Color::from_hex(0xe8b494), Color::from_hex(0x9a6240))
                            }
                            _ => identity_color(platform),
                        }
                        .to_rgb565();
                        let foreground = pixels.data().iter().filter(|&&p| p == color).count();
                        // The translucent badge alone must never satisfy this.
                        assert!(
                            foreground >= side as usize / 2,
                            "missing foreground: {platform}, {side}px, {offset:?}, light={light}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn composition_zero_and_max_counters_stay_bounded_with_complete_segments() {
        let _restore = RestoreAppearance::capture();
        let size = Size::new(96.0, 8.0);
        let offset = Offset::new(23.0, 17.0);
        for light in [false, true] {
            Folio::configure(light, 0);
            let empty = render(composition_bar(0, 0, 0, 96.0), size, offset);
            assert_outside_unchanged(&empty, size, offset);
            assert_eq!(empty.data()[21 * 200 + 71], Folio::raised().to_rgb565());
            let filled = render(
                composition_bar(u64::MAX, u64::MAX, u64::MAX, 96.0),
                size,
                offset,
            );
            assert_outside_unchanged(&filled, size, offset);
            for (center, color) in [
                (39, Folio::blue()),
                (71, Folio::green()),
                (103, Folio::orange()),
            ] {
                assert_eq!(filled.data()[21 * 200 + center], color.to_rgb565());
            }
            // All three segments touch on the middle scanline; there is no seam.
            for x in 25..117 {
                assert_ne!(filled.data()[21 * 200 + x], BACKGROUND.to_rgb565());
            }
            // Rounded corners remain a partial coverage rather than square fills.
            assert_ne!(filled.data()[17 * 200 + 23], Folio::blue().to_rgb565());
        }
    }

    #[test]
    fn identity_color_is_independent_of_evaluation_order_and_returns_matching_palette() {
        let _restore = RestoreAppearance::capture();
        Folio::configure(false, 0);
        let first = identity_color("gpt-6-sol");
        for key in ["another model", "user:42", "gpt-6-astra"] {
            let _ = identity_color(key);
        }
        assert_eq!(identity_color("gpt-6-sol"), first);
        Folio::configure(true, 0);
        let light = identity_color("gpt-6-sol");
        assert_ne!(light, first);
        Folio::configure(false, 0);
        assert_eq!(identity_color("gpt-6-sol"), first);
    }
}
