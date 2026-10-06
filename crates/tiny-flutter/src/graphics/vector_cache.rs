//! Bounded exact RGB565 replay for static vectors. Keep SVG geometry as source.
//! A cached result is usable only when the pixels underneath still match. This
//! retains translucent edges, fractional placement and changing glass materials
//! without reconstructing alpha from an already quantized RGB565 image.
use super::{Canvas, Color, Rect, VectorIcon};
use std::cell::RefCell;

const PIXEL_BUDGET: usize = 6 * 1024 * 1024;
const ENTRY_LIMIT: usize = 128;

#[derive(Clone, PartialEq)]
struct Key {
    layers: usize,
    count: usize,
    geometry: [u32; 16],
    tint: Color,
    opaque: bool,
}
#[derive(Clone, Copy, PartialEq, Debug)]
struct Region {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}
impl Region {
    fn intersect(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.w).min(other.x + other.w);
        let bottom = (self.y + self.h).min(other.y + other.h);
        (right > x && bottom > y).then(|| Self {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    }
}
struct Entry {
    key: Key,
    region: Region,
    before: Vec<u16>,
    after: Vec<u16>,
    used: u64,
}
impl Entry {
    fn bytes(&self) -> usize {
        (self.before.capacity() + self.after.capacity()) * 2
    }
}
#[derive(Default)]
struct Cache {
    entries: Vec<Entry>,
    bytes: usize,
    building_bytes: usize,
    stamp: u64,
    hits: u64,
    misses: u64,
}
thread_local! { static CACHE: RefCell<Cache> = RefCell::new(Cache::default()); }

/// Reserve both captures before allocating either of them. `draw` may enter
/// another cached surface, so pending captures must share the same budget.
struct CaptureReservation {
    bytes: usize,
}
impl CaptureReservation {
    fn new(key: &Key, bytes: usize) -> Option<Self> {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if bytes > PIXEL_BUDGET.checked_sub(cache.building_bytes)? {
                return None;
            }
            if let Some(index) = cache.entries.iter().position(|e| &e.key == key) {
                let old = cache.entries.swap_remove(index);
                cache.bytes -= old.bytes();
            }
            while cache.bytes + cache.building_bytes + bytes > PIXEL_BUDGET
                || cache.entries.len() >= ENTRY_LIMIT
            {
                let index = cache
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.used)?
                    .0;
                let old = cache.entries.swap_remove(index);
                cache.bytes -= old.bytes();
            }
            cache.building_bytes += bytes;
            Some(Self { bytes })
        })
    }
    fn finish(mut self, mut entry: Entry) {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let bytes = entry.bytes();
            // Exact reservations normally match Vec capacities. Keep the
            // fallback safe if that allocation contract ever changes.
            if bytes > self.bytes {
                return;
            }
            if let Some(index) = cache.entries.iter().position(|e| e.key == entry.key) {
                let old = cache.entries.swap_remove(index);
                cache.bytes -= old.bytes();
            }
            while cache.entries.len() >= ENTRY_LIMIT {
                let index = cache
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.used)
                    .unwrap()
                    .0;
                let old = cache.entries.swap_remove(index);
                cache.bytes -= old.bytes();
            }
            if cache.entries.try_reserve(1).is_ok() {
                entry.used = cache.stamp;
                cache.entries.push(entry);
                cache.bytes += bytes;
                cache.building_bytes -= self.bytes;
                self.bytes = 0;
            }
        });
    }
}
impl Drop for CaptureReservation {
    fn drop(&mut self) {
        if self.bytes != 0 {
            CACHE.with(|cache| cache.borrow_mut().building_bytes -= self.bytes);
        }
    }
}

/// Entries, retained pixel bytes, cache hits and misses on the current UI thread.
/// No user content is exposed; retained and in-progress pixel captures share a
/// 6 MiB budget across icons and static surfaces. Metadata is not pixel storage.
pub fn vector_cache_stats() -> (usize, usize, u64, u64) {
    CACHE.with(|c| {
        let c = c.borrow();
        (c.entries.len(), c.bytes, c.hits, c.misses)
    })
}

fn copy_region(canvas: &Canvas, r: Region) -> Option<Vec<u16>> {
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(r.w * r.h).ok()?;
    let stride = canvas.pixel_stride();
    for y in r.y..r.y + r.h {
        pixels.extend_from_slice(&canvas.pixels_rgb565()[y * stride + r.x..y * stride + r.x + r.w]);
    }
    Some(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::numix_icons_generated::*;
    use crate::tiny_gfx::Pixmap565;

    fn render(
        icon: &VectorIcon,
        cached: bool,
        bg: Color,
        clip: Option<Rect>,
        scale_canvas: bool,
    ) -> Pixmap565 {
        let mut pixels = Pixmap565::new(190, 180).unwrap();
        let mut canvas = Canvas::new(pixels.as_mut());
        canvas.clear(bg);
        if let Some(clip) = clip {
            canvas.clip_rect(clip);
        }
        canvas.translate(2.25, 3.5);
        if scale_canvas {
            canvas.scale(0.85, 0.9);
        }
        let bounds = Rect::from_ltwh(14.2, 8.7, 140.0, 140.0);
        if cached {
            icon.paint(&mut canvas, bounds, Color::WHITE);
        } else {
            icon.paint_layers(
                &mut canvas,
                bounds.x,
                bounds.y,
                140.0 / 128.0,
                Color::WHITE,
                1.0,
            );
        }
        pixels
    }

    #[test]
    fn cached_vectors_match_direct_raster_on_changed_background_and_clipped_updates() {
        CACHE.with(|c| *c.borrow_mut() = Cache::default());
        for (_, icon) in ALL_NUMIX_ICONS {
            for (bg, clip, scale) in [
                (Color::from_hex(0x123456), None, false),
                (Color::from_hex(0x123456), None, false),
                (
                    Color::from_hex(0x123456),
                    Some(Rect::from_ltwh(52.0, 30.0, 75.0, 60.0)),
                    false,
                ),
                (Color::from_hex(0xbbeedd), None, false),
                (
                    Color::BLACK,
                    Some(Rect::from_ltwh(4.5, 6.5, 160.0, 140.0)),
                    false,
                ),
                (Color::BLACK, None, true),
            ] {
                assert_eq!(
                    render(icon, true, bg, clip, scale),
                    render(icon, false, bg, clip, scale)
                );
            }
        }
        assert!(vector_cache_stats().2 >= 84);
    }

    #[test]
    fn partially_warmed_cache_does_not_replay_unpainted_pixels() {
        CACHE.with(|c| *c.borrow_mut() = Cache::default());
        let icon = &NUMIX_DARK_CALCULATOR;
        let clip = Some(Rect::from_ltwh(52.0, 30.0, 75.0, 60.0));
        render(icon, true, Color::BLACK, clip, false);
        assert_eq!(
            render(icon, true, Color::BLACK, None, false),
            render(icon, false, Color::BLACK, None, false)
        );
        assert_eq!(vector_cache_stats().2, 0);
        render(icon, true, Color::BLACK, clip, false);
        assert_eq!(vector_cache_stats().2, 1);
    }

    #[test]
    fn cache_evicts_old_geometry_with_a_fixed_pixel_budget() {
        CACHE.with(|c| *c.borrow_mut() = Cache::default());
        let mut pixels = Pixmap565::new(260, 260).unwrap();
        for i in 0..120 {
            let mut canvas = Canvas::new(pixels.as_mut());
            canvas.clear(Color::BLACK);
            NUMIX_LIGHT_CLOCK.paint(
                &mut canvas,
                Rect::from_ltwh(2.0, 2.0, 130.0 + i as f32, 130.0 + i as f32),
                Color::WHITE,
            );
            let (entries, bytes, _, _) = vector_cache_stats();
            assert!(bytes <= PIXEL_BUDGET && entries <= ENTRY_LIMIT);
        }
        assert!(vector_cache_stats().0 < 120);
    }

    fn assert_capture_budget() {
        CACHE.with(|c| {
            let c = c.borrow();
            assert!(c.bytes + c.building_bytes <= PIXEL_BUDGET);
        });
    }

    fn full_surface(canvas: &mut Canvas, id: usize, draw: impl FnOnce(&mut Canvas)) {
        surface(
            canvas,
            Rect::from_ltwh(0.0, 0.0, 1024.0, 600.0),
            id,
            10,
            [0; 12],
            false,
            draw,
        );
    }

    #[test]
    fn nested_capture_evicts_before_allocation_and_falls_back_without_changing_pixels() {
        CACHE.with(|c| *c.borrow_mut() = Cache::default());
        let mut pixels = Pixmap565::new(1024, 600).unwrap();
        let mut canvas = Canvas::new(pixels.as_mut());
        for id in 1..=2 {
            canvas.clear(Color::BLACK);
            full_surface(&mut canvas, id, |canvas| canvas.clear(Color::WHITE));
        }
        assert_eq!(vector_cache_stats().1, 1024 * 600 * 4 * 2);
        canvas.clear(Color::BLACK);
        let expected = Color::from_hex(0x3175a9);
        full_surface(&mut canvas, 3, |canvas| {
            assert_capture_budget();
            CACHE.with(|c| {
                let c = c.borrow();
                assert_eq!(c.building_bytes, 1024 * 600 * 4);
                assert_eq!(c.entries.len(), 1, "evict old storage before capture");
            });
            full_surface(canvas, 4, |canvas| {
                assert_capture_budget();
                full_surface(canvas, 5, |canvas| {
                    assert_capture_budget();
                    // Three full translucent captures exceed the budget. The
                    // innermost draw must still execute without caching it.
                    CACHE.with(|c| assert_eq!(c.borrow().building_bytes, 1024 * 600 * 8));
                    canvas.clear(expected);
                });
            });
        });
        assert_capture_budget();
        CACHE.with(|c| assert_eq!(c.borrow().building_bytes, 0));
        assert!(pixels.data().iter().all(|p| *p == expected.to_rgb565()));
        assert_eq!(vector_cache_stats().0, 2);
    }

    #[test]
    fn abandoned_capture_returns_its_reservation() {
        CACHE.with(|c| *c.borrow_mut() = Cache::default());
        let mut pixels = Pixmap565::new(1024, 600).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            full_surface(&mut Canvas::new(pixels.as_mut()), 1, |_| {
                panic!("synthetic paint failure")
            });
        }));
        assert!(result.is_err());
        CACHE.with(|c| assert_eq!(c.borrow().building_bytes, 0));
        assert_eq!(vector_cache_stats().1, 0);
    }
}

pub(super) fn paint(
    icon: &VectorIcon,
    canvas: &mut Canvas,
    bounds: Rect,
    area: Rect,
    tint: Color,
    opacity: f32,
    draw: impl FnOnce(&mut Canvas),
) {
    // Animated sizes/opacity and arbitrary transforms use the original path.
    let Some((tx, ty)) = canvas.translation_only() else {
        draw(canvas);
        return;
    };
    if opacity != 1.0 || area.width > 258.0 || area.height > 258.0 {
        draw(canvas);
        return;
    }
    let key = Key {
        layers: icon.layers.as_ptr() as usize,
        count: icon.layers.len(),
        tint,
        opaque: false,
        geometry: [
            bounds.x.to_bits(),
            bounds.y.to_bits(),
            bounds.width.to_bits(),
            bounds.height.to_bits(),
            tx.to_bits(),
            ty.to_bits(),
            icon.width.to_bits(),
            icon.height.to_bits(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    };
    replay(canvas, area, key, draw);
}

/// The supplied signature must include every source revision and paint parameter.
/// Opaque surfaces must replace every pixel in the supplied bounds.
pub(super) fn surface(
    canvas: &mut Canvas,
    area: Rect,
    source: usize,
    tag: usize,
    signature: [u32; 12],
    opaque: bool,
    draw: impl FnOnce(&mut Canvas),
) {
    let Some((tx, ty)) = canvas.translation_only() else {
        draw(canvas);
        return;
    };
    let mut geometry = [0; 16];
    geometry[..12].copy_from_slice(&signature);
    geometry[12..].copy_from_slice(&[tx.to_bits(), ty.to_bits(), canvas.width(), canvas.height()]);
    replay(
        canvas,
        area,
        Key {
            layers: source,
            count: usize::MAX - tag,
            geometry,
            tint: Color::TRANSPARENT,
            opaque,
        },
        draw,
    );
}

fn replay(canvas: &mut Canvas, area: Rect, key: Key, draw: impl FnOnce(&mut Canvas)) {
    let Some((tx, ty)) = canvas.translation_only() else {
        draw(canvas);
        return;
    };
    let screen = Rect::from_ltwh(0.0, 0.0, canvas.width() as f32, canvas.height() as f32);
    let clip = canvas.current_clip().unwrap_or(screen);
    // Pixel-aligned clipping can replay a subset of an earlier complete draw.
    if [clip.x, clip.y, clip.right(), clip.bottom()]
        .iter()
        .any(|v| !v.is_finite() || v.fract() != 0.0)
    {
        draw(canvas);
        return;
    }
    let to_region = |r: Rect| -> Option<Region> {
        let x = r.x.floor().max(0.0).min(screen.width) as usize;
        let y = r.y.floor().max(0.0).min(screen.height) as usize;
        let right = r.right().ceil().max(0.0).min(screen.width) as usize;
        let bottom = r.bottom().ceil().max(0.0).min(screen.height) as usize;
        (right > x && bottom > y).then(|| Region {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    };
    let Some(region) = to_region(Rect::from_ltwh(
        area.x + tx,
        area.y + ty,
        area.width,
        area.height,
    )) else {
        draw(canvas);
        return;
    };
    let Some(visible) = to_region(clip).and_then(|c| region.intersect(c)) else {
        return;
    };
    let hit = CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.stamp = cache.stamp.wrapping_add(1);
        let stamp = cache.stamp;
        let stride = canvas.pixel_stride();
        if let Some(entry) = cache
            .entries
            .iter_mut()
            .find(|e| e.key == key && e.region.intersect(visible) == Some(visible))
        {
            let region = entry.region;
            let same = key.opaque
                || (visible.y..visible.y + visible.h).all(|y| {
                    let source = (y - region.y) * region.w + visible.x - region.x;
                    let dest = y * stride + visible.x;
                    canvas.pixels_rgb565()[dest..dest + visible.w]
                        == entry.before[source..source + visible.w]
                });
            if same {
                for y in visible.y..visible.y + visible.h {
                    let source = (y - region.y) * region.w + visible.x - region.x;
                    let dest = y * stride + visible.x;
                    canvas.pixels_rgb565_mut()[dest..dest + visible.w]
                        .copy_from_slice(&entry.after[source..source + visible.w]);
                }
                entry.used = stamp;
                cache.hits += 1;
                return true;
            }
        }
        cache.misses += 1;
        false
    });
    if hit {
        return;
    }
    // Capture only the region actually painted. Later partial draws may reuse
    // a containing entry; a larger clip must rasterize its uncovered pixels.
    let region = visible;
    let Some(bytes) = region
        .w
        .checked_mul(region.h)
        .and_then(|n| n.checked_mul(if key.opaque { 2 } else { 4 }))
    else {
        draw(canvas);
        return;
    };
    let Some(reservation) = CaptureReservation::new(&key, bytes) else {
        draw(canvas);
        return;
    };
    let before = if key.opaque {
        Some(Vec::new())
    } else {
        copy_region(canvas, region)
    };
    draw(canvas);
    let Some(before) = before else {
        return;
    };
    let Some(after) = copy_region(canvas, region) else {
        return;
    };
    reservation.finish(Entry {
        key,
        region,
        before,
        after,
        used: 0,
    });
}
