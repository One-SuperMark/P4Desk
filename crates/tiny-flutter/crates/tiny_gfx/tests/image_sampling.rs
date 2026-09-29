use tiny_gfx::{blend_rgb565, Canvas, Pixmap565, Rect};

#[test]
fn native_coverage_preserves_white_corners_and_partial_transparency() {
    let rgb = vec![0xffff; 24];
    let mut alpha = vec![255; 24];
    alpha[7] = 128;
    alpha[8] = 0;
    let background = 0x3186;
    let mut pixmap = Pixmap565::new(6, 4).unwrap();
    pixmap.fill(background);
    Canvas::new(pixmap.as_mut()).blit_image_565_with_alpha(0, 0, 6, 4, &rgb, &alpha);
    assert_eq!(
        pixmap.data()[0],
        0xffff,
        "a supplied opaque corner is artwork, not a halo"
    );
    for (pixel, alpha) in pixmap.data().iter().zip(alpha) {
        assert_eq!(*pixel, blend_rgb565(background, 0xffff, alpha));
    }
}

#[test]
fn bilinear_edge_is_smooth_and_transparent_rgb_cannot_add_a_color_halo() {
    let alpha = [255, 0, 255, 0];
    let render = |transparent_color| {
        let mut pixmap = Pixmap565::new(9, 5).unwrap();
        Canvas::new(pixmap.as_mut()).blit_image_565_with_alpha_scaled(
            0,
            0,
            9,
            5,
            2,
            2,
            &[0xf800, transparent_color, 0xf800, transparent_color],
            &alpha,
        );
        pixmap
    };
    let blue = render(0x001f);
    assert_eq!(blue, render(0x07e0));
    assert_eq!(blue, render(0x0000));
    let row = &blue.data()[18..27];
    assert_eq!(row[0], 0xf800);
    assert_eq!(row[8], 0);
    assert!((14..=17).contains(&(row[4] >> 11)));
    assert!(row.windows(2).all(|pair| pair[0] >= pair[1]));
    assert!(row.iter().all(|pixel| pixel & 0x07ff == 0));
    let mut tones = row.to_vec();
    tones.sort_unstable();
    tones.dedup();
    assert!(
        tones.len() >= 5,
        "an enlarged coverage edge needs intermediate tones"
    );
}

#[test]
fn identity_scaling_is_pixel_exact_even_with_a_negative_origin_and_clip() {
    let rgb: Vec<u16> = (0..25).map(|i| 0xffff - i * 617).collect();
    let alpha: Vec<u8> = (0..25).map(|i| (i * 43 % 256) as u8).collect();
    let mut native = Pixmap565::new(10, 8).unwrap();
    native.fill(0x7bde);
    let mut scaled = native.clone();
    let clip = Rect::from_ltwh(1.0, 2.0, 6.0, 4.0);
    let mut canvas = Canvas::new(native.as_mut());
    canvas.clip_rect(clip);
    canvas.blit_image_565_with_alpha(-2, 1, 5, 5, &rgb, &alpha);
    let mut canvas = Canvas::new(scaled.as_mut());
    canvas.clip_rect(clip);
    canvas.blit_image_565_with_alpha_scaled(-2, 1, 5, 5, 5, 5, &rgb, &alpha);
    assert_eq!(native, scaled);
}

#[test]
fn clipped_scaled_image_matches_full_render_without_shifting_the_sampling_grid() {
    let rgb = [0xffff, 0xf800, 0x07e0, 0x001f];
    let alpha = [32, 255, 128, 0];
    let mut full = Pixmap565::new(9, 7).unwrap();
    full.fill(0x3186);
    let mut clipped = full.clone();
    Canvas::new(full.as_mut()).blit_image_565_with_alpha_scaled(-3, -2, 11, 9, 2, 2, &rgb, &alpha);
    let mut canvas = Canvas::new(clipped.as_mut());
    canvas.clip_rect(Rect::from_ltwh(2.0, 1.0, 4.0, 3.0));
    canvas.blit_image_565_with_alpha_scaled(-3, -2, 11, 9, 2, 2, &rgb, &alpha);
    for y in 0..7 {
        for x in 0..9 {
            assert_eq!(
                clipped.data()[y * 9 + x],
                if (2..6).contains(&x) && (1..4).contains(&y) {
                    full.data()[y * 9 + x]
                } else {
                    0x3186
                }
            );
        }
    }
}

#[test]
fn bad_dimensions_and_short_buffers_preserve_the_target() {
    let mut target = Pixmap565::new(4, 4).unwrap();
    target.fill(0x1234);
    let before = target.clone();
    let mut canvas = Canvas::new(target.as_mut());
    canvas.blit_image_565_with_alpha(0, 0, u32::MAX, 2, &[], &[]);
    canvas.blit_image_565_with_alpha(0, 0, 2, 2, &[0xffff; 3], &[255; 4]);
    canvas.blit_image_565_with_alpha_scaled(0, 0, u32::MAX, 2, 1, 1, &[0xffff], &[255]);
    canvas.blit_image_565_with_alpha_scaled(0, 0, 4, 4, 2, 2, &[0xffff; 4], &[255; 3]);
    canvas.blit_image_565_with_alpha_scaled(i32::MAX, i32::MAX, 4, 4, 1, 1, &[0xffff], &[255]);
    assert_eq!(target, before);
}
