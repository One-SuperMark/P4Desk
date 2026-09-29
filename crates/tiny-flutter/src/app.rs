use crate::graphics::canvas::Canvas;
use crate::graphics::color::{extract_rect_to_rgb565, Color};
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::platform::backend::PlatformBackend;
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::dirty::DirtyRegion;
use crate::rendering::render_box::{RenderBox, TouchEvent};
use crate::tiny_gfx::Pixmap565;
use crate::widgets::widget::Widget;

/// Main application runner and state orchestrator for tiny-flutter.
pub struct App {
    root: Box<dyn RenderBox>,
    size: Size,
    pixmap: Pixmap565,
    scratch_rgb565: Vec<u16>,
    dirty: DirtyRegion,
    is_first_frame: bool,
    pub screen_on: bool,
    touch_down: Option<Point>,
    rebuild_requested: bool,
}

impl App {
    /// Create a new App instance given root Widget and screen dimensions.
    pub fn new(root_widget: impl Widget, size: Size) -> Self {
        let width = size.width as u32;
        let height = size.height as u32;

        let pixmap = Pixmap565::new(width, height).expect("Failed to allocate Pixmap565");
        let scratch_rgb565 = Vec::new();
        let mut root = root_widget.create_render_object();
        root.layout(&BoxConstraints::tight(size));

        Self {
            root,
            size,
            pixmap,
            scratch_rgb565,
            dirty: DirtyRegion::new(),
            is_first_frame: true,
            screen_on: true,
            touch_down: None,
            rebuild_requested: false,
        }
    }

    /// Returns true if the screen is currently powered on and active.
    pub fn is_screen_on(&self) -> bool {
        self.screen_on
    }

    /// Toggle or set screen power. When powered off, background state continues to run
    /// uninterrupted, while UI rasterization and frame output halt.
    pub fn set_screen_power(&mut self, backend: &mut dyn PlatformBackend, on: bool) {
        if self.screen_on != on {
            self.screen_on = on;
            backend.set_screen_power(on);
            if on {
                self.dirty.mark_all_dirty(self.size);
            }
        }
    }

    /// Mark the entire app or a subregion as dirty to trigger repainting.
    pub fn mark_dirty(&mut self, rect: Rect) {
        self.dirty.mark_dirty(rect);
    }

    /// Rebuild after external clock, USB or resource events, even when there was no touch.
    pub fn request_rebuild(&mut self) {
        self.rebuild_requested = true;
    }

    /// Cancel gestures on mode changes so a finger cannot activate a newly mounted app.
    pub fn cancel_touch(&mut self) {
        self.root.dispatch_touch(&TouchEvent::Cancel);
        self.touch_down = None;
    }

    pub fn framebuffer(&self) -> &[u16] {
        self.pixmap.data()
    }

    /// Re-mount or update the root widget hierarchy.
    pub fn set_root(&mut self, root_widget: impl Widget) {
        let mut root = root_widget.create_render_object();
        root.layout(&BoxConstraints::tight(self.size));
        if let Some(p) = self.touch_down {
            root.set_pressed_at(p, true);
        }
        self.root = root;
        self.dirty.mark_all_dirty(self.size);
    }

    /// Resize the app rendering surface and box constraints.
    pub fn resize(&mut self, new_size: Size) {
        if self.size == new_size {
            return;
        }
        let width = (new_size.width as u32).max(1);
        let height = (new_size.height as u32).max(1);
        self.size = new_size;
        self.pixmap = Pixmap565::new(width, height).expect("Failed to allocate Pixmap565");
        self.scratch_rgb565 = Vec::new();
        self.dirty.mark_all_dirty(new_size);
        self.root.layout(&BoxConstraints::tight(new_size));
    }

    /// Get current screen size.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Single frame step with static widget tree.
    pub fn step(&mut self, backend: &mut dyn PlatformBackend) {
        while let Some(event) = backend.poll_touch() {
            match event {
                TouchEvent::HardwarePower => {
                    self.set_screen_power(backend, !self.screen_on);
                }
                TouchEvent::Down(p) if !self.screen_on => {
                    self.touch_down = Some(p);
                    self.set_screen_power(backend, true);
                }
                TouchEvent::Down(p) => {
                    self.touch_down = Some(p);
                    if self.screen_on {
                        if self.root.dispatch_touch(&event) {
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                }
                TouchEvent::Up(_) => {
                    if self.screen_on {
                        if self.root.dispatch_touch(&event) {
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                    self.touch_down = None;
                }
                TouchEvent::Move(p) => {
                    if self.touch_down.is_some() {
                        self.touch_down = Some(p);
                    }
                    if self.screen_on {
                        if self.root.dispatch_touch(&event) {
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                }
                TouchEvent::Cancel => {
                    if self.screen_on {
                        if self.root.dispatch_touch(&event) {
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                    self.touch_down = None;
                }
                _ => {}
            }
        }
        if self.screen_on {
            self.render_dirty(backend);
        }
    }

    /// Single frame step with reactive widget builder.
    /// Supports both zero-arg closures `|| widget` and size-aware closures `|size| widget`.
    /// When touch interaction triggers a state update (on TouchEvent::Up), the builder
    /// is automatically invoked to rebuild the widget tree before rendering.
    pub fn step_with_builder<W: Widget, M>(
        &mut self,
        backend: &mut dyn PlatformBackend,
        mut builder: impl ResponsiveBuilder<M, Output = W>,
    ) {
        let mut needs_rebuild = std::mem::take(&mut self.rebuild_requested);
        #[cfg(feature = "profile")]
        let mut event_count = 0;

        while let Some(event) = backend.poll_touch() {
            #[cfg(feature = "profile")]
            {
                event_count += 1;
            }
            match event {
                TouchEvent::HardwarePower => {
                    log::info!(
                        "[App-Event] HardwarePower toggling screen: {} -> {}",
                        self.screen_on,
                        !self.screen_on
                    );
                    self.set_screen_power(backend, !self.screen_on);
                    if self.screen_on {
                        needs_rebuild = true;
                    }
                }
                TouchEvent::Down(p) if !self.screen_on => {
                    log::info!("[App-Event] Touch Down waking screen up");
                    self.touch_down = Some(p);
                    self.set_screen_power(backend, true);
                    needs_rebuild = true;
                }
                _ if !self.screen_on => {
                    // Screen is off: ignore UI events, but app state continues running
                }
                TouchEvent::Down(p) => {
                    self.touch_down = Some(p);
                    log::debug!("[App-Event] Down at ({:.1}, {:.1})", p.x, p.y);
                    if self.root.dispatch_touch(&event) {
                        if let Some(hit_r) = self.root.hit_rect(p) {
                            let x1 = ((hit_r.x - 2.0).max(0.0) as u32 / 2 * 2) as f32;
                            let y1 = ((hit_r.y - 2.0).max(0.0) as u32 / 2 * 2) as f32;
                            let x2 = (((hit_r.right() + 2.0).min(self.size.width) as u32 + 1) / 2
                                * 2) as f32;
                            let y2 = (((hit_r.bottom() + 2.0).min(self.size.height) as u32 + 1) / 2
                                * 2) as f32;
                            let dirty_r = Rect::from_ltrb(x1, y1, x2, y2);
                            log::trace!(
                                "[App-Dirty] Touch ({:.1}, {:.1}) -> hit_r: {:?}, dirty_r: {:?}",
                                p.x,
                                p.y,
                                hit_r,
                                dirty_r
                            );
                            self.dirty.mark_dirty(dirty_r);
                        } else {
                            log::trace!(
                                "[App-Dirty] hit_rect returned None for ({:.1}, {:.1})",
                                p.x,
                                p.y
                            );
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                }
                TouchEvent::Up(p) => {
                    self.touch_down = None;
                    log::debug!("[App-Event] Up at ({:.1}, {:.1})", p.x, p.y);
                    if self.root.dispatch_touch(&event) {
                        needs_rebuild = true;
                    }
                }
                TouchEvent::Move(p) => {
                    if self.touch_down.is_some() {
                        self.touch_down = Some(p);
                    }
                    log::debug!("[App-Event] Move at ({:.1}, {:.1})", p.x, p.y);
                    if self.root.dispatch_touch(&event) {
                        if let Some(hit_r) = self.root.hit_rect(p) {
                            let x1 = ((hit_r.x - 2.0).max(0.0) as u32 / 2 * 2) as f32;
                            let y1 = ((hit_r.y - 2.0).max(0.0) as u32 / 2 * 2) as f32;
                            let x2 = (((hit_r.right() + 2.0).min(self.size.width) as u32 + 1) / 2
                                * 2) as f32;
                            let y2 = (((hit_r.bottom() + 2.0).min(self.size.height) as u32 + 1) / 2
                                * 2) as f32;
                            let dirty_r = Rect::from_ltrb(x1, y1, x2, y2);
                            log::trace!(
                                "[App-Dirty] Touch ({:.1}, {:.1}) -> hit_r: {:?}, dirty_r: {:?}",
                                p.x,
                                p.y,
                                hit_r,
                                dirty_r
                            );
                            self.dirty.mark_dirty(dirty_r);
                        } else {
                            self.dirty.mark_all_dirty(self.size);
                        }
                    }
                }
                TouchEvent::Cancel => {
                    self.touch_down = None;
                    log::debug!("[App-Event] Cancel");
                    if self.root.dispatch_touch(&event) {
                        self.dirty.mark_all_dirty(self.size);
                    }
                }
                TouchEvent::HardwareBack => {
                    log::debug!("[App-Event] HardwareBack");
                    if self.root.dispatch_touch(&event) {
                        needs_rebuild = true;
                        self.dirty.mark_all_dirty(self.size);
                    }
                }
                TouchEvent::HardwareKill => {
                    log::debug!("[App-Event] HardwareKill");
                    if self.root.dispatch_touch(&event) {
                        needs_rebuild = true;
                        self.dirty.mark_all_dirty(self.size);
                    }
                }
            }
        }

        if !self.screen_on {
            return;
        }

        if needs_rebuild || self.root.needs_rebuild() {
            #[cfg(feature = "profile")]
            let t0 = std::time::Instant::now();

            if self.touch_down.is_none() {
                self.set_root(builder.build(self.size));
                self.dirty.mark_all_dirty(self.size);
            } else {
                self.rebuild_requested = true;
            }

            #[cfg(feature = "profile")]
            log::info!("[App-Profile] Rebuild widget tree took {:?}", t0.elapsed());
        }

        #[cfg(feature = "profile")]
        let t_frame_start = std::time::Instant::now();

        self.render_dirty(backend);

        #[cfg(feature = "profile")]
        if event_count > 0 || self.is_first_frame {
            log::info!(
                "[App-Profile] Frame total (render_dirty) took {:?}",
                t_frame_start.elapsed()
            );
        }
    }

    /// Single frame step for continuously animated applications (e.g. 60fps vector animations).
    /// Dispatches touch events and always rebuilds the widget tree and marks dirty for continuous rendering.
    pub fn step_animated<W: Widget, M>(
        &mut self,
        backend: &mut dyn PlatformBackend,
        mut builder: impl ResponsiveBuilder<M, Output = W>,
    ) {
        while let Some(event) = backend.poll_touch() {
            match event {
                TouchEvent::HardwarePower => {
                    self.set_screen_power(backend, !self.screen_on);
                }
                TouchEvent::Down(p) if !self.screen_on => {
                    self.touch_down = Some(p);
                    self.set_screen_power(backend, true);
                }
                TouchEvent::Down(p) => {
                    self.touch_down = Some(p);
                    if self.screen_on {
                        self.root.dispatch_touch(&event);
                    }
                }
                TouchEvent::Up(_) => {
                    if self.screen_on {
                        self.root.dispatch_touch(&event);
                    }
                    self.touch_down = None;
                }
                TouchEvent::Move(p) => {
                    if self.touch_down.is_some() {
                        self.touch_down = Some(p);
                    }
                    if self.screen_on {
                        self.root.dispatch_touch(&event);
                    }
                }
                TouchEvent::Cancel => {
                    if self.screen_on {
                        self.root.dispatch_touch(&event);
                    }
                    self.touch_down = None;
                }
                _ if self.screen_on => {
                    self.root.dispatch_touch(&event);
                }
                _ => {}
            }
        }

        if self.screen_on {
            if self.touch_down.is_none() {
                self.set_root(builder.build(self.size));
            }
            self.dirty.mark_all_dirty(self.size);
            self.render_dirty(backend);
        }
    }

    fn render_dirty(&mut self, backend: &mut dyn PlatformBackend) {
        if !self.screen_on {
            return;
        }
        let dirty_rect = if self.is_first_frame {
            self.is_first_frame = false;
            // The forced full first frame consumes any requested dirty region.
            self.dirty.take();
            Some(Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height))
        } else {
            self.dirty.take()
        };

        if let Some(rect) = dirty_rect {
            #[cfg(feature = "profile")]
            let t_start = std::time::Instant::now();

            let constraints = BoxConstraints::tight(self.size);
            self.root.layout(&constraints);

            #[cfg(feature = "profile")]
            let t_layout = std::time::Instant::now();

            {
                let mut canvas = Canvas::new(self.pixmap.as_mut());
                canvas.save();
                canvas.clip_rect(rect);
                canvas.clear(Color::BLACK);
                self.root.paint(&mut canvas, Offset::ZERO);
                canvas.restore();
            }

            #[cfg(feature = "profile")]
            let t_paint = std::time::Instant::now();

            let (x1, y1, x2, y2) = self.pixmap.rect_bounds(tiny_gfx::Rect::from_ltwh(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            ));
            let count = (x2 - x1) as usize * (y2 - y1) as usize;
            // Whole-width rows are already tightly packed. The synchronous
            // backend may borrow them directly; cropped columns still pack.
            let pixels = if x1 == 0 && x2 == self.pixmap.width() as i32 {
                let stride = self.pixmap.width() as usize;
                &self.pixmap.data()[y1 as usize * stride..y2 as usize * stride]
            } else {
                if self.scratch_rgb565.len() < count {
                    self.scratch_rgb565.resize(count, 0);
                }
                extract_rect_to_rgb565(&self.pixmap, rect, &mut self.scratch_rgb565);
                &self.scratch_rgb565[..count]
            };

            #[cfg(feature = "profile")]
            let t_extract = std::time::Instant::now();

            log::trace!(
                "[Render-Dirty] rect: {:?}, extract: ({},{})-({},{}) count: {}",
                rect,
                x1,
                y1,
                x2,
                y2,
                count
            );

            if count > 0 {
                let actual_flush_rect = Rect::from_ltrb(x1 as f32, y1 as f32, x2 as f32, y2 as f32);
                backend.begin_frame();
                backend.flush(actual_flush_rect, pixels);
                backend.end_frame();
            }

            #[cfg(feature = "profile")]
            let t_flush = std::time::Instant::now();

            #[cfg(feature = "profile")]
            log::info!(
                "[App-Render] layout: {:?}, paint: {:?}, extract: {:?}, flush: {:?}",
                t_layout.duration_since(t_start),
                t_paint.duration_since(t_layout),
                t_extract.duration_since(t_paint),
                t_flush.duration_since(t_extract)
            );
        }
    }
}

/// Helper trait allowing `step_with_builder` and `run_simulator` to accept either
/// `|| -> Widget` (simple) or `|size| -> Widget` (screen-size aware).
pub trait ResponsiveBuilder<M> {
    type Output: Widget;
    fn build(&mut self, size: Size) -> Self::Output;
}

pub struct NoArgBuilder;
pub struct SizeArgBuilder;

impl<F, W: Widget> ResponsiveBuilder<NoArgBuilder> for F
where
    F: FnMut() -> W,
{
    type Output = W;
    fn build(&mut self, _size: Size) -> W {
        self()
    }
}

impl<F, W: Widget> ResponsiveBuilder<SizeArgBuilder> for F
where
    F: FnMut(Size) -> W,
{
    type Output = W;
    fn build(&mut self, size: Size) -> W {
        self(size)
    }
}
