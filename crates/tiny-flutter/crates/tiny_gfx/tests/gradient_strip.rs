use tiny_gfx::*;

// Original per-pixel quantizer is the compatibility oracle for the strip copy.
fn reference(width: usize, x: usize, y: usize, a: (u8, u8, u8), b: (u8, u8, u8)) -> u16 {
    const B: [u8; 64] = [
        0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38,
        60, 28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57,
        25, 15, 47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
    ];
    let inv = if width > 1 {
        1.0 / (width - 1) as f32
    } else {
        0.0
    };
    let t = x as f32 * inv;
    let noise = (B[(y & 7) * 8 + (x & 7)] as f32 / 64.0 - 0.5) * 6.0;
    let q =
        |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t + noise + 0.5).clamp(0.0, 255.0) as u8;
    rgb888_to_rgb565(q(a.0, b.0), q(a.1, b.1), q(a.2, b.2))
}
#[test]
fn repeated_strip_is_pixel_exact_to_old_quantization_with_clip_and_narrow_surfaces() {
    for (w, h) in [(1, 1), (2, 9), (17, 33), (1024, 600)] {
        for (a, b) in [((12, 28, 46), (18, 63, 78)), ((255, 3, 0), (0, 253, 255))] {
            for clipped in [false, true] {
                let mut p = Pixmap565::new(w, h).unwrap();
                p.fill(0x2104);
                let (x1, y1, x2, y2) = if clipped {
                    (
                        w as usize / 4,
                        h as usize / 3,
                        (w as usize * 3 / 4).max(1),
                        (h as usize * 2 / 3).max(1),
                    )
                } else {
                    (0, 0, w as usize, h as usize)
                };
                let mut c = Canvas::new(p.as_mut());
                if clipped {
                    c.clip_rect(Rect::from_ltwh(
                        x1 as f32,
                        y1 as f32,
                        (x2 - x1) as f32,
                        (y2 - y1) as f32,
                    ));
                }
                c.fill_dithered_horizontal_gradient(a, b);
                for y in 0..h as usize {
                    for x in 0..w as usize {
                        assert_eq!(
                            p.data()[y * w as usize + x],
                            if x >= x1 && x < x2 && y >= y1 && y < y2 {
                                reference(w as usize, x, y, a, b)
                            } else {
                                0x2104
                            }
                        );
                    }
                }
            }
        }
    }
}
