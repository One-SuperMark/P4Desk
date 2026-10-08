//! File-backed TF typeface over the bounded FreeType C HAL.
//! Font initialization and visible-name warmup run on the monitor worker.
use app_launcher::usage::Data;
use tiny_flutter::graphics::font::Font;

#[cfg(any(target_os = "espidf", test))]
mod requests {
    use std::collections::VecDeque;
    const LIMIT: usize = 256;
    #[derive(Default)]
    pub(super) struct Requests {
        pending: VecDeque<(char, u16)>,
        attempted: VecDeque<((char, u16), std::time::Instant)>,
    }
    impl Requests {
        pub(super) fn enqueue(&mut self, key: (char, u16), now: std::time::Instant) -> bool {
            if key.0.is_control()
                || !(8..=128).contains(&key.1)
                || self.pending.contains(&key)
                || self.pending.len() >= LIMIT
                || self.attempted.iter().any(|(k, when)| {
                    *k == key && now.saturating_duration_since(*when).as_secs() < 1
                })
            {
                return false;
            }
            self.pending.push_back(key);
            true
        }
        pub(super) fn is_empty(&self) -> bool {
            self.pending.is_empty()
        }
        pub(super) fn batch(&mut self) -> Vec<(char, u16)> {
            let count = self.pending.len().min(32);
            self.pending.drain(..count).collect()
        }
        pub(super) fn attempted(&mut self, key: (char, u16), now: std::time::Instant) {
            self.attempted.retain(|(k, _)| *k != key);
            if self.attempted.len() >= LIMIT {
                self.attempted.pop_front();
            }
            self.attempted.push_back((key, now));
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn queued_glyphs_are_bounded_deduplicated_batched_and_retryable() {
            let now = std::time::Instant::now();
            let mut q = Requests::default();
            assert!(!q.enqueue(('\n', 18), now));
            assert!(!q.enqueue(('钟', 129), now));
            assert!(q.enqueue(('钟', 18), now));
            assert!(!q.enqueue(('钟', 18), now));
            assert_eq!(q.batch(), vec![('钟', 18)]);
            q.attempted(('钟', 18), now);
            assert!(!q.enqueue(('钟', 18), now));
            assert!(q.enqueue(('钟', 18), now + std::time::Duration::from_secs(1)));
            q.batch();
            for cp in 0x4e00..0x4f00 {
                assert!(q.enqueue((char::from_u32(cp).unwrap(), 18), now));
            }
            assert!(!q.enqueue(('邃', 18), now));
            assert_eq!(q.batch().len(), 32);
            assert!(!q.is_empty());
        }
    }
}

#[cfg(target_os = "espidf")]
mod device {
    use super::requests::Requests;
    use std::cell::Cell;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex, OnceLock, RwLock};
    use tiny_flutter::graphics::font::{
        install_typeface_provider, notify_typeface_glyphs_ready, Font, Glyph, TypefaceMetrics,
        TypefaceProvider,
    };
    type Waker = Arc<dyn Fn() + Send + Sync>;
    static WAKER: RwLock<Option<Waker>> = RwLock::new(None);
    static REQUESTS: OnceLock<Mutex<Requests>> = OnceLock::new();
    thread_local! { static IO_ALLOWED: Cell<bool> = const { Cell::new(false) }; }
    fn requests() -> &'static Mutex<Requests> {
        REQUESTS.get_or_init(Default::default)
    }
    fn queue(ch: char, px: u16) {
        let added = requests()
            .lock()
            .unwrap()
            .enqueue((ch, px), std::time::Instant::now());
        if added {
            let wake = WAKER.read().unwrap().clone();
            if let Some(wake) = wake {
                wake();
            }
        }
    }
    pub(super) fn register_waker(waker: Waker) {
        *WAKER.write().unwrap() = Some(waker);
    }
    pub(super) fn pending() -> bool {
        !requests().lock().unwrap().is_empty()
    }
    pub(super) fn with_io<T>(operation: impl FnOnce() -> T) -> T {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                IO_ALLOWED.with(|v| v.set(self.0));
            }
        }
        let previous = IO_ALLOWED.with(|v| v.replace(true));
        let _restore = Restore(previous);
        operation()
    }
    pub(super) fn drain() {
        let batch = requests().lock().unwrap().batch();
        let before = METRICS_OK.load(Ordering::Relaxed);
        with_io(|| {
            for key in batch {
                let _ = Font::dynamic_font().glyph(key.0, key.1 as f32);
                requests()
                    .lock()
                    .unwrap()
                    .attempted(key, std::time::Instant::now());
            }
        });
        if METRICS_OK.load(Ordering::Relaxed) != before {
            notify_typeface_glyphs_ready();
        }
    }
    pub(super) fn measured() -> u32 { METRICS_OK.load(Ordering::Relaxed) }

    #[repr(C)]
    #[derive(Default)]
    struct CMetrics {
        width: u16,
        height: u16,
        xmin: i16,
        ymin: i16,
        advance: f32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct CMemory {
        current_bytes: u32,
        peak_bytes: u32,
        allocation_failures: u32,
        successful_allocations: u32,
    }
    const _: () = {
        assert!(std::mem::size_of::<CMetrics>() == 12);
        assert!(std::mem::align_of::<CMetrics>() == 4);
        assert!(std::mem::offset_of!(CMetrics, advance) == 8);
        assert!(std::mem::size_of::<CMemory>() == 16);
    };
    extern "C" {
        fn p4desk_typeface_init() -> i32;
        fn p4desk_typeface_ready() -> bool;
        fn p4desk_typeface_metrics(ch: u32, px: u16, out: *mut CMetrics) -> i32;
        fn p4desk_typeface_render(
            ch: u32,
            px: u16,
            alpha: *mut u8,
            capacity: usize,
            out: *mut CMetrics,
        ) -> i32;
        fn p4desk_typeface_memory(out: *mut CMemory);
    }
    impl CMetrics {
        fn convert(&self) -> Option<TypefaceMetrics> {
            if self.width > 256
                || self.height > 256
                || !self.advance.is_finite()
                || !(0.0..=512.0).contains(&self.advance)
            {
                return None;
            }
            Some(TypefaceMetrics {
                width: self.width as usize,
                height: self.height as usize,
                xmin: self.xmin as i32,
                ymin: self.ymin as i32,
                advance: self.advance,
            })
        }
    }
    struct TfTypeface;
    static METRICS_OK: AtomicU32 = AtomicU32::new(0);
    static RENDER_OK: AtomicU32 = AtomicU32::new(0);
    impl TypefaceProvider for TfTypeface {
        fn metrics(&self, ch: char, size: u16) -> Option<TypefaceMetrics> {
            if !IO_ALLOWED.with(Cell::get) {
                queue(ch, size);
                return None;
            }
            let mut out = CMetrics::default();
            if unsafe { p4desk_typeface_metrics(ch as u32, size, &mut out) } != 0 {
                return None;
            }
            let metrics = out.convert()?;
            METRICS_OK.fetch_add(1, Ordering::Relaxed);
            Some(metrics)
        }
        fn rasterize(&self, ch: char, size: u16) -> Option<Glyph> {
            if !IO_ALLOWED.with(Cell::get) {
                queue(ch, size);
                return None;
            }
            let metrics = self.metrics(ch, size)?;
            let mut alpha = vec![0; metrics.width.checked_mul(metrics.height)?];
            let mut rendered = CMetrics::default();
            if unsafe {
                p4desk_typeface_render(
                    ch as u32,
                    size,
                    alpha.as_mut_ptr(),
                    alpha.len(),
                    &mut rendered,
                )
            } != 0
                || rendered.convert()? != metrics
            {
                return None;
            }
            let glyph = Glyph::from_alpha8(metrics, alpha).ok()?;
            RENDER_OK.fetch_add(1, Ordering::Relaxed);
            Some(glyph)
        }
    }
    pub(super) fn prepare() {
        if unsafe { p4desk_typeface_ready() } {
            return;
        }
        let status = unsafe { p4desk_typeface_init() };
        if status == 0 {
            install_typeface_provider(Some(Arc::new(TfTypeface)));
        }
        let mut memory = CMemory::default();
        unsafe { p4desk_typeface_memory(&mut memory) };
        crate::diagnostics::diagnostic!(
            "p4desk_typeface: ready={} code={} memory_bytes={} peak_bytes={} alloc_failures={}",
            status == 0,
            status,
            memory.current_bytes,
            memory.peak_bytes,
            memory.allocation_failures,
        );
    }
    pub(super) fn report() {
        let mut memory = CMemory::default();
        unsafe { p4desk_typeface_memory(&mut memory) };
        crate::diagnostics::diagnostic!(
            "p4desk_typeface: memory_bytes={} peak_bytes={} alloc_failures={} metrics_ok={} render_ok={}",
            memory.current_bytes, memory.peak_bytes, memory.allocation_failures,
            METRICS_OK.load(Ordering::Relaxed), RENDER_OK.load(Ordering::Relaxed),
        );
    }
}

/// Only the first visible five names and their initials are warmed. A 200-user
/// response must not expand 200 × 80 characters at several sizes or evict all
/// currently visible glyphs. Later pages resolve through the same bounded cache.
pub(crate) fn prepare_and_warm(data: Option<&Data>) {
    #[cfg(target_os = "espidf")]
    {
        let before = device::measured();
        device::with_io(|| {
            device::prepare();
            warm_names(data);
        });
        if device::measured() != before {
            tiny_flutter::graphics::font::notify_typeface_glyphs_ready();
        }
        device::report();
    }
    #[cfg(not(target_os = "espidf"))]
    warm_names(data);
}

pub(crate) fn register_waker(waker: std::sync::Arc<dyn Fn() + Send + Sync>) {
    #[cfg(target_os = "espidf")]
    device::register_waker(waker);
    #[cfg(not(target_os = "espidf"))]
    let _ = waker;
}
pub(crate) fn pending() -> bool {
    #[cfg(target_os = "espidf")]
    {
        device::pending()
    }
    #[cfg(not(target_os = "espidf"))]
    {
        false
    }
}
pub(crate) fn drain_pending() {
    #[cfg(target_os = "espidf")]
    device::drain();
}

fn warm_names(data: Option<&Data>) {
    let Some(data) = data else {
        return;
    };
    if data.users.is_empty() {
        return;
    }
    let font = Font::dynamic_font();
    for user in data.users.iter().take(5) {
        let name = user.display_name();
        for ch in name.chars().take(32).filter(|ch| !ch.is_control()) {
            let _ = font.glyph(ch, 18.0);
            let _ = font.glyph(ch, 14.0);
        }
        if let Some(initial) = name
            .chars()
            .find(|ch| ch.is_alphanumeric())
            .and_then(|ch| ch.to_uppercase().next())
        {
            let _ = font.glyph(initial, 14.0);
        }
    }
}
