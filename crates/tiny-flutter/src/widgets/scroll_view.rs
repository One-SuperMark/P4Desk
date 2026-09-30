use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::{Arc, Mutex};

/// Controller to observe or manipulate the scroll position of a `SingleChildScrollView`.
#[derive(Clone, Default)]
pub struct ScrollController {
    offset: Arc<Mutex<f32>>,
    raster: Arc<Mutex<super::scroll_raster::ScrollRaster>>,
}

impl ScrollController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_offset(initial: f32) -> Self {
        Self {
            offset: Arc::new(Mutex::new(initial)),
            ..Self::default()
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
    raster_cache: Option<(u64, Color)>,
}

impl SingleChildScrollView {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            controller: None,
            auto_scroll_to_bottom: false,
            raster_cache: None,
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

    /// For static content over a uniform opaque background only. Change the key
    /// whenever any rendered value changes. At most 2 MiB is retained per controller.
    pub fn raster_cache(mut self, revision: u64, background: Color) -> Self {
        self.raster_cache = Some((revision, background));
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
            touch_start_x: 0.0,
            child_canceled: false,
            touch_start_offset: 0.0,
            size: Size::ZERO,
            offset: Offset::ZERO,
            raster_cache: self.raster_cache,
            raster: self
                .controller
                .as_ref()
                .map(|c| c.raster.clone())
                .unwrap_or_default(),
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
    touch_start_x: f32,
    child_canceled: bool,
    touch_start_offset: f32,
    size: Size,
    offset: Offset,
    raster_cache: Option<(u64, Color)>,
    raster: Arc<Mutex<super::scroll_raster::ScrollRaster>>,
}

impl RenderBox for RenderSingleChildScrollView {
    fn drag_dirty(&self) -> Option<Rect> {
        (self.is_dragging && self.child_canceled && self.raster_cache.is_some())
            .then(|| Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
    }
    fn paint_opaque_region(&self, canvas: &mut Canvas, offset: Offset, dirty: Rect) -> bool {
        if self.is_dragging && !self.child_canceled {
            return false;
        }
        let Some((key, background)) = self.raster_cache else {
            return false;
        };
        let child_size = self.child.size();
        if child_size.width < self.size.width
            || child_size.height < self.size.height
            || dirty.x < offset.dx
            || dirty.y < offset.dy
            || dirty.right() > offset.dx + self.size.width
            || dirty.bottom() > offset.dy + self.size.height
        {
            return false;
        }
        self.raster.lock().unwrap().paint(
            self.child.as_ref(),
            key,
            background,
            canvas,
            Offset::new(offset.dx, offset.dy - self.scroll_offset),
        )
    }
    fn captures_touch(&self) -> bool {
        self.is_dragging
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
        // Pending taps use the actual pressed child. Once scrolling has canceled
        // the tap, replay the resting snapshot instead of rasterizing SVGs at
        // every new y coordinate. The controller retains it across root rebuilds.
        let cached = (!self.is_dragging || self.child_canceled)
            && self.raster_cache.is_some_and(|(key, background)| {
                self.raster.lock().unwrap().paint(
                    self.child.as_ref(),
                    key,
                    background,
                    canvas,
                    child_offset,
                )
            });
        if !cached {
            self.child.paint(canvas, child_offset);
        }

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
                self.touch_start_x = pt.x;
                self.child_canceled = false;
                self.touch_start_offset = self.scroll_offset;
                self.child
                    .dispatch_touch(&event.transform(Point::new(pt.x, pt.y + self.scroll_offset)));
                true
            }
            TouchEvent::Move(pt) if self.is_dragging => {
                let dy = pt.y - self.touch_start_y;
                let dx = pt.x - self.touch_start_x;
                // A horizontal slider may claim its gesture; vertical movement
                // still cancels it and belongs to this scroll view.
                if !self.child_canceled
                    && (self.child.captures_touch()
                        || (dx.abs() > 8.0 && dx.abs() > dy.abs() * 1.2))
                {
                    let handled = self.child.dispatch_touch(
                        &event.transform(Point::new(pt.x, pt.y + self.scroll_offset)),
                    );
                    if handled && self.child.captures_touch() {
                        return true;
                    }
                }
                let canceled_now = !self.child_canceled && (dy.abs() > 8.0 || dx.abs() > 8.0);
                if canceled_now {
                    self.child.dispatch_touch(&TouchEvent::Cancel);
                    self.child_canceled = true;
                }
                if !self.child_canceled {
                    return false;
                }
                let new_offset = (self.touch_start_offset - dy).clamp(0.0, self.max_scroll);
                if (new_offset - self.scroll_offset).abs() > 0.5 {
                    self.scroll_offset = new_offset;
                    if let Some(ctrl) = &self.controller {
                        ctrl.set_offset(new_offset);
                    }
                    true
                } else {
                    canceled_now
                }
            }
            TouchEvent::Up(pt) if self.is_dragging => {
                self.is_dragging = false;
                if self.child.captures_touch() {
                    return self.child.dispatch_touch(
                        &event.transform(Point::new(pt.x, pt.y + self.scroll_offset)),
                    );
                }
                if self.child_canceled
                    || (pt.y - self.touch_start_y).abs() > 8.0
                    || (pt.x - self.touch_start_x).abs() > 8.0
                {
                    self.child.dispatch_touch(&TouchEvent::Cancel);
                    self.scroll_offset = (self.touch_start_offset - (pt.y - self.touch_start_y))
                        .clamp(0.0, self.max_scroll);
                } else {
                    self.child.dispatch_touch(
                        &event.transform(Point::new(pt.x, pt.y + self.scroll_offset)),
                    );
                }
                if let Some(ctrl) = &self.controller {
                    ctrl.set_offset(self.scroll_offset);
                }
                true
            }
            TouchEvent::Cancel => {
                self.is_dragging = false;
                self.child.dispatch_touch(event);
                true
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
        if self.hit_test(point) || self.is_dragging {
            let viewport = Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height);
            if self.is_dragging && !self.child_canceled {
                // A press only changes its button, not the entire scroll page.
                // A press on empty space needs no paint but still owns the drag.
                Some(
                    self.child
                        .hit_rect(Point::new(point.x, point.y + self.scroll_offset))
                        .and_then(|r| {
                            r.shift(Offset::new(0.0, -self.scroll_offset))
                                .intersect(&viewport)
                        })
                        .unwrap_or(Rect::ZERO),
                )
            } else {
                Some(viewport)
            }
        } else {
            None
        }
    }
}
