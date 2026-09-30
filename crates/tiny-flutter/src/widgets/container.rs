use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;
use crate::graphics::geometry::{EdgeInsets, Offset, Point, RRect, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::theme::GlassMaterial;
use crate::widgets::widget::Widget;

pub struct Container {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: Option<EdgeInsets>,
    pub margin: Option<EdgeInsets>,
    pub color: Option<Color>,
    pub border_radius: Option<f32>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub glass: Option<GlassMaterial>,
    pub child: Option<Box<dyn Widget>>,
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            padding: None,
            margin: None,
            color: None,
            border_radius: None,
            border_color: None,
            border_width: None,
            glass: None,
            child: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = Some(width);
        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
    pub fn glass_material(mut self, material: GlassMaterial) -> Self {
        self.glass = Some(material);
        self
    }

    pub fn glass(mut self, enabled: bool) -> Self {
        self.glass = enabled.then_some(GlassMaterial::Page);
        self
    }
}

impl Widget for Container {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderContainer {
            width: self.width,
            height: self.height,
            padding: self.padding.unwrap_or(EdgeInsets::ZERO),
            margin: self.margin.unwrap_or(EdgeInsets::ZERO),
            color: self.color,
            border_radius: self.border_radius,
            border_color: self.border_color,
            border_width: self.border_width,
            glass: self.glass,
            child: self.child.as_ref().map(|c| c.create_render_object()),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderContainer {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub color: Option<Color>,
    pub border_radius: Option<f32>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub glass: Option<GlassMaterial>,
    pub child: Option<Box<dyn RenderBox>>,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderContainer {
    fn drag_dirty(&self) -> Option<Rect> {
        self.child
            .as_ref()
            .and_then(|c| c.drag_dirty().map(|r| r.shift(c.offset())))
    }
    fn paint_opaque_region(&self, canvas: &mut Canvas, offset: Offset, dirty: Rect) -> bool {
        if self.border_color.is_some() && self.border_width.unwrap_or(0.0) > 0.0 {
            return false;
        }
        self.child
            .as_ref()
            .is_some_and(|c| c.paint_opaque_region(canvas, offset + c.offset(), dirty))
    }
    fn captures_touch(&self) -> bool {
        self.child.as_ref().is_some_and(|c| c.captures_touch())
    }
    fn animation_dirty(&self) -> Option<Rect> {
        let child = self.child.as_ref()?;
        child.animation_dirty().map(|r| r.shift(child.offset()))
    }
    fn needs_rebuild(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.needs_rebuild())
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
        let margin_h = self.margin.horizontal();
        let margin_v = self.margin.vertical();

        let explicit_w = self.width;
        let explicit_h = self.height;

        let content_constraints = BoxConstraints::new(
            explicit_w.unwrap_or(0.0),
            explicit_w.unwrap_or(constraints.max_width - margin_h),
            explicit_h.unwrap_or(0.0),
            explicit_h.unwrap_or(constraints.max_height - margin_v),
        );

        let (inner_w, inner_h) = if let Some(child) = &mut self.child {
            let child_constraints = content_constraints.deflate(self.padding);
            let child_size = child.layout(&child_constraints);
            child.set_offset(Offset::new(
                self.margin.left + self.padding.left,
                self.margin.top + self.padding.top,
            ));
            let padded_size = self.padding.inflate_size(child_size);
            (
                explicit_w.unwrap_or(padded_size.width),
                explicit_h.unwrap_or(padded_size.height),
            )
        } else {
            (
                explicit_w.unwrap_or(self.padding.horizontal()),
                explicit_h.unwrap_or(self.padding.vertical()),
            )
        };

        let total_w = inner_w + margin_h;
        let total_h = inner_h + margin_v;
        self.size = constraints.constrain(Size::new(total_w, total_h));
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let content_x = offset.dx + self.margin.left;
        let content_y = offset.dy + self.margin.top;
        let content_w = (self.size.width - self.margin.horizontal()).max(0.0);
        let content_h = (self.size.height - self.margin.vertical()).max(0.0);
        let content_rect = Rect::from_ltwh(content_x, content_y, content_w, content_h);

        // 1. Paint background
        if let Some(color) = self.color {
            if let Some(material) = self.glass {
                canvas.liquid_glass_material(
                    RRect::from_rect_circular(content_rect, self.border_radius.unwrap_or(0.0)),
                    color,
                    false,
                    material,
                );
            } else if let Some(radius) = self.border_radius {
                canvas.draw_rrect_aa(RRect::from_rect_circular(content_rect, radius), color);
            } else {
                canvas.draw_rect(content_rect, color);
            }
        }

        // 2. Paint child
        if let Some(child) = &self.child {
            child.paint(canvas, offset + child.offset());
        }

        // 3. Paint border on top
        if let (Some(b_color), Some(b_width)) = (self.border_color, self.border_width) {
            if b_width > 0.0 {
                if let Some(radius) = self.border_radius {
                    canvas.draw_rrect_stroke(
                        RRect::from_rect_circular(content_rect, radius),
                        b_color,
                        b_width,
                    );
                } else {
                    canvas.draw_rrect_stroke(
                        RRect::from_rect_circular(content_rect, 0.0),
                        b_color,
                        b_width,
                    );
                }
            }
        }
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if let Some(child) = &mut self.child {
            let child_offset = child.offset();
            let local_p = event.point() - child_offset;
            let hit = child.hit_test(local_p);
            let is_up = matches!(event, TouchEvent::Up(_));
            if hit
                || (matches!(event, TouchEvent::Move(_)) && child.captures_touch())
                || is_up
                || matches!(event, TouchEvent::Cancel)
            {
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
            if child.hit_test(local_p) || child.captures_touch() {
                if let Some(r) = child.hit_rect(local_p) {
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
