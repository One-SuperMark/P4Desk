use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::Arc;

/// A delegate trait for drawing custom 2D graphics onto a Canvas during paint phase.
pub trait CustomPainter: Send + Sync {
    /// Draw custom graphics within the given Size.
    fn paint(&self, canvas: &mut Canvas, size: Size);
}

/// A widget that provides a canvas on which to draw custom vector shapes and graphics.
pub struct CustomPaint {
    pub painter: Arc<dyn CustomPainter>,
    pub size: Size,
    pub child: Option<Box<dyn Widget>>,
}

impl CustomPaint {
    /// Create a new CustomPaint widget with the specified painter.
    pub fn new(painter: impl CustomPainter + 'static) -> Self {
        Self {
            painter: Arc::new(painter),
            size: Size::ZERO,
            child: None,
        }
    }

    /// Set an explicit size constraint for the custom paint canvas.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Add an optional child widget to be rendered on top of the custom paint canvas.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Widget for CustomPaint {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderCustomPaint {
            painter: self.painter.clone(),
            desired_size: self.size,
            child: self.child.as_ref().map(|c| c.create_render_object()),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderCustomPaint {
    pub painter: Arc<dyn CustomPainter>,
    pub desired_size: Size,
    pub child: Option<Box<dyn RenderBox>>,
    pub size: Size,
    pub offset: Offset,
}

impl RenderBox for RenderCustomPaint {
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
        if let Some(child) = &mut self.child {
            let child_size = child.layout(constraints);
            let w = if self.desired_size.width > 0.0 {
                self.desired_size.width
            } else {
                child_size.width
            };
            let h = if self.desired_size.height > 0.0 {
                self.desired_size.height
            } else {
                child_size.height
            };
            self.size = constraints.constrain(Size::new(w, h));
            let child_dx = (self.size.width - child_size.width) / 2.0;
            let child_dy = (self.size.height - child_size.height) / 2.0;
            child.set_offset(Offset::new(child_dx.max(0.0), child_dy.max(0.0)));
        } else {
            let w = if self.desired_size.width > 0.0 {
                self.desired_size.width
            } else {
                constraints.max_width
            };
            let h = if self.desired_size.height > 0.0 {
                self.desired_size.height
            } else {
                constraints.max_height
            };
            self.size = constraints.constrain(Size::new(w, h));
        }
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        canvas.save();
        canvas.translate(offset.dx, offset.dy);
        self.painter.paint(canvas, self.size);
        canvas.restore();

        if let Some(child) = &self.child {
            child.paint(canvas, offset + child.offset());
        }
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if let Some(child) = &mut self.child {
            let child_offset = child.offset();
            let local_p = event.point() - child_offset;
            let hit = child.hit_test(local_p);
            let is_up = matches!(event, TouchEvent::Up(_));
            let is_cancel = matches!(event, TouchEvent::Cancel);
            if hit || is_up || is_cancel {
                let child_event = event.transform(local_p);
                return child.dispatch_touch(&child_event);
            }
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if let Some(child) = &mut self.child {
            child.set_pressed_at(point - child.offset(), pressed);
        }
    }
    fn hit_rect(&self, point: Point) -> Option<Rect> {
        let child = self.child.as_ref()?;
        let offset = child.offset();
        child.hit_rect(point - offset).map(|r| r.shift(offset))
    }
    fn needs_rebuild(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.needs_rebuild())
    }
}
