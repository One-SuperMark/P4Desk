use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::widgets::widget::Widget;
use std::sync::{Arc, Mutex};

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
        }
    }

    pub fn with_initial_page(page: usize) -> Self {
        Self {
            page: Arc::new(Mutex::new(page)),
            drag_offset: Arc::new(Mutex::new(0.0)),
            is_settling: Arc::new(Mutex::new(false)),
            fade_factor: Arc::new(Mutex::new(1.0)),
        }
    }

    pub fn page(&self) -> usize {
        self.page.lock().map(|p| *p).unwrap_or(0)
    }

    pub fn set_page(&self, page: usize) {
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
        if let Ok(mut s) = self.is_settling.lock() {
            *s = settling;
        }
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
            current_page: self.controller.page(),
            drag_offset: self.controller.drag_offset(),
            is_swiping: false,
            gesture_handled: false,
            touch_start_x: 0.0,
            touch_start_y: 0.0,
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
    current_page: usize,
    drag_offset: f32,
    is_swiping: bool,
    gesture_handled: bool,
    touch_start_x: f32,
    touch_start_y: f32,
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
        self.controller.is_settling()
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
        } else {
            // Smooth physics-based decay towards resting position (spring settling)
            if self.controller.is_settling() {
                let next_offset = self.drag_offset * 0.58;
                if next_offset.abs() < 1.5 {
                    self.drag_offset = 0.0;
                    self.controller.set_drag_offset(0.0);
                    self.controller.set_settling(false);
                } else {
                    self.drag_offset = next_offset;
                    self.controller.set_drag_offset(next_offset);
                }
            }
        }

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        if self.children.is_empty() {
            return;
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
            self.children[current_page].paint(canvas, current_offset);

            // 2. Paint neighbor page if dragging or settling
            if drag_x < 0.0 && current_page + 1 < self.children.len() {
                // Revealing next page on the right
                let next_offset = offset + Offset::new(self.size.width + drag_x, 0.0);
                self.children[current_page + 1].paint(canvas, next_offset);
            } else if drag_x > 0.0 && current_page > 0 {
                // Revealing previous page on the left
                let prev_offset = offset + Offset::new(-self.size.width + drag_x, 0.0);
                self.children[current_page - 1].paint(canvas, prev_offset);
            }
        }

        canvas.restore();
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if self.children.is_empty() {
            return false;
        }

        let p = event.point();
        let inside = self.hit_test(p);
        let current_page = self.current_page.min(self.children.len() - 1);

        match event {
            TouchEvent::Down(pt) if inside => {
                self.touch_start_x = pt.x;
                self.touch_start_y = pt.y;
                self.is_swiping = false;
                self.gesture_handled = false;
                self.drag_offset = 0.0;
                if self.transition == PageTransition::Slide {
                    self.controller.set_settling(false);
                }

                // Pass down to active child so button hover/press can track
                self.children[current_page].dispatch_touch(event);
                true
            }
            TouchEvent::Move(pt) if inside || self.is_swiping => {
                let dx = pt.x - self.touch_start_x;
                let dy = pt.y - self.touch_start_y;

                if self.transition == PageTransition::None
                    || self.transition == PageTransition::Fade
                {
                    if !self.gesture_handled {
                        // Gesture threshold: > 32px horizontally and predominantly horizontal
                        if dx.abs() > 32.0 && dx.abs() > dy.abs() * 1.2 {
                            self.is_swiping = true;
                            self.gesture_handled = true;
                            // Cancel active button tap on child
                            self.children[current_page].dispatch_touch(&TouchEvent::Cancel);

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

                    self.children[current_page].dispatch_touch(event)
                } else {
                    if !self.is_swiping && (dx.abs() > 12.0 || dy.abs() > 12.0) {
                        if dx.abs() > dy.abs() {
                            self.is_swiping = true;
                            self.controller.set_settling(false);
                            // Cancel touch on child so button doesn't trigger tap
                            self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        }
                    }

                    if self.is_swiping {
                        // Limit overscroll resistance
                        let clamped_dx = if (current_page == 0 && dx > 0.0)
                            || (current_page + 1 == self.children.len() && dx < 0.0)
                        {
                            dx * 0.35 // rubber-band resistance at boundaries
                        } else {
                            dx
                        };

                        // Slew-rate limiter & EMA filter:
                        let step = clamped_dx - self.drag_offset;
                        let max_step = 32.0;
                        let final_dx = self.drag_offset + step.clamp(-max_step, max_step);

                        self.drag_offset = final_dx;
                        self.controller.set_drag_offset(final_dx);
                        true
                    } else {
                        self.children[current_page].dispatch_touch(event)
                    }
                }
            }
            TouchEvent::Up(pt) => {
                if self.transition == PageTransition::Fade
                    || self.transition == PageTransition::None
                {
                    let dx = pt.x - self.touch_start_x;
                    let dy = pt.y - self.touch_start_y;
                    let was_swiping = self.is_swiping || self.gesture_handled;
                    self.is_swiping = false;

                    if !self.gesture_handled && dx.abs() > 28.0 && dx.abs() > dy.abs() {
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

                    if was_swiping {
                        // User was performing a swipe; swallow tap so app does not launch
                        self.children[current_page].dispatch_touch(&TouchEvent::Cancel);
                        true
                    } else {
                        self.children[current_page].dispatch_touch(event)
                    }
                } else {
                    if self.is_swiping {
                        let dx = pt.x - self.touch_start_x;
                        self.is_swiping = false;

                        let mut page_changed = false;
                        if dx < -50.0 && current_page + 1 < self.children.len() {
                            self.current_page += 1;
                            self.controller.set_page(self.current_page);
                            let new_drag = self.size.width + self.drag_offset;
                            self.drag_offset = new_drag;
                            self.controller.set_drag_offset(new_drag);
                            self.controller.set_settling(true);
                            page_changed = true;
                        } else if dx > 50.0 && current_page > 0 {
                            self.current_page -= 1;
                            self.controller.set_page(self.current_page);
                            let new_drag = -self.size.width + self.drag_offset;
                            self.drag_offset = new_drag;
                            self.controller.set_drag_offset(new_drag);
                            self.controller.set_settling(true);
                            page_changed = true;
                        } else if self.drag_offset.abs() > 1.0 {
                            self.controller.set_settling(true);
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
                        self.children[current_page].dispatch_touch(event)
                    }
                }
            }
            TouchEvent::Cancel => {
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
        if self.hit_test(point) {
            Some(Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
        } else {
            None
        }
    }
}
