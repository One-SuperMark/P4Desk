use tiny_gfx::*;

fn tick() -> Path {
    let mut path = PathBuilder::new();
    path.move_to(8.3, 23.4);
    path.line_to(19.6, 35.2);
    path.line_to(43.7, 10.1);
    path.finish().unwrap()
}
fn stroke(color: Color, clip: Option<Rect>, translated: bool) -> Pixmap565 {
    let mut pixmap = Pixmap565::new(64, 48).unwrap();
    let mut canvas = Canvas::new(pixmap.as_mut());
    if let Some(clip) = clip {
        canvas.clip_rect(clip);
    }
    if translated {
        canvas.translate(2.25, -3.4);
    }
    canvas.stroke_path(
        &tick(),
        &Paint::new(color),
        &Stroke {
            width: 5.4,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        },
    );
    pixmap
}
#[test]
fn diagonal_round_tick_has_coverage_and_never_double_blends_the_joint() {
    let opaque = stroke(Color::WHITE, None, false);
    let colors = opaque
        .data()
        .iter()
        .copied()
        .filter(|p| *p != 0 && *p != 0xffff)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        colors.len() >= 16,
        "vector diagonals need intermediate edge tones"
    );
    let translucent = stroke(Color::from_rgba(255, 255, 255, 128), None, false);
    let center = blend_rgb565(0, 0xffff, 128);
    assert_eq!(translucent.data()[33 * 64 + 20], center);
    assert!(translucent.data().iter().all(|p| *p <= center));
}
#[test]
fn vector_clip_matches_full_pixels_after_fractional_translation() {
    let clip = Rect::from_ltwh(15.0, 9.0, 19.0, 26.0);
    let full = stroke(Color::WHITE, None, true);
    let cut = stroke(Color::WHITE, Some(clip), true);
    for y in 0..48 {
        for x in 0..64 {
            assert_eq!(
                cut.data()[y * 64 + x],
                if (15..34).contains(&x) && (9..35).contains(&y) {
                    full.data()[y * 64 + x]
                } else {
                    0
                }
            );
        }
    }
}
#[test]
fn compound_fill_honors_even_odd_and_nonzero_interior_rules() {
    let mut path = PathBuilder::new();
    path.push_rect(Rect::from_ltwh(5.0, 5.0, 40.0, 36.0));
    path.push_rect(Rect::from_ltwh(14.0, 14.0, 20.0, 20.0));
    let path = path.finish().unwrap();
    for rule in [FillRule::EvenOdd, FillRule::Winding] {
        let mut pixmap = Pixmap565::new(50, 48).unwrap();
        Canvas::new(pixmap.as_mut()).fill_path(&path, &Paint::new(Color::WHITE), rule);
        assert_eq!(pixmap.data()[10 * 50 + 10], 0xffff);
        assert_eq!(
            pixmap.data()[24 * 50 + 24],
            if rule == FillRule::EvenOdd { 0 } else { 0xffff }
        );
    }
}
#[test]
fn filled_circle_is_symmetric_and_has_a_solid_center() {
    let mut path = PathBuilder::new();
    path.push_circle(24.0, 24.0, 18.75);
    let mut pixmap = Pixmap565::new(48, 48).unwrap();
    Canvas::new(pixmap.as_mut()).fill_path(
        &path.finish().unwrap(),
        &Paint::new(Color::WHITE),
        FillRule::Winding,
    );
    assert_eq!(pixmap.data()[24 * 48 + 24], 0xffff);
    assert_eq!(pixmap.data()[0], 0);
    for y in 0..48 {
        for x in 0..48 {
            assert_eq!(
                pixmap.data()[y * 48 + x],
                pixmap.data()[(47 - y) * 48 + 47 - x]
            );
        }
    }
}
#[test]
fn a_translated_scaled_gradient_remains_attached_to_its_path() {
    let mut path = PathBuilder::new();
    path.push_rect(Rect::from_ltwh(0.0, 0.0, 20.0, 20.0));
    let mut paint = Paint::new(Color::RED);
    paint.shader = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 20.0),
        vec![
            GradientStop::new(0.0, Color::RED),
            GradientStop::new(1.0, Color::BLUE),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    let mut pixmap = Pixmap565::new(80, 80).unwrap();
    let mut canvas = Canvas::new(pixmap.as_mut());
    canvas.scale(2.0, 2.0);
    canvas.translate(10.0, 16.0);
    canvas.fill_path(&path.finish().unwrap(), &paint, FillRule::Winding);
    let (top_r, _, top_b) = rgb565_to_rgb888(pixmap.data()[18 * 80 + 20]);
    let (bottom_r, _, bottom_b) = rgb565_to_rgb888(pixmap.data()[53 * 80 + 20]);
    assert!(top_r > 220 && top_b < 40);
    assert!(bottom_b > 220 && bottom_r < 40);
}
#[test]
fn clipped_offscreen_and_transparent_paths_cannot_write_outside_the_visible_row() {
    for transparent in [true, false] {
        let mut pixmap = Pixmap565::new(32, 24).unwrap();
        pixmap.fill(0x2104);
        let mut canvas = Canvas::new(pixmap.as_mut());
        if !transparent {
            canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, 4.0, 4.0));
            canvas.clip_rect(Rect::from_ltwh(20.0, 20.0, 4.0, 4.0));
        }
        canvas.stroke_path(
            &tick(),
            &Paint::new(Color::from_rgba(
                255,
                255,
                255,
                if transparent { 0 } else { 255 },
            )),
            &Stroke {
                width: 10.0,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Default::default()
            },
        );
        assert!(pixmap.data().iter().all(|p| *p == 0x2104));
    }
    let pixmap = stroke(
        Color::WHITE,
        Some(Rect::from_ltwh(-10.0, -10.0, 20.0, 20.0)),
        true,
    );
    assert!(pixmap
        .data()
        .iter()
        .enumerate()
        .all(|(i, p)| *p == 0 || (i % 64 < 10 && i / 64 < 10)));
}

#[test]
fn analytic_circle_and_rounded_rect_outlines_keep_holes_and_alpha_single_blended() {
    let expected = blend_rgb565(0, 0xffff, 128);
    for circle in [true, false] {
        let mut pixmap = Pixmap565::new(64, 64).unwrap();
        let mut canvas = Canvas::new(pixmap.as_mut());
        let paint = Paint::new(Color::from_rgba(255, 255, 255, 128));
        if circle {
            canvas.paint_circle(Point::new(32.0, 32.0), 22.0, &paint, Some(6.0));
        } else {
            canvas.paint_rrect(
                RRect::from_rect_xy(Rect::from_ltwh(10.0, 10.0, 44.0, 44.0), 14.0, 14.0),
                &paint,
                Some(6.0),
            );
        }
        assert_eq!(
            pixmap.data()[32 * 64 + 32],
            0,
            "outline must leave its interior empty"
        );
        assert_eq!(pixmap.data()[10 * 64 + 32], expected);
        assert!(pixmap.data().iter().all(|p| *p <= expected));
        assert!(pixmap.data().iter().any(|p| *p != 0 && *p != expected));
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(
                    pixmap.data()[y * 64 + x],
                    pixmap.data()[(63 - y) * 64 + 63 - x]
                );
            }
        }
    }
}

#[test]
fn analytic_shapes_obey_fractional_transform_gradient_and_partial_clip() {
    let mut paint = Paint::new(Color::RED);
    paint.shader = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 30.0),
        vec![
            GradientStop::new(0.0, Color::RED),
            GradientStop::new(1.0, Color::BLUE),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap();
    for circle in [true, false] {
        let mut full = Pixmap565::new(80, 80).unwrap();
        let mut cut = Pixmap565::new(80, 80).unwrap();
        for (pixels, clip) in [(&mut full, false), (&mut cut, true)] {
            let mut canvas = Canvas::new(pixels.as_mut());
            if clip {
                canvas.clip_rect(Rect::from_ltwh(13.0, 16.0, 35.0, 30.0));
            }
            canvas.scale(1.5, 1.5);
            canvas.translate(10.25, 12.5);
            if circle {
                canvas.paint_circle(Point::new(15.0, 15.0), 14.5, &paint, None);
            } else {
                canvas.paint_rrect(
                    RRect::from_rect_xy(Rect::from_ltwh(0.0, 0.0, 30.0, 30.0), 7.0, 7.0),
                    &paint,
                    None,
                );
            }
        }
        for y in 0..80 {
            for x in 0..80 {
                let expected = if (13..48).contains(&x) && (16..46).contains(&y) {
                    full.data()[y * 80 + x]
                } else {
                    0
                };
                assert_eq!(cut.data()[y * 80 + x], expected);
            }
        }
        let (top_r, _, top_b) = rgb565_to_rgb888(full.data()[18 * 80 + 32]);
        let (bottom_r, _, bottom_b) = rgb565_to_rgb888(full.data()[52 * 80 + 32]);
        assert!(top_r > 200 && top_b < 55);
        assert!(bottom_b > 200 && bottom_r < 55);
    }
}

#[test]
fn analytic_primitives_ignore_invisible_shapes_and_fill_a_stroke_wider_than_the_shape() {
    let mut pixels = Pixmap565::new(32, 32).unwrap();
    let mut canvas = Canvas::new(pixels.as_mut());
    let white = Paint::new(Color::WHITE);
    canvas.paint_circle(Point::new(16.0, 16.0), 4.0, &white, Some(16.0));
    assert_eq!(pixels.data()[16 * 32 + 16], 0xffff);
    pixels.fill(0x2104);
    let mut canvas = Canvas::new(pixels.as_mut());
    canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, 3.0, 3.0));
    canvas.clip_rect(Rect::from_ltwh(20.0, 20.0, 3.0, 3.0));
    canvas.paint_circle(Point::new(16.0, 16.0), 12.0, &white, None);
    canvas.paint_rrect(
        RRect::from_rect_xy(Rect::from_ltwh(0.0, 0.0, 30.0, 30.0), 8.0, 8.0),
        &white,
        None,
    );
    assert!(pixels.data().iter().all(|p| *p == 0x2104));
}
