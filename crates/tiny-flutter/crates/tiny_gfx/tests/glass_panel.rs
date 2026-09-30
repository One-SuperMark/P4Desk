use tiny_gfx::{Canvas, Color, Pixmap565, Rect};
fn backdrop() -> Vec<u8> {
    (0..16 * 12)
        .flat_map(|i| [(i % 16 * 15) as u8, (i / 16 * 20) as u8, 160])
        .collect()
}
fn draw(c: &mut Canvas, source: &[u8], pressed: bool) {
    c.glass_panel(
        Rect::from_ltwh(8.0, 9.0, 82.0, 61.0),
        18.0,
        Color::from_rgb(32, 42, 56),
        source,
        16,
        12,
        65,
        pressed,
    );
}
#[test]
fn clipped_refresh_matches_complete_glass_without_sampling_old_frame() {
    let source = backdrop();
    let mut full = Pixmap565::new(110, 90).unwrap();
    full.fill(0x3a6b);
    draw(&mut Canvas::new(full.as_mut()), &source, false);
    let mut partial = Pixmap565::new(110, 90).unwrap();
    partial.fill(0x3a6b);
    for rect in [
        Rect::from_ltwh(0.0, 0.0, 37.0, 90.0),
        Rect::from_ltwh(37.0, 0.0, 73.0, 45.0),
        Rect::from_ltwh(37.0, 45.0, 73.0, 45.0),
    ] {
        let mut c = Canvas::new(partial.as_mut());
        c.clip_rect(rect);
        draw(&mut c, &source, false);
    }
    assert_eq!(full.data(), partial.data());
    assert_eq!(
        full.data()[9 * 110 + 8],
        0x3a6b,
        "rounded corner stays outside glass"
    );
    assert_ne!(full.data()[40 * 110 + 40], 0x3a6b);
}
#[test]
fn press_highlight_retains_the_same_shape_and_invalid_texture_is_safe() {
    let source = backdrop();
    let mut normal = Pixmap565::new(110, 90).unwrap();
    normal.fill(0xf81f);
    draw(&mut Canvas::new(normal.as_mut()), &source, false);
    let mut pressed = Pixmap565::new(110, 90).unwrap();
    pressed.fill(0xf81f);
    draw(&mut Canvas::new(pressed.as_mut()), &source, true);
    assert_ne!(normal.data(), pressed.data());
    for (i, (a, b)) in normal.data().iter().zip(pressed.data()).enumerate() {
        // RGB565 rounding can hide a very faint AA edge in one state. Check
        // the stable interior/exterior against the rounded-rectangle shape,
        // allowing only the one-pixel antialias band to quantize differently.
        let x = (i % 110) as f32 + 0.5;
        let y = (i / 110) as f32 + 0.5;
        let cx = x.clamp(26.0, 72.0);
        let cy = y.clamp(27.0, 52.0);
        let distance = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        if distance > 19.0 {
            assert_eq!(
                (*a, *b),
                (0xf81f, 0xf81f),
                "press stays inside the same outline"
            );
        } else if distance < 17.0 {
            assert_ne!(*a, 0xf81f, "normal interior is filled");
            assert_ne!(*b, 0xf81f, "pressed interior is filled");
        }
    }
    let before = pressed.data().to_vec();
    draw(&mut Canvas::new(pressed.as_mut()), &[], false);
    assert_eq!(before, pressed.data());
}

#[test]
fn lens_card_refraction_is_identical_during_clipped_refresh() {
    let source = backdrop();
    for finish in [tiny_gfx::GlassFinish::Lens, tiny_gfx::GlassFinish::Frosted] {
        let draw_lens = |c: &mut Canvas| {
            c.glass_panel_with_finish(
                Rect::from_ltwh(8.0, 9.0, 82.0, 61.0),
                18.0,
                Color::from_rgb(32, 42, 56),
                &source,
                16,
                12,
                65,
                false,
                finish,
            );
        };
        let mut full = Pixmap565::new(110, 90).unwrap();
        full.fill(0xf81f);
        draw_lens(&mut Canvas::new(full.as_mut()));
        let mut partial = Pixmap565::new(110, 90).unwrap();
        partial.fill(0xf81f);
        for x in (0..110).step_by(11) {
            let mut c = Canvas::new(partial.as_mut());
            c.clip_rect(Rect::from_ltwh(x as f32, 0.0, 11.0, 90.0));
            draw_lens(&mut c);
        }
        assert_eq!(full.data(), partial.data());
        assert_eq!(full.data()[9 * 110 + 8], 0xf81f);
    }
}
