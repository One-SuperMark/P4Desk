use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;

/// Touch interaction event passed down the render tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEvent {
    Down(Point),
    Move(Point),
    Up(Point),
    Cancel,
    HardwareBack,
    HardwareKill,
    HardwarePower,
}

impl TouchEvent {
    pub fn point(&self) -> Point {
        match *self {
            TouchEvent::Down(p) | TouchEvent::Move(p) | TouchEvent::Up(p) => p,
            TouchEvent::Cancel
            | TouchEvent::HardwareBack
            | TouchEvent::HardwareKill
            | TouchEvent::HardwarePower => Point::ZERO,
        }
    }

    pub fn transform(&self, new_point: Point) -> TouchEvent {
        match *self {
            TouchEvent::Down(_) => TouchEvent::Down(new_point),
            TouchEvent::Move(_) => TouchEvent::Move(new_point),
            TouchEvent::Up(_) => TouchEvent::Up(new_point),
            TouchEvent::Cancel => TouchEvent::Cancel,
            TouchEvent::HardwareBack => TouchEvent::HardwareBack,
            TouchEvent::HardwareKill => TouchEvent::HardwareKill,
            TouchEvent::HardwarePower => TouchEvent::HardwarePower,
        }
    }
}

/// The core RenderObject in 2D cartesian coordinate space.
/// Responsible for layout, painting, and hit-testing.
pub trait RenderBox: Send + Sync {
    /// Compute box dimensions according to BoxConstraints.
    fn layout(&mut self, constraints: &BoxConstraints) -> Size;

    /// Paint this box onto the canvas at the specified global offset.
    fn paint(&self, canvas: &mut Canvas, offset: Offset);

    /// Get current calculated size.
    fn size(&self) -> Size;

    /// Get local offset relative to parent.
    fn offset(&self) -> Offset;

    /// Set local offset relative to parent.
    fn set_offset(&mut self, offset: Offset);

    /// Test if a local coordinate point lies within this box.
    fn hit_test(&self, point: Point) -> bool {
        let s = self.size();
        point.x >= 0.0 && point.x <= s.width && point.y >= 0.0 && point.y <= s.height
    }

    /// Dispatch touch event to this box or its children. Returns true if handled.
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let _ = event;
        false
    }

    /// Set pressed visual state for any interactive component at the local point.
    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        let _ = (point, pressed);
    }

    /// Flex weight factor when placed inside a Row or Column (0 means non-flexible).
    fn flex(&self) -> u32 {
        0
    }

    /// Compute full bounding box in parent coordinates.
    fn paint_bounds(&self) -> Rect {
        let o = self.offset();
        let s = self.size();
        Rect::from_ltwh(o.dx, o.dy, s.width, s.height)
    }

    /// Returns the bounding rectangle of the interactive component hit at the local coordinate, if any.
    fn hit_rect(&self, point: Point) -> Option<Rect> {
        let _ = point;
        None
    }

    /// Check if this render object or any child requests a widget tree rebuild.
    fn needs_rebuild(&self) -> bool {
        false
    }
}
