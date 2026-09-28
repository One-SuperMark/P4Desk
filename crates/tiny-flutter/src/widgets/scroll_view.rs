use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::{Arc, Mutex};

/// Controller to observe or manipulate the scroll position of a `SingleChildScrollView`.
#[derive(Clone, Default)]
pub struct ScrollController {
    offset: Arc<Mutex<f32>>,
}

impl ScrollController {
    pub fn new() -> Self {
        Self {
            offset: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn with_offset(initial: f32) -> Self {
        Self {
            offset: Arc::new(Mutex::new(initial)),
        }
    }

    pub fn offset(&self) -> f32 {
        *self.offset.lock().unwrap()
    }

    pub fn set_offset(&self, val: f32) {
        if let Ok(mut o) = self.offset.lock() {
            *o = val;
        }
    }
}

/// A box in which a single widget can be scrolled vertically via touch gestures.
pub struct SingleChildScrollView {
    pub child: Box<dyn Widget>,
    pub controller: Option<ScrollController>,
    pub auto_scroll_to_bottom: bool,
}

impl SingleChildScrollView {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            controller: None,
            auto_scroll_to_bottom: false,
        }
    }

    pub fn controller(mut self, controller: ScrollController) -> Self {
        self.controller = Some(controller);
        self
    }

    pub fn auto_scroll_to_bottom(mut self, auto: bool) -> Self {
        self.auto_scroll_to_bottom = auto;
        self
    }
}

impl Widget for SingleChildScrollView {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let initial_offset = self.controller.as_ref().map(|c| c.offset()).unwrap_or(0.0);
        Box::new(RenderSingleChildScrollView {
            child: self.child.create_render_object(),
            controller: self.controller.clone(),
            auto_scroll_to_bottom: self.auto_scroll_to_bottom,
            scroll_offset: initial_offset,
            max_scroll: 0.0,
            is_dragging: false,
            touch_start_y: 0.0,
            touch_start_offset: 0.0,
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderSingleChildScrollView {
    pub child: Box<dyn RenderBox>,
    pub controller: Option<ScrollController>,
    pub auto_scroll_to_bottom: bool,
    scroll_offset: f32,
    max_scroll: f32,
    is_dragging: bool,
    touch_start_y: f32,
    touch_start_offset: f32,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderSingleChildScrollView {
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
        let w = if constraints.max_width.is_finite() {
            constraints.max_width
        } else {
            constraints.min_width
        };
        let h = if constraints.max_height.is_finite() {
            constraints.max_height
        } else {
            constraints.min_height
        };
        self.size = constraints.constrain(Size::new(w, h));

        // Child can be as tall as intrinsically required
        let child_constraints = BoxConstraints::new(
            constraints.min_width,
            constraints.max_width,
            0.0,
            f32::INFINITY,
        );
        let child_size = self.child.layout(&child_constraints);

        self.max_scroll = (child_size.height - self.size.height).max(0.0);

        if self.auto_scroll_to_bottom {
            self.scroll_offset = self.max_scroll;
            self.auto_scroll_to_bottom = false; // Consumed on initial layout so drag gestures aren't negated!
            if let Some(ctrl) = &self.controller {
                ctrl.set_offset(self.scroll_offset);
            }
        } else if let Some(ctrl) = &self.controller {
            self.scroll_offset = ctrl.offset().clamp(0.0, self.max_scroll);
        } else {
            self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll);
        }

        self.child.set_offset(Offset::ZERO);
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        if self.size.width <= 0.0 || self.size.height <= 0.0 {
            return;
        }

        canvas.save();
        canvas.clip_rect(Rect::from_ltwh(
            offset.dx,
            offset.dy,
            self.size.width,
            self.size.height,
        ));

        let child_offset = Offset::new(offset.dx, offset.dy - self.scroll_offset);
        self.child.paint(canvas, child_offset);

        canvas.restore();
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let p = event.point();
        let inside = self.hit_test(p);

        match event {
            TouchEvent::Down(pt) if inside => {
                self.auto_scroll_to_bottom = false;
                self.is_dragging = true;
                self.touch_start_y = pt.y;
                self.touch_start_offset = self.scroll_offset;
                true
            }
            TouchEvent::Move(pt) if self.is_dragging => {
                let dy = pt.y - self.touch_start_y;
                let new_offset = (self.touch_start_offset - dy).clamp(0.0, self.max_scroll);
                if (new_offset - self.scroll_offset).abs() > 0.5 {
                    self.scroll_offset = new_offset;
                    if let Some(ctrl) = &self.controller {
                        ctrl.set_offset(new_offset);
                    }
                    true
                } else {
                    false
                }
            }
            TouchEvent::Up(_) if self.is_dragging => {
                self.is_dragging = false;
                if let Some(ctrl) = &self.controller {
                    ctrl.set_offset(self.scroll_offset);
                }
                true
            }
            TouchEvent::Cancel => {
                self.is_dragging = false;
                false
            }
            _ => {
                let local_p = Point::new(p.x, p.y + self.scroll_offset);
                if self.child.hit_test(local_p) {
                    let child_event = event.transform(local_p);
                    self.child.dispatch_touch(&child_event)
                } else {
                    false
                }
            }
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if self.hit_test(point) {
            Some(Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
        } else {
            None
        }
    }
}
