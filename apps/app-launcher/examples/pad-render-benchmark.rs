//! Synthetic render timings and allocation counts; not device FPS/latency.
use app_launcher::{build_launcher_ui, LauncherState};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Instant;
use tiny_flutter::prelude::*;

struct CountedAllocator;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(size, Ordering::Relaxed);
        System.realloc(ptr, layout, size)
    }
}
#[global_allocator]
static ALLOCATOR: CountedAllocator = CountedAllocator;

fn main() {
    let size = Size::new(1024.0, 600.0);
    let mut results = Vec::new();
    for (name, elapsed) in [
        ("desktop", None),
        ("wallpaper", None),
        ("circle_150ms", Some(150)),
        ("circle_220ms", Some(220)),
        ("bridge_300ms", Some(300)),
        ("reveal_640ms", Some(640)),
        ("timer", Some(940)),
    ] {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        if let Some(now) = elapsed {
            // Production prepares the backdrop during the initial complete
            // desktop paint, before the user presses the icon.
            let mut desktop = build_launcher_ui(state.clone(), size).create_render_object();
            desktop.layout(&BoxConstraints::tight(size));
            let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
            desktop.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
            let mut s = state.lock().unwrap();
            s.launch_app("timer", Rect::from_ltwh(323.0, 108.0, 137.24, 137.24));
            s.tick(now, 0);
        }
        let mut root = if name == "wallpaper" {
            CustomPaint::new(app_launcher::widgets::WallpaperPainter)
                .size(size)
                .create_render_object()
        } else {
            build_launcher_ui(state, size).create_render_object()
        };
        root.layout(&BoxConstraints::tight(size));
        let mut pixels = tiny_gfx::Pixmap565::new(1024, 600).unwrap();
        for _ in 0..3 {
            root.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
        }
        let mut times = Vec::with_capacity(40);
        ALLOCS.store(0, Ordering::Relaxed);
        ALLOC_BYTES.store(0, Ordering::Relaxed);
        for _ in 0..40 {
            let started = Instant::now();
            root.paint(&mut Canvas::new(pixels.as_mut()), Offset::ZERO);
            times.push(started.elapsed().as_nanos() as u64);
        }
        let allocations = ALLOCS.load(Ordering::Relaxed) / 40;
        let bytes = ALLOC_BYTES.load(Ordering::Relaxed) / 40;
        times.sort_unstable();
        results.push(serde_json::json!({"case":name,"median_us":times[20] as f64/1000.0,"p95_us":times[38] as f64/1000.0,"allocations_per_paint":allocations,"allocated_bytes_per_paint":bytes}));
    }
    println!(
        "{}",
        serde_json::json!({"scope":"synthetic arm64 release render-only; not device latency or resident heap","samples_per_case":40,"results":results})
    );
}
