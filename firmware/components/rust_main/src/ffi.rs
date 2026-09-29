#[cfg(not(espidf_time64))]
compile_error!("P4Desk requires --cfg espidf_time64 to match ESP-IDF 6.0.2 time_t ABI");
const _: () = {
    assert!(std::mem::size_of::<bool>() == 1);
    assert!(std::mem::size_of::<i32>() == 4);
    assert!(std::mem::size_of::<libc::time_t>() == 8);
    assert!(std::mem::size_of::<esp_idf_sys::time_t>() == 8);
    assert!(std::mem::size_of::<libc::timeval>() == std::mem::size_of::<esp_idf_sys::timeval>());
};
use crate::runtime::{DeviceRuntime, Hal};
use p4desk_protocol::{DeviceMessage, Mode, KIND_CONTROL, KIND_RESOURCE, MAX_CONTROL};
use std::ffi::c_char;
use tiny_flutter::{
    App, PlatformBackend, Point, RawTouchFrame, RawTouchPoint, Rect, Size, TouchEvent,
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CTouchPoint {
    x: u16,
    y: u16,
    id: u8,
    reserved: [u8; 3],
}
#[repr(C)]
#[derive(Default)]
struct CTouchFrame {
    stamp_us: u64,
    session: u32,
    sequence: u16,
    count: u8,
    reserved: u8,
    points: [CTouchPoint; 5],
}
const _: () = {
    assert!(std::mem::size_of::<CTouchPoint>() == 8);
    assert!(std::mem::size_of::<CTouchFrame>() == 56);
    assert!(std::mem::offset_of!(CTouchFrame, points) == 16);
};

extern "C" {
    fn host_lcd_draw_bitmap(x1: i32, y1: i32, x2: i32, y2: i32, pixels: *const u16);
    fn host_touch_get_point(x: *mut i32, y: *mut i32) -> bool;
    fn host_lcd_set_power(on: bool);
    fn p4desk_pad_frame_begin();
    fn p4desk_pad_frame_end();
    fn p4desk_get_mode() -> u32;
    fn p4desk_set_mode(mode: u32, session: u32) -> bool;
    fn p4desk_direct_jpeg_rotation_degrees() -> u32;
    fn p4desk_set_mode_with_jpeg_rotation(
        mode: u32,
        session: u32,
        jpeg_rotation_degrees: u32,
    ) -> bool;
    fn p4desk_set_brightness(percent: u8);
    fn p4desk_monotonic_us() -> i64;
    fn p4desk_delay_ms(ms: u32);
    fn p4desk_sd_ready() -> bool;
    fn p4desk_sd_free_bytes() -> u64;
    fn p4desk_poll_packet(
        kind: *mut u8,
        sequence: *mut u16,
        buffer: *mut u8,
        capacity: usize,
    ) -> usize;
    fn p4desk_send_control(json: *const c_char, length: usize, sequence: u16) -> bool;
    fn p4desk_send_media(usage: u16);
    fn p4desk_usb_connected() -> bool;
    fn p4desk_heartbeat_received();
    fn p4desk_host_active() -> bool;
    fn p4desk_ui_invalidated() -> bool;
    fn p4desk_time_set(unix_ms: i64);
    fn p4desk_unix_ms() -> i64;
    fn p4desk_get_raw_touch(out: *mut CTouchFrame) -> bool;
}
pub struct EspHal;
impl Hal for EspHal {
    fn connected(&self) -> bool {
        unsafe { p4desk_usb_connected() }
    }
    fn host_active(&self) -> bool {
        unsafe { p4desk_host_active() }
    }
    fn sd_ready(&self) -> bool {
        unsafe { p4desk_sd_ready() }
    }
    fn sd_free_bytes(&self) -> u64 {
        unsafe { p4desk_sd_free_bytes() }
    }
    fn mode(&self) -> Mode {
        if unsafe { p4desk_get_mode() } == 1 {
            Mode::Display
        } else {
            Mode::Pad
        }
    }
    fn set_mode(&mut self, m: Mode, s: u32) -> bool {
        unsafe { p4desk_set_mode(m.as_u32(), s) }
    }
    fn direct_jpeg_rotation_degrees(&self) -> u16 {
        u16::try_from(unsafe { p4desk_direct_jpeg_rotation_degrees() }).unwrap_or(0)
    }
    fn set_mode_with_jpeg_rotation(&mut self, m: Mode, s: u32, degrees: u16) -> bool {
        unsafe { p4desk_set_mode_with_jpeg_rotation(m.as_u32(), s, u32::from(degrees)) }
    }
    fn heartbeat(&mut self) {
        unsafe { p4desk_heartbeat_received() }
    }
    fn unix_ms(&self) -> i64 {
        unsafe { p4desk_unix_ms() }
    }
    fn set_time(&mut self, ms: i64) {
        unsafe { p4desk_time_set(ms) }
    }
    fn monotonic_ms(&self) -> u64 {
        unsafe { p4desk_monotonic_us() }.max(0) as u64 / 1000
    }
    fn brightness(&mut self, v: u8) {
        unsafe { p4desk_set_brightness(v) }
    }
    fn screen(&mut self, on: bool) {
        unsafe { host_lcd_set_power(on) }
    }
    fn media(&mut self, v: u16) {
        unsafe { p4desk_send_media(v) }
    }
    fn send(&mut self, message: &DeviceMessage, sequence: u16) -> bool {
        let Ok(json) = serde_json::to_vec(message) else {
            return false;
        };
        if json.len() > MAX_CONTROL {
            return false;
        }
        unsafe { p4desk_send_control(json.as_ptr().cast(), json.len(), sequence) }
    }
}
struct P4Backend {
    last: Option<Point>,
    last_raw_stamp: Option<u64>,
}
impl PlatformBackend for P4Backend {
    fn begin_frame(&mut self) {
        unsafe { p4desk_pad_frame_begin() }
    }
    fn end_frame(&mut self) {
        unsafe { p4desk_pad_frame_end() }
    }
    fn screen_size(&self) -> Size {
        Size::new(1024.0, 600.0)
    }
    fn poll_raw_touch(&mut self) -> Option<RawTouchFrame> {
        let mut raw = CTouchFrame::default();
        if !unsafe { p4desk_get_raw_touch(&mut raw) }
            || raw.count > 5
            || self.last_raw_stamp == Some(raw.stamp_us)
        {
            return None;
        }
        self.last_raw_stamp = Some(raw.stamp_us);
        Some(RawTouchFrame {
            session: raw.session,
            sequence: raw.sequence,
            stamp_us: raw.stamp_us,
            points: raw.points[..raw.count as usize]
                .iter()
                .map(|p| RawTouchPoint {
                    id: p.id,
                    x: p.x.min(1023),
                    y: p.y.min(599),
                })
                .collect(),
        })
    }
    fn set_screen_power(&mut self, on: bool) {
        unsafe { host_lcd_set_power(on) }
    }
    fn flush(&mut self, r: Rect, data: &[u16]) {
        let (x1, y1, x2, y2) = (r.x as i32, r.y as i32, r.right() as i32, r.bottom() as i32);
        if x1 < 0
            || y1 < 0
            || x2 > 1024
            || y2 > 600
            || x2 <= x1
            || y2 <= y1
            || data.len() != ((x2 - x1) * (y2 - y1)) as usize
        {
            return;
        }
        unsafe { host_lcd_draw_bitmap(x1, y1, x2, y2, data.as_ptr()) }
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        let (mut x, mut y) = (0, 0);
        let active = unsafe { host_touch_get_point(&mut x, &mut y) };
        let point = Point::new(x.clamp(0, 1023) as f32, y.clamp(0, 599) as f32);
        let event = match (self.last, active) {
            (None, true) => Some(TouchEvent::Down(point)),
            (Some(last), true) if last != point => Some(TouchEvent::Move(point)),
            (Some(last), false) => Some(TouchEvent::Up(last)),
            _ => None,
        };
        self.last = if active { Some(point) } else { None };
        event
    }
}

#[no_mangle]
pub extern "C" fn rust_main_entry() {
    esp_idf_sys::link_patches();
    let mut runtime = DeviceRuntime::new(EspHal, "/sdcard/p4desk", "/flash");
    let state = runtime.state.clone();
    let size = Size::new(1024.0, 600.0);
    let mut app = App::new(app_launcher::build_launcher_ui(state.clone(), size), size);
    let mut backend = P4Backend {
        last: None,
        last_raw_stamp: None,
    };
    let mut buffer = vec![0u8; MAX_CONTROL];
    let mut mode = Mode::Pad;
    let mut revision = 0;
    loop {
        for _ in 0..8 {
            let (mut kind, mut sequence) = (0u8, 0u16);
            let n = unsafe {
                p4desk_poll_packet(&mut kind, &mut sequence, buffer.as_mut_ptr(), buffer.len())
            };
            if n == 0 {
                break;
            }
            if n > buffer.len() {
                continue;
            }
            match kind {
                KIND_CONTROL => runtime.control(sequence, &buffer[..n]),
                KIND_RESOURCE => runtime.resource(sequence, &buffer[..n]),
                _ => (),
            }
        }
        runtime.tick();
        let next_mode = runtime.hal.mode();
        if next_mode != mode {
            app.cancel_touch();
            backend.last = None;
            mode = next_mode;
            if mode == Mode::Pad {
                app.set_screen_power(&mut backend, true);
                state.lock().unwrap().settings.screen_on = true;
                app.request_rebuild();
            }
        }
        let current_revision = state.lock().unwrap().revision;
        if current_revision != revision || unsafe { p4desk_ui_invalidated() } {
            app.request_rebuild();
            revision = current_revision;
        }
        if mode == Mode::Pad {
            let screen_on = state.lock().unwrap().settings.screen_on;
            app.set_screen_power(&mut backend, screen_on);
            app.step_with_builder(&mut backend, |size| {
                app_launcher::build_launcher_ui(state.clone(), size)
            });
            state.lock().unwrap().settings.screen_on = app.is_screen_on();
        }
        if runtime.process_commands() {
            app.request_rebuild();
        }
        unsafe { p4desk_delay_ms(16) };
    }
}
