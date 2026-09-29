use tiny_gfx::{blend_rgb565, Canvas, Color, Pixmap565, RRect, Rect};

fn render(color: Color, clip: Option<Rect>, rect: Rect) -> Pixmap565 {
    let mut p = Pixmap565::new(64, 48).unwrap();
    p.fill(0x2104);
    let mut c = Canvas::new(p.as_mut());
    if let Some(clip) = clip {
        c.clip_rect(clip);
    }
    c.draw_rrect_aa(RRect::from_rect_xy(rect, 14.0, 14.0), color);
    p
}
#[test]
fn capsule_has_symmetric_coverage_and_intermediate_edge_tones() {
    let p = render(Color::WHITE, None, Rect::from_ltwh(6.0, 10.0, 52.0, 28.0));
    let background = 0x2104;
    let mut tones = std::collections::BTreeSet::new();
    for y in 10..38 {
        for x in 6..58 {
            let px = p.data()[y * 64 + x];
            assert_eq!(px, p.data()[(47 - y) * 64 + x]);
            assert_eq!(px, p.data()[y * 64 + (63 - x)]);
            if px != background && px != 0xffff {
                tones.insert(px);
            }
        }
    }
    assert!(tones.len() >= 8);
    assert_eq!(p.data()[24 * 64 + 32], 0xffff);
    assert_eq!(p.data()[10 * 64 + 6], background);
}
#[test]
fn translucent_capsule_blends_each_pixel_once() {
    let p = render(
        Color::from_rgba(255, 255, 255, 128),
        None,
        Rect::from_ltwh(6.0, 10.0, 52.0, 28.0),
    );
    let center = blend_rgb565(0x2104, 0xffff, 128);
    assert_eq!(p.data()[24 * 64 + 32], center);
    assert!(p.data().iter().all(|px| *px <= center));
    assert!(p.data().iter().any(|px| *px > 0x2104 && *px < center));
}
#[test]
fn fractional_shape_clip_preserves_original_sampling_positions() {
    let rect = Rect::from_ltwh(-3.35, 5.6, 53.7, 32.8);
    let clip = Rect::from_ltwh(4.0, 8.0, 25.0, 21.0);
    let full = render(Color::WHITE, None, rect);
    let cut = render(Color::WHITE, Some(clip), rect);
    for y in 0..48 {
        for x in 0..64 {
            assert_eq!(
                cut.data()[y * 64 + x],
                if (4..29).contains(&x) && (8..29).contains(&y) {
                    full.data()[y * 64 + x]
                } else {
                    0x2104
                }
            );
        }
    }
}
#[test]
fn fully_transparent_shape_and_empty_intersection_leave_pixels_intact() {
    let transparent = render(
        Color::from_rgba(255, 255, 255, 0),
        None,
        Rect::from_ltwh(1.0, 1.0, 20.0, 20.0),
    );
    assert!(transparent.data().iter().all(|px| *px == 0x2104));
    let outside = render(
        Color::WHITE,
        Some(Rect::from_ltwh(40.0, 30.0, 10.0, 10.0)),
        Rect::from_ltwh(1.0, 1.0, 20.0, 20.0),
    );
    assert!(outside.data().iter().all(|px| *px == 0x2104));
}
