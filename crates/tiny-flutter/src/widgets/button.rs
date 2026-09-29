use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;
use crate::graphics::geometry::{EdgeInsets, Offset, Point, RRect, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::text::Text;
use crate::widgets::widget::Widget;
use std::sync::Arc;

/// Reusable styling configuration for ElevatedButton.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub color: Color,
    pub pressed_color: Color,
    pub border_radius: f32,
    pub antialias: bool,
    pub padding: EdgeInsets,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            color: Color::PRIMARY,
            pressed_color: Color::from_rgb(21, 101, 192),
            border_radius: 8.0,
            antialias: false,
            padding: EdgeInsets::symmetric(8.0, 16.0),
            width: None,
            height: None,
        }
    }
}

impl ButtonStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn pressed_color(mut self, color: Color) -> Self {
        self.pressed_color = color;
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
    /// Coverage rendering for capsule edges. Existing app styles retain their raster path.
    pub fn antialias(mut self, enabled: bool) -> Self {
        self.antialias = enabled;
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// A versatile, generic Flutter-like ElevatedButton component.
pub struct ElevatedButton {
    pub child: Box<dyn Widget>,
    pub on_pressed: Option<Arc<dyn Fn() + Send + Sync>>,
    pub color: Color,
    pub pressed_color: Color,
    pub padding: EdgeInsets,
    pub border_radius: f32,
    pub antialias: bool,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub trigger_on_down: bool,
}

impl ElevatedButton {
    /// Create a button with arbitrary child widget.
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            on_pressed: None,
            color: Color::PRIMARY,
            pressed_color: Color::from_rgb(21, 101, 192),
            padding: EdgeInsets::symmetric(8.0, 16.0),
            border_radius: 8.0,
            antialias: false,
            width: None,
            height: None,
            trigger_on_down: false,
        }
    }

    /// Apply a comprehensive ButtonStyle to configure colors, dimensions, and padding.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.color = style.color;
        self.pressed_color = style.pressed_color;
        self.border_radius = style.border_radius;
        self.antialias = style.antialias;
        self.padding = style.padding;
        if style.width.is_some() {
            self.width = style.width;
        }
        if style.height.is_some() {
            self.height = style.height;
        }
        self
    }

    /// Convenience: create a button with centered text label.
    pub fn text(label: impl Into<String>) -> Self {
        Self::new(Text::new(label).color(Color::WHITE))
    }

    /// Convenience: create a button with centered text label and fixed dimensions.
    pub fn text_sized(label: impl Into<String>, width: f32, height: f32) -> Self {
        Self::text(label).width(width).height(height)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn trigger_on_down(mut self, trigger: bool) -> Self {
        self.trigger_on_down = trigger;
        self
    }

    pub fn on_pressed(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_pressed = Some(Arc::new(handler));
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn pressed_color(mut self, color: Color) -> Self {
        self.pressed_color = color;
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
}

impl Widget for ElevatedButton {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderElevatedButton {
            child: self.child.create_render_object(),
            on_pressed: self.on_pressed.clone(),
            color: self.color,
            pressed_color: self.pressed_color,
            padding: self.padding,
            border_radius: self.border_radius,
            antialias: self.antialias,
            width: self.width,
            height: self.height,
            trigger_on_down: self.trigger_on_down,
            is_pressed: false,
            touch_origin: None,
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderElevatedButton {
    pub child: Box<dyn RenderBox>,
    pub on_pressed: Option<Arc<dyn Fn() + Send + Sync>>,
    pub color: Color,
    pub pressed_color: Color,
    pub padding: EdgeInsets,
    pub border_radius: f32,
    pub antialias: bool,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub trigger_on_down: bool,
    is_pressed: bool,
    touch_origin: Option<Point>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderElevatedButton {
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
        let min_w = self
            .width
            .unwrap_or(constraints.min_width)
            .clamp(constraints.min_width, constraints.max_width);
        let max_w = self
            .width
            .unwrap_or(constraints.max_width)
            .clamp(min_w, constraints.max_width);

        let min_h = self
            .height
            .unwrap_or(constraints.min_height)
            .clamp(constraints.min_height, constraints.max_height);
        let max_h = self
            .height
            .unwrap_or(constraints.max_height)
            .clamp(min_h, constraints.max_height);

        let effective_constraints = BoxConstraints::new(min_w, max_w, min_h, max_h);

        // Allow child to size intrinsically within the button bounds
        let max_child_w = (effective_constraints.max_width - self.padding.horizontal()).max(0.0);
        let max_child_h = (effective_constraints.max_height - self.padding.vertical()).max(0.0);
        let child_constraints = BoxConstraints::new(0.0, max_child_w, 0.0, max_child_h);
        let child_size = self.child.layout(&child_constraints);

        let content_w = child_size.width + self.padding.horizontal();
        let content_h = child_size.height + self.padding.vertical();
        self.size = effective_constraints.constrain(Size::new(content_w, content_h));

        // Perfectly center child inside button bounds
        let dx = (self.size.width - child_size.width) / 2.0;
        let dy = (self.size.height - child_size.height) / 2.0;
        self.child.set_offset(Offset::new(dx.max(0.0), dy.max(0.0)));

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let btn_rect = Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height);
        let active_color = if self.is_pressed {
            self.pressed_color
        } else {
            self.color
        };

        let shape = RRect::from_rect_circular(btn_rect, self.border_radius);
        if self.antialias {
            canvas.draw_rrect_aa(shape, active_color);
        } else {
            canvas.draw_rrect(shape, active_color);
        }

        self.child.paint(canvas, offset + self.child.offset());
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let p = event.point();
        let inside = self.hit_test(p);

        match event {
            TouchEvent::Down(p) if inside => {
                self.touch_origin = Some(*p);
                self.is_pressed = true;
                if self.trigger_on_down {
                    if let Some(handler) = &self.on_pressed {
                        handler();
                    }
                }
                true
            }
            TouchEvent::Up(_) => {
                let was_pressed = self.is_pressed;
                self.is_pressed = false;
                if was_pressed && inside {
                    if !self.trigger_on_down {
                        if let Some(handler) = &self.on_pressed {
                            handler();
                        }
                    }
                    return true;
                }
                was_pressed
            }
            TouchEvent::Move(p) => {
                let moved = self
                    .touch_origin
                    .is_some_and(|o| (p.x - o.x).abs() > 12.0 || (p.y - o.y).abs() > 12.0);
                if (!inside || moved) && self.is_pressed {
                    self.is_pressed = false;
                    return true;
                }
                false
            }
            TouchEvent::Cancel => {
                let was = self.is_pressed;
                self.is_pressed = false;
                was
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
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if self.hit_test(point) {
            Some(Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
        } else {
            None
        }
    }
}
