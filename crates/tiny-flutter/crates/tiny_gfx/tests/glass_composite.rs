use tiny_gfx::*;

fn backdrop() -> Pixmap565 {
    let mut pixels = Pixmap565::new(64, 48).unwrap();
    for y in 0..48 {
        for x in 0..64 {
            pixels.data_mut()[y * 64 + x] = rgb888_to_rgb565(
                (x * 4) as u8,
                (y * 5) as u8,
                if (x / 4 + y / 4) % 2 == 0 { 220 } else { 30 },
            );
        }
    }
    pixels
}
#[test]
fn glass_tint_and_transparent_aperture_keep_the_correct_side_of_the_circle() {
    let source = backdrop();
    let tint = Color::from_rgb(241, 139, 119);
    for fill in [GlassFill::Inside, GlassFill::Outside, GlassFill::EdgeOnly] {
        let mut pixels = source.clone();
        Canvas::new(pixels.as_mut()).glass_circle(
            Point::new(32.0, 24.0),
            15.0,
            tint,
            fill,
            5.0,
            1.0,
        );
        assert_eq!(
            pixels.data()[24 * 64 + 32],
            if fill == GlassFill::Inside {
                tint.to_rgb565()
            } else {
                source.data()[24 * 64 + 32]
            }
        );
        assert_eq!(
            pixels.data()[0],
            if fill == GlassFill::Outside {
                tint.to_rgb565()
            } else {
                source.data()[0]
            }
        );
        assert_ne!(
            pixels.data()[24 * 64 + 46],
            source.data()[24 * 64 + 46],
            "refraction must remain visible over a patterned backdrop"
        );
    }
    for fill in [GlassFill::Inside, GlassFill::Outside, GlassFill::EdgeOnly] {
        let mut pixels = source.clone();
        Canvas::new(pixels.as_mut()).glass_circle(
            Point::new(32.0, 24.0),
            100.0,
            tint,
            fill,
            5.0,
            1.0,
        );
        if fill == GlassFill::Inside {
            assert!(pixels.data().iter().all(|p| *p == tint.to_rgb565()));
        } else {
            assert_eq!(
                pixels, source,
                "aperture beyond the corners must erase all tint and glass"
            );
        }
    }
}
#[test]
fn expansion_and_reveal_add_no_white_lip_or_inner_outline() {
    let base = Color::from_rgb(23, 26, 29).to_rgb565();
    let channels = |p: u16| [p >> 11, (p >> 5) & 63, p & 31];
    for tint in [
        Color::from_rgb(241, 139, 119),
        Color::from_rgb(142, 212, 181),
        Color::from_rgb(157, 189, 237),
    ] {
        for fill in [GlassFill::Inside, GlassFill::Outside, GlassFill::EdgeOnly] {
            let mut pixels = Pixmap565::new(64, 48).unwrap();
            pixels.data_mut().fill(base);
            Canvas::new(pixels.as_mut()).glass_circle(
                Point::new(31.5, 23.75),
                16.25,
                tint,
                fill,
                5.0,
                1.0,
            );
            let a = channels(base);
            let b = channels(tint.to_rgb565());
            for pixel in pixels.data() {
                for (i, v) in channels(*pixel).into_iter().enumerate() {
                    assert!(
                        v >= a[i].min(b[i]) && v <= a[i].max(b[i]),
                        "edge introduced a contrasting outline"
                    );
                }
            }
            if fill == GlassFill::EdgeOnly {
                assert!(
                    pixels.data().iter().all(|p| *p == base),
                    "flat background must not acquire a ring"
                );
            }
        }
    }
}
#[test]
fn glass_refraction_uses_the_unmodified_row_and_obeys_clip_transform_and_empty_clip() {
    let source = backdrop();
    let tint = Color::from_rgba(141, 212, 181, 230);
    for fill in [GlassFill::Inside, GlassFill::Outside, GlassFill::EdgeOnly] {
        for radius in [0.0, 12.75, 76.0] {
            let mut full = source.clone();
            let mut partial = source.clone();
            for (pixels, clip) in [(&mut full, false), (&mut partial, true)] {
                let mut canvas = Canvas::new(pixels.as_mut());
                if clip {
                    canvas.clip_rect(Rect::from_ltwh(10.0, 8.0, 30.0, 25.0));
                }
                canvas.scale(1.25, 1.25);
                canvas.translate(2.25, -1.5);
                canvas.glass_circle(Point::new(24.0, 20.0), radius, tint, fill, 5.0, 1.0);
            }
            for y in 0..48 {
                for x in 0..64 {
                    let i = y * 64 + x;
                    assert_eq!(
                        partial.data()[i],
                        if (10..40).contains(&x) && (8..33).contains(&y) {
                            full.data()[i]
                        } else {
                            source.data()[i]
                        }
                    );
                }
            }
        }
    }
    let mut pixels = source.clone();
    let mut canvas = Canvas::new(pixels.as_mut());
    canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, 3.0, 3.0));
    canvas.clip_rect(Rect::from_ltwh(20.0, 20.0, 3.0, 3.0));
    canvas.glass_circle(
        Point::new(32.0, 24.0),
        15.0,
        tint,
        GlassFill::Outside,
        5.0,
        1.0,
    );
    assert_eq!(pixels, source);
    for radius in [-1.0, f32::NAN, f32::INFINITY] {
        Canvas::new(pixels.as_mut()).glass_circle(
            Point::new(32.0, 24.0),
            radius,
            tint,
            GlassFill::Outside,
            5.0,
            1.0,
        );
        assert_eq!(pixels, source);
    }
}
