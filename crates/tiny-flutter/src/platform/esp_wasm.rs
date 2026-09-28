use crate::graphics::geometry::{Point, Rect, Size};
use crate::platform::backend::PlatformBackend;
use crate::rendering::render_box::TouchEvent;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn host_lcd_flush(x1: i32, y1: i32, x2: i32, y2: i32, pixels_offset: *const u16);
    fn host_get_touch(x: *mut i32, y: *mut i32, pressed: *mut i32) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn host_lcd_flush(_x1: i32, _y1: i32, _x2: i32, _y2: i32, _pixels_offset: *const u16) {}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn host_get_touch(_x: *mut i32, _y: *mut i32, _pressed: *mut i32) -> i32 {
    0
}

pub struct EspWasmBackend {
    width: u32,
    height: u32,
    was_pressed: bool,
    last_point: Point,
}

impl EspWasmBackend {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            was_pressed: false,
            last_point: Point::ZERO,
        }
    }
}

impl PlatformBackend for EspWasmBackend {
    fn screen_size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    fn flush(&mut self, rect: Rect, rgb565_data: &[u16]) {
        let x1 = (rect.x.floor() as i32).clamp(0, self.width as i32);
        let y1 = (rect.y.floor() as i32).clamp(0, self.height as i32);
        let x2 = (rect.right().ceil() as i32).clamp(x1, self.width as i32);
        let y2 = (rect.bottom().ceil() as i32).clamp(y1, self.height as i32);

        if x2 > x1 && y2 > y1 && !rgb565_data.is_empty() {
            unsafe {
                host_lcd_flush(x1, y1, x2, y2, rgb565_data.as_ptr());
            }
        }
    }

    fn poll_touch(&mut self) -> Option<TouchEvent> {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut state: i32 = 0;

        let ret = unsafe { host_get_touch(&mut x, &mut y, &mut state) };
        let is_pressed = ret == 1 || state == 1;
        let current_pt = Point::new(x as f32, y as f32);

        let event = if is_pressed {
            if !self.was_pressed {
                self.last_point = current_pt;
                Some(TouchEvent::Down(current_pt))
            } else {
                let dx = (current_pt.x - self.last_point.x).abs();
                let dy = (current_pt.y - self.last_point.y).abs();
                // Filter capacitive sensor noise (deadband = 4.0px)
                if dx >= 4.0 || dy >= 4.0 {
                    self.last_point = current_pt;
                    Some(TouchEvent::Move(current_pt))
                } else {
                    None
                }
            }
        } else if self.was_pressed {
            Some(TouchEvent::Up(self.last_point))
        } else {
            None
        };

        self.was_pressed = is_pressed;
        event
    }
}
