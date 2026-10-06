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
impl Layer {
    fn bytes(&self) -> usize {
        self.rgb.capacity() * 2
            + self.alpha.capacity()
            + self.spans.capacity() * std::mem::size_of::<Span>()
    }
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
    #[cfg(test)]
    build_peak_bytes: usize,
}
impl PageRasterCache {
    pub fn contains(&self, key: u64, size: Size) -> bool {
        self.sets.iter().any(|s| s.key == key && s.size == size)
    }
    pub fn prepare(&mut self, key: u64, size: Size, pages: &[Box<dyn RenderBox>]) {
        self.prepare_with_budget(key, size, pages, BUDGET);
    }
    fn note_build_bytes(&mut self, bytes: usize, budget: usize) {
        let bytes = bytes + self.sets.iter().map(|s| s.bytes).sum::<usize>();
        debug_assert!(bytes <= budget);
        #[cfg(test)]
        {
            self.build_peak_bytes = self.build_peak_bytes.max(bytes);
        }
    }
    fn prepare_with_budget(
        &mut self,
        key: u64,
        size: Size,
        pages: &[Box<dyn RenderBox>],
        budget: usize,
    ) {
        if self
            .sets
            .iter()
            .any(|s| s.key == key && s.size == size && s.layers.len() == pages.len())
        {
            return;
        }
        if !size.width.is_finite() || !size.height.is_finite() {
            return;
        }
        let (w, h) = (size.width as u32, size.height as u32);
        if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
            return;
        }
        let Some(pixels) = (w as usize).checked_mul(h as usize) else {
            return;
        };
        let Some(worst_pixels) = pixels
            .checked_mul(3)
            .and_then(|n| n.checked_mul(pages.len()))
        else {
            return;
        };
        let Some(scratch_bytes) = pixels.checked_mul(4) else {
            return;
        };
        if worst_pixels > budget || scratch_bytes > budget {
            return;
        }
        // A changed theme must release its old pages before constructing the
        // replacement. Black/white capture buffers and spans share this cache's
        // budget too; failure leaves authoritative widgets available to paint.
        self.sets.clear();
        let mut layers = Vec::new();
        let mut built_bytes = 0;
        for page in pages {
            if built_bytes + scratch_bytes > budget {
                return;
            }
            let Some(mut black) = tiny_gfx::Pixmap565::new(w, h) else {
                return;
            };
            let Some(mut white) = tiny_gfx::Pixmap565::new(w, h) else {
                return;
            };
            self.note_build_bytes(built_bytes + scratch_bytes, budget);
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
            if visible > (budget - built_bytes - scratch_bytes) / 3 {
                return;
            }
            if alpha.try_reserve_exact(visible).is_err() {
                return;
            }
            let mut rgb = Vec::new();
            if rgb.try_reserve_exact(visible).is_err() {
                return;
            }
            let pixel_bytes = rgb.capacity() * 2 + alpha.capacity();
            if pixel_bytes > budget - built_bytes - scratch_bytes {
                return;
            }
            self.note_build_bytes(built_bytes + scratch_bytes + pixel_bytes, budget);
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
                    if spans.len() == spans.capacity() {
                        let available_spans = (budget - built_bytes - scratch_bytes - pixel_bytes)
                            / std::mem::size_of::<Span>();
                        let additional = available_spans.saturating_sub(spans.capacity()).min(128);
                        if additional == 0 || spans.try_reserve_exact(additional).is_err() {
                            return;
                        }
                        let bytes = built_bytes
                            + scratch_bytes
                            + pixel_bytes
                            + spans.capacity() * std::mem::size_of::<Span>();
                        if bytes > budget {
                            return;
                        }
                        self.note_build_bytes(bytes, budget);
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
            let layer = Layer { rgb, alpha, spans };
            built_bytes += layer.bytes();
            self.note_build_bytes(built_bytes + scratch_bytes, budget);
            layers.push(layer);
        }
        let bytes = built_bytes;
        self.sets.push(Set {
            key,
            size,
            layers,
            bytes,
        });
        debug_assert!(self.sets.iter().map(|s| s.bytes).sum::<usize>() <= budget);
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Pattern {
        color: Color,
    }
    impl CustomPainter for Pattern {
        fn paint(&self, canvas: &mut Canvas, size: Size) {
            for y in 0..size.height as usize {
                for x in (0..size.width as usize).step_by(2) {
                    canvas.draw_rect(Rect::from_ltwh(x as f32, y as f32, 1.0, 1.0), self.color);
                }
            }
        }
    }
    fn page(size: Size, color: Color) -> Box<dyn RenderBox> {
        let mut page = CustomPaint::new(Pattern { color })
            .size(size)
            .create_render_object();
        page.layout(&BoxConstraints::tight(size));
        page
    }

    #[test]
    fn replacement_and_dense_spans_share_budget_with_capture_scratch() {
        let size = Size::new(64.0, 32.0);
        let pages = vec![page(size, Color::WHITE)];
        let mut cache = PageRasterCache::default();
        let budget = 32 * 1024;
        cache.prepare_with_budget(1, size, &pages, budget);
        assert!(cache.contains(1, size));
        let old_bytes = cache.sets[0].bytes;
        assert!(old_bytes > 10 * 1024, "exercise fragmented span capacity");
        cache.prepare_with_budget(2, size, &pages, budget);
        assert!(!cache.contains(1, size));
        assert!(cache.contains(2, size));
        assert!(cache.build_peak_bytes <= budget);
        assert_eq!(cache.sets.len(), 1);

        let mut reference = tiny_gfx::Pixmap565::new(64, 32).unwrap();
        reference.fill(0x1042);
        pages[0].paint(&mut Canvas::new(reference.as_mut()), Offset::ZERO);
        let mut cached = tiny_gfx::Pixmap565::new(64, 32).unwrap();
        cached.fill(0x1042);
        assert!(cache.paint(2, size, 0, &mut Canvas::new(cached.as_mut()), Offset::ZERO));
        assert_eq!(cached, reference);

        // A second dense layer would fit the old RGB/alpha estimate but its
        // spans plus the black/white scratch exceed the construction budget.
        let pages = vec![page(size, Color::WHITE), page(size, Color::WHITE)];
        cache.prepare_with_budget(3, size, &pages, budget);
        assert!(!cache.contains(3, size));
        assert!(cache.sets.is_empty());
        assert!(cache.build_peak_bytes <= budget);
        assert!(!cache.paint(3, size, 0, &mut Canvas::new(cached.as_mut()), Offset::ZERO));
        // The caller's live widget fallback remains exactly the reference.
        cached.fill(0x1042);
        pages[0].paint(&mut Canvas::new(cached.as_mut()), Offset::ZERO);
        assert_eq!(cached, reference);
    }

    #[test]
    fn invalid_dimensions_are_rejected_before_multiplication_or_paint() {
        let size = Size::new(64.0, 32.0);
        let pages = vec![page(size, Color::WHITE)];
        let mut cache = PageRasterCache::default();
        for size in [
            Size::new(f32::INFINITY, 32.0),
            Size::new(u32::MAX as f32, u32::MAX as f32),
        ] {
            cache.prepare(1, size, &pages);
        }
        assert!(cache.sets.is_empty());
        assert_eq!(cache.build_peak_bytes, 0);
    }
}
