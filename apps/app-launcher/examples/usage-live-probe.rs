//! Fixed synthetic comparison of the old explicit-rebuild sample path and
//! live render objects. Run with --release; host timing is not device FPS.
use app_launcher::{build_launcher_ui, usage::*, LauncherState};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tiny_flutter::{App, PlatformBackend, Rect, Size, TouchEvent};

struct Allocator;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static REQUESTED: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
fn allocated(size: usize) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    REQUESTED.fetch_add(size as u64, Ordering::Relaxed);
    let live = LIVE.fetch_add(size as u64, Ordering::Relaxed) + size as u64;
    PEAK.fetch_max(live, Ordering::Relaxed);
}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            allocated(layout.size());
        }
        ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            allocated(layout.size());
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let next = System.realloc(ptr, layout, new_size);
        if !next.is_null() {
            LIVE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
            allocated(new_size);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

const NOW: i64 = 1_791_287_400_000;
struct Sink {
    pixels: Vec<u16>,
    frame: bool,
}
impl Sink {
    fn new() -> Self {
        Self {
            pixels: vec![0; 1024 * 600],
            frame: false,
        }
    }
}
impl PlatformBackend for Sink {
    fn screen_size(&self) -> Size {
        Size::new(1024., 600.)
    }
    fn poll_touch(&mut self) -> Option<TouchEvent> {
        None
    }
    fn begin_frame(&mut self) {
        assert!(!self.frame);
        self.frame = true;
    }
    fn end_frame(&mut self) {
        assert!(self.frame);
        self.frame = false;
    }
    fn flush(&mut self, _: Rect, _: &[u16]) {
        panic!("probe requires strided output")
    }
    fn flush_strided(&mut self, rect: Rect, pixels: &[u16], stride: usize) -> bool {
        assert!(self.frame);
        assert_eq!(stride, 1024);
        let (x, y, w, h) = (
            rect.x as usize,
            rect.y as usize,
            rect.width as usize,
            rect.height as usize,
        );
        assert!(x + w <= 1024 && y + h <= 600 && h > 0 && w > 0);
        assert_eq!(pixels.len(), (h - 1) * stride + w);
        for row in 0..h {
            self.pixels[(y + row) * 1024 + x..(y + row) * 1024 + x + w]
                .copy_from_slice(&pixels[row * stride..row * stride + w]);
        }
        true
    }
}
fn fixture() -> Arc<Mutex<LauncherState>> {
    let mut s = LauncherState::new();
    s.open_app("sub2api-monitor");
    s.tick(10_000, NOW);
    s.usage.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    s.usage.scope = Some(api::scope(Period::Day, Page::Overview, None, NOW, 480).unwrap());
    let trend: Vec<_> = (0..12)
        .map(|i| Point {
            date: format!("2026-10-06 {i:02}:00"),
            totals: Totals {
                tokens: 50_000 + (i * i % 11) * 81_000,
                cost: i as f64 * 0.14,
                requests: Some(i * 2),
            },
        })
        .collect();
    let models = ["synthetic-a", "synthetic-b", "synthetic-c"]
        .into_iter()
        .enumerate()
        .map(|(i, n)| Model {
            name: n.into(),
            totals: Totals {
                tokens: 500_000 / (i as u64 + 1),
                cost: 1.,
                requests: Some(12),
            },
            input: 40,
            output: 20,
            cache: 10,
        })
        .collect();
    s.usage.data = Some(Arc::new(Data {
        totals: Some(Totals {
            tokens: 270_879_573,
            cost: 142.87,
            requests: Some(428),
        }),
        accounts: vec![Account {
            id: 1,
            label: "synthetic".into(),
            platform: "openai".into(),
            kind: "oauth".into(),
            plan_label: "PRO 20X".into(),
            usage_updated_ms: Some(NOW),
            five: Some(Window {
                used: 24.,
                reset_ms: Some(NOW + 3_600_000),
                window_minutes: Some(300),
            }),
            seven: Some(Window {
                used: 61.,
                reset_ms: Some(NOW + 86_400_000),
                window_minutes: Some(10080),
            }),
        }],
        trend: trend.clone(),
        yesterday_trend: trend,
        models,
        sampled_ms: NOW,
        trend_label: "全站 Token 趋势".into(),
        ..Data::default()
    }));
    Arc::new(Mutex::new(s))
}
struct ResultRow {
    metrics: tiny_flutter::app::FrameMetrics,
    allocations: u64,
    requested: u64,
    peak_extra: u64,
    pixels: Vec<u16>,
}
fn run(rebuild: bool) -> ResultRow {
    let state = fixture();
    let size = Size::new(1024., 600.);
    let mut sink = Sink::new();
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step_with_builder(&mut sink, |size| build_launcher_ui(state.clone(), size));
    app.take_frame_metrics();
    ALLOCATIONS.store(0, Ordering::Relaxed);
    REQUESTED.store(0, Ordering::Relaxed);
    let baseline = LIVE.load(Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    for sample in 1..=10u64 {
        let mono = 10_000 + sample * 5_000;
        {
            let mut s = state.lock().unwrap();
            if sample == 8 {
                s.usage.stale = true;
                s.usage.status = "网络连接失败".into();
            } else {
                let mut d = (**s.usage.data.as_ref().unwrap()).clone();
                let totals = d.totals.as_mut().unwrap();
                totals.tokens = 270_879_573 + sample * 113_421;
                totals.cost = 142.87 + sample as f64 * 0.17;
                if sample % 2 == 0 {
                    let last = d.trend.last_mut().unwrap();
                    last.totals.tokens += sample * 11_173;
                    last.totals.cost += 0.01;
                }
                d.sampled_ms = NOW + sample as i64 * 5_000;
                d.warning = (sample == 4).then(|| "趋势未更新".into());
                s.usage.stale = sample == 4;
                s.usage.status = d.warning.clone().unwrap_or_else(|| {
                    format!(
                        "更新于 14:30:{:02} · 实时采样 5 秒 / 看板 60 秒",
                        sample * 5
                    )
                });
                s.usage.data = Some(Arc::new(d));
            }
            s.tick(mono, NOW + sample as i64 * 5_000);
        }
        if rebuild {
            app.request_rebuild();
        }
        for frame in 0..=36u64 {
            let elapsed = frame * 33;
            state
                .lock()
                .unwrap()
                .tick(mono + elapsed, NOW + sample as i64 * 5_000 + elapsed as i64);
            app.step_with_builder(&mut sink, |size| build_launcher_ui(state.clone(), size));
        }
    }
    let row = ResultRow {
        metrics: app.take_frame_metrics(),
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        requested: REQUESTED.load(Ordering::Relaxed),
        peak_extra: PEAK.load(Ordering::Relaxed).saturating_sub(baseline),
        pixels: sink.pixels,
    };
    row
}
fn report(name: &str, r: &ResultRow) -> serde_json::Value {
    let m = &r.metrics;
    serde_json::json!({"path":name,"samples":10,"frames":m.frames,"draw_us":m.draw_us,"layout_us":m.layout_us,
        "advance_layouts":m.advance_layouts,"advance_layout_us":m.advance_layout_us,
        "paint_us":m.paint_us,"output_us":m.output_us,"mounts":m.mounts,"mount_us":m.mount_us,"builder_us":m.builder_us,
        "damage_pixels":m.damage_pixels,"full_region_frames":m.full_region_frames,"max_total_us":m.max_total_us,
        "allocations":r.allocations,"requested_bytes":r.requested,"peak_extra_bytes":r.peak_extra})
}
fn main() {
    // Warm both paths using the identical sequence, then measure without PNG,
    // network, storage or real credentials. Only the sample rebuild policy
    // differs; both paths use the current cache/raster code.
    drop(run(true));
    drop(run(false));
    let before = run(true);
    let after = run(false);
    let mismatch = before
        .pixels
        .iter()
        .zip(&after.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(mismatch, 0, "comparison produced different final pixels");
    println!(
        "{}",
        serde_json::json!({"environment":"host release synthetic; not device FPS", "pixel_mismatches":mismatch,
        "explicit_rebuild":report("explicit_rebuild",&before),"live":report("live",&after)})
    );
}
