use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{EdgeInsets, Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;

// =========================================================================
// Padding
// =========================================================================

pub struct Padding {
    pub padding: EdgeInsets,
    pub child: Box<dyn Widget>,
}

impl Padding {
    pub fn new(padding: EdgeInsets, child: impl Widget + 'static) -> Self {
        Self {
            padding,
            child: Box::new(child),
        }
    }
}

impl Widget for Padding {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderPadding {
            padding: self.padding,
            child: self.child.create_render_object(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderPadding {
    pub padding: EdgeInsets,
    pub child: Box<dyn RenderBox>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderPadding {
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
        let inner_constraints = constraints.deflate(self.padding);
        let child_size = self.child.layout(&inner_constraints);
        self.child
            .set_offset(Offset::new(self.padding.left, self.padding.top));
        self.size = constraints.constrain(self.padding.inflate_size(child_size));
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
// Center
// =========================================================================

pub struct Center {
    pub child: Box<dyn Widget>,
}

impl Center {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl Widget for Center {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderCenter {
            child: self.child.create_render_object(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderCenter {
    pub child: Box<dyn RenderBox>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderCenter {
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
        let child_size = self.child.layout(&constraints.loosen());
        let width = if constraints.has_bounded_width() {
            constraints.max_width
        } else {
            child_size.width
        };
        let height = if constraints.has_bounded_height() {
            constraints.max_height
        } else {
            child_size.height
        };

        let dx = (width - child_size.width) / 2.0;
        let dy = (height - child_size.height) / 2.0;
        self.child.set_offset(Offset::new(dx, dy));

        self.size = constraints.constrain(Size::new(width, height));
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
// SizedBox
// =========================================================================

pub struct SizedBox {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub child: Option<Box<dyn Widget>>,
}

impl SizedBox {
    /// Creates a box with zero width and zero height.
    pub fn shrink() -> Self {
        Self::from_size(Size::ZERO)
    }

    pub fn from_size(size: Size) -> Self {
        Self {
            width: Some(size.width),
            height: Some(size.height),
            child: None,
        }
    }

    pub fn square(dimension: f32) -> Self {
        Self {
            width: Some(dimension),
            height: Some(dimension),
            child: None,
        }
    }

    pub fn with_child(
        width: Option<f32>,
        height: Option<f32>,
        child: impl Widget + 'static,
    ) -> Self {
        Self {
            width,
            height,
            child: Some(Box::new(child)),
        }
    }
}

impl Widget for SizedBox {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderSizedBox {
            width: self.width,
            height: self.height,
            child: self.child.as_ref().map(|c| c.create_render_object()),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderSizedBox {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub child: Option<Box<dyn RenderBox>>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderSizedBox {
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
        let additional = BoxConstraints::tight_for(self.width, self.height);
        let actual_constraints = additional.enforce(constraints);

        if let Some(child) = &mut self.child {
            let child_size = child.layout(&actual_constraints);
            child.set_offset(Offset::ZERO);
            self.size = actual_constraints.constrain(child_size);
        } else {
            self.size = actual_constraints.constrain(Size::new(
                self.width.unwrap_or(0.0),
                self.height.unwrap_or(0.0),
            ));
        }

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
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
            if hit || is_up || matches!(event, TouchEvent::Cancel) {
                let child_event = event.transform(local_p);
                return child.dispatch_touch(&child_event);
            }
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if let Some(child) = &mut self.child {
            let child_offset = child.offset();
            let local_p = point - child_offset;
            if child.hit_test(local_p) {
                child.set_pressed_at(local_p, pressed);
            }
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if let Some(child) = &self.child {
            let child_offset = child.offset();
            let local_p = point - child_offset;
            if child.hit_test(local_p) {
                return child.hit_rect(local_p).map(|r| {
                    Rect::from_ltwh(
                        r.x + child_offset.dx,
                        r.y + child_offset.dy,
                        r.width,
                        r.height,
                    )
                });
            }
        }
        None
    }
}

// =========================================================================
// Expanded
// =========================================================================

/// A widget that expands a child of a Row or Column so that the child fills the available space.
pub struct Expanded {
    pub flex: u32,
    pub child: Box<dyn Widget>,
}

impl Expanded {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            flex: 1,
            child: Box::new(child),
        }
    }

    pub fn flex(mut self, flex: u32) -> Self {
        self.flex = flex.max(1);
        self
    }
}

impl Widget for Expanded {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderExpanded {
            flex: self.flex,
            child: self.child.create_render_object(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderExpanded {
    pub flex: u32,
    pub child: Box<dyn RenderBox>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderExpanded {
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn flex(&self) -> u32 {
        self.flex
    }

    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        let child_size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        self.size = constraints.constrain(child_size);
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
// FractionallySizedBox
// =========================================================================

/// A widget that sizes its child to a fraction of the total available space.
pub struct FractionallySizedBox {
    pub width_factor: Option<f32>,
    pub height_factor: Option<f32>,
    pub child: Option<Box<dyn Widget>>,
}

impl FractionallySizedBox {
    pub fn new() -> Self {
        Self {
            width_factor: None,
            height_factor: None,
            child: None,
        }
    }

    pub fn width_factor(mut self, factor: f32) -> Self {
        self.width_factor = Some(factor);
        self
    }

    pub fn height_factor(mut self, factor: f32) -> Self {
        self.height_factor = Some(factor);
        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Default for FractionallySizedBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for FractionallySizedBox {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderFractionallySizedBox {
            width_factor: self.width_factor,
            height_factor: self.height_factor,
            child: self.child.as_ref().map(|c| c.create_render_object()),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderFractionallySizedBox {
    pub width_factor: Option<f32>,
    pub height_factor: Option<f32>,
    pub child: Option<Box<dyn RenderBox>>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderFractionallySizedBox {
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
        let target_w = self.width_factor.map(|f| constraints.max_width * f);
        let target_h = self.height_factor.map(|f| constraints.max_height * f);

        let child_constraints = BoxConstraints::tight_for(target_w, target_h);

        if let Some(child) = &mut self.child {
            let child_size = child.layout(&child_constraints);
            let total_w = target_w.unwrap_or(child_size.width);
            let total_h = target_h.unwrap_or(child_size.height);

            let dx = ((total_w - child_size.width) / 2.0).max(0.0);
            let dy = ((total_h - child_size.height) / 2.0).max(0.0);
            child.set_offset(Offset::new(dx, dy));

            self.size = constraints.constrain(Size::new(total_w, total_h));
        } else {
            self.size =
                constraints.constrain(Size::new(target_w.unwrap_or(0.0), target_h.unwrap_or(0.0)));
        }

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
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
            if hit || is_up || matches!(event, TouchEvent::Cancel) {
                let child_event = event.transform(local_p);
                return child.dispatch_touch(&child_event);
            }
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if let Some(child) = &mut self.child {
            let child_offset = child.offset();
            let local_p = point - child_offset;
            if child.hit_test(local_p) {
                child.set_pressed_at(local_p, pressed);
            }
        }
    }
}

// Re-export Positioned and Stack from modular stack widget
pub use crate::widgets::stack::{Positioned, RenderPositioned, RenderStack, Stack};
