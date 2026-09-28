use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;

// =========================================================================
// Positioned
// =========================================================================

pub struct Positioned {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub child: Box<dyn Widget>,
}

impl Positioned {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            child: Box::new(child),
        }
    }

    pub fn left(mut self, left: f32) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: f32) -> Self {
        self.top = Some(top);
        self
    }

    pub fn right(mut self, right: f32) -> Self {
        self.right = Some(right);
        self
    }

    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = Some(bottom);
        self
    }
}

impl Widget for Positioned {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderPositioned {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            child: self.child.create_render_object(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderPositioned {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub child: Box<dyn RenderBox>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderPositioned {
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
        let child_constraints = constraints.loosen();
        let child_size = self.child.layout(&child_constraints);

        let dx = self.left.unwrap_or_else(|| {
            if let Some(r) = self.right {
                (constraints.max_width - child_size.width - r).max(0.0)
            } else {
                0.0
            }
        });
        let dy = self.top.unwrap_or_else(|| {
            if let Some(b) = self.bottom {
                (constraints.max_height - child_size.height - b).max(0.0)
            } else {
                0.0
            }
        });
        self.child.set_offset(Offset::new(dx, dy));
        self.size = constraints.constrain(Size::new(
            if constraints.max_width.is_finite() {
                constraints.max_width
            } else {
                dx + child_size.width
            },
            if constraints.max_height.is_finite() {
                constraints.max_height
            } else {
                dy + child_size.height
            },
        ));
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        self.child.paint(canvas, offset + self.child.offset());
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let child_offset = self.child.offset();
        let local_p = event.point() - child_offset;
        let hit = self.child.hit_test(local_p);
        let is_up = matches!(event, TouchEvent::Up(_));
        if hit || is_up || matches!(event, TouchEvent::Cancel) {
            let child_event = event.transform(local_p);
            return self.child.dispatch_touch(&child_event);
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        let child_offset = self.child.offset();
        let local_p = point - child_offset;
        if self.child.hit_test(local_p) {
            self.child.set_pressed_at(local_p, pressed);
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        let child_offset = self.child.offset();
        let local_p = point - child_offset;
        if self.child.hit_test(local_p) {
            self.child.hit_rect(local_p).map(|r| {
                Rect::from_ltwh(
                    r.x + child_offset.dx,
                    r.y + child_offset.dy,
                    r.width,
                    r.height,
                )
            })
        } else {
            None
        }
    }
}

// =========================================================================
// Stack
// =========================================================================

pub struct Stack {
    pub children: Vec<Box<dyn Widget>>,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn push(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl Widget for Stack {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let render_children = self
            .children
            .iter()
            .map(|c| c.create_render_object())
            .collect();
        Box::new(RenderStack {
            children: render_children,
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderStack {
    pub children: Vec<Box<dyn RenderBox>>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderStack {
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
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;

        let child_constraints = constraints.loosen();
        for child in self.children.iter_mut() {
            let child_size = child.layout(&child_constraints);
            if child_size.width > max_w {
                max_w = child_size.width;
            }
            if child_size.height > max_h {
                max_h = child_size.height;
            }
        }

        let w = if constraints.max_width.is_finite() {
            constraints.max_width
        } else {
            max_w
        };
        let h = if constraints.max_height.is_finite() {
            constraints.max_height
        } else {
            max_h
        };

        self.size = constraints.constrain(Size::new(w, h));
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        for child in &self.children {
            let child_offset = offset + child.offset();
            child.paint(canvas, child_offset);
        }
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let local_p = event.point();
        let is_up = matches!(event, TouchEvent::Up(_));

        for child in self.children.iter_mut().rev() {
            let child_offset = child.offset();
            let child_local_p = local_p - child_offset;
            if child.hit_test(child_local_p) || is_up || matches!(event, TouchEvent::Cancel) {
                let child_event = event.transform(child_local_p);
                if child.dispatch_touch(&child_event) {
                    return true;
                }
            }
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        for child in self.children.iter_mut().rev() {
            let child_offset = child.offset();
            let child_local_p = point - child_offset;
            if child.hit_test(child_local_p) {
                child.set_pressed_at(child_local_p, pressed);
            }
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        for child in self.children.iter().rev() {
            let child_offset = child.offset();
            let child_local_p = point - child_offset;
            if child.hit_test(child_local_p) {
                if let Some(r) = child.hit_rect(child_local_p) {
                    return Some(Rect::from_ltwh(
                        r.x + child_offset.dx,
                        r.y + child_offset.dy,
                        r.width,
                        r.height,
                    ));
                }
            }
        }
        None
    }
}
