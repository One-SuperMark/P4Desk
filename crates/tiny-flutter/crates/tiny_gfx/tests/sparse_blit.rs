use tiny_gfx::raster::ImageSpan565;
use tiny_gfx::{Canvas, Color, Pixmap565, Rect};
#[test]
fn batched_spans_match_row_blits_across_clipping_translation_and_alpha() {
    let spans = [
        ImageSpan565 {
            start: 0,
            x: 0,
            y: 0,
            length: 8,
            opaque: true,
        },
        ImageSpan565 {
            start: 8,
            x: 4,
            y: 1,
            length: 10,
            opaque: false,
        },
        ImageSpan565 {
            start: 18,
            x: 2,
            y: 9,
            length: 8,
            opaque: true,
        },
        ImageSpan565 {
            start: 26,
            x: 10,
            y: 9,
            length: 5,
            opaque: false,
        },
    ];
    let rgb: Vec<u16> = (0..31).map(|i| (i * 1831) as u16).collect();
    let alpha: Vec<u8> = (0..31).map(|i| [0, 1, 128, 254, 255][i % 5]).collect();
    for x in [-40, -5, 0, 11, 29, 40] {
        for y in [-12, -3, 0, 15, 28] {
            for clip in [None, Some(Rect::from_ltwh(3.25, 2.5, 18.25, 14.25))] {
                let mut expected = Pixmap565::new(32, 24).unwrap();
                let mut actual = Pixmap565::new(32, 24).unwrap();
                for (pixels, batch) in [(&mut expected, false), (&mut actual, true)] {
                    let mut canvas = Canvas::new(pixels.as_mut());
                    canvas.clear(Color::from_rgb(35, 80, 125));
                    if let Some(r) = clip {
                        canvas.clip_rect(r);
                    }
                    canvas.translate(2.0, -1.0);
                    if batch {
                        canvas.blit_image_565_spans(x, y, &rgb, &alpha, &spans);
                    } else {
                        for s in &spans {
                            let a = s.start as usize;
                            let b = a + s.length as usize;
                            if s.opaque {
                                canvas.blit_image_565(
                                    x + s.x as i32,
                                    y + s.y as i32,
                                    s.length as u32,
                                    1,
                                    &rgb[a..b],
                                );
                            } else {
                                canvas.blit_image_565_with_alpha(
                                    x + s.x as i32,
                                    y + s.y as i32,
                                    s.length as u32,
                                    1,
                                    &rgb[a..b],
                                    &alpha[a..b],
                                );
                            }
                        }
                    }
                }
                assert!(
                    actual.data() == expected.data(),
                    "x={x} y={y} clip={clip:?}"
                );
            }
        }
    }
}
