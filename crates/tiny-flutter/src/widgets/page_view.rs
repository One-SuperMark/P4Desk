use super::page_motion::{DragVelocity, Settle};
use super::page_raster::PageRasterCache;
use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Transition animation style between pages.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PageTransition {
    /// In-place immediate page switch with zero fade, zero animation, zero flicker.
    #[default]
    None,
    /// In-place page flip with smooth color fade-in.
    Fade,
    /// Continuous horizontal sliding translation.
    Slide,
}

/// Controller to observe or manipulate the active page of a `PageView`.
#[derive(Clone)]
pub struct PageController {
    page: Arc<Mutex<usize>>,
    drag_offset: Arc<Mutex<f32>>,
    is_settling: Arc<Mutex<bool>>,
    fade_factor: Arc<Mutex<f32>>,
    settle: Arc<Mutex<Option<Settle>>>,
    rasters: Arc<Mutex<PageRasterCache>>,
}

impl Default for PageController {
    fn default() -> Self {
        Self::new()
    }
}

impl PageController {
    pub fn new() -> Self {
        Self {
            page: Arc::new(Mutex::new(0)),
            drag_offset: Arc::new(Mutex::new(0.0)),
            is_settling: Arc::new(Mutex::new(false)),
            fade_factor: Arc::new(Mutex::new(1.0)),
            settle: Arc::new(Mutex::new(None)),
            rasters: Arc::new(Mutex::new(PageRasterCache::default())),
        }
    }

    pub fn with_initial_page(page: usize) -> Self {
        Self {
            page: Arc::new(Mutex::new(page)),
            drag_offset: Arc::new(Mutex::new(0.0)),
            is_settling: Arc::new(Mutex::new(false)),
            fade_factor: Arc::new(Mutex::new(1.0)),
            settle: Arc::new(Mutex::new(None)),
            rasters: Arc::new(Mutex::new(PageRasterCache::default())),
        }
    }

    pub fn page(&self) -> usize {
        self.page.lock().map(|p| *p).unwrap_or(0)
    }

    pub fn has_cached_pages(&self, key: u64, size: Size) -> bool {
        self.rasters.lock().unwrap().contains(key, size)
    }

    pub fn set_page(&self, page: usize) {
        *self.settle.lock().unwrap() = None;
        if let Ok(mut p) = self.page.lock() {
            *p = page;
        }
        if let Ok(mut d) = self.drag_offset.lock() {
            *d = 0.0;
        }
        if let Ok(mut s) = self.is_settling.lock() {
            *s = false;
        }
        if let Ok(mut f) = self.fade_factor.lock() {
            *f = 1.0;
        }
    }

    pub fn drag_offset(&self) -> f32 {
        self.drag_offset.lock().map(|d| *d).unwrap_or(0.0)
    }

    pub fn set_drag_offset(&self, offset: f32) {
        if let Ok(mut d) = self.drag_offset.lock() {
            *d = offset;
        }
    }

    pub fn is_settling(&self) -> bool {
        self.is_settling.lock().map(|s| *s).unwrap_or(false)
    }

    pub fn set_settling(&self, settling: bool) {
        *self.settle.lock().unwrap() =
            settling.then(|| Settle::new(self.drag_offset(), 0.0, 864.0));
        if let Ok(mut s) = self.is_settling.lock() {
            *s = settling;
        }
    }

    fn settle_with_velocity(&self, velocity: f32, width: f32) {
        *self.settle.lock().unwrap() = Some(Settle::new(self.drag_offset(), velocity, width));
        *self.is_settling.lock().unwrap() = true;
    }

    pub fn fade_factor(&self) -> f32 {
        self.fade_factor.lock().map(|f| *f).unwrap_or(1.0)
    }

    pub fn set_fade_factor(&self, factor: f32) {
        if let Ok(mut f) = self.fade_factor.lock() {
            *f = factor.clamp(0.0, 1.0);
        }
    }

    pub fn start_fade(&self) {
        if let Ok(mut f) = self.fade_factor.lock() {
            *f = 0.05;
        }
        if let Ok(mut s) = self.is_settling.lock() {
            *s = true;
        }
    }
}

/// A scrollable list of pages that work along horizontal swipe gestures or in-place transitions.
pub struct PageView {
    pub children: Vec<Box<dyn Widget>>,
    pub controller: PageController,
    pub on_page_changed: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    pub transition: PageTransition,
    pub raster_cache_key: Option<u64>,
}

impl PageView {
    pub fn new(children: Vec<impl Widget + 'static>) -> Self {
        Self {
            children: children
                .into_iter()
                .map(|c| Box::new(c) as Box<dyn Widget>)
                .collect(),
            controller: PageController::new(),
            on_page_changed: None,
            transition: PageTransition::None,
            raster_cache_key: None,
        }
    }

    pub fn controller(mut self, controller: PageController) -> Self {
        self.controller = controller;
        self
    }

    pub fn transition(mut self, transition: PageTransition) -> Self {
        self.transition = transition;
        self
    }

    /// Cache transparent page content for smooth sliding over a stationary background.
    pub fn raster_cache_key(mut self, key: u64) -> Self {
        self.raster_cache_key = Some(key);
        self
    }

    pub fn on_page_changed(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_page_changed = Some(Arc::new(callback));
        self
    }
}

impl Widget for PageView {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let child_objects = self
            .children
            .iter()
            .map(|c| c.create_render_object())
            .collect();
        Box::new(RenderPageView {
            children: child_objects,
            controller: self.controller.clone(),
            on_page_changed: self.on_page_changed.clone(),
            transition: self.transition,
            raster_cache_key: self.raster_cache_key,
            current_page: self.controller.page(),
            drag_offset: self.controller.drag_offset(),
            touch_active: false,
            child_touch_canceled: false,
            is_swiping: false,
            gesture_handled: false,
            last_event_changed: true,
            touch_start_x: 0.0,
            touch_start_y: 0.0,
            touch_start_drag: 0.0,
            velocity: DragVelocity::default(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderPageView {
    pub children: Vec<Box<dyn RenderBox>>,
    pub controller: PageController,
    pub on_page_changed: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    pub transition: PageTransition,
    pub raster_cache_key: Option<u64>,
    current_page: usize,
    drag_offset: f32,
    touch_active: bool,
    child_touch_canceled: bool,
    is_swiping: bool,
    gesture_handled: bool,
    last_event_changed: bool,
    touch_start_x: f32,
    touch_start_y: f32,
    touch_start_drag: f32,
    velocity: DragVelocity,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderPageView {
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn needs_rebuild(&self) -> bool {
        self.transition == PageTransition::Fade && self.controller.is_settling()
    }

    fn captures_touch(&self) -> bool {
        self.touch_active
    }
    fn animation_dirty(&self) -> Option<Rect> {
        (self.transition == PageTransition::Slide && self.controller.is_settling())
            .then(|| Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
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

        let page_constraints = BoxConstraints::tight(self.size);
        for child in &mut self.children {
            child.layout(&page_constraints);
            child.set_offset(Offset::ZERO);
        }

        self.current_page = self
            .controller
            .page()
            .min(self.children.len().saturating_sub(1));
        self.drag_offset = self.controller.drag_offset();

        if self.transition == PageTransition::None {
            self.controller.set_fade_factor(1.0);
            self.controller.set_settling(false);
        } else if self.transition == PageTransition::Fade {
            if self.controller.is_settling() {
                let current_fade = self.controller.fade_factor();
                let next_fade = current_fade + 0.30;
                if next_fade >= 1.0 {
                    self.controller.set_fade_factor(1.0);
                    self.controller.set_settling(false);
                } else {
                    self.controller.set_fade_factor(next_fade);
                }
            }
        } else if self.controller.is_settling() {
            let mut settle = self.controller.settle.lock().unwrap();
            if let Some(motion) = *settle {
                self.drag_offset = motion.position(motion.started.elapsed().as_secs_f32());
                self.controller.set_drag_offset(self.drag_offset);
                if self.drag_offset.abs() < 0.25 {
                    self.drag_offset = 0.0;
                    self.controller.set_drag_offset(0.0);
                    *self.controller.is_settling.lock().unwrap() = false;
                    *settle = None;
                }
            }
        }

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        if self.children.is_empty() {
            return;
        }

        if let Some(key) = self
            .raster_cache_key
            .filter(|_| !self.touch_active && !self.controller.is_settling())
        {
            self.controller
                .rasters
                .lock()
                .unwrap()
                .prepare(key, self.size, &self.children);
        }
        canvas.save();
        canvas.clip_rect(Rect::from_ltwh(
            offset.dx,
            offset.dy,
            self.size.width,
            self.size.height,
        ));

        let current_page = self.current_page.min(self.children.len() - 1);

        if self.transition == PageTransition::None || self.transition == PageTransition::Fade {
            // Strictly stationary painting in-place at offset: ZERO spatial shift
            self.children[current_page].paint(canvas, offset);
        } else {
            let drag_x = self.drag_offset;

            // 1. Paint current active page
            let current_offset = offset + Offset::new(drag_x, 0.0);
            self.paint_slide_page(current_page, canvas, current_offset);

            // 2. Paint neighbor page if dragging or settling
            if drag_x < 0.0 && current_page + 1 < self.children.len() {
                // Revealing next page on the right
                let next_offset = offset + Offset::new(self.size.width + drag_x, 0.0);
                self.paint_slide_page(current_page + 1, canvas, next_offset);
            } else if drag_x > 0.0 && current_page > 0 {
                // Revealing previous page on the left
                let prev_offset = offset + Offset::new(-self.size.width + drag_x, 0.0);
                self.paint_slide_page(current_page - 1, canvas, prev_offset);
            }
        }

        canvas.restore();
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        self.last_event_changed = true;
        if self.children.is_empty() {
            return false;
        }

        let p = event.point();
        let inside = self.hit_test(p);
        let current_page = self.current_page.min(self.children.len() - 1);

        match event {
            TouchEvent::Down(pt) if inside => {
                self.touch_active = true;
                self.child_touch_canceled = false;
                self.touch_start_x = pt.x;
                self.touch_start_y = pt.y;
                self.is_swiping = false;
                self.gesture_handled = false;
                self.touch_start_drag = if self.transition == PageTransition::Slide {
                    self.controller.drag_offset()
                } else {
                    0.0
                };
                self.drag_offset = self.touch_start_drag;
                // Invert edge resistance before grabbing an unfinished bounce,
                // so the next sample continues exactly where the page was shown.
                self.touch_start_drag = unresisted_drag(
                    self.touch_start_drag,
                    current_page,
                    self.children.len(),
                    self.size.width,
                );
                self.velocity = DragVelocity::default();
                self.velocity.record(Instant::now(), self.drag_offset);
                if self.transition == PageTransition::Slide {
                    self.controller.set_settling(false);
                    if self.drag_offset.abs() > 0.25 {
                        self.is_swiping = true;
                        self.child_touch_canceled = true;
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        return true;
                    }
                }

                // Pass down to active child so button hover/press can track
                self.children[current_page].dispatch_touch(event);
                true
            }
            // A valid Down owns the gesture through release, including moves
            // beyond the page bounds. Late samples after Cancel/Up are ignored.
            TouchEvent::Move(pt) if self.touch_active => {
                let dx = pt.x - self.touch_start_x;
                let dy = pt.y - self.touch_start_y;

                if self.transition == PageTransition::None
                    || self.transition == PageTransition::Fade
                {
                    // Continue owning this swipe without repainting the same
                    // stationary page for every remaining finger sample.
                    if self.gesture_handled {
                        self.last_event_changed = false;
                        return true;
                    }
                    if !self.child_touch_canceled && (dx.abs() > 12.0 || dy.abs() > 12.0) {
                        // The page owns movement through release, including a
                        // move into another row. Cancel the original button even
                        // when Flex hit testing would route Move to another child.
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        self.child_touch_canceled = true;
                    }
                    if !self.gesture_handled {
                        // Gesture threshold: > 32px horizontally and predominantly horizontal
                        if dx.abs() > 32.0 && dx.abs() > dy.abs() * 1.2 {
                            self.is_swiping = true;
                            self.gesture_handled = true;
                            // The 12px movement gate already canceled the button.

                            let mut page_changed = false;
                            if dx < -32.0 && current_page + 1 < self.children.len() {
                                self.current_page += 1;
                                self.controller.set_page(self.current_page);
                                page_changed = true;
                            } else if dx > 32.0 && current_page > 0 {
                                self.current_page -= 1;
                                self.controller.set_page(self.current_page);
                                page_changed = true;
                            }

                            if page_changed {
                                if let Some(ref cb) = self.on_page_changed {
                                    (cb)(self.current_page);
                                }
                            }
                            return true;
                        }
                    }

                    if self.is_swiping {
                        return true;
                    }

                    if self.child_touch_canceled {
                        return true;
                    }

                    self.children[current_page].dispatch_touch(event)
                } else {
                    if !self.child_touch_canceled && (dx.abs() > 8.0 || dy.abs() > 8.0) {
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        self.child_touch_canceled = true;
                    }
                    if !self.is_swiping && dx.abs() > 8.0 && dx.abs() > dy.abs() * 1.1 {
                        self.is_swiping = true;
                        self.controller.set_settling(false);
                    }
                    if self.is_swiping {
                        self.drag_offset = drag_position(
                            self.touch_start_drag + dx,
                            current_page,
                            self.children.len(),
                            self.size.width,
                        );
                        self.velocity.record(Instant::now(), self.drag_offset);
                        self.controller.set_drag_offset(self.drag_offset);
                        true
                    } else if self.child_touch_canceled {
                        true
                    } else {
                        self.children[current_page].dispatch_touch(event)
                    }
                }
            }

            TouchEvent::Up(pt) if self.touch_active => {
                self.touch_active = false;
                if self.transition == PageTransition::Fade
                    || self.transition == PageTransition::None
                {
                    let dx = pt.x - self.touch_start_x;
                    let dy = pt.y - self.touch_start_y;
                    let was_swiping = self.is_swiping || self.gesture_handled;
                    self.is_swiping = false;

                    if !self.gesture_handled && dx.abs() > 28.0 && dx.abs() > dy.abs() {
                        // Up can be the first sample reaching the swipe
                        // threshold. Cancel the original child before switching
                        // pages, even when already at the first/last page.
                        if !self.child_touch_canceled {
                            self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                            self.child_touch_canceled = true;
                        }
                        self.gesture_handled = true;
                        let mut page_changed = false;
                        if dx < -28.0 && current_page + 1 < self.children.len() {
                            self.current_page += 1;
                            self.controller.set_page(self.current_page);
                            page_changed = true;
                        } else if dx > 28.0 && current_page > 0 {
                            self.current_page -= 1;
                            self.controller.set_page(self.current_page);
                            page_changed = true;
                        }

                        if page_changed {
                            if let Some(ref cb) = self.on_page_changed {
                                (cb)(self.current_page);
                            }
                        }
                        return true;
                    }

                    if was_swiping || self.child_touch_canceled {
                        // Its original button was already canceled. Swallow the
                        // release without delivering Cancel to a newly shown page.
                        true
                    } else {
                        self.children[current_page].dispatch_touch(event)
                    }
                } else {
                    let dx = pt.x - self.touch_start_x;
                    let dy = pt.y - self.touch_start_y;
                    if !self.child_touch_canceled && (dx.abs() > 8.0 || dy.abs() > 8.0) {
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        self.child_touch_canceled = true;
                    }
                    if !self.is_swiping && dx.abs() > 8.0 && dx.abs() > dy.abs() * 1.1 {
                        self.is_swiping = true;
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        self.child_touch_canceled = true;
                    }
                    if self.is_swiping {
                        self.drag_offset = drag_position(
                            self.touch_start_drag + dx,
                            current_page,
                            self.children.len(),
                            self.size.width,
                        );
                        let now = Instant::now();
                        self.velocity.record(now, self.drag_offset);
                        let velocity = self.velocity.velocity(now);
                        self.is_swiping = false;

                        let mut page_changed = false;
                        if self.drag_offset < -self.size.width * 0.5
                            && current_page + 1 < self.children.len()
                        {
                            self.current_page += 1;
                            self.controller.set_page(self.current_page);
                            let new_drag = self.size.width + self.drag_offset;
                            self.drag_offset = new_drag;
                            self.controller.set_drag_offset(new_drag);
                            self.controller
                                .settle_with_velocity(velocity, self.size.width);
                            page_changed = true;
                        } else if self.drag_offset > self.size.width * 0.5 && current_page > 0 {
                            self.current_page -= 1;
                            self.controller.set_page(self.current_page);
                            let new_drag = -self.size.width + self.drag_offset;
                            self.drag_offset = new_drag;
                            self.controller.set_drag_offset(new_drag);
                            self.controller
                                .settle_with_velocity(velocity, self.size.width);
                            page_changed = true;
                        } else if self.drag_offset.abs() > 1.0 {
                            self.controller.set_drag_offset(self.drag_offset);
                            self.controller
                                .settle_with_velocity(velocity, self.size.width);
                        } else {
                            self.drag_offset = 0.0;
                            self.controller.set_drag_offset(0.0);
                            self.controller.set_settling(false);
                        }

                        if page_changed {
                            if let Some(ref cb) = self.on_page_changed {
                                (cb)(self.current_page);
                            }
                        }
                        true
                    } else {
                        self.drag_offset = 0.0;
                        self.controller.set_drag_offset(0.0);
                        self.controller.set_settling(false);
                        if self.child_touch_canceled {
                            true
                        } else {
                            self.children[current_page].dispatch_touch(event)
                        }
                    }
                }
            }
            TouchEvent::Cancel => {
                self.touch_active = false;
                self.child_touch_canceled = false;
                self.is_swiping = false;
                self.gesture_handled = false;
                if self.transition == PageTransition::Slide {
                    if self.drag_offset.abs() > 1.0 {
                        self.controller.set_settling(true);
                    } else {
                        self.drag_offset = 0.0;
                        self.controller.set_drag_offset(0.0);
                        self.controller.set_settling(false);
                    }
                }
                self.children[current_page].dispatch_touch(event)
            }
            TouchEvent::Down(_) | TouchEvent::Move(_) | TouchEvent::Up(_) => false,
            _ => self.children[current_page].dispatch_touch(event),
        }
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if !self.children.is_empty() {
            let current_page = self.current_page.min(self.children.len() - 1);
            self.children[current_page].set_pressed_at(point, pressed);
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if !self.last_event_changed {
            return Some(Rect::from_ltwh(0.0, 0.0, 0.0, 0.0));
        }
        if !self.is_swiping
            && !self.controller.is_settling()
            && self.drag_offset == 0.0
            && !self.children.is_empty()
        {
            let child = &self.children[self.current_page.min(self.children.len() - 1)];
            let current = child.hit_rect(point);
            if self.touch_active {
                let origin = child.hit_rect(Point::new(self.touch_start_x, self.touch_start_y));
                return match (origin, current) {
                    (Some(a), Some(b)) => Some(a.union(&b)),
                    (a, b) => a.or(b),
                };
            }
            return current;
        }
        if self.hit_test(point) || self.touch_active || self.controller.is_settling() {
            Some(Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
        } else {
            None
        }
    }
}

fn unresisted_drag(offset: f32, page: usize, count: usize, width: f32) -> f32 {
    if (page == 0 && offset > 0.0) || (page + 1 == count && offset < 0.0) {
        let limit = width * 0.18;
        offset.signum() * limit * offset.abs() / (limit - offset.abs()).max(1.0)
    } else {
        offset
    }
}
fn drag_position(dx: f32, page: usize, count: usize, width: f32) -> f32 {
    if (page == 0 && dx > 0.0) || (page + 1 == count && dx < 0.0) {
        let distance = dx.abs();
        dx.signum() * width * 0.18 * distance / (width * 0.18 + distance)
    } else {
        dx.clamp(-width, width)
    }
}
impl RenderPageView {
    fn paint_slide_page(&self, page: usize, canvas: &mut Canvas, offset: Offset) {
        if self.drag_offset.abs() > 0.0 {
            if let Some(key) = self.raster_cache_key {
                if self
                    .controller
                    .rasters
                    .lock()
                    .unwrap()
                    .paint(key, self.size, page, canvas, offset)
                {
                    return;
                }
            }
        }
        self.children[page].paint(canvas, offset);
    }
}
