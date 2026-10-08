//! Sub2API monitor: bounded device data model, independent of its HTTPS transport.
pub mod api;
pub mod headline;
pub mod trend;
pub mod ui;
pub mod user_names;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Arc;
use tiny_flutter::ScrollController;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub site: String,
    pub key: String,
}
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MonitorConfig([redacted])")
    }
}
impl Config {
    pub fn new(site: &str, key: &str) -> Result<Self, &'static str> {
        let site = site.trim().trim_end_matches('/');
        let site = if site.contains("://") {
            site.to_owned()
        } else {
            format!("https://{site}")
        };
        let rest = site.strip_prefix("https://").ok_or("请填写 HTTPS 站点")?;
        if site.len() > 240
            || !site.is_ascii()
            || rest.is_empty()
            || rest
                .bytes()
                .any(|b| b.is_ascii_whitespace() || b.is_ascii_control() || b"@?#\\%".contains(&b))
        {
            return Err("站点不能包含凭据、查询参数或空格");
        }
        let authority = rest.split('/').next().unwrap_or("");
        let mut parts = authority.split(':');
        let host = parts.next().unwrap_or("");
        if host.is_empty()
            || host.starts_with('.')
            || host.ends_with('.')
            || !host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
            || parts
                .next()
                .is_some_and(|p| p.parse::<u16>().ok().filter(|v| *v > 0).is_none())
            || parts.next().is_some()
            || rest.split('/').any(|p| matches!(p, "." | ".."))
        {
            return Err("站点地址格式不正确");
        }
        let key = key.trim();
        if key.is_empty() || key.len() > 512 || !key.bytes().all(|b| (33..=126).contains(&b)) {
            return Err("请输入有效的管理员 API Key");
        }
        Ok(Self {
            site: site.trim_end_matches("/api/v1").to_owned(),
            key: key.to_owned(),
        })
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Period {
    #[default]
    Day,
    Month,
    Total,
}
impl Period {
    pub fn label(self) -> &'static str {
        match self {
            Self::Day => "今日",
            Self::Month => "本月",
            Self::Total => "累计",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Page {
    #[default]
    Overview,
    Accounts,
    Models,
    Users,
    Connection,
}
impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "总览",
            Self::Accounts => "账号额度",
            Self::Models => "模型",
            Self::Users => "用户",
            Self::Connection => "连接设置",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub period: Period,
    pub page: Page,
    pub detail: Option<i64>,
    pub date: String,
    pub timezone_minutes: i32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Totals {
    pub tokens: u64,
    pub cost: f64,
    pub requests: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub date: String,
    pub totals: Totals,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub totals: Totals,
    pub input: u64,
    pub output: u64,
    pub cache: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    /// Actual display name from the user profile. Older snapshots omit it.
    #[serde(default)]
    pub name: Option<String>,
    pub label: String,
    pub totals: Totals,
}
impl User {
    pub fn display_name(&self) -> String {
        self.name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("用户#{}", self.id))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Window {
    pub used: f64,
    pub reset_ms: Option<i64>,
    #[serde(default)]
    pub window_minutes: Option<u32>,
}
impl Window {
    pub fn remaining(&self) -> f64 {
        (100.0 - self.used).clamp(0.0, 100.0)
    }
    /// Time remaining in this server window. An expired window stays at zero
    /// until the server publishes a new reset; it never refreshes quota locally.
    pub fn remaining_time_ratio(&self, now: i64, fallback_minutes: u32) -> Option<f64> {
        let minutes = self.window_minutes.unwrap_or(fallback_minutes);
        if minutes == 0 {
            return None;
        }
        Some(
            (self.reset_ms?.saturating_sub(now).max(0) as f64 / (minutes as f64 * 60_000.0))
                .clamp(0.0, 1.0),
        )
    }
    pub fn caption(&self, now: i64) -> String {
        match self.reset_ms {
            Some(ms) if ms <= now => "等待额度更新".into(),
            Some(ms) => {
                let m = (ms - now + 59_999) / 60_000;
                if m >= 1440 {
                    format!("{} 天 {} 小时后重置", m / 1440, m % 1440 / 60)
                } else {
                    format!("{} 小时 {} 分后重置", m / 60, m % 60)
                }
            }
            None => "重置时间未知".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub label: String,
    pub platform: String,
    pub kind: String,
    #[serde(default)]
    pub plan_label: String,
    #[serde(default)]
    pub usage_updated_ms: Option<i64>,
    pub five: Option<Window>,
    pub seven: Option<Window>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    pub totals: Option<Totals>,
    pub yesterday: Option<Totals>,
    /// Complete previous day, while today's curve ends at the observed hour.
    #[serde(default)]
    pub yesterday_trend: Vec<Point>,
    pub trend: Vec<Point>,
    pub models: Vec<Model>,
    pub users: Vec<User>,
    pub accounts: Vec<Account>,
    pub sampled_ms: i64,
    pub server_stale: bool,
    pub warning: Option<String>,
    pub trend_label: String,
    pub total_label: String,
}
#[derive(Clone, Debug)]
pub enum Command {
    Save(Config),
    Forget,
    Refresh,
}
pub struct Editor {
    pub site: String,
    pub key: String,
    pub field: usize,
    pub keyboard: bool,
    pub uppercase: bool,
    pub symbols: bool,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            site: "https://".into(),
            key: String::new(),
            field: 0,
            keyboard: false,
            uppercase: false,
            symbols: false,
        }
    }
}
pub struct State {
    pub config: Option<Config>,
    pub page: Page,
    pub period: Period,
    pub detail: Option<i64>,
    pub scroll: ScrollController,
    pub data: Option<Arc<Data>>,
    pub scope: Option<Scope>,
    pub status: String,
    pub busy: bool,
    pub stale: bool,
    pub editor: Editor,
    pub chart_selected: Option<usize>,
    pub model_selected: Option<usize>,
    pub list_page: usize,
    pub configuration_result: Option<bool>,
    pub revision: u64,
    pub refresh: bool,
    pub next_refresh_ms: u64,
    /// Local presentation state; never replaces authoritative Data.totals.
    pub headline: headline::HeadlineState,
    // RAM only. The digest separates sites/keys without retaining a second
    // plaintext credential or exposing it through a Debug implementation.
    page_snapshots: VecDeque<PageSnapshot>,
    clock_context: Option<(i64, i32)>,
    headline_identity: Option<HeadlineIdentity>,
    headline_observed: Option<HeadlineObservation>,
}
#[derive(PartialEq, Eq)]
struct HeadlineIdentity {
    config_digest: [u8; 32],
    scope: Scope,
}
struct HeadlineObservation {
    data_ptr: usize,
    tokens: Option<u64>,
    config_digest: Option<[u8; 32]>,
    page: Page,
    period: Period,
    detail: Option<i64>,
    calendar_day: Option<i64>,
    timezone_minutes: Option<i32>,
    scope: Option<Scope>,
}
struct PageSnapshot {
    config_digest: [u8; 32],
    scope: Scope,
    data: Arc<Data>,
}
const PAGE_CACHE_CAPACITY: usize = 4;
fn config_digest(config: &Config) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(config.site.as_bytes());
    digest.update([0]);
    digest.update(config.key.as_bytes());
    digest.finalize().into()
}
impl Default for State {
    fn default() -> Self {
        Self {
            config: None,
            page: Page::Overview,
            period: Period::Day,
            detail: None,
            scroll: ScrollController::new(),
            data: None,
            scope: None,
            status: "连接站点后开始监控".into(),
            busy: false,
            stale: false,
            editor: Editor::default(),
            chart_selected: None,
            model_selected: None,
            list_page: 0,
            configuration_result: None,
            revision: 0,
            refresh: true,
            next_refresh_ms: 0,
            headline: headline::HeadlineState::default(),
            page_snapshots: VecDeque::new(),
            clock_context: None,
            headline_identity: None,
            headline_observed: None,
        }
    }
}
impl State {
    pub fn set_clock_context(&mut self, unix_ms: i64, timezone_minutes: i32) {
        self.clock_context = Some((unix_ms, timezone_minutes));
    }
    /// Called from LauncherState::tick and, if a worker completed after that
    /// tick, once before building the UI. No global revision or repaint is
    /// raised here: the headline render object consumes its own local damage.
    pub fn sync_headline(&mut self, now_ms: u64, visible: bool) {
        let data_ptr = self
            .data
            .as_ref()
            .map_or(0, |data| Arc::as_ptr(data) as usize);
        let tokens = self
            .data
            .as_ref()
            .and_then(|data| data.totals.as_ref().map(|t| t.tokens));
        let digest = self.config.as_ref().map(config_digest);
        let calendar_day = self.clock_context.and_then(|(ms, tz)| {
            (ms > 0).then(|| (ms.div_euclid(1_000) + tz as i64 * 60).div_euclid(86_400))
        });
        let timezone_minutes = self.clock_context.map(|(_, tz)| tz);
        let unchanged = self.headline_observed.as_ref().is_some_and(|observed| {
            observed.data_ptr == data_ptr
                && observed.tokens == tokens
                && observed.config_digest == digest
                && observed.page == self.page
                && observed.period == self.period
                && observed.detail == self.detail
                && observed.calendar_day == calendar_day
                && observed.timezone_minutes == timezone_minutes
                && observed.scope == self.scope
        });
        if unchanged {
            if !visible {
                self.headline.settle();
            }
            return;
        }
        // Avoid allocating/formatting a calendar scope on every animation
        // tick. Validation occurs only when the snapshot or context changed.
        let valid_scope = self.scope.as_ref().filter(|scope| {
            (scope.page, scope.period, scope.detail) == (self.page, self.period, self.detail)
                && self.clock_context.is_none_or(|(now, tz)| {
                    api::scope(self.period, self.page, self.detail, now, tz).as_ref() == Ok(*scope)
                })
        });
        let value = self.data.as_ref().and_then(|data| {
            // Unknown-scope synthetic/direct snapshots can display directly,
            // but cannot animate across an unvalidated identity. Explicitly
            // mismatched network scopes cannot be shown on the new page.
            (self.scope.is_none() || valid_scope.is_some())
                .then(|| data.totals.as_ref().map(|totals| totals.tokens))
                .flatten()
        });
        let identity = digest
            .zip(valid_scope)
            .map(|(config_digest, scope)| HeadlineIdentity {
                config_digest,
                scope: scope.clone(),
            });
        let duration = identity
            .as_ref()
            .zip(self.headline_identity.as_ref())
            .and_then(|(next, previous)| {
                if next == previous {
                    Some(headline::UPDATE_DURATION_MS)
                } else if next.config_digest == previous.config_digest
                    && next.scope.page == previous.scope.page
                    && next.scope.detail == previous.scope.detail
                    && next.scope.date == previous.scope.date
                    && next.scope.timezone_minutes == previous.scope.timezone_minutes
                    && next.scope.period != previous.scope.period
                {
                    Some(headline::PERIOD_DURATION_MS)
                } else {
                    None
                }
            });
        if visible {
            if let Some(duration) = duration {
                self.headline.update(value, now_ms, duration);
            } else {
                self.headline.snap(value);
            }
        } else {
            self.headline.snap(value);
        }
        self.headline_identity = identity;
        self.headline_observed = Some(HeadlineObservation {
            data_ptr,
            tokens,
            config_digest: digest,
            page: self.page,
            period: self.period,
            detail: self.detail,
            calendar_day,
            timezone_minutes,
            scope: self.scope.clone(),
        });
    }
    pub fn cached_page_count(&self) -> usize {
        self.page_snapshots.len()
    }
    /// Resolve a detail title from the current list or a snapshot belonging to
    /// this exact configuration. Names follow stable IDs across ranking changes.
    pub fn user_display_name(&self, id: i64) -> String {
        if let Some(user) = self
            .data
            .as_ref()
            .and_then(|d| d.users.iter().find(|u| u.id == id))
        {
            return user.display_name();
        }
        if let Some(config) = &self.config {
            let digest = config_digest(config);
            for snapshot in self
                .page_snapshots
                .iter()
                .rev()
                .filter(|p| p.config_digest == digest)
            {
                if let Some(user) = snapshot.data.users.iter().find(|u| u.id == id) {
                    // An explicitly nameless newer profile supersedes old names.
                    return user.display_name();
                }
            }
        }
        format!("用户#{id}")
    }
    pub fn clear_page_cache(&mut self) {
        self.page_snapshots.clear();
    }
    /// Remember a validated worker snapshot, including an explicitly marked
    /// partial response. LRU eviction keeps at most four shared Data values.
    pub fn remember_page(&mut self, scope: &Scope, data: Arc<Data>) {
        let Some(config) = self.config.as_ref() else {
            self.clear_page_cache();
            return;
        };
        let digest = config_digest(config);
        self.page_snapshots
            .retain(|p| p.config_digest != digest || p.scope != *scope);
        self.page_snapshots.push_back(PageSnapshot {
            config_digest: digest,
            scope: scope.clone(),
            data,
        });
        while self.page_snapshots.len() > PAGE_CACHE_CAPACITY {
            self.page_snapshots.pop_front();
        }
    }
    /// Restore only the exact current site/key and calendar scope. A cached
    /// page is visible immediately, but still requires an asynchronous update.
    pub fn load_cached_page(&mut self, scope: &Scope) -> bool {
        self.data = None;
        self.scope = Some(scope.clone());
        self.stale = false;
        let Some(config) = self.config.as_ref() else {
            self.clear_page_cache();
            return false;
        };
        let Some((now, tz)) = self.clock_context else {
            return false;
        };
        if api::scope(scope.period, scope.page, scope.detail, now, tz).as_ref() != Ok(scope) {
            return false;
        }
        let digest = config_digest(config);
        let Some(index) = self
            .page_snapshots
            .iter()
            .position(|p| p.config_digest == digest && p.scope == *scope)
        else {
            return false;
        };
        let snapshot = self.page_snapshots.remove(index).unwrap();
        self.data = Some(snapshot.data.clone());
        self.page_snapshots.push_back(snapshot);
        self.stale = true;
        self.refresh = true;
        self.status = "上次数据，正在刷新".into();
        self.clamp_selection();
        true
    }
    pub fn clamp_selection(&mut self) {
        let Some(data) = &self.data else {
            return;
        };
        let (count, per_page) = if self.detail.is_some() {
            (data.models.len(), 3)
        } else {
            match self.page {
                Page::Accounts => (data.accounts.len(), 2),
                Page::Models => (data.models.len(), 5),
                Page::Users => (data.users.len(), 5),
                _ => (1, 1),
            }
        };
        let pages = count.div_ceil(per_page).max(1);
        self.list_page = self.list_page.min(pages - 1);
        if self.detail.is_some() || self.page == Page::Models {
            let first = self.list_page * per_page;
            if self
                .model_selected
                .is_some_and(|n| n < first || n >= (first + per_page).min(count))
            {
                self.model_selected = None;
            }
        }
    }
    pub fn navigate(&mut self, page: Page, period: Period, detail: Option<i64>) {
        if (self.page, self.period, self.detail) == (page, period, detail) {
            return;
        }
        self.page = page;
        self.period = period;
        self.detail = detail;
        self.scroll.set_offset(0.0);
        self.data = None;
        self.scope = None;
        self.chart_selected = None;
        self.model_selected = None;
        self.list_page = 0;
        self.revision += 1;
        self.busy = false;
        self.stale = false;
        self.refresh = true;
        self.status = if self.config.is_some() {
            "等待刷新"
        } else {
            "连接站点后开始监控"
        }
        .into();
        if page != Page::Connection {
            if let Some((now, tz)) = self.clock_context {
                if let Ok(scope) = api::scope(period, page, detail, now, tz) {
                    self.load_cached_page(&scope);
                }
            }
        }
    }
    pub fn edit_connection(&mut self) {
        self.editor = Editor {
            site: self
                .config
                .as_ref()
                .map(|c| c.site.clone())
                .unwrap_or("https://".into()),
            ..Default::default()
        };
        self.navigate(Page::Connection, self.period, None);
    }
}
pub fn tokens(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1e9)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1e3)
    } else {
        n.to_string()
    }
}
pub fn masked(label: &str, id: i64) -> String {
    if label.is_empty() {
        format!("用户 {id}")
    } else {
        format!(
            "{}********",
            label
                .chars()
                .filter(|c| !c.is_control())
                .take(2)
                .collect::<String>()
        )
    }
}
