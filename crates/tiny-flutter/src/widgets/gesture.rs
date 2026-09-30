use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::Arc;

pub struct GestureDetector {
    pub child: Box<dyn Widget>,
    pub on_tap: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl GestureDetector {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            on_tap: None,
        }
    }

    pub fn on_tap(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_tap = Some(Arc::new(handler));
        self
    }
}

impl Widget for GestureDetector {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderGestureDetector {
            child: self.child.create_render_object(),
            on_tap: self.on_tap.clone(),
            is_pressed: false,
            touch_origin: None,
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderGestureDetector {
    pub child: Box<dyn RenderBox>,
    pub on_tap: Option<Arc<dyn Fn() + Send + Sync>>,
    is_pressed: bool,
    touch_origin: Option<Point>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderGestureDetector {
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        self.child.paint(canvas, offset + self.child.offset());
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let p = event.point();
        let inside = self.hit_test(p);

        let child_offset = self.child.offset();
        let local_p = p - child_offset;
        let is_up = matches!(event, TouchEvent::Up(_));
        let is_cancel = matches!(event, TouchEvent::Cancel);
        if self.child.hit_test(local_p) || is_up || is_cancel {
            let child_event = event.transform(local_p);
            if self.child.dispatch_touch(&child_event) {
                self.is_pressed = false;
                return true;
            }
        }

        match event {
            TouchEvent::Down(p) if inside => {
                self.touch_origin = Some(*p);
                self.is_pressed = true;
                true
            }
            TouchEvent::Up(_) => {
                if self.is_pressed && inside {
                    self.is_pressed = false;
                    if let Some(on_tap) = &self.on_tap {
                        on_tap();
                    }
                    return true;
                }
                self.is_pressed = false;
                false
            }
            TouchEvent::Move(p) => {
                let moved = self
                    .touch_origin
                    .is_some_and(|o| (p.x - o.x).abs() > 12.0 || (p.y - o.y).abs() > 12.0);
                if !inside || moved {
                    self.is_pressed = false;
                }
                inside
            }
            TouchEvent::Cancel => {
                self.touch_origin = None;
                self.is_pressed = false;
                false
            }
            _ => false,
        }
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if self.hit_test(point) {
            self.is_pressed = pressed;
        } else if !pressed && self.is_pressed {
            self.is_pressed = false;
        }
        let child_offset = self.child.offset();
        let local_p = point - child_offset;
        self.child.set_pressed_at(local_p, pressed);
    }

    fn hit_rect(&self, point: Point) -> Option<crate::graphics::geometry::Rect> {
        let child_offset = self.child.offset();
        let local_p = point - child_offset;
        if let Some(r) = self.child.hit_rect(local_p) {
            Some(crate::graphics::geometry::Rect::from_ltwh(
                r.x + child_offset.dx,
                r.y + child_offset.dy,
                r.width,
                r.height,
            ))
        } else if self.hit_test(point) {
            Some(crate::graphics::geometry::Rect::from_ltwh(
                0.0,
                0.0,
                self.size.width,
                self.size.height,
            ))
        } else {
            None
        }
    }
}

/// A widget that intercepts hardware back/escape button events and kill events (Flutter-like PopScope).
pub struct BackListener {
    pub child: Box<dyn Widget>,
    pub on_back: Arc<dyn Fn() + Send + Sync>,
    pub on_kill: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl BackListener {
    pub fn new(child: impl Widget + 'static, on_back: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            child: Box::new(child),
            on_back: Arc::new(on_back),
            on_kill: None,
        }
    }

    pub fn on_kill(mut self, on_kill: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_kill = Some(Arc::new(on_kill));
        self
    }
}

impl Widget for BackListener {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderBackListener {
            child: self.child.create_render_object(),
            on_back: self.on_back.clone(),
            on_kill: self.on_kill.clone(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderBackListener {
    pub child: Box<dyn RenderBox>,
    pub on_back: Arc<dyn Fn() + Send + Sync>,
    pub on_kill: Option<Arc<dyn Fn() + Send + Sync>>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderBackListener {
    fn drag_dirty(&self) -> Option<Rect> {
        self.child
            .drag_dirty()
            .map(|r| r.shift(self.child.offset()))
    }
    fn paint_opaque_region(&self, canvas: &mut Canvas, offset: Offset, dirty: Rect) -> bool {
        self.child
            .paint_opaque_region(canvas, offset + self.child.offset(), dirty)
    }
    fn captures_touch(&self) -> bool {
        self.child.captures_touch()
    }
    fn animation_dirty(&self) -> Option<Rect> {
        self.child
            .animation_dirty()
            .map(|r| r.shift(self.child.offset()))
    }
    fn needs_rebuild(&self) -> bool {
        self.child.needs_rebuild()
    }
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        self.child.paint(canvas, offset + self.child.offset());
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if matches!(event, TouchEvent::HardwareBack) {
            (self.on_back)();
            return true;
        }
        if matches!(event, TouchEvent::HardwareKill) {
            if let Some(ref on_kill) = self.on_kill {
                (on_kill)();
            } else {
                (self.on_back)();
            }
            return true;
        }
        self.child.dispatch_touch(event)
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        let child_offset = self.child.offset();
        let local_p = point - child_offset;
        self.child.set_pressed_at(local_p, pressed);
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        self.child.hit_rect(point)
    }
}
