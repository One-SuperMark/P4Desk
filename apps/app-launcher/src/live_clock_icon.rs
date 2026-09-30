//! Colloid SVG dial with antialiased hands driven by the same local time as the desktop.
#[cfg(test)]
use tiny_flutter::graphics::colloid_icons_generated::COLLOID_DARK_CLOCK as DESKTOP_CLOCK;
use tiny_flutter::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
    second: u8,
}
impl ClockTime {
    pub fn parse(clock: &str) -> Option<Self> {
        let b = clock.as_bytes();
        if b.len() != 8 || b[2] != b':' || b[5] != b':' {
            return None;
        }
        let pair = |i: usize| {
            (b[i].is_ascii_digit() && b[i + 1].is_ascii_digit())
                .then(|| (b[i] - b'0') * 10 + b[i + 1] - b'0')
        };
        let result = Self {
            hour: pair(0)?,
            minute: pair(3)?,
            second: pair(6)?,
        };
        (result.hour < 24 && result.minute < 60 && result.second < 60).then_some(result)
    }
    fn turns(self) -> [f32; 3] {
        let minute = self.minute as f32 + self.second as f32 / 60.0;
        [
            ((self.hour % 12) as f32 + minute / 60.0) / 12.0,
            minute / 60.0,
            self.second as f32 / 60.0,
        ]
    }
}

pub fn paint_app_icon(
    asset: &'static VectorIcon,
    canvas: &mut Canvas,
    bounds: Rect,
    clock: Option<ClockTime>,
    opacity: f32,
) {
    asset.paint_with_opacity(canvas, bounds, Color::WHITE, opacity);
    if tiny_flutter::widgets::page_raster::is_page_snapshot() {
        return;
    }
    if !crate::app_icons::is_live_clock(asset)
        || bounds.width <= 0.0
        || bounds.height <= 0.0
        || opacity <= 0.0
    {
        return;
    }
    paint_clock_hands(canvas, bounds, clock, opacity);
}

pub fn paint_clock_hands(
    canvas: &mut Canvas,
    bounds: Rect,
    clock: Option<ClockTime>,
    opacity: f32,
) {
    paint_clock_hands_for_theme(canvas, bounds, clock, opacity, crate::icon_theme::current());
}

pub fn paint_clock_hands_for_theme(
    canvas: &mut Canvas,
    bounds: Rect,
    clock: Option<ClockTime>,
    opacity: f32,
    theme: crate::icon_theme::IconTheme,
) {
    let Some(clock) = clock else { return }; // An unsynced dial must not claim a fixed time.
    if !canvas.is_rect_visible(bounds) {
        return;
    }
    let scale = (bounds.width / 128.0).min(bounds.height / 128.0);
    let center = Point::new(
        bounds.x + (bounds.width - 128.0 * scale) * 0.5 + 64.0 * scale,
        bounds.y + (bounds.height - 128.0 * scale) * 0.5 + 64.0 * scale,
    );
    let colloid = matches!(
        theme,
        crate::icon_theme::IconTheme::Colloid | crate::icon_theme::IconTheme::WhiteSur
    );
    let light = crate::icon_theme::is_light();
    let whitesur = theme == crate::icon_theme::IconTheme::WhiteSur;
    let blue = Color::from_hex(if whitesur {
        0xedf5ff
    } else if colloid {
        if light {
            0x345e62
        } else {
            0xd1e9e8
        }
    } else {
        0x304b5c
    })
    .with_opacity(opacity);
    let second_color = Color::from_hex(if whitesur {
        if light {
            0xed6575
        } else {
            0xff8b91
        }
    } else if colloid {
        if light {
            0xc74e58
        } else {
            0xff9ba3
        }
    } else {
        0xc96455
    })
    .with_opacity(opacity);
    let lengths = match theme {
        crate::icon_theme::IconTheme::Folio => [14.0, 20.0, 22.0],
        crate::icon_theme::IconTheme::Numix => [21.0, 29.0, 32.0],
        crate::icon_theme::IconTheme::Colloid | crate::icon_theme::IconTheme::WhiteSur => {
            [26.0, 36.0, 39.0]
        }
    };
    // Short hour, long minute and fine second hand, all inside the supplied Colloid dial.
    for (i, turn) in clock.turns().into_iter().enumerate() {
        let radians = turn * std::f32::consts::TAU;
        let (dx, dy) = (radians.sin(), -radians.cos());
        let length = lengths[i] * scale;
        let tail = if i == 2 { 3.6 * scale } else { 0.0 };
        let mut line = tiny_gfx::PathBuilder::new();
        line.move_to(center.x - dx * tail, center.y - dy * tail);
        line.line_to(center.x + dx * length, center.y + dy * length);
        if let Some(path) = line.finish() {
            canvas.stroke_path(
                &path,
                &tiny_gfx::Paint::new(if i == 2 { second_color } else { blue }.to_gfx()),
                &tiny_gfx::Stroke {
                    width: [4.2, 3.0, 1.25][i] * scale,
                    line_cap: tiny_gfx::LineCap::Round,
                    ..Default::default()
                },
            );
        }
    }
    canvas.paint_circle(
        center,
        3.0 * scale,
        &tiny_gfx::Paint::new(blue.to_gfx()),
        None,
    );
    canvas.paint_circle(
        center,
        1.25 * scale,
        &tiny_gfx::Paint::new(second_color.to_gfx()),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_uses_twelve_hour_time_and_fractional_hour_minute_positions() {
        assert_eq!(ClockTime::parse("00:00:00").unwrap().turns(), [0.0; 3]);
        assert_eq!(ClockTime::parse("12:00:00").unwrap().turns(), [0.0; 3]);
        let positions = ClockTime::parse("15:30:30").unwrap().turns();
        assert!((positions[0] - (3.0 + 30.5 / 60.0) / 12.0).abs() < 0.00001);
        assert!((positions[1] - 30.5 / 60.0).abs() < 0.00001);
        assert_eq!(positions[2], 0.5);
        for invalid in [
            "--:--:--",
            "24:00:00",
            "12:60:00",
            "12:00:60",
            "12:03",
            "12：03：04",
        ] {
            assert_eq!(ClockTime::parse(invalid), None);
        }
    }

    #[test]
    fn moving_hands_preserve_silhouette_and_respect_partial_redraw() {
        let render = |clock, clip| {
            let mut image = tiny_gfx::Pixmap565::new(140, 140).unwrap();
            let mut canvas = Canvas::new(image.as_mut());
            if let Some(clip) = clip {
                canvas.clip_rect(clip);
            }
            paint_app_icon(
                &DESKTOP_CLOCK,
                &mut canvas,
                Rect::from_ltwh(0.0, 0.0, 140.0, 140.0),
                clock,
                1.0,
            );
            image
        };
        let empty = render(None, None);
        let a = render(ClockTime::parse("03:00:00"), None);
        let b = render(ClockTime::parse("03:00:01"), None);
        assert_ne!(a.data(), b.data());
        for y in 0..140usize {
            for x in 0..140usize {
                if (x as f32 - 70.0).hypot(y as f32 - 70.0) > 45.0 {
                    assert_eq!(a.data()[y * 140 + x], empty.data()[y * 140 + x]);
                }
            }
        }
        let clipped = render(
            ClockTime::parse("03:00:00"),
            Some(Rect::from_ltwh(53.0, 39.0, 30.0, 58.0)),
        );
        for y in 0..140usize {
            for x in 0..140usize {
                assert_eq!(
                    clipped.data()[y * 140 + x],
                    if (53..83).contains(&x) && (39..97).contains(&y) {
                        a.data()[y * 140 + x]
                    } else {
                        0
                    }
                );
            }
        }
    }
}
