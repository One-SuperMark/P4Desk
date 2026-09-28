use std::collections::VecDeque;
use tiny_flutter::{PlatformBackend, Rect, Size, TouchEvent};

pub struct HeadlessBackend {
    pub pixels: Vec<u16>,
    width: usize,
    height: usize,
    events: VecDeque<TouchEvent>,
    pub frames: usize,
    begin: bool,
}
impl HeadlessBackend {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![0; width * height],
            width,
            height,
            events: VecDeque::new(),
            frames: 0,
            begin: false,
        }
    }
    pub fn event(&mut self, event: TouchEvent) {
        self.events.push_back(event);
    }
    #[cfg(feature = "screenshots")]
    pub fn screenshot(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::create(path)?;
        let mut e = png::Encoder::new(
            std::io::BufWriter::new(f),
            self.width as u32,
            self.height as u32,
        );
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        let mut w = e.write_header()?;
        let mut rgb = Vec::with_capacity(self.pixels.len() * 3);
        for p in &self.pixels {
            rgb.extend_from_slice(&[
                (((p >> 11) & 31) * 255 / 31) as u8,
                (((p >> 5) & 63) * 255 / 63) as u8,
                ((p & 31) * 255 / 31) as u8,
            ]);
        }
        w.write_image_data(&rgb)?;
        Ok(())
    }
}
impl PlatformBackend for HeadlessBackend {
    fn begin_frame(&mut self) {
        assert!(!self.begin);
        self.begin = true;
    }
    fn end_frame(&mut self) {
        assert!(self.begin);
        self.begin = false;
        self.frames += 1;
    }
    fn screen_size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        self.events.pop_front()
    }
    fn flush(&mut self, r: Rect, data: &[u16]) {
        assert!(self.begin);
        let x = r.x as usize;
        let y = r.y as usize;
        let w = r.width as usize;
        let h = r.height as usize;
        assert_eq!(w * h, data.len());
        assert!(x + w <= self.width && y + h <= self.height);
        for row in 0..h {
            let offset = (y + row) * self.width + x;
            self.pixels[offset..offset + w].copy_from_slice(&data[row * w..row * w + w]);
        }
    }
}
