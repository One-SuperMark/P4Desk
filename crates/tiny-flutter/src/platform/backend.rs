use crate::graphics::geometry::{Rect, Size};
use crate::rendering::render_box::TouchEvent;

/// Multitouch transport is independent of the local UI's single-pointer TouchEvent stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTouchPoint {
    pub id: u8,
    pub x: u16,
    pub y: u16,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTouchFrame {
    pub session: u32,
    pub sequence: u16,
    pub stamp_us: u64,
    pub points: Vec<RawTouchPoint>,
}

/// Hardware abstraction interface for screen output and touch input.
pub trait PlatformBackend {
    /// Obtain display ownership for one Pad frame; flushes are synchronous.
    fn begin_frame(&mut self) {}

    /// Release ownership and present the completed frame.
    fn end_frame(&mut self) {}

    /// Flush a dirty rectangular buffer of 16-bit RGB565 pixels to the physical display.
    /// Rows are tightly packed: stride is rect.width, length is width * height.
    fn flush(&mut self, rect: Rect, rgb565_data: &[u16]);

    /// Optional synchronous copy from a framebuffer subregion. `pixels` starts at
    /// rect's first pixel; row starts are `stride` words apart. Return false only
    /// if unsupported, without writing; the caller then supplies a packed flush.
    fn flush_strided(&mut self, _rect: Rect, _pixels: &[u16], _stride: usize) -> bool {
        false
    }

    /// Poll any pending touch interaction events.
    fn poll_touch(&mut self) -> Option<TouchEvent>;

    /// Optional raw contacts for a host bridge; never synthesize them from single-pointer UI events.
    fn poll_raw_touch(&mut self) -> Option<RawTouchFrame> {
        None
    }

    /// Target display dimensions in logical/physical pixels.
    fn screen_size(&self) -> Size;

    /// Control physical display power (e.g. AMOLED panel sleep/wake).
    /// Default implementation is a no-op so backends remain fully compatible.
    fn set_screen_power(&mut self, _on: bool) {}
}
