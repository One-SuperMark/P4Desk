use tiny_flutter::{tiny_gfx::Pixmap565, Canvas, Color, Font, Point, Rect};

#[test]
fn translated_text_is_drawn_and_clipped_in_framebuffer_coordinates() {
    let mut p = Pixmap565::new(128, 160).unwrap();
    let mut canvas = Canvas::new(p.as_mut());
    canvas.translate(20.0, 90.0);
    canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, 80.0, 24.0));
    canvas.draw_text(
        "12345",
        Font::default_font(),
        22.0,
        Point::new(0.0, 0.0),
        Color::WHITE,
    );
    assert!(p.data()[90 * 128..114 * 128]
        .iter()
        .any(|pixel| *pixel != 0));
    for y in 0..160 {
        for x in 0..128 {
            if !(90..114).contains(&y) || !(20..100).contains(&x) {
                assert_eq!(p.data()[y * 128 + x], 0);
            }
        }
    }
}
