use tiny_flutter::prelude::*;

struct Pattern;
impl CustomPainter for Pattern {
    fn paint(&self, canvas: &mut Canvas, _: Size) {
        canvas.fill_dithered_horizontal_gradient((12, 51, 129), (209, 88, 34));
    }
}
#[derive(Default)]
struct TrackedBackend {
    pointer: usize,
    rect: Option<Rect>,
    pixels: Vec<u16>,
    events: Vec<&'static str>,
}
impl PlatformBackend for TrackedBackend {
    fn screen_size(&self) -> Size {
        Size::new(13.0, 9.0)
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        None
    }
    fn begin_frame(&mut self) {
        self.events.push("begin");
    }
    fn flush(&mut self, rect: Rect, pixels: &[u16]) {
        self.events.push("flush");
        self.pointer = pixels.as_ptr() as usize;
        self.rect = Some(rect);
        self.pixels = pixels.to_vec();
    }
    fn end_frame(&mut self) {
        self.events.push("end");
    }
}
fn verify(app: &mut App, backend: &mut TrackedBackend, rect: Rect, borrowed: bool) {
    backend.events.clear();
    app.mark_dirty(rect);
    app.step(backend);
    let mut reference = vec![0u16; app.framebuffer().len()];
    let pixmap = tiny_gfx::Pixmap565::from_vec(
        app.size().width as u32,
        app.size().height as u32,
        app.framebuffer().to_vec(),
    )
    .unwrap();
    let (x1, y1, x2, y2, n) = pixmap.extract_rect(
        tiny_gfx::Rect::from_ltwh(rect.x, rect.y, rect.width, rect.height),
        &mut reference,
    );
    assert_eq!(
        backend.rect,
        Some(Rect::from_ltrb(x1 as f32, y1 as f32, x2 as f32, y2 as f32))
    );
    assert_eq!(backend.pixels, reference[..n]);
    assert_eq!(backend.events, ["begin", "flush", "end"]);
    let offset = y1 as usize * app.size().width as usize + x1 as usize;
    let direct = app.framebuffer()[offset..].as_ptr() as usize;
    assert_eq!(backend.pointer == direct, borrowed);
}

#[test]
fn full_width_rows_borrow_the_framebuffer_and_partial_columns_stay_packed() {
    let size = Size::new(13.0, 9.0);
    let mut app = App::new(CustomPaint::new(Pattern).size(size), size);
    let mut backend = TrackedBackend::default();
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(0.0, 0.0, 13.0, 9.0),
        true,
    );
    // Fractional, clipped whole-width strip must point to its first pixel row.
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(-2.0, 2.2, 18.0, 3.3),
        true,
    );
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(2.2, 1.2, 4.1, 3.1),
        false,
    );
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(0.0, 0.0, 13.0, 9.0),
        true,
    );
    app.resize(Size::new(20.0, 12.0));
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(0.0, 0.0, 20.0, 12.0),
        true,
    );
    verify(
        &mut app,
        &mut backend,
        Rect::from_ltwh(17.0, 10.0, 8.0, 9.0),
        false,
    );
}
