use tiny_gfx::paint::MappedGradient;
use tiny_gfx::{Canvas, Color, GradientStop, Paint, Pixmap565, Point, Shader, Transform};
fn gradient(radial: bool) -> MappedGradient {
    MappedGradient {
        radial,
        transform: Transform {
            sx: 0.1,
            sy: 0.2,
            kx: 0.0,
            ky: 0.0,
            tx: -2.0,
            ty: -4.0,
        },
        stops: vec![
            GradientStop::new(0.0, Color::RED),
            GradientStop::new(0.5, Color::GREEN),
            GradientStop::new(1.0, Color::from_rgba(0, 0, 255, 128)),
        ],
    }
}
#[test]
fn gradient_uses_all_stops_radial_axes_and_clamps_endpoints() {
    let g = gradient(true);
    assert_eq!(g.color_at(20.0, 20.0), Color::RED);
    assert_eq!(g.color_at(25.0, 20.0), Color::GREEN);
    assert_eq!(g.color_at(20.0, 22.5), Color::GREEN);
    assert_eq!(g.color_at(40.0, 20.0), Color::from_rgba(0, 0, 255, 128));
    let g = gradient(false);
    assert_eq!(g.color_at(0.0, 50.0), Color::RED);
    assert_eq!(g.color_at(25.0, 100.0), Color::GREEN);
}
#[test]
fn mapped_gradient_moves_with_translated_geometry() {
    for radial in [false, true] {
        let mut a = Pixmap565::new(48, 48).unwrap();
        let mut b = Pixmap565::new(48, 48).unwrap();
        for (pixels, shift) in [(&mut a, 0.0), (&mut b, 8.0)] {
            let mut c = Canvas::new(pixels.as_mut());
            c.clear(Color::BLACK);
            c.translate(shift, shift);
            let mut p = Paint::new(Color::RED);
            p.shader = Shader::Mapped(gradient(radial));
            c.paint_circle(Point::new(20.0, 20.0), 12.0, &p, None);
        }
        for y in 8..32 {
            for x in 8..32 {
                let a = a.data()[y * 48 + x];
                let b = b.data()[(y + 8) * 48 + x + 8];
                // Affine f32 translation can cross a Bayer quantization threshold
                // by one code value; it must never shift the gradient geometrically.
                for (shift, mask) in [(11, 31), (5, 63), (0, 31)] {
                    assert!(
                        (((a >> shift) & mask) as i32 - ((b >> shift) & mask) as i32).abs() <= 1
                    );
                }
            }
        }
    }
}
