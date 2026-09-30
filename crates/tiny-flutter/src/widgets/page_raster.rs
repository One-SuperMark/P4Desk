//! Temporary transparent rendering cache. Original widgets remain authoritative
//! at rest; cached pixels are only composited while dragging/settling pages.
use crate::prelude::*;
use std::cell::Cell;
use tiny_gfx::raster::ImageSpan565 as Span;
thread_local! { static CAPTURING: Cell<bool> = const { Cell::new(false) }; }
pub fn is_page_snapshot() -> bool {
    CAPTURING.with(Cell::get)
}
struct SnapshotScope(bool);
impl SnapshotScope {
    fn enter() -> Self {
        Self(CAPTURING.with(|c| c.replace(true)))
    }
}
impl Drop for SnapshotScope {
    fn drop(&mut self) {
        CAPTURING.with(|c| c.set(self.0));
    }
}
const BUDGET: usize = 4 * 1024 * 1024;
struct Layer {
    rgb: Vec<u16>,
    alpha: Vec<u8>,
    spans: Vec<Span>,
}
struct Set {
    key: u64,
    size: Size,
    layers: Vec<Layer>,
    bytes: usize,
}
#[derive(Default)]
pub(super) struct PageRasterCache {
    sets: Vec<Set>,
}
impl PageRasterCache {
    pub fn contains(&self, key: u64, size: Size) -> bool {
        self.sets.iter().any(|s| s.key == key && s.size == size)
    }
    pub fn prepare(&mut self, key: u64, size: Size, pages: &[Box<dyn RenderBox>]) {
        if self
            .sets
            .iter()
            .any(|s| s.key == key && s.size == size && s.layers.len() == pages.len())
        {
            return;
        }
        let (w, h) = (size.width as u32, size.height as u32);
        let worst_pixels = w as usize * h as usize * 3 * pages.len();
        if worst_pixels > BUDGET || w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
            return;
        }
        let mut layers = Vec::new();
        for page in pages {
            let Some(mut black) = tiny_gfx::Pixmap565::new(w, h) else {
                return;
            };
            let Some(mut white) = tiny_gfx::Pixmap565::new(w, h) else {
                return;
            };
            white.fill(0xffff);
            {
                let _scope = SnapshotScope::enter();
                page.paint(&mut Canvas::new(black.as_mut()), Offset::ZERO);
                page.paint(&mut Canvas::new(white.as_mut()), Offset::ZERO);
            }
            let mut alpha = Vec::new();
            let visible = black
                .data()
                .iter()
                .zip(white.data())
                .filter(|(b, w)| **b != 0 || **w != 0xffff)
                .count();
            if alpha.try_reserve_exact(visible).is_err() {
                return;
            }
            let mut rgb = Vec::new();
            if rgb.try_reserve_exact(visible).is_err() {
                return;
            }
            let mut spans: Vec<Span> = Vec::new();
            for (index, (b, white_pixel)) in black.data().iter().zip(white.data()).enumerate() {
                if *b == 0 && *white_pixel == 0xffff {
                    continue;
                }
                let (br, bg, bb) = tiny_gfx::rgb565_to_rgb888(*b);
                let (wr, wg, wb) = tiny_gfx::rgb565_to_rgb888(*white_pixel);
                let a = 255u32.saturating_sub(
                    ((wr.saturating_sub(br) as u32)
                        + (wg.saturating_sub(bg) as u32) * 2
                        + wb.saturating_sub(bb) as u32
                        + 2)
                        / 4,
                );
                if a == 0 {
                    continue;
                }
                let x = (index % w as usize) as u16;
                let y = (index / w as usize) as u16;
                let opaque = a == 255;
                if let Some(last) = spans
                    .last_mut()
                    .filter(|s| s.y == y && s.x + s.length == x && s.opaque == opaque)
                {
                    last.length += 1;
                } else {
                    if spans.len() == spans.capacity() && spans.try_reserve(128).is_err() {
                        return;
                    }
                    spans.push(Span {
                        start: rgb.len() as u32,
                        x,
                        y,
                        length: 1,
                        opaque,
                    });
                }
                alpha.push(a as u8);
                let unpremul = |c: u8| {
                    if a == 0 {
                        0
                    } else {
                        (c as u32 * 255 / a).min(255) as u8
                    }
                };
                rgb.push(tiny_gfx::rgb888_to_rgb565(
                    unpremul(br),
                    unpremul(bg),
                    unpremul(bb),
                ));
            }
            layers.push(Layer { rgb, alpha, spans });
        }
        let bytes = layers
            .iter()
            .map(|l| {
                l.rgb.capacity() * 2
                    + l.alpha.capacity()
                    + l.spans.capacity() * std::mem::size_of::<Span>()
            })
            .sum::<usize>();
        if bytes > BUDGET {
            return;
        }
        while self.sets.iter().map(|s| s.bytes).sum::<usize>() + bytes > BUDGET {
            self.sets.remove(0);
        }
        self.sets.push(Set {
            key,
            size,
            layers,
            bytes,
        });
    }
    pub fn paint(
        &self,
        key: u64,
        size: Size,
        page: usize,
        canvas: &mut Canvas,
        offset: Offset,
    ) -> bool {
        let Some(layer) = self
            .sets
            .iter()
            .find(|s| s.key == key && s.size == size)
            .and_then(|s| s.layers.get(page))
        else {
            return false;
        };
        let x = offset.dx.round() as i32;
        let y = offset.dy.round() as i32;
        canvas.blit_image_565_spans(x, y, &layer.rgb, &layer.alpha, &layer.spans);
        true
    }
}
