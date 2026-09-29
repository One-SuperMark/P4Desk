//! Spread the app theme from its stationary icon, then shrink it toward screen center.
//! Reuses a normal desktop RGB565 backdrop during expansion, released on exit.
use crate::app_icons::get_app_icon_asset;
use crate::launcher_state::LauncherState;
use p4desk_protocol::Mode;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tiny_flutter::prelude::*;

pub const APP_LAUNCH_DURATION_MS: u64 = 940;
pub const EXPAND_END_MS: u64 = 260;
pub const REVEAL_START_MS: u64 = 340;
pub const APP_LAUNCH_FRAME_INTERVAL_MS: u64 = 16;

pub struct DesktopBackdrop {
    size: Size,
    pixels: Vec<u16>,
    erased_source: Option<Rect>,
}
pub type DesktopBackdropCache = Arc<Mutex<Option<DesktopBackdrop>>>;

#[derive(Clone, Copy)]
pub struct AppLaunchFrame {
    pub id: &'static str,
    pub source: Rect,
    pub elapsed_ms: u64,
}

#[derive(Default)]
pub struct AppLaunchState {
    id: Option<&'static str>,
    source: Rect,
    started_ms: Option<u64>,
    last_frame: Option<u64>,
    // Keep ownership after animation expiry. A finger held on the splash must
    // not release onto an app control (for example the timer's Start button).
    touch_owned: bool,
    display_hold: bool,
}
impl AppLaunchState {
    pub fn start(&mut self, id: &'static str, source: Rect, now_ms: u64) {
        self.id = Some(id);
        self.source = source;
        self.started_ms = Some(now_ms);
        self.last_frame = None;
        self.display_hold = false;
    }
    pub fn hold_for_display(&mut self) {
        self.display_hold = true;
    }
    /// Release a failed USB wait without skipping the existing expansion/reveal.
    pub fn resume_after_display_failure(&mut self, now_ms: u64) {
        if self.display_hold {
            if self.elapsed(now_ms).is_some_and(|ms| ms >= REVEAL_START_MS) {
                self.started_ms = Some(now_ms.saturating_sub(REVEAL_START_MS));
            }
            self.display_hold = false;
            self.last_frame = None;
        }
    }
    fn elapsed(&self, now_ms: u64) -> Option<u64> {
        let elapsed = now_ms.saturating_sub(self.started_ms?);
        Some(if self.display_hold {
            elapsed.min(REVEAL_START_MS)
        } else {
            elapsed
        })
    }
    pub fn cancel(&mut self) {
        self.started_ms = None;
        self.display_hold = false;
        self.last_frame = None;
    }
    pub fn frame(&self, now_ms: u64) -> Option<AppLaunchFrame> {
        let elapsed_ms = self.elapsed(now_ms)?;
        (elapsed_ms < APP_LAUNCH_DURATION_MS).then_some(AppLaunchFrame {
            id: self.id?,
            source: self.source,
            elapsed_ms,
        })
    }
    pub fn take_dirty(&mut self, now_ms: u64, size: Size) -> Option<Rect> {
        let elapsed = self.elapsed(now_ms)?;
        if elapsed >= APP_LAUNCH_DURATION_MS {
            self.cancel();
        } else {
            // All three phases move; the brief full-color bridge fades the SVG.
            // A phase boundary must repaint even within the same 16 ms bucket.
            let phase = if elapsed < EXPAND_END_MS {
                0
            } else if elapsed < REVEAL_START_MS {
                1
            } else {
                2
            };
            let frame = (phase << 56) | (elapsed / APP_LAUNCH_FRAME_INTERVAL_MS);
            if self.last_frame == Some(frame) {
                return None;
            }
            self.last_frame = Some(frame);
        }
        Some(Rect::from_ltwh(0.0, 0.0, size.width, size.height))
    }
}

fn sample(state: &LauncherState) -> Option<AppLaunchFrame> {
    if state.mode != Mode::Pad || !state.settings.screen_on {
        return None;
    }
    state
        .app_launch
        .frame(state.monotonic_ms)
        .filter(|frame| state.active_app.id() == Some(frame.id))
}

pub struct AppLaunchOverlay {
    state: Arc<Mutex<LauncherState>>,
    child: Box<dyn Widget>,
    desktop: Option<Box<dyn Widget>>,
    backdrop: DesktopBackdropCache,
    capture_desktop: bool,
}
impl AppLaunchOverlay {
    pub fn new(
        state: Arc<Mutex<LauncherState>>,
        child: Box<dyn Widget>,
        desktop: Option<Box<dyn Widget>>,
        backdrop: DesktopBackdropCache,
        capture_desktop: bool,
    ) -> Self {
        Self {
            state,
            child,
            desktop,
            backdrop,
            capture_desktop,
        }
    }
}
impl Widget for AppLaunchOverlay {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderAppLaunchOverlay {
            state: self.state.clone(),
            child: self.child.create_render_object(),
            desktop: self.desktop.as_ref().map(|w| w.create_render_object()),
            backdrop: self.backdrop.clone(),
            capture_desktop: self.capture_desktop,
            desktop_touch_active: AtomicBool::new(false),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}
struct RenderAppLaunchOverlay {
    state: Arc<Mutex<LauncherState>>,
    child: Box<dyn RenderBox>,
    desktop: Option<Box<dyn RenderBox>>,
    backdrop: DesktopBackdropCache,
    capture_desktop: bool,
    desktop_touch_active: AtomicBool,
    size: Size,
    offset: Offset,
}
impl RenderAppLaunchOverlay {
    fn bounds(&self) -> Rect {
        Rect::from_ltwh(0.0, 0.0, self.size.width, self.size.height)
    }
    fn intercepting(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.app_launch.touch_owned || sample(&state).is_some()
    }
    fn paint_cached_desktop(&self, canvas: &mut Canvas, offset: Offset, source: Rect) -> bool {
        let mut cache = self.backdrop.lock().unwrap();
        if let Some(image) = cache.as_mut().filter(|image| image.size == self.size) {
            if image.erased_source != Some(source) {
                // Patch the existing backdrop once, before the stationary SVG is
                // drawn. The cache belongs to this launch and is freed at exit.
                erase_source_icon(
                    &mut Canvas::new(tiny_gfx::Pixmap565Mut::new(
                        &mut image.pixels,
                        self.size.width as u32,
                        self.size.height as u32,
                    )),
                    self.size,
                    source,
                );
                image.erased_source = Some(source);
            }
            canvas.blit_image_565(
                offset.dx.round() as i32,
                offset.dy.round() as i32,
                self.size.width as u32,
                self.size.height as u32,
                &image.pixels,
            );
            true
        } else {
            false
        }
    }
    fn capture(&self, canvas: &Canvas, offset: Offset) {
        // Only cache a complete, normal desktop. Small press updates must not
        // replace it with a pressed icon, and clipped frames are incomplete.
        if !self.capture_desktop
            || self.desktop_touch_active.load(Ordering::Relaxed)
            || offset != Offset::ZERO
            || self.size.width as u32 != canvas.width()
            || self.size.height as u32 != canvas.height()
            || canvas.current_clip().is_some_and(|clip| {
                clip.x > 0.0
                    || clip.y > 0.0
                    || clip.right() < self.size.width
                    || clip.bottom() < self.size.height
            })
        {
            return;
        }
        {
            let state = self.state.lock().unwrap();
            if state.mode != Mode::Pad
                || !state.settings.screen_on
                || state.active_app.id().is_some()
                || state.status_panel_open
            {
                return;
            }
        }
        let mut cache = self.backdrop.lock().unwrap();
        let count = canvas.pixels_rgb565().len();
        if cache.as_ref().is_none_or(|image| image.size != self.size) {
            let mut pixels = Vec::new();
            if pixels.try_reserve_exact(count).is_err() {
                return; // Rendering remains functional under memory pressure.
            }
            pixels.resize(count, 0);
            *cache = Some(DesktopBackdrop {
                size: self.size,
                pixels,
                erased_source: None,
            });
        }
        let image = cache.as_mut().unwrap();
        image.pixels.copy_from_slice(canvas.pixels_rgb565());
        image.erased_source = None;
    }
}
impl RenderBox for RenderAppLaunchOverlay {
    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = self.child.layout(constraints);
        self.child.set_offset(Offset::ZERO);
        if let Some(desktop) = &mut self.desktop {
            desktop.layout(&BoxConstraints::tight(self.size));
            desktop.set_offset(Offset::ZERO);
        }
        self.size
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let animation = sample(&self.state.lock().unwrap());
        let mut cached_desktop = false;
        match animation {
            Some(frame) if frame.elapsed_ms < EXPAND_END_MS => {
                cached_desktop = self.paint_cached_desktop(canvas, offset, frame.source);
                if cached_desktop {
                    // Reuse the last complete desktop; no vector/gradient work.
                } else if let Some(desktop) = &self.desktop {
                    // If allocation failed, skip the opaque square inscribed in
                    // the circle. Keep the narrow refractive rim's source intact.
                    if frame.elapsed_ms == 0 {
                        desktop.paint(canvas, offset);
                    } else {
                        let (center, radius, rim) = circle_geometry(self.size, frame);
                        let half = (radius - rim - 3.0).max(0.0) * std::f32::consts::FRAC_1_SQRT_2;
                        let core = Rect::from_ltwh(
                            center.x - half,
                            center.y - half,
                            half * 2.0,
                            half * 2.0,
                        );
                        // Use disjoint integer clips. Fractional strip edges
                        // can round into the same row and double-blend wallpaper
                        // circles/text where top/bottom meet the side strips.
                        let interior = Rect::from_ltrb(
                            core.x.ceil(),
                            core.y.ceil(),
                            core.right().floor(),
                            core.bottom().floor(),
                        );
                        let rectangles = [
                            Rect::from_ltrb(0.0, 0.0, self.size.width, interior.y),
                            Rect::from_ltrb(
                                0.0,
                                interior.bottom(),
                                self.size.width,
                                self.size.height,
                            ),
                            Rect::from_ltrb(0.0, interior.y, interior.x, interior.bottom()),
                            Rect::from_ltrb(
                                interior.right(),
                                interior.y,
                                self.size.width,
                                interior.bottom(),
                            ),
                        ];
                        canvas.save();
                        canvas.translate(offset.dx, offset.dy);
                        for exposed in rectangles {
                            if exposed.width > 0.0 && exposed.height > 0.0 {
                                canvas.save();
                                canvas.clip_rect(exposed);
                                desktop.paint(canvas, Offset::ZERO);
                                canvas.restore();
                            }
                        }
                        canvas.restore();
                    }
                } else {
                    self.child.paint(canvas, offset);
                }
            }
            Some(frame) if frame.elapsed_ms <= REVEAL_START_MS => (),
            _ => self.child.paint(canvas, offset),
        }
        if let Some(frame) = animation {
            canvas.save();
            canvas.translate(offset.dx, offset.dy);
            canvas.clip_rect(self.bounds());
            if frame.elapsed_ms < EXPAND_END_MS && !cached_desktop {
                erase_source_icon(canvas, self.size, frame.source);
            }
            paint_launch(canvas, self.size, frame);
            canvas.restore();
        } else {
            self.capture(canvas, offset);
        }
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
    fn hit_test(&self, point: Point) -> bool {
        if self.intercepting() {
            self.bounds().contains(point)
        } else {
            self.child.hit_test(point)
        }
    }
    fn hit_rect(&self, point: Point) -> Option<Rect> {
        if self.intercepting() {
            self.bounds().contains(point).then(|| self.bounds())
        } else {
            self.child.hit_rect(point)
        }
    }
    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        if !self.intercepting() {
            self.child.set_pressed_at(point, pressed);
        }
    }
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        if self.capture_desktop {
            match event {
                TouchEvent::Down(_) => self.desktop_touch_active.store(true, Ordering::Relaxed),
                TouchEvent::Up(_) | TouchEvent::Cancel => {
                    self.desktop_touch_active.store(false, Ordering::Relaxed)
                }
                _ => (),
            }
        }
        let mut state = self.state.lock().unwrap();
        let active = sample(&state).is_some();
        let owned = state.app_launch.touch_owned;
        let handled = match *event {
            TouchEvent::Down(point) if active && self.bounds().contains(point) => {
                state.app_launch.touch_owned = true;
                true
            }
            TouchEvent::Up(_) | TouchEvent::Cancel if owned => {
                state.app_launch.touch_owned = false;
                true
            }
            TouchEvent::Move(_) if owned => true,
            TouchEvent::Up(_) | TouchEvent::Move(_) | TouchEvent::Cancel if active => true,
            _ => false,
        };
        drop(state);
        if handled {
            self.child.dispatch_touch(&TouchEvent::Cancel);
            true
        } else {
            self.child.dispatch_touch(event)
        }
    }
    fn needs_rebuild(&self) -> bool {
        self.child.needs_rebuild()
    }
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
/// Launcher colors follow the desktop SVG palette; timer keeps its accepted coral.
fn theme_color(id: &str) -> Color {
    Color::from_hex(match id {
        "clock" => 0x626bd8,
        "timer" => 0xf18b77,
        "calculator" => 0x3faa80,
        "notes" => 0xedb247,
        "mac" => 0x409bdb,
        "settings" => 0x687c9c,
        "display" => 0x509faf,
        _ => 0x409bdb,
    })
}
fn circle_geometry(size: Size, frame: AppLaunchFrame) -> (Point, f32, f32) {
    let scale = (size.width / 1024.0).min(size.height / 600.0);
    let revealing = frame.elapsed_ms >= REVEAL_START_MS;
    let center = if revealing {
        Point::new(size.width * 0.5, size.height * 0.5)
    } else {
        Point::new(
            frame.source.x + frame.source.width * 0.5,
            frame.source.y + frame.source.height * 0.5,
        )
    };
    // Expansion begins off-center. The farthest corner, not half the screen's
    // diagonal, determines the radius needed to hide the entire desktop.
    let dx = center.x.max(size.width - center.x);
    let dy = center.y.max(size.height - center.y);
    let max_radius = (dx * dx + dy * dy).sqrt() + 14.0 * scale;
    let (start, progress) = if revealing {
        (
            0.0,
            smooth(
                (frame.elapsed_ms - REVEAL_START_MS) as f32
                    / (APP_LAUNCH_DURATION_MS - REVEAL_START_MS) as f32,
            ),
        )
    } else {
        (
            frame.source.width * 0.4,
            smooth(frame.elapsed_ms as f32 / EXPAND_END_MS as f32),
        )
    };
    let radius = if revealing {
        max_radius * (1.0 - progress)
    } else {
        lerp(start, max_radius, progress)
    };
    (center, radius, 8.0 * scale)
}
fn erase_source_icon(canvas: &mut Canvas, size: Size, source: Rect) {
    // The recorded source is the 94% pressed SVG. Restore only its unpressed
    // bounds from the procedural wallpaper, so its normal-size edge cannot peek around the pressed SVG.
    // Labels and neighboring icons are outside this patch. No second frame cache.
    let padding = source.width * (1.0 / 0.94 - 1.0) * 0.5 + 1.0;
    canvas.save();
    canvas.clip_rect(Rect::from_ltrb(
        (source.x - padding).floor(),
        (source.y - padding).floor(),
        (source.right() + padding).ceil(),
        (source.bottom() + padding).ceil(),
    ));
    crate::widgets::WallpaperPainter.paint(canvas, size);
    canvas.restore();
}
fn paint_launch(canvas: &mut Canvas, size: Size, frame: AppLaunchFrame) {
    use tiny_gfx::GlassFill;
    let theme = theme_color(frame.id);
    let (circle_center, radius, rim) = circle_geometry(size, frame);
    if frame.elapsed_ms < EXPAND_END_MS {
        if radius > 0.0 {
            canvas.glass_circle(circle_center, radius, theme, GlassFill::Inside, rim, 1.0);
        }
    } else if frame.elapsed_ms < REVEAL_START_MS {
        canvas.draw_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height), theme);
    } else {
        // Keep the theme inside a shrinking disc: the app appears at the
        // outside edges first, and the final color disappears at screen center.
        // Drop the subpixel disc rather than leave a refracted center speck.
        if radius > rim / 8.0 {
            canvas.glass_circle(circle_center, radius, theme, GlassFill::Inside, rim, 1.0);
        }
        return; // Never leave the SVG over the revealed application.
    }
    let opacity = 1.0
        - smooth(
            frame.elapsed_ms.saturating_sub(EXPAND_END_MS) as f32
                / (REVEAL_START_MS - EXPAND_END_MS) as f32,
        );
    if let Some(icon) = get_app_icon_asset(frame.id) {
        icon.paint_with_opacity(canvas, frame.source, Color::WHITE, opacity);
    }
}

/// Called only on a display owner's exclusively owned BUILDING JPEG buffer.
/// Symmetric about screen center, so the existing physical 180-degree rotation
/// needs no special case. Reuses the exact Pad circle/refraction compositor.
pub fn paint_usb_display_reveal(
    canvas: &mut Canvas,
    size: Size,
    elapsed_ms: u32,
    duration_ms: u32,
) {
    if duration_ms == 0 || elapsed_ms >= duration_ms {
        return;
    }
    let scale = (size.width / 1024.0).min(size.height / 600.0);
    let center = Point::new(size.width * 0.5, size.height * 0.5);
    let radius = (center.x.hypot(center.y) + 14.0 * scale)
        * (1.0 - smooth(elapsed_ms as f32 / duration_ms as f32));
    if radius > scale {
        canvas.glass_circle(
            center,
            radius,
            theme_color("display"),
            tiny_gfx::GlassFill::Inside,
            8.0 * scale,
            1.0,
        );
    }
}

#[cfg(test)]
mod occlusion_tests {
    use super::*;

    #[test]
    fn circle_starts_at_stationary_icon_and_shrinks_inward_without_white_rim() {
        let size = Size::new(1024.0, 600.0);
        let background = Color::from_hex(0x182434).to_rgb565();
        for (id, x, y) in [
            ("clock", 151.0, 177.0),
            ("timer", 392.0, 177.0),
            ("notes", 632.0, 177.0),
            ("calculator", 873.0, 177.0),
            ("mac", 151.0, 403.0),
            ("settings", 392.0, 403.0),
            ("display", 632.0, 403.0),
        ] {
            let theme = theme_color(id).to_rgb565();
            let render = |elapsed_ms| {
                let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
                pixels.fill(background);
                paint_launch(
                    &mut Canvas::new(pixels.as_mut()),
                    size,
                    AppLaunchFrame {
                        id,
                        source: Rect::from_ltwh(x - 68.62, y - 68.62, 137.24, 137.24),
                        elapsed_ms,
                    },
                );
                pixels
            };
            let initial = render(0);
            for ms in [80, 160, EXPAND_END_MS - 1] {
                let frame = render(ms);
                // Opaque SVG interior keeps its exact geometry and position.
                for row in y as usize - 20..y as usize + 20 {
                    assert_eq!(
                        &initial.data()[row * 1024 + x as usize - 20..row * 1024 + x as usize + 20],
                        &frame.data()[row * 1024 + x as usize - 20..row * 1024 + x as usize + 20],
                        "SVG moved or resized for {id}"
                    );
                }
            }
            let early = render(70);
            let sample_y = if y < 300.0 {
                y as usize + 100
            } else {
                y as usize - 100
            };
            assert_eq!(
                early.data()[sample_y * 1024 + x as usize],
                theme,
                "spread must originate at clicked icon"
            );
            let far_x = if x < 512.0 { 1023 } else { 0 };
            assert_eq!(
                early.data()[far_x],
                background,
                "far side revealed too early"
            );
            for ms in [EXPAND_END_MS - 1, EXPAND_END_MS] {
                let covered = render(ms);
                for index in [0, 1023, 599 * 1024, 600 * 1024 - 1] {
                    assert_eq!(
                        covered.data()[index],
                        theme,
                        "off-center circle misses corner for {id}"
                    );
                }
            }
            assert!(
                render(REVEAL_START_MS).data().iter().all(|p| *p == theme),
                "SVG must vanish before reveal"
            );
            let reveal = render((REVEAL_START_MS + APP_LAUNCH_DURATION_MS) / 2);
            assert_eq!(
                reveal.data()[300 * 1024 + 512],
                theme,
                "theme must remain at center during inward shrink"
            );
            assert_eq!(
                reveal.data()[0],
                background,
                "app must appear at the outside edges first"
            );
            for pixel in reveal.data() {
                for (shift, mask) in [(11, 31), (5, 63), (0, 31)] {
                    assert!(
                        (pixel >> shift) & mask
                            <= ((theme >> shift) & mask).max((background >> shift) & mask)
                    );
                }
            }
            assert!(
                render(APP_LAUNCH_DURATION_MS - 1)
                    .data()
                    .iter()
                    .all(|p| *p == background),
                "last aperture leaves edge pixels"
            );
        }
    }

    #[test]
    fn launch_source_patch_reuses_the_cache_allocation_and_is_only_applied_once() {
        let size = Size::new(1024.0, 600.0);
        let state = Arc::new(Mutex::new(LauncherState::new()));
        let cache = state.lock().unwrap().desktop_backdrop.clone();
        let mut desktop = crate::build_launcher_ui(state.clone(), size).create_render_object();
        desktop.layout(&BoxConstraints::tight(size));
        let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
        desktop.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        let allocation = cache.lock().unwrap().as_ref().unwrap().pixels.as_ptr();
        let source = Rect::from_ltwh(323.38, 108.38, 137.24, 137.24);
        state.lock().unwrap().launch_app("timer", source);
        let mut launch = crate::build_launcher_ui(state.clone(), size).create_render_object();
        launch.layout(&BoxConstraints::tight(size));
        launch.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        let patched = cache.lock().unwrap().as_ref().unwrap().pixels.clone();
        for ms in [80, 160, EXPAND_END_MS - 1] {
            state.lock().unwrap().tick(ms, 0);
            launch.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
            let c = cache.lock().unwrap();
            let image = c.as_ref().unwrap();
            assert_eq!(image.pixels.as_ptr(), allocation);
            assert_eq!(image.erased_source, Some(source));
            assert_eq!(image.pixels, patched);
        }
    }

    #[test]
    fn normal_backdrop_is_bounded_reused_and_never_captures_a_pressed_full_frame() {
        let size = Size::new(1024.0, 600.0);
        let state = Arc::new(Mutex::new(LauncherState::new()));
        let cache = state.lock().unwrap().desktop_backdrop.clone();
        let mut desktop = crate::build_launcher_ui(state, size).create_render_object();
        desktop.layout(&BoxConstraints::tight(size));
        let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
        desktop.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        let normal = pixels.data().to_vec();
        let allocation = {
            let c = cache.lock().unwrap();
            let image = c.as_ref().unwrap();
            assert_eq!(image.pixels.len() * 2, 1_228_800);
            assert_eq!(image.pixels, normal);
            image.pixels.as_ptr()
        };
        desktop.dispatch_touch(&TouchEvent::Down(Point::new(392.0, 177.0)));
        desktop.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        assert_ne!(pixels.data(), normal);
        assert_eq!(cache.lock().unwrap().as_ref().unwrap().pixels, normal);
        desktop.dispatch_touch(&TouchEvent::Cancel);
        desktop.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        assert_eq!(pixels.data(), normal);
        let c = cache.lock().unwrap();
        assert_eq!(c.as_ref().unwrap().pixels.as_ptr(), allocation);
        assert_eq!(c.as_ref().unwrap().pixels, normal);
    }

    #[test]
    fn cached_desktop_matches_full_vector_backdrop_at_every_app_position() {
        let size = Size::new(1024.0, 600.0);
        for (id, x, y) in [
            ("clock", 151.0, 177.0),
            ("timer", 392.0, 177.0),
            ("notes", 632.0, 177.0),
            ("calculator", 873.0, 177.0),
            ("mac", 151.0, 403.0),
            ("settings", 392.0, 403.0),
            ("display", 632.0, 403.0),
        ] {
            let state = Arc::new(Mutex::new(LauncherState::new()));
            let mut desktop = crate::build_launcher_ui(state.clone(), size).create_render_object();
            desktop.layout(&BoxConstraints::tight(size));
            let mut normal = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
            desktop.paint(&mut Canvas::new(normal.as_mut()), Offset::ZERO);
            state
                .lock()
                .unwrap()
                .launch_app(id, Rect::from_ltwh(x - 68.62, y - 68.62, 137.24, 137.24));
            let mut live = crate::build_launcher_ui(state.clone(), size).create_render_object();
            live.layout(&BoxConstraints::tight(size));
            for ms in [0, 40, 80, 120, 160, 200, 240, EXPAND_END_MS - 1] {
                let frame = {
                    let mut s = state.lock().unwrap();
                    s.tick(ms, 0);
                    s.app_launch.frame(ms).unwrap()
                };
                let mut expected = normal.clone();
                erase_source_icon(&mut Canvas::new(expected.as_mut()), size, frame.source);
                paint_launch(&mut Canvas::new(expected.as_mut()), size, frame);
                let mut actual = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
                live.paint(&mut Canvas::new(actual.as_mut()), Offset::ZERO);
                assert_eq!(
                    actual.data(),
                    expected.data(),
                    "cached backdrop differs for {id} at {ms} ms"
                );
            }
        }
    }

    #[test]
    fn uncached_circular_occlusion_matches_full_backdrop_for_every_app_position() {
        let size = Size::new(1024.0, 600.0);
        for (id, x, y) in [
            ("clock", 151.0, 177.0),
            ("timer", 392.0, 177.0),
            ("notes", 632.0, 177.0),
            ("calculator", 873.0, 177.0),
            ("mac", 151.0, 403.0),
            ("settings", 392.0, 403.0),
            ("display", 632.0, 403.0),
        ] {
            let state = Arc::new(Mutex::new(LauncherState::new()));
            let mut desktop =
                crate::launcher_ui::build_desktop(state.clone(), size).create_render_object();
            desktop.layout(&BoxConstraints::tight(size));
            state
                .lock()
                .unwrap()
                .launch_app(id, Rect::from_ltwh(x - 68.62, y - 68.62, 137.24, 137.24));
            let mut live = crate::build_launcher_ui(state.clone(), size).create_render_object();
            live.layout(&BoxConstraints::tight(size));
            for ms in [0, 40, 80, 120, 160, 200, 240, EXPAND_END_MS - 1] {
                let frame = {
                    let mut s = state.lock().unwrap();
                    s.tick(ms, 0);
                    s.app_launch.frame(ms).unwrap()
                };
                let mut expected = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
                let mut actual = expected.clone();
                let mut canvas = Canvas::new(expected.as_mut());
                desktop.paint(&mut canvas, Offset::ZERO);
                erase_source_icon(&mut canvas, size, frame.source);
                paint_launch(&mut canvas, size, frame);
                live.paint(&mut Canvas::new(actual.as_mut()), Offset::ZERO);
                assert_eq!(
                    actual.data(),
                    expected.data(),
                    "clipped backdrop differs for {id} at {ms} ms"
                );
            }
        }
    }
}
