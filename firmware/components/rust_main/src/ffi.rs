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
use std::sync::atomic::{AtomicU8, Ordering};
// The display owner is a different thread from the UI's thread-local palette.
static DISPLAY_REVEAL_STYLE: AtomicU8 = AtomicU8::new(4);
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
#[repr(C)]
#[derive(Default)]
struct CPadTouchEvent {
    kind: u32,
    x: i32,
    y: i32,
}
const _: () = {
    assert!(std::mem::size_of::<CPadTouchEvent>() == 12);
    assert!(std::mem::offset_of!(CPadTouchEvent, x) == 4);
    assert!(std::mem::offset_of!(CPadTouchEvent, y) == 8);
    assert!(std::mem::size_of::<CTouchPoint>() == 8);
    assert!(std::mem::size_of::<CTouchFrame>() == 56);
    assert!(std::mem::offset_of!(CTouchFrame, points) == 16);
};

extern "C" {
    fn p4desk_radio_snapshot(out: *mut app_launcher::radio::RadioSnapshot, last: u32) -> bool;
    fn p4desk_radio_submit(op: u32, id: u32, data: *const u8, length: usize) -> bool;
    fn host_lcd_draw_bitmap(x1: i32, y1: i32, x2: i32, y2: i32, pixels: *const u16);
    fn p4desk_pad_blit_rgb565(
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        pixels: *const u16,
        pixel_count: usize,
        stride: usize,
    ) -> bool;
    fn p4desk_poll_pad_touch(event: *mut CPadTouchEvent) -> bool;
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
    fn p4desk_arm_display_transition(duration_ms: u32) -> bool;
    fn p4desk_cancel_display_transition();
    fn p4desk_display_transition_pending() -> bool;
    fn p4desk_set_brightness(percent: u8);
    fn p4desk_monotonic_us() -> i64;
    fn p4desk_delay_ms(ms: u32);
    fn p4desk_battery_voltage_mv() -> i32;
    fn p4desk_typec_host_connected() -> bool;
    fn p4desk_reset_reason() -> u32;
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
    fn radio_snapshot(&self, revision: u32) -> Option<app_launcher::radio::RadioSnapshot> {
        let mut out = app_launcher::radio::RadioSnapshot::default();
        unsafe { p4desk_radio_snapshot(&mut out, revision) }.then_some(out)
    }
    fn radio_command(&mut self, c: &app_launcher::radio::RadioCommand) -> bool {
        let (op, id, data) = c.parts();
        unsafe { p4desk_radio_submit(op, id, data.as_ptr(), data.len()) }
    }
    fn connected(&self) -> bool {
        unsafe { p4desk_usb_connected() }
    }
    fn host_active(&self) -> bool {
        unsafe { p4desk_host_active() }
    }
    fn battery_reading(&self) -> app_launcher::battery::BatteryReading {
        app_launcher::battery::BatteryReading {
            voltage_mv: u16::try_from(unsafe { p4desk_battery_voltage_mv() }).ok(),
            charge: if unsafe { p4desk_typec_host_connected() } {
                app_launcher::battery::ChargeState::PluggedInAssumed
            } else {
                // No SOF does not prove battery power: a wall charger or the
                // separate CH343 UART Type-C can still be supplying power.
                app_launcher::battery::ChargeState::Unknown
            },
        }
    }
    fn reset_reason(&self) -> u32 {
        unsafe { p4desk_reset_reason() }
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
    fn arm_display_transition(&mut self, duration_ms: u32) -> bool {
        unsafe { p4desk_arm_display_transition(duration_ms) }
    }
    fn display_transition_pending(&self) -> bool {
        unsafe { p4desk_display_transition_pending() }
    }
    fn cancel_display_transition(&mut self) {
        unsafe { p4desk_cancel_display_transition() }
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
    touch_batch: crate::cadence::TouchBatch,
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
    fn flush_strided(&mut self, r: Rect, data: &[u16], stride: usize) -> bool {
        unsafe {
            p4desk_pad_blit_rgb565(
                r.x as i32,
                r.y as i32,
                r.right() as i32,
                r.bottom() as i32,
                data.as_ptr(),
                data.len(),
                stride,
            )
        }
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        // Present Down feedback before consuming a queued Up. Also rebuild an
        // app switch before routing the next gesture to its new widget tree.
        if !self.touch_batch.ready() {
            return None;
        }
        let mut raw = CPadTouchEvent::default();
        if !unsafe { p4desk_poll_pad_touch(&mut raw) } {
            return None;
        }
        self.touch_batch.consumed(raw.kind);
        let point = Point::new(raw.x.clamp(0, 1023) as f32, raw.y.clamp(0, 599) as f32);
        Some(match raw.kind {
            1 => TouchEvent::Down(point),
            2 => TouchEvent::Move(point),
            3 => TouchEvent::Up(point),
            _ => TouchEvent::Cancel,
        })
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
        touch_batch: crate::cadence::TouchBatch::default(),
        last_raw_stamp: None,
    };
    let mut buffer = vec![0u8; MAX_CONTROL];
    let mut mode = Mode::Pad;
    let mut revision = 0;
    let mut next_ui_metrics_us = 0i64;
    loop {
        let iteration_started_us = unsafe { p4desk_monotonic_us() };
        let style = {
            let s = state.lock().unwrap();
            (s.settings.icon_theme.index() << 1) | u8::from(s.settings.light_icons())
        };
        DISPLAY_REVEAL_STYLE.store(style, Ordering::Release);
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
            backend.touch_batch.reset();
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
            if let Some(rect) = state
                .lock()
                .unwrap()
                .take_launch_animation_dirty(app.size())
            {
                app.mark_dirty(rect);
            }
            if let Some(rect) = state.lock().unwrap().take_clock_animation_dirty(app.size()) {
                app.mark_dirty(rect);
            }
            if let Some(rect) = state.lock().unwrap().take_timer_animation_dirty(app.size()) {
                app.mark_dirty(rect);
            }
            backend.touch_batch.reset();
            app.step_with_builder(&mut backend, |size| {
                app_launcher::build_launcher_ui(state.clone(), size)
            });
            state.lock().unwrap().settings.screen_on = app.is_screen_on();
        }
        if runtime.process_commands() {
            app.request_rebuild();
        }
        if iteration_started_us >= next_ui_metrics_us {
            next_ui_metrics_us = iteration_started_us.saturating_add(30_000_000);
            let m = app.take_frame_metrics();
            if m.frames > 0 {
                let (entries, bytes, hits, misses) = tiny_flutter::vector_cache_stats();
                println!("p4desk_ui_perf: frames={} draw_avg_us={} output_avg_us={} draw_max_us={} total_max_us={} render_cache_entries={} render_cache_bytes={} cache_hits={} cache_misses={}",
                    m.frames, m.draw_us / m.frames, m.output_us / m.frames,
                    m.max_draw_us, m.max_total_us, entries, bytes, hits, misses);
                if m.drag_frames > 0 {
                    println!("p4desk_drag_perf: frames={} draw_avg_us={} output_avg_us={} total_max_us={} opaque_frames={}",
                        m.drag_frames, m.drag_draw_us / m.drag_frames, m.drag_output_us / m.drag_frames, m.drag_max_us, m.opaque_frames);
                }
                let s = tiny_flutter::widgets::take_scroll_metrics();
                if s.paints > 0 {
                    println!("p4desk_scroll_perf: builds={} build_us={} paints={} blit_avg_us={} blit_max_us={}",
                        s.builds, s.build_us, s.paints, s.blit_us/s.paints, s.blit_max_us);
                }
            }
        }
        let elapsed_us = unsafe { p4desk_monotonic_us() }.saturating_sub(iteration_started_us);
        unsafe { p4desk_delay_ms(crate::cadence::idle_delay_ms(elapsed_us)) };
    }
}

/// The C display owner holds this buffer in BUILDING until the function returns.
#[no_mangle]
pub unsafe extern "C" fn rust_p4desk_display_reveal(
    pixels: *mut u16,
    count: usize,
    elapsed_ms: u32,
    duration_ms: u32,
) -> bool {
    if pixels.is_null()
        || (pixels as usize) % std::mem::align_of::<u16>() != 0
        || count != 1024 * 600
        || duration_ms == 0
        || duration_ms > 2000
    {
        return false;
    }
    let data = std::slice::from_raw_parts_mut(pixels, count);
    let mut canvas =
        tiny_flutter::Canvas::new(tiny_flutter::tiny_gfx::Pixmap565Mut::new(data, 1024, 600));
    let style = DISPLAY_REVEAL_STYLE.load(Ordering::Acquire);
    app_launcher::app_launch::paint_usb_display_reveal(
        &mut canvas,
        Size::new(1024.0, 600.0),
        elapsed_ms,
        duration_ms,
        style & 1 != 0,
        app_launcher::icon_theme::IconTheme::from_index(style >> 1),
    );
    true
}
