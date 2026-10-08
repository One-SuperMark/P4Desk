//! Single background worker: latest pending request wins; network never holds UI locks.
use app_launcher::{
    launcher_state::{SystemError, SystemNotice},
    storage::MonitorStore,
    usage::{
        self,
        api::{self, Error, Transport},
        Command, Config, Data, Page, Period, Scope,
    },
    ActiveApp, LauncherState,
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Condvar, Mutex,
};

pub struct Service {
    mailbox: Arc<(Mutex<Mailbox>, Condvar)>,
    generation: Arc<AtomicU32>,
    request_key: Option<(u64, Scope)>,
    failures: u32,
    configuring: bool,
    last_preflight: Option<Error>,
    in_flight: Option<u32>,
    cadence: Option<Cadence>,
    retry_not_before_ms: u64,
}
#[derive(Clone)]
enum Work {
    Fetch(Config, Scope, i64, Option<Error>, FetchKind),
    Save(Config, Scope),
    Forget,
}
#[derive(Clone)]
enum FetchKind {
    Full,
    Headline(Arc<Data>),
}
impl FetchKind {
    fn label(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Headline(_) => "headline",
        }
    }
    fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }
}
const HEADLINE_INTERVAL_MS: u64 = 5_000;
const FULL_INTERVAL_MS: u64 = 60_000;
/// Slot deadlines are anchored to the first request start for a Scope. A busy
/// worker advances past elapsed slots instead of enqueuing catch-up requests.
struct Cadence {
    origin_ms: u64,
    next_headline_ms: u64,
}
impl Cadence {
    fn new(now: u64) -> Self {
        Self {
            origin_ms: now,
            next_headline_ms: now.saturating_add(HEADLINE_INTERVAL_MS),
        }
    }
    fn next_after(&self, now: u64, interval: u64) -> u64 {
        let slots = now.saturating_sub(self.origin_ms) / interval + 1;
        self.origin_ms
            .saturating_add(slots.saturating_mul(interval))
    }
    fn skip_elapsed_headlines(&mut self, now: u64) {
        if self.next_headline_ms <= now {
            self.next_headline_ms = self.next_after(now, HEADLINE_INTERVAL_MS);
        }
    }
}

struct Job {
    id: u32,
    work: Work,
}
struct Done {
    id: u32,
    work: Work,
    result: Result<Option<Data>, Error>,
    cached: Option<Data>,
}

fn configuration_idle_status(configured: bool) -> &'static str {
    if configured { "等待采样" } else { "连接站点后开始监控" }
}

fn same_totals(a: &usage::Totals, b: &usage::Totals) -> bool {
    a.tokens == b.tokens && a.cost.to_bits() == b.cost.to_bits() && a.requests == b.requests
}
fn same_window(a: &Option<usage::Window>, b: &Option<usage::Window>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            a.used.to_bits() == b.used.to_bits()
                && a.reset_ms == b.reset_ms
                && a.window_minutes == b.window_minutes
        }
        (None, None) => true,
        _ => false,
    }
}
/// The live widgets cover totals, chart and status. Any change outside those
/// regions must retain a regular rebuild, even if a future fast API starts
/// returning refreshed quota/ranking data. Comparisons borrow the bounded
/// snapshots and allocate nothing.
fn same_static_cards(a: &Data, b: &Data) -> bool {
    let yesterday = match (&a.yesterday, &b.yesterday) {
        (Some(a), Some(b)) => same_totals(a, b),
        (None, None) => true,
        _ => false,
    };
    yesterday
        && a.accounts.len() == b.accounts.len()
        && a.accounts.iter().zip(&b.accounts).all(|(a, b)| {
            a.id == b.id
                && a.label == b.label
                && a.platform == b.platform
                && a.kind == b.kind
                && a.plan_label == b.plan_label
                && a.usage_updated_ms == b.usage_updated_ms
                && same_window(&a.five, &b.five)
                && same_window(&a.seven, &b.seven)
        })
        && a.models.len() == b.models.len()
        && a.models.iter().zip(&b.models).all(|(a, b)| {
            a.name == b.name
                && same_totals(&a.totals, &b.totals)
                && a.input == b.input
                && a.output == b.output
                && a.cache == b.cache
        })
        && a.users.len() == b.users.len()
        && a.users
            .iter()
            .zip(&b.users)
            .all(|(a, b)| a.id == b.id && a.label == b.label && same_totals(&a.totals, &b.totals))
}
fn live_completion(state: &LauncherState, done: &Done) -> bool {
    let Work::Fetch(_, scope, _, _, FetchKind::Headline(base)) = &done.work else {
        return false;
    };
    if (scope.page, scope.period, scope.detail) != (Page::Overview, Period::Day, None)
        || (state.usage.page, state.usage.period, state.usage.detail)
            != (Page::Overview, Period::Day, None)
        || state.usage.scope.as_ref() != Some(scope)
        || !state
            .usage
            .data
            .as_ref()
            .is_some_and(|data| Arc::ptr_eq(data, base))
    {
        return false;
    }
    match &done.result {
        Ok(Some(next)) => same_static_cards(base, next),
        // A failed sample retains the same card tree; LiveStatus immediately
        // presents its error and stale state while HeadlineView keeps the
        // existing value. A loaded fallback snapshot would need a rebuild.
        Err(_) => done.cached.is_none(),
        _ => false,
    }
}
#[derive(Default)]
struct Mailbox {
    pending: Option<Job>,
    done: Option<Done>,
    stop: bool,
}
struct Guarded<T> {
    inner: T,
    id: u32,
    generation: Arc<AtomicU32>,
}
impl<T: Transport> Transport for Guarded<T> {
    fn request(&mut self, c: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        if self.generation.load(Ordering::Acquire) != self.id {
            return Err(Error::Cancelled);
        }
        self.inner.request(c, path, body)
    }
    fn finish_batch(&mut self) {
        self.inner.finish_batch()
    }
}
impl Service {
    pub fn new(
        transport: Box<dyn Transport + Send>,
        store: MonitorStore,
    ) -> Result<Self, std::io::Error> {
        let mailbox = Arc::new((Mutex::new(Mailbox::default()), Condvar::new()));
        let generation = Arc::new(AtomicU32::new(0));
        let shared = mailbox.clone();
        let epoch = generation.clone();
        std::thread::Builder::new().name("p4desk-monitor".into()).stack_size(16*1024).spawn(move||{
            let mut transport=Guarded{inner:transport,id:0,generation:epoch.clone()};
            let mut last_start = None::<std::time::Instant>;
            let mut starts = 0u32;
            loop {
                let job={let(mut m,cv)= (shared.0.lock().unwrap(),&shared.1);
                    while m.pending.is_none()&&!m.stop{m=cv.wait(m).unwrap();}
                    if m.stop { transport.finish_batch(); break;} m.pending.take().unwrap()};
                transport.id=job.id;
                let mut cached=None;
                let began = std::time::Instant::now();
                let interval_ms = last_start.map_or(0, |last| began.saturating_duration_since(last).as_millis());
                last_start = Some(began);
                starts = starts.saturating_add(1);
                // A short initial trace plus one sample/minute thereafter is
                // sufficient to measure real cadence without continuous spam.
                if starts <= 24 || starts % 12 == 0 {
                    crate::diagnostics::diagnostic!("p4desk_monitor: request=begin kind={} interval_ms={}", match &job.work { Work::Save(..)=>"configure",Work::Fetch(.., kind)=>kind.label(),Work::Forget=>"forget" }, interval_ms);
                }
                let result=match &job.work {
                    Work::Fetch(c,s,now,preflight,kind)=>{
                        let result=if let Some(e)=preflight { Err(*e) } else { match kind {
                            FetchKind::Full => api::fetch(&mut transport,c,s,*now),
                            FetchKind::Headline(base) => api::fetch_headline(&mut transport,c,s,*now,base),
                        }};
                        // Flash/TF persistence is bounded to full snapshots;
                        // a five-second sample is never written to storage.
                        if kind.is_full() {
                            match &result {Ok(d)=>{if epoch.load(Ordering::Acquire)==job.id{let _=store.save_cache(c,s,d);}},Err(_)=>cached=store.load_cache(c,s)}
                        }
                        result.map(Some)
                    },
                    Work::Save(c,s)=>api::validate(&mut transport,c,s).and_then(|()|{
                        if epoch.load(Ordering::Acquire)!=job.id{return Err(Error::Cancelled)}
                        store.save_config(Some(c)).map_err(|_|Error::Storage)?;Ok(None)
                    }),
                    Work::Forget=>store.save_config(None).map(|()|None).map_err(|_|Error::Storage),
                };
                // Reuse is confined to this batch; no socket or API-key header
                // survives the worker's wait, even after cancellation/errors.
                transport.finish_batch();
                crate::diagnostics::diagnostic!("p4desk_monitor: request=end ok={} duration_ms={} error={}",result.is_ok(),began.elapsed().as_millis(),result.as_ref().err().map(|e|format!("{e:?}")).unwrap_or("none".into()));
                if let Ok(Some(d))=&result { crate::diagnostics::diagnostic!("p4desk_monitor: data models={} users={} accounts={} trend={} partial={}",d.models.len(),d.users.len(),d.accounts.len(),d.trend.len(),d.warning.is_some()); }
                if epoch.load(Ordering::Acquire)==job.id {
                    shared.0.lock().unwrap().done=Some(Done{id:job.id,work:job.work,result,cached});
                }
            }
        })?;
        Ok(Self {
            mailbox,
            generation,
            request_key: None,
            failures: 0,
            configuring: false,
            last_preflight: None,
            in_flight: None,
            cadence: None,
            retry_not_before_ms: 0,
        })
    }
    fn submit(&mut self, work: Work) {
        let id = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        self.in_flight = Some(id);
        let mut m = self.mailbox.0.lock().unwrap();
        m.pending = Some(Job { id, work });
        m.done = None;
        self.mailbox.1.notify_one();
    }
    pub fn command(&mut self, state: &mut LauncherState, command: Command) {
        let now = state.monotonic_ms;
        match command {
            Command::Refresh => {
                if !state.usage.busy {
                    self.retry_not_before_ms = 0;
                    state.usage.refresh = true;
                    state.usage.next_refresh_ms = now;
                    state.changed();
                }
            }
            Command::Forget => {
                self.configuring = true;
                state.usage.configuration_result = None;
                self.submit(Work::Forget);
                state.usage.busy = true;
                state.usage.status = "正在清除配置".into();
                state.usage.editor.key.clear();
                state.changed();
            }
            Command::Save(c) => {
                let result = preflight(state).map_or_else(
                    || {
                        api::scope(
                            Period::Day,
                            Page::Overview,
                            None,
                            state.unix_ms,
                            state.settings.timezone_minutes,
                        )
                    },
                    Err,
                );
                match result {
                    Ok(s) => {
                        self.configuring = true;
                        state.usage.configuration_result = None;
                        self.submit(Work::Save(c, s));
                        state.usage.busy = true;
                        state.usage.editor.keyboard = false;
                        state.usage.status = "正在验证管理员接口".into();
                    }
                    Err(e) => {
                        state.usage.status = configuration_idle_status(state.usage.config.is_some()).into();
                        state.usage.busy = false;
                        state.usage.configuration_result = Some(false);
                        state.show_error(SystemError::Monitor(e));
                    }
                }
                state.changed();
            }
        }
    }
    pub fn poll(&mut self, state: &mut LauncherState) {
        let now = state.monotonic_ms;
        state
            .usage
            .set_clock_context(state.unix_ms, state.settings.timezone_minutes);
        let done = self.mailbox.0.lock().unwrap().done.take();
        if let Some(done) = done.filter(|d| d.id == self.generation.load(Ordering::Acquire)) {
            self.in_flight = None;
            if let Work::Fetch(config, scope, _, _, _) = &done.work {
                if self
                    .request_key
                    .as_ref()
                    .is_none_or(|(r, s)| *r != state.usage.revision || s != scope)
                    || state.usage.config.as_ref() != Some(config)
                {
                    return;
                }
            }
            let fast_sample = matches!(&done.work, Work::Fetch(.., FetchKind::Headline(_)));
            let live_update = live_completion(state, &done);
            let open_after = matches!(done.work, Work::Save(..)) && done.result.is_ok();
            // Only explicit configuration work produces a one-shot result.
            // Automatic full/headline fetches retain their live status and
            // backoff, including failures, without reopening a dialog.
            let configuration_dialog = match (&done.work, &done.result) {
                (Work::Save(..), Ok(_)) => Some(Ok(SystemNotice::MonitorSaved)),
                (Work::Forget, Ok(_)) => Some(Ok(SystemNotice::MonitorCleared)),
                (Work::Save(..) | Work::Forget, Err(error)) => Some(Err(*error)),
                _ => None,
            };
            let u = &mut state.usage;
            u.busy = false;
            if matches!(done.work, Work::Save(..) | Work::Forget) {
                self.configuring = false;
                u.configuration_result = Some(done.result.is_ok());
            }
            match (done.work, done.result) {
                (Work::Save(c, _), Ok(_)) => {
                    u.clear_page_cache();
                    u.config = Some(c);
                    u.data = None;
                    u.scope = None;
                    u.editor.key.clear();
                    u.editor.keyboard = false;
                    u.navigate(Page::Overview, u.period, None);
                    u.refresh = true;
                    u.status = configuration_idle_status(true).into();
                    self.request_key = None;
                    self.failures = 0;
                    self.last_preflight = None;
                    self.cadence = None;
                    self.retry_not_before_ms = 0;
                }
                (Work::Forget, Ok(_)) => {
                    *u = usage::State::default();
                    u.page = Page::Connection;
                    u.configuration_result = Some(true);
                    self.request_key = None;
                    self.last_preflight = None;
                    self.cadence = None;
                    self.retry_not_before_ms = 0;
                }
                (Work::Fetch(_, s, _, _, kind), Ok(Some(d))) => {
                    // Same network generation can still precede a UI navigation.
                    if self
                        .request_key
                        .as_ref()
                        .is_some_and(|(r, scope)| *r == u.revision && *scope == s)
                    {
                        u.stale = d.server_stale || d.warning.is_some();
                        let stamp = app_launcher::launcher_state::clock_strings(
                            d.sampled_ms,
                            state.settings.timezone_minutes,
                        )
                        .0;
                        u.status = d.warning.clone().unwrap_or_else(|| {
                            if d.server_stale {
                                "服务端数据已过期".into()
                            } else {
                                if s.page == Page::Overview
                                    && s.period == Period::Day
                                    && s.detail.is_none()
                                {
                                    format!("更新于 {stamp} · 实时采样 5 秒 / 看板 60 秒")
                                } else {
                                    format!("更新于 {stamp} · 自动刷新 60 秒")
                                }
                            }
                        });
                        let data = Arc::new(d);
                        u.remember_page(&s, data.clone());
                        u.data = Some(data);
                        u.scope = Some(s);
                        u.clamp_selection();
                        if let Some(cadence) = &mut self.cadence {
                            cadence.skip_elapsed_headlines(now);
                            if kind.is_full() && u.next_refresh_ms <= now {
                                u.next_refresh_ms = cadence.next_after(now, FULL_INTERVAL_MS);
                            }
                        }
                        self.failures = 0;
                        self.retry_not_before_ms = 0;
                    }
                }
                (_, Err(e)) => {
                    u.stale = u.data.is_some() || done.cached.is_some();
                    if u.data.is_none() {
                        u.data = done.cached.map(Arc::new);
                    }
                    u.clamp_selection();
                    u.status = if configuration_dialog.is_some() {
                        configuration_idle_status(u.config.is_some())
                    } else { e.message() }.into();
                    self.failures = (self.failures + 1).min(4);
                    u.next_refresh_ms = now.saturating_add(15_000 * (1u64 << self.failures));
                    self.retry_not_before_ms = u.next_refresh_ms;
                }
                _ => {}
            }
            if open_after {
                state.open_app("sub2api-monitor");
            }
            match configuration_dialog {
                Some(Ok(notice)) => { state.show_notice(notice); }
                Some(Err(error)) if error != Error::Cancelled => {
                    state.show_error(SystemError::Monitor(error));
                }
                _ => {}
            }
            if !fast_sample || (matches!(state.active_app, ActiveApp::Usage) && !live_update) {
                state.changed();
            }
        }
        // Configuration commands complete even when the application is hidden.
        if self.configuring {
            return;
        }
        let active = matches!(state.active_app, ActiveApp::Usage)
            || state.running_apps.contains_key("sub2api-monitor");
        if !active || state.usage.page == Page::Connection {
            return;
        }
        let Some(config) = state.usage.config.clone() else {
            return;
        };
        let pre = preflight(state);
        if self.in_flight.is_none() {
            let recovered =
                pre.is_none() && matches!(self.last_preflight, Some(Error::Offline | Error::Time));
            self.last_preflight = pre;
            if recovered {
                // Network and NTP readiness are local events, so a completed
                // preflight failure should not delay the first online sample.
                state.usage.refresh = true;
                state.usage.next_refresh_ms = now;
                self.retry_not_before_ms = 0;
                self.cadence = Some(Cadence::new(now));
            }
        }
        let s = match api::scope(
            state.usage.period,
            state.usage.page,
            state.usage.detail,
            state.unix_ms,
            state.settings.timezone_minutes,
        ) {
            Ok(s) => s,
            Err(e) => {
                if state.usage.status != e.message() {
                    state.usage.status = e.message().into();
                    state.changed();
                }
                return;
            }
        };
        let key = (state.usage.revision, s.clone());
        let changed = self.request_key.as_ref() != Some(&key);
        if changed {
            self.cadence = Some(Cadence::new(now));
            self.retry_not_before_ms = 0;
        }
        let idle = self.in_flight.is_none();
        let full_due = state.usage.refresh || now >= state.usage.next_refresh_ms;
        let headline_due = s.page == Page::Overview
            && s.period == Period::Day
            && s.detail.is_none()
            && state.usage.scope.as_ref() == Some(&s)
            && state.usage.data.is_some()
            && self
                .cadence
                .as_ref()
                .is_some_and(|c| now >= c.next_headline_ms);
        if changed || (idle && now >= self.retry_not_before_ms && (full_due || headline_due)) {
            let kind = if changed || full_due {
                FetchKind::Full
            } else {
                FetchKind::Headline(state.usage.data.as_ref().unwrap().clone())
            };
            let full = kind.is_full();
            self.last_preflight = pre;
            self.submit(Work::Fetch(config, s.clone(), state.unix_ms, pre, kind));
            self.request_key = Some(key);
            let u = &mut state.usage;
            if changed && u.scope.as_ref() != Some(&s) {
                u.load_cached_page(&s);
            }
            let cadence = self.cadence.get_or_insert_with(|| Cadence::new(now));
            cadence.next_headline_ms = cadence.next_after(now, HEADLINE_INTERVAL_MS);
            if full {
                u.next_refresh_ms = cadence.next_after(now, FULL_INTERVAL_MS);
                u.busy = true;
                u.refresh = false;
                u.status = if u.stale && u.data.is_some() {
                    "上次数据，正在刷新"
                } else {
                    "正在刷新"
                }
                .into();
                state.changed();
            }
            // Fast work has its own in-flight guard. The visible Refresh
            // control remains steady instead of flashing busy every 5 seconds.
        }
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.mailbox.0.lock().unwrap().stop = true;
        self.mailbox.1.notify_one();
    }
}
fn preflight(s: &LauncherState) -> Option<Error> {
    if s.radio.wifi_indicator() != app_launcher::radio::WifiIndicator::Connected {
        Some(Error::Offline)
    } else if !s.time_valid || s.time_estimated {
        Some(Error::Time)
    } else {
        None
    }
}

#[cfg(target_os = "espidf")]
pub struct EspTransport;
#[cfg(target_os = "espidf")]
impl Transport for EspTransport {
    fn finish_batch(&mut self) {
        extern "C" {
            fn p4desk_monitor_http_reset();
        }
        unsafe { p4desk_monitor_http_reset() }
    }
    fn request(&mut self, c: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        use std::ffi::{c_char, CString};
        extern "C" {
            fn p4desk_monitor_http(
                url: *const c_char,
                key: *const c_char,
                body: *const c_char,
                out: *mut u8,
                capacity: usize,
                length: *mut usize,
            ) -> i32;
        }
        let url = CString::new(format!("{}/api/v1{}", c.site, path)).map_err(|_| Error::Format)?;
        let key = CString::new(c.key.as_str()).map_err(|_| Error::Format)?;
        let body = body
            .map(CString::new)
            .transpose()
            .map_err(|_| Error::Format)?;
        let mut out = Vec::new();
        out.try_reserve_exact(api::MAX_RESPONSE)
            .map_err(|_| Error::TooLarge)?;
        out.resize(api::MAX_RESPONSE, 0);
        let mut len = 0;
        let result = unsafe {
            p4desk_monitor_http(
                url.as_ptr(),
                key.as_ptr(),
                body.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
                out.as_mut_ptr(),
                out.len(),
                &mut len,
            )
        };
        match result {
            200 if len <= out.len() => {
                out.truncate(len);
                Ok(out)
            }
            401 | 403 => Err(Error::Authentication),
            -2 => Err(Error::TooLarge),
            n if n >= 100 => Err(Error::Http(n as u16)),
            _ => Err(Error::Transport),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    struct Fake {
        fail: Arc<AtomicBool>,
    }
    impl Transport for Fake {
        fn request(&mut self, _: &Config, path: &str, _: Option<&str>) -> Result<Vec<u8>, Error> {
            if self.fail.load(Ordering::Relaxed) {
                return Err(Error::Authentication);
            }
            let data = if path == "/admin/dashboard/stats" {
                serde_json::json!({"today_tokens":10,"total_tokens":20,"today_actual_cost":1,"total_actual_cost":2,"today_requests":2,"total_requests":4})
            } else if path.starts_with("/admin/accounts?") {
                serde_json::json!({"items":[],"total":0})
            } else if path.starts_with("/admin/dashboard/trend?") {
                serde_json::json!({"trend":[]})
            } else if path.starts_with("/admin/dashboard/models?") {
                serde_json::json!({"models":[]})
            } else {
                serde_json::json!({"users":[]})
            };
            Ok(serde_json::to_vec(&serde_json::json!({"data":data})).unwrap())
        }
    }
    fn setup(
        name: &str,
    ) -> (
        Service,
        LauncherState,
        MonitorStore,
        Arc<AtomicBool>,
        std::path::PathBuf,
    ) {
        let root = std::env::temp_dir().join(format!("p4-usage-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let store =
            app_launcher::GenerationStore::new(root.join("sd"), root.join("flash")).monitor_store();
        let fail = Arc::new(AtomicBool::new(false));
        let service = Service::new(Box::new(Fake { fail: fail.clone() }), store.clone()).unwrap();
        let mut state = LauncherState::new();
        state.open_app("sub2api-monitor");
        state.tick(100, 1_791_287_400_000);
        state.radio.backend = 2;
        state.radio.wifi_on = 1;
        state.radio.wifi_phase = 5;
        (service, state, store, fail, root)
    }
    fn finish(
        service: &mut Service,
        state: &mut LauncherState,
        predicate: impl Fn(&LauncherState) -> bool,
    ) {
        for _ in 0..300 {
            service.poll(state);
            if predicate(state) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        panic!("background worker did not finish");
    }
    #[test]
    fn live_completion_requires_exact_baseline_and_preserves_static_card_updates() {
        let mut state = LauncherState::new();
        state.open_app("sub2api-monitor");
        state.tick(10_000, 1_791_287_400_000);
        let scope = api::scope(Period::Day, Page::Overview, None, state.unix_ms, 480).unwrap();
        let config = Config::new("test.example", "synthetic-key").unwrap();
        let base = Arc::new(Data::default());
        state.usage.scope = Some(scope.clone());
        state.usage.data = Some(base.clone());
        let mut done = Done {
            id: 1,
            work: Work::Fetch(
                config,
                scope,
                state.unix_ms,
                None,
                FetchKind::Headline(base.clone()),
            ),
            result: Ok(Some(Data {
                totals: Some(usage::Totals {
                    tokens: 123,
                    cost: 2.5,
                    requests: None,
                }),
                warning: Some("趋势未更新".into()),
                ..Data::default()
            })),
            cached: None,
        };
        assert!(live_completion(&state, &done));
        if let Ok(Some(next)) = &mut done.result {
            next.models.push(usage::Model {
                name: "synthetic".into(),
                totals: Default::default(),
                input: 0,
                output: 0,
                cache: 0,
            });
        }
        assert!(
            !live_completion(&state, &done),
            "new ranking content must rebuild"
        );
        done.result = Err(Error::Transport);
        assert!(live_completion(&state, &done));
        done.cached = Some(Data::default());
        assert!(
            !live_completion(&state, &done),
            "new fallback structure must rebuild"
        );
        done.cached = None;
        state.usage.data = Some(Arc::new(Data::default()));
        assert!(
            !live_completion(&state, &done),
            "equal values are not the same request baseline"
        );
        state.usage.data = Some(base);
        state.usage.navigate(Page::Overview, Period::Month, None);
        assert!(
            !live_completion(&state, &done),
            "old DAY completion must not mutate the MONTH tree"
        );
    }
    #[test]
    fn validates_before_saving_and_keeps_previous_connection_on_failure() {
        let (mut service, mut state, store, fail, root) = setup("validate");
        let config = Config::new("one.example", "test-key").unwrap();
        service.command(&mut state, Command::Save(config.clone()));
        finish(&mut service, &mut state, |s| {
            s.usage.configuration_result == Some(true)
        });
        let dialog = state.active_dialog().unwrap();
        assert_eq!(dialog.content, app_launcher::launcher_state::DialogContent::Notice(SystemNotice::MonitorSaved));
        assert!(state.dismiss_dialog(dialog.id));
        assert_ne!(state.usage.status, "配置已保存");
        assert_eq!(store.load_config().unwrap(), Some(config.clone()));
        finish(&mut service, &mut state, |s| !s.usage.busy);
        fail.store(true, Ordering::Relaxed);
        service.command(
            &mut state,
            Command::Save(Config::new("two.example", "wrong-key").unwrap()),
        );
        finish(&mut service, &mut state, |s| {
            s.usage.configuration_result == Some(false)
        });
        let dialog = state.active_dialog().unwrap();
        assert_eq!(dialog.content, app_launcher::launcher_state::DialogContent::Error(SystemError::Monitor(Error::Authentication)));
        assert_ne!(state.usage.status, Error::Authentication.message());
        assert!(state.dismiss_dialog(dialog.id));
        assert_eq!(store.load_config().unwrap(), Some(config));
        service.command(&mut state, Command::Forget);
        finish(&mut service, &mut state, |s| {
            s.usage.config.is_none() && !s.usage.busy
        });
        let dialog = state.active_dialog().unwrap();
        assert_eq!(dialog.content, app_launcher::launcher_state::DialogContent::Notice(SystemNotice::MonitorCleared));
        assert_eq!(state.usage.status, "连接站点后开始监控");
        assert!(state.dismiss_dialog(dialog.id));
        for _ in 0..5 { service.poll(&mut state); }
        assert!(state.active_dialog().is_none(), "consumed operation results must not reopen");
        assert!(store.load_config().unwrap().is_none());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn offline_does_not_validate_or_save_credentials() {
        let (mut service, mut state, store, _, root) = setup("offline");
        state.radio.wifi_phase = 0;
        service.command(
            &mut state,
            Command::Save(Config::new("test.example", "test-key").unwrap()),
        );
        assert_eq!(state.usage.configuration_result, Some(false));
        assert!(!state.usage.busy);
        assert_eq!(state.active_dialog().unwrap().content,
            app_launcher::launcher_state::DialogContent::Error(SystemError::Monitor(Error::Offline)));
        assert_ne!(state.usage.status, Error::Offline.message());
        assert!(store.load_config().unwrap().is_none());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn time_preflight_failure_is_dismissible_without_a_persistent_result() {
        let (mut service, mut state, store, _, root) = setup("notice-time");
        state.time_valid = false;
        service.command(&mut state, Command::Save(Config::new("test.example", "test-key").unwrap()));
        let dialog = state.active_dialog().unwrap();
        assert_eq!(dialog.content,
            app_launcher::launcher_state::DialogContent::Error(SystemError::Monitor(Error::Time)));
        assert_eq!(state.usage.configuration_result, Some(false));
        assert_eq!(state.usage.status, "连接站点后开始监控");
        assert!(store.load_config().unwrap().is_none());
        assert!(state.dismiss_dialog(dialog.id));
        for _ in 0..5 { service.poll(&mut state); }
        assert!(state.active_dialog().is_none());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn save_and_forget_storage_failures_preserve_config_and_report_once() {
        for forget in [false, true] {
            let name = if forget { "notice-forget-storage" } else { "notice-save-storage" };
            let (mut service, mut state, _, _, root) = setup(name);
            let previous = Config::new("previous.example", "synthetic-key").unwrap();
            state.usage.config = Some(previous.clone());
            state.usage.page = Page::Connection;
            // A regular file in the temporary fixture prevents creating the
            // configuration directory. No device or user filesystem is used.
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("flash"), b"blocked fixture").unwrap();
            let command = if forget { Command::Forget }
                else { Command::Save(Config::new("next.example", "synthetic-key").unwrap()) };
            service.command(&mut state, command);
            finish(&mut service, &mut state, |s| s.usage.configuration_result == Some(false));
            let dialog = state.active_dialog().unwrap();
            assert_eq!(dialog.content,
                app_launcher::launcher_state::DialogContent::Error(SystemError::Monitor(Error::Storage)));
            assert_eq!(state.usage.config, Some(previous));
            assert_eq!(state.usage.status, "等待采样");
            assert!(state.dismiss_dialog(dialog.id));
            for _ in 0..5 { service.poll(&mut state); }
            assert!(state.active_dialog().is_none());
            drop(service);
            let _ = std::fs::remove_dir_all(root);
        }
    }
    #[test]
    fn automatic_full_and_headline_failures_remain_silent() {
        let (mut service, mut state, _, fail, root) = setup("notice-auto-fetch");
        state.usage.config = Some(Config::new("test.example", "synthetic-key").unwrap());
        fail.store(true, Ordering::Relaxed);
        finish(&mut service, &mut state, |s| !s.usage.busy && s.usage.status == Error::Authentication.message());
        assert!(state.active_dialog().is_none());
        fail.store(false, Ordering::Relaxed);
        state.monotonic_ms = state.usage.next_refresh_ms;
        finish(&mut service, &mut state, |s| s.usage.data.is_some() && !s.usage.busy);
        assert!(state.active_dialog().is_none());
        fail.store(true, Ordering::Relaxed);
        state.monotonic_ms += HEADLINE_INTERVAL_MS + 1;
        finish(&mut service, &mut state, |s| s.usage.status == Error::Authentication.message());
        assert!(state.usage.data.is_some(), "automatic failure retains the last data");
        assert!(state.active_dialog().is_none(), "five-second samples must not reopen operation alerts");
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn wifi_recovery_retries_before_the_offline_backoff_deadline() {
        let (mut service, mut state, _, fail, root) = setup("wifi-ready");
        state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
        state.radio.wifi_phase = 0;
        // Any accidental HTTPS call while offline would report Authentication.
        fail.store(true, Ordering::Relaxed);
        finish(&mut service, &mut state, |s| {
            !s.usage.busy && s.usage.status == Error::Offline.message()
        });
        let deadline = state.usage.next_refresh_ms;
        assert!(deadline > state.monotonic_ms);
        state.radio.wifi_phase = 5;
        fail.store(false, Ordering::Relaxed);
        service.poll(&mut state);
        assert!(state.usage.busy);
        assert!(state.monotonic_ms < deadline);
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        assert_eq!(
            state
                .usage
                .data
                .as_ref()
                .unwrap()
                .totals
                .as_ref()
                .unwrap()
                .tokens,
            10
        );
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn trusted_time_recovery_retries_without_https_while_time_is_untrusted() {
        for estimated in [false, true] {
            let (mut service, mut state, _, fail, root) = setup(if estimated {
                "time-estimated"
            } else {
                "time-invalid"
            });
            state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
            state.time_valid = estimated;
            state.time_estimated = estimated;
            fail.store(true, Ordering::Relaxed);
            finish(&mut service, &mut state, |s| {
                !s.usage.busy && s.usage.status == Error::Time.message()
            });
            let deadline = state.usage.next_refresh_ms;
            assert!(deadline > state.monotonic_ms);
            for _ in 0..5 {
                service.poll(&mut state);
            }
            assert!(!state.usage.busy);
            assert_eq!(state.usage.status, Error::Time.message());
            state.time_valid = true;
            state.time_estimated = false;
            fail.store(false, Ordering::Relaxed);
            service.poll(&mut state);
            assert!(state.usage.busy);
            assert!(state.monotonic_ms < deadline);
            finish(&mut service, &mut state, |s| {
                s.usage.data.is_some() && !s.usage.busy
            });
            drop(service);
            let _ = std::fs::remove_dir_all(root);
        }
    }
    #[test]
    fn authentication_failure_keeps_its_existing_backoff() {
        let (mut service, mut state, _, fail, root) = setup("auth-backoff");
        state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
        fail.store(true, Ordering::Relaxed);
        finish(&mut service, &mut state, |s| {
            !s.usage.busy && s.usage.status == Error::Authentication.message()
        });
        let deadline = state.usage.next_refresh_ms;
        assert!(deadline > state.monotonic_ms);
        fail.store(false, Ordering::Relaxed);
        for _ in 0..5 {
            service.poll(&mut state);
        }
        assert!(!state.usage.busy);
        assert!(state.usage.data.is_none());
        assert_eq!(state.usage.next_refresh_ms, deadline);
        assert_eq!(state.usage.status, Error::Authentication.message());
        state.monotonic_ms = deadline;
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn old_generation_cannot_overwrite_new_page_or_period() {
        let (mut service, mut state, _, _, root) = setup("scope");
        state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
        service.poll(&mut state);
        state.usage.navigate(Page::Models, Period::Month, None);
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        assert_eq!(state.usage.scope.as_ref().unwrap().page, Page::Models);
        assert_eq!(state.usage.scope.as_ref().unwrap().period, Period::Month);
        assert!(state.usage.data.as_ref().unwrap().trend.is_empty());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn cached_page_is_visible_during_a_required_fresh_request_and_after_failure() {
        let (mut service, mut state, _, fail, root) = setup("page-cache");
        state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        let original = state.usage.data.clone().unwrap();
        assert_eq!(state.usage.cached_page_count(), 1);
        state.usage.navigate(Page::Models, Period::Day, None);
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        fail.store(true, Ordering::Relaxed);
        state.usage.navigate(Page::Overview, Period::Day, None);
        assert!(Arc::ptr_eq(state.usage.data.as_ref().unwrap(), &original));
        assert!(state.usage.refresh && state.usage.stale);
        service.poll(&mut state);
        assert!(state.usage.busy);
        assert_eq!(state.usage.status, "上次数据，正在刷新");
        finish(&mut service, &mut state, |s| !s.usage.busy);
        assert!(Arc::ptr_eq(state.usage.data.as_ref().unwrap(), &original));
        assert!(state.usage.status.contains("密钥无效"));
        assert!(state.usage.stale);
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn successfully_replacing_or_forgetting_configuration_clears_ram_snapshots() {
        let (mut service, mut state, _, _, root) = setup("cache-connection");
        state.usage.config = Some(Config::new("one.example", "test-key").unwrap());
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        assert_eq!(state.usage.cached_page_count(), 1);
        let next = Config::new("two.example", "next-key").unwrap();
        service.command(&mut state, Command::Save(next.clone()));
        finish(&mut service, &mut state, |s| {
            s.usage.configuration_result == Some(true)
        });
        assert_eq!(state.usage.config.as_ref(), Some(&next));
        assert_eq!(state.usage.cached_page_count(), 0);
        assert!(state.usage.data.is_none());
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        assert_eq!(state.usage.cached_page_count(), 1);
        service.command(&mut state, Command::Forget);
        finish(&mut service, &mut state, |s| {
            s.usage.config.is_none() && !s.usage.busy
        });
        assert_eq!(state.usage.cached_page_count(), 0);
        assert!(state.usage.data.is_none());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn canceled_inflight_page_never_enters_ram_cache() {
        struct Slow {
            fake: Fake,
            started: Arc<AtomicBool>,
            released: Arc<AtomicBool>,
            finished: Arc<AtomicU32>,
        }
        impl Transport for Slow {
            fn request(
                &mut self,
                c: &Config,
                path: &str,
                body: Option<&str>,
            ) -> Result<Vec<u8>, Error> {
                if path == "/admin/dashboard/stats" {
                    self.started.store(true, Ordering::Release);
                    while !self.released.load(Ordering::Acquire) {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                self.fake.request(c, path, body)
            }
            fn finish_batch(&mut self) {
                self.finished.fetch_add(1, Ordering::AcqRel);
            }
        }
        let root =
            std::env::temp_dir().join(format!("p4-usage-cache-cancel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let store =
            app_launcher::GenerationStore::new(root.join("sd"), root.join("flash")).monitor_store();
        let started = Arc::new(AtomicBool::new(false));
        let released = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicU32::new(0));
        let mut service = Service::new(
            Box::new(Slow {
                fake: Fake {
                    fail: Arc::new(AtomicBool::new(false)),
                },
                started: started.clone(),
                released: released.clone(),
                finished: finished.clone(),
            }),
            store,
        )
        .unwrap();
        let mut state = LauncherState::new();
        state.open_app("sub2api-monitor");
        state.tick(100, 1_791_287_400_000);
        state.radio.backend = 2;
        state.radio.wifi_on = 1;
        state.radio.wifi_phase = 5;
        state.usage.config = Some(Config::new("test.example", "test-key").unwrap());
        service.poll(&mut state);
        for _ in 0..300 {
            if started.load(Ordering::Acquire) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(started.load(Ordering::Acquire));
        state.usage.navigate(Page::Models, Period::Month, None);
        service.poll(&mut state);
        released.store(true, Ordering::Release);
        finish(&mut service, &mut state, |s| {
            s.usage.data.is_some() && !s.usage.busy
        });
        assert_eq!(state.usage.cached_page_count(), 1);
        assert_eq!(
            finished.load(Ordering::Acquire),
            2,
            "cancelled and replacement batches must both release their session"
        );
        assert_eq!(state.usage.scope.as_ref().unwrap().page, Page::Models);
        state.usage.navigate(Page::Overview, Period::Day, None);
        assert!(state.usage.data.is_none());
        drop(service);
        let _ = std::fs::remove_dir_all(root);
    }
}
