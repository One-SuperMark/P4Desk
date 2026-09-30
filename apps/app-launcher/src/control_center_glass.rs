//! Capture the actual desktop once per modal opening. Blur includes icons and
//! cards; it never changes the background source of ordinary application pages.
use std::sync::{Arc, Mutex};
use tiny_flutter::graphics::canvas::BlurredBackdrop;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::{Folio, GlassMaterial};

pub(crate) struct Capture {
    size: Size,
    light: bool,
    pixels: Vec<u16>,
    blurred: Option<BlurredBackdrop>,
}
pub(crate) type Backdrop = Arc<Mutex<Option<Capture>>>;

pub(crate) struct FrozenDesktop {
    pub child: Box<dyn Widget>,
    pub cache: Backdrop,
}
impl Widget for FrozenDesktop {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderFrozen {
            child: self.child.create_render_object(),
            cache: self.cache.clone(),
            size: Size::ZERO,
            offset: Offset::ZERO,
        })
    }
}
struct RenderFrozen {
    child: Box<dyn RenderBox>,
    cache: Backdrop,
    size: Size,
    offset: Offset,
}
impl RenderBox for RenderFrozen {
    fn layout(&mut self, c: &BoxConstraints) -> Size {
        self.size = self.child.layout(c);
        self.size
    }
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, o: Offset) {
        self.offset = o;
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(c) = cache
            .as_ref()
            .filter(|c| c.size == self.size && c.light == Folio::is_light())
        {
            canvas.blit_image_565(
                offset.dx as i32,
                offset.dy as i32,
                c.size.width as u32,
                c.size.height as u32,
                &c.pixels,
            );
            return;
        }
        self.child.paint(canvas, offset);
        if offset != Offset::ZERO
            || canvas.width() != self.size.width as u32
            || canvas.height() != self.size.height as u32
            || canvas.current_clip().is_some_and(|r| {
                r.x > 0.0
                    || r.y > 0.0
                    || r.right() < self.size.width
                    || r.bottom() < self.size.height
            })
        {
            return;
        }
        let mut pixels = Vec::new();
        if pixels
            .try_reserve_exact(canvas.pixels_rgb565().len())
            .is_err()
        {
            return;
        }
        pixels.extend_from_slice(canvas.pixels_rgb565());
        *cache = Some(Capture {
            size: self.size,
            light: Folio::is_light(),
            pixels,
            blurred: canvas.capture_blurred_backdrop(),
        });
    }
    // The modal overlay owns input. Its frozen desktop must never activate apps.
}

pub(crate) struct Panel(pub Backdrop);
impl CustomPainter for Panel {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let rect =
            RRect::from_rect_circular(Rect::from_ltwh(0.0, 0.0, size.width, size.height), 28.0);
        let cache = self.0.lock().unwrap();
        if let Some(source) = cache.as_ref().and_then(|c| c.blurred.as_ref()) {
            canvas.frosted_backdrop(rect, Folio::surface(), source);
        } else {
            canvas.liquid_glass_material(rect, Folio::surface(), false, GlassMaterial::DesktopCard);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::HeadlessBackend;
    use crate::status_bar::StatusPanelKind;
    use crate::{build_launcher_ui, LauncherState};

    #[test]
    fn modal_capture_is_reused_then_released_and_recaptured_on_the_next_page() {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        let size = Size::new(1024.0, 600.0);
        let render = || {
            let mut backend = HeadlessBackend::new(1024, 600);
            App::new(build_launcher_ui(state.clone(), size), size).step(&mut backend);
        };
        {
            let mut s = state.lock().unwrap();
            s.status_panel_open = true;
            s.status_panel_kind = StatusPanelKind::Control;
        }
        render();
        let cache = state.lock().unwrap().control_center_backdrop.clone();
        let original = cache.lock().unwrap().as_ref().unwrap().pixels.clone();
        let pointer = cache.lock().unwrap().as_ref().unwrap().pixels.as_ptr();
        state.lock().unwrap().tick(123_000, 1_790_712_418_000);
        render();
        assert_eq!(
            cache.lock().unwrap().as_ref().unwrap().pixels.as_ptr(),
            pointer
        );
        assert_eq!(cache.lock().unwrap().as_ref().unwrap().pixels, original);
        state.lock().unwrap().status_panel_open = false;
        render();
        assert!(cache.lock().unwrap().is_none());
        {
            let mut s = state.lock().unwrap();
            s.page_controller.set_page(1);
            s.status_panel_open = true;
        }
        render();
        assert_ne!(cache.lock().unwrap().as_ref().unwrap().pixels, original);
        assert!(cache.lock().unwrap().as_ref().unwrap().blurred.is_some());
    }
}
