//! One bounded opaque snapshot for static scroll content. No cache is used unless
//! the caller supplies a revision covering all content and a uniform background.
use crate::prelude::*;
use std::{cell::Cell, time::Instant};

const BUDGET: usize = 2 * 1024 * 1024;

#[derive(Default, Clone, Copy)]
pub struct ScrollMetrics {
    pub builds: u64,
    pub build_us: u64,
    pub paints: u64,
    pub blit_us: u64,
    pub blit_max_us: u64,
}
thread_local! { static METRICS: Cell<ScrollMetrics> = Cell::new(ScrollMetrics::default()); }
pub fn take_scroll_metrics() -> ScrollMetrics {
    METRICS.with(|m| m.replace(ScrollMetrics::default()))
}

#[derive(Clone, Copy, PartialEq)]
struct Key {
    revision: u64,
    width: u32,
    height: u32,
    background: Color,
}
#[derive(Default)]
pub(super) struct ScrollRaster {
    attempted: Option<Key>,
    pixels: Option<tiny_gfx::Pixmap565>,
}
impl ScrollRaster {
    pub fn paint(
        &mut self,
        child: &dyn RenderBox,
        revision: u64,
        background: Color,
        canvas: &mut Canvas,
        offset: Offset,
    ) -> bool {
        let size = child.size();
        if !size.width.is_finite() || !size.height.is_finite() || background.a != 255 {
            return false;
        }
        let key = Key {
            revision,
            width: size.width.ceil().max(0.0) as u32,
            height: size.height.ceil().max(0.0) as u32,
            background,
        };
        if self.attempted != Some(key) {
            // Drop the old allocation first so changing appearance cannot double
            // this cache's peak storage. Failed allocations use live painting.
            self.pixels = None;
            self.attempted = Some(key);
            if let Some(bytes) = (key.width as usize)
                .checked_mul(key.height as usize)
                .and_then(|n| n.checked_mul(2))
                .filter(|n| *n <= BUDGET)
            {
                if bytes > 0 {
                    if let Some(mut pixels) = tiny_gfx::Pixmap565::new(key.width, key.height) {
                        let start = Instant::now();
                        pixels.fill(background.to_rgb565());
                        child.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
                        self.pixels = Some(pixels);
                        METRICS.with(|m| {
                            let mut value = m.get();
                            value.builds += 1;
                            value.build_us += start.elapsed().as_micros() as u64;
                            m.set(value);
                        });
                    }
                }
            }
        }
        let Some(pixels) = &self.pixels else {
            return false;
        };
        let start = Instant::now();
        canvas.blit_image_565(
            offset.dx.round() as i32,
            offset.dy.round() as i32,
            pixels.width(),
            pixels.height(),
            pixels.data(),
        );
        METRICS.with(|m| {
            let mut value = m.get();
            let elapsed = start.elapsed().as_micros() as u64;
            value.paints += 1;
            value.blit_us += elapsed;
            value.blit_max_us = value.blit_max_us.max(elapsed);
            m.set(value);
        });
        true
    }
}
