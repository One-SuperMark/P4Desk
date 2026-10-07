//! Management API semantics ported from the local Sub2APIMonitor source.
//! Never retain account credentials, request bodies, or full user email addresses.
use super::*;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, Timelike, Utc};
use serde_json::Value;
pub const MAX_RESPONSE: usize = 256 * 1024;
pub const MAX_ACCOUNTS: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Offline,
    Time,
    Timezone,
    Transport,
    Authentication,
    Http(u16),
    Format,
    TooLarge,
    Storage,
    Cancelled,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Offline => "Wi-Fi 未连接，保留上次数据",
            Self::Time => "请先联网对时，再连接监控站点",
            Self::Timezone => "监控暂不支持此时区，请使用整点时区",
            Self::Transport => "连接失败，请检查网络与站点证书",
            Self::Authentication => "管理员密钥无效或权限不足",
            Self::Http(429) => "请求过于频繁，稍后重试",
            Self::Http(_) => "服务器请求失败，稍后重试",
            Self::Format => "接口数据格式不兼容",
            Self::TooLarge => "数据超出设备容量，请缩小查询范围",
            Self::Storage => "配置保存失败，请检查存储",
            Self::Cancelled => "已取消请求",
        }
    }
}
pub trait Transport {
    /// HTTPS only, no redirects, status checked before parsing; bounded response.
    fn request(
        &mut self,
        config: &Config,
        path: &str,
        body: Option<&str>,
    ) -> Result<Vec<u8>, Error>;
    /// End the single worker's bounded request batch, including failed or
    /// cancelled work. Device transports release their TLS session here.
    fn finish_batch(&mut self) {}
}
impl<T: Transport + ?Sized> Transport for Box<T> {
    fn request(&mut self, c: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        (**self).request(c, path, body)
    }
    fn finish_batch(&mut self) {
        (**self).finish_batch()
    }
}
fn request(
    t: &mut impl Transport,
    c: &Config,
    path: &str,
    body: Option<&str>,
) -> Result<Value, Error> {
    let bytes = t.request(c, path, body)?;
    if bytes.len() > MAX_RESPONSE {
        return Err(Error::TooLarge);
    }
    let mut v: Value = serde_json::from_slice(&bytes).map_err(|_| Error::Format)?;
    if v.get("code").is_some_and(|c| c.as_i64() != Some(0)) {
        return Err(Error::Format);
    }
    if !v.get("data").is_some_and(Value::is_object) {
        return Err(Error::Format);
    }
    Ok(v["data"].take())
}
fn uint(v: &Value, key: &str) -> Result<u64, Error> {
    v.get(key).and_then(Value::as_u64).ok_or(Error::Format)
}
fn money(v: &Value, key: &str) -> Result<f64, Error> {
    v.get(key)
        .and_then(Value::as_f64)
        .filter(|n| n.is_finite() && *n >= 0.0)
        .ok_or(Error::Format)
}
fn string(v: &Value, key: &str, max: usize) -> String {
    v[key]
        .as_str()
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect()
}
fn array<'a>(v: &'a Value, key: &str, max: usize) -> Result<&'a Vec<Value>, Error> {
    let a = v[key].as_array().ok_or(Error::Format)?;
    if a.len() > max {
        Err(Error::TooLarge)
    } else {
        Ok(a)
    }
}
fn totals(v: &Value) -> Result<Totals, Error> {
    Ok(Totals {
        tokens: uint(v, "total_tokens")?,
        cost: money(v, "actual_cost")?,
        requests: v.get("requests").map(|_| uint(v, "requests")).transpose()?,
    })
}
pub fn scope(
    period: Period,
    page: Page,
    detail: Option<i64>,
    unix_ms: i64,
    tz: i32,
) -> Result<Scope, Error> {
    let offset =
        FixedOffset::east_opt(tz.checked_mul(60).ok_or(Error::Time)?).ok_or(Error::Time)?;
    let date = DateTime::<Utc>::from_timestamp_millis(unix_ms)
        .ok_or(Error::Time)?
        .with_timezone(&offset);
    if !(2020..=2099).contains(&date.year()) {
        return Err(Error::Time);
    }
    Ok(Scope {
        period,
        page,
        detail,
        date: date.format("%Y-%m-%d").to_string(),
        timezone_minutes: tz,
    })
}
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn range(s: &Scope, chart: bool) -> Result<(NaiveDate, NaiveDate), Error> {
    let date = NaiveDate::parse_from_str(&s.date, "%Y-%m-%d").map_err(|_| Error::Time)?;
    let start = match s.period {
        Period::Day => date,
        Period::Month => date.with_day(1).unwrap(),
        Period::Total if chart => date - Duration::days(89),
        Period::Total => NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
    };
    Ok((start, date))
}
pub fn query(s: &Scope, chart: bool) -> Result<String, Error> {
    let (start, _) = range(s, chart)?;
    // Use IANA identifiers; numeric offsets may silently fall back on servers.
    let tz = if s.timezone_minutes == 480 {
        "Asia/Shanghai".to_string()
    } else if s.timezone_minutes == 0 {
        "UTC".to_string()
    } else if s.timezone_minutes % 60 == 0 && (-720..=840).contains(&s.timezone_minutes) {
        format!(
            "Etc/GMT{}{hour}",
            if s.timezone_minutes > 0 { "-" } else { "+" },
            hour = s.timezone_minutes.abs() / 60
        )
    } else {
        return Err(Error::Timezone);
    };
    let mut q = format!(
        "start_date={start}&end_date={}&timezone={}&granularity={}",
        s.date,
        encode(&tz),
        if s.period == Period::Day && chart {
            "hour"
        } else {
            "day"
        }
    );
    if let Some(id) = s.detail {
        if id <= 0 {
            return Err(Error::Format);
        }
        let key = match s.page {
            Page::Users => "user_id",
            Page::Accounts => "account_id",
            _ => return Err(Error::Format),
        };
        q.push_str(&format!("&{key}={id}"));
    }
    Ok(q)
}
fn timestamp(v: &Value, key: &str) -> Option<i64> {
    v[key]
        .as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis())
}
fn window(v: &Value, percent: &str, reset: &str, minutes: &str) -> Option<Window> {
    let used = v[percent].as_f64().filter(|n| n.is_finite() && *n >= 0.0)?;
    let window_minutes = v[minutes].as_u64().and_then(|n| u32::try_from(n).ok());
    Some(Window {
        used,
        reset_ms: timestamp(v, reset),
        window_minutes,
    })
}
fn plan_label(v: &Value, platform: &str) -> String {
    let value = [
        string(&v["credentials"], "plan_type", 40),
        string(v, "parent_plan_type", 40),
    ]
    .into_iter()
    .map(|s| s.trim().to_string())
    .find(|s| !s.is_empty())
    .unwrap_or_default();
    let normalized: String = value
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '_' | '-'))
        .flat_map(char::to_lowercase)
        .collect();
    let label = if platform == "openai" {
        match normalized.as_str() {
            "prolite" | "pro100" => "Pro 100",
            "pro" | "chatgptpro" | "pro200" => "Pro 200",
            "promax" | "pro500" => "Pro 500",
            "pro20x" => "Pro 20X",
            "free" => "Free",
            "go" => "Go",
            "plus" => "Plus",
            "team" | "selfservebusinessusagebased" => "Business",
            "selfservebusinessprolite" => "Business Premium",
            "business" | "enterprise" | "ent26" | "enterprisecbpusagebased" => "Enterprise",
            "enterprisecbpautomation" => "Enterprise (Automation)",
            "edu" => "Edu",
            "eduplus" => "Edu Plus",
            "edupro" => "Edu Pro",
            "" | "unknown" => "UNKNOWN",
            _ => return value.chars().take(24).collect(),
        }
    } else {
        match normalized.as_str() {
            "pro" => "PRO",
            "plus" => "PLUS",
            "free" => "FREE",
            "team" => "TEAM",
            "business" => "BUSINESS",
            "enterprise" => "ENTERPRISE",
            "edu" => "EDU",
            "" | "unknown" => "UNKNOWN",
            _ => return value.chars().take(24).collect(),
        }
    };
    label.into()
}
pub fn accounts(t: &mut impl Transport, c: &Config) -> Result<Vec<Account>, Error> {
    let mut out = Vec::new();
    let mut expected = None;
    for page in 1..=8 {
        let v = request(
            t,
            c,
            &format!("/admin/accounts?page={page}&page_size=25&lite=1"),
            None,
        )?;
        let total = uint(&v, "total")?;
        if total > MAX_ACCOUNTS as u64 {
            return Err(Error::TooLarge);
        }
        let total = total as usize;
        if expected.is_some_and(|n| n != total) {
            return Err(Error::Format);
        }
        expected = Some(total);
        let rows = array(&v, "items", 25)?;
        for row in rows {
            let id = row["id"].as_i64().filter(|n| *n > 0).ok_or(Error::Format)?;
            if out.iter().any(|a: &Account| a.id == id) {
                return Err(Error::Format);
            }
            let platform = string(row, "platform", 24);
            out.push(Account {
                id,
                label: masked(&string(row, "name", 80), id),
                platform: platform.clone(),
                kind: string(row, "type", 24),
                plan_label: plan_label(row, &platform),
                usage_updated_ms: if platform == "openai" {
                    timestamp(&row["extra"], "codex_usage_updated_at")
                } else {
                    None
                },
                five: if platform == "openai" {
                    window(
                        &row["extra"],
                        "codex_5h_used_percent",
                        "codex_5h_reset_at",
                        "codex_5h_window_minutes",
                    )
                } else {
                    None
                },
                seven: if platform == "openai" {
                    window(
                        &row["extra"],
                        "codex_7d_used_percent",
                        "codex_7d_reset_at",
                        "codex_7d_window_minutes",
                    )
                } else {
                    None
                },
            });
        }
        if out.len() == total {
            return Ok(out);
        }
        if rows.is_empty() || out.len() > total {
            return Err(Error::Format);
        }
    }
    Err(Error::TooLarge)
}
fn stats(t: &mut impl Transport, c: &Config, s: &Scope) -> Result<(Totals, bool), Error> {
    let v = request(t, c, "/admin/dashboard/stats", None)?;
    let prefix = if s.period == Period::Day {
        "today"
    } else {
        "total"
    };
    Ok((
        Totals {
            tokens: uint(&v, &format!("{prefix}_tokens"))?,
            cost: money(&v, &format!("{prefix}_actual_cost"))?,
            requests: Some(uint(&v, &format!("{prefix}_requests"))?),
        },
        v["stats_stale"].as_bool().unwrap_or(false),
    ))
}
fn bucket_time(value: &str, offset: FixedOffset) -> Result<NaiveDateTime, Error> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return Ok(value.with_timezone(&offset).naive_local());
    }
    for pattern in [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(value) = NaiveDateTime::parse_from_str(value, pattern) {
            return Ok(value);
        }
    }
    Err(Error::Format)
}
/// Normalize only a completed API response. Missing buckets in that successful
/// response are zeros, while transport/parse failures remain errors to the UI.
pub fn normalize_points(
    source: Vec<Point>,
    s: &Scope,
    sampled_ms: i64,
) -> Result<Vec<Point>, Error> {
    let (start, end) = range(s, true)?;
    let offset = FixedOffset::east_opt(s.timezone_minutes.checked_mul(60).ok_or(Error::Time)?)
        .ok_or(Error::Time)?;
    let observed = DateTime::<Utc>::from_timestamp_millis(sampled_ms)
        .ok_or(Error::Time)?
        .with_timezone(&offset);
    if !(2020..=2099).contains(&observed.year()) || end > observed.date_naive() {
        return Err(Error::Time);
    }
    let mut indexed = std::collections::BTreeMap::new();
    for point in source {
        let (date, hour) = if s.period == Period::Day {
            let time = bucket_time(&point.date, offset)?;
            if time.minute() != 0 || time.second() != 0 || time.nanosecond() != 0 {
                return Err(Error::Format);
            }
            (time.date(), Some(time.hour()))
        } else if let Ok(date) = NaiveDate::parse_from_str(&point.date, "%Y-%m-%d") {
            (date, None)
        } else {
            (bucket_time(&point.date, offset)?.date(), None)
        };
        if date < start || date > end {
            return Err(Error::Format);
        }
        // A full-day template may include empty future buckets. Actual usage
        // beyond the observation hour is incompatible with this snapshot;
        // report it rather than silently losing recorded tokens or charges.
        if s.period == Period::Day
            && date == observed.date_naive()
            && hour.is_some_and(|hour| hour > observed.hour())
            && (point.totals.tokens != 0
                || point.totals.cost != 0.0
                || point.totals.requests.is_some_and(|n| n != 0))
        {
            return Err(Error::Format);
        }
        let key = match hour {
            Some(hour) => format!("{date} {hour:02}:00"),
            None => date.to_string(),
        };
        if indexed.insert(key, point.totals).is_some() {
            return Err(Error::Format);
        }
    }
    let zero = || Totals {
        requests: Some(0),
        ..Default::default()
    };
    if s.period == Period::Day {
        let upper = if end == observed.date_naive() {
            observed.hour()
        } else {
            23
        };
        return Ok((0..=upper)
            .map(|hour| {
                let date = format!("{end} {hour:02}:00");
                let totals = indexed.remove(&date).unwrap_or_else(zero);
                Point { date, totals }
            })
            .collect());
    }
    let mut cursor = start;
    let mut out = Vec::new();
    while cursor <= end {
        let date = cursor.to_string();
        let totals = indexed.remove(&date).unwrap_or_else(zero);
        out.push(Point { date, totals });
        cursor = cursor.succ_opt().ok_or(Error::Time)?;
    }
    Ok(out)
}
fn points(t: &mut impl Transport, c: &Config, s: &Scope, now: i64) -> Result<Vec<Point>, Error> {
    let v = request(
        t,
        c,
        &format!("/admin/dashboard/trend?{}", query(s, true)?),
        None,
    )?;
    let mut out = Vec::new();
    for row in array(&v, "trend", 240)? {
        let date = string(row, "date", 40);
        if date.is_empty() || out.iter().any(|p: &Point| p.date == date) {
            return Err(Error::Format);
        }
        out.push(Point {
            date,
            totals: totals(row)?,
        });
    }
    normalize_points(out, s, now)
}
fn models(t: &mut impl Transport, c: &Config, s: &Scope) -> Result<Vec<Model>, Error> {
    let v = request(
        t,
        c,
        &format!("/admin/dashboard/models?{}", query(s, false)?),
        None,
    )?;
    let mut out = Vec::new();
    for row in array(&v, "models", 200)? {
        let name = string(row, "model", 80);
        if name.is_empty() {
            return Err(Error::Format);
        }
        out.push(Model {
            name,
            totals: totals(row)?,
            input: uint(row, "input_tokens")?,
            output: uint(row, "output_tokens")?,
            cache: uint(row, "cache_read_tokens")?,
        });
    }
    out.sort_by(|a, b| b.totals.tokens.cmp(&a.totals.tokens));
    Ok(out)
}
fn sum<'a>(rows: impl Iterator<Item = &'a Totals>) -> Result<Totals, Error> {
    let mut out = Totals {
        requests: Some(0),
        ..Default::default()
    };
    for row in rows {
        out.tokens = out.tokens.checked_add(row.tokens).ok_or(Error::Format)?;
        out.cost += row.cost;
        out.requests = out
            .requests
            .zip(row.requests)
            .and_then(|(a, b)| a.checked_add(b));
    }
    if !out.cost.is_finite() {
        return Err(Error::Format);
    }
    Ok(out)
}
fn warn(data: &mut Data, section: &str, r: Result<(), Error>) {
    if let Err(e) = r {
        let label = format!("{section}未更新");
        match &mut data.warning {
            Some(message) if !message.contains(&label) => {
                message.push_str(" · ");
                message.push_str(&label);
            }
            None => data.warning = Some(format!("{label}：{}", e.message())),
            _ => {}
        }
    }
}

/// A bounded live update for the currently observed DAY overview. Account
/// membership, quotas, models and yesterday remain the last full snapshot;
/// only online headline/trend endpoints are sampled here. The caller must also
/// keep the snapshot associated with its exact site/key and Scope.
pub fn fetch_headline(
    t: &mut impl Transport,
    c: &Config,
    s: &Scope,
    now: i64,
    base: &Data,
) -> Result<Data, Error> {
    if s.page != Page::Overview
        || s.period != Period::Day
        || s.detail.is_some()
        || scope(s.period, s.page, s.detail, now, s.timezone_minutes)? != *s
        || scope(
            s.period,
            s.page,
            s.detail,
            base.sampled_ms,
            s.timezone_minutes,
        )? != *s
    {
        return Err(Error::Format);
    }
    if base.accounts.len() > MAX_ACCOUNTS {
        return Err(Error::TooLarge);
    }
    let mut d = base.clone();
    // Live section warnings are reevaluated each sample, while slower quota,
    // model and yesterday warnings stay visible until their full refresh.
    d.warning = base.warning.as_ref().and_then(|warning| {
        let kept = warning
            .split(" · ")
            .filter(|part| {
                !["全站统计未更新", "今日账号汇总未更新", "趋势未更新"]
                    .iter()
                    .any(|prefix| part.starts_with(prefix))
            })
            .collect::<Vec<_>>()
            .join(" · ");
        (!kept.is_empty()).then_some(kept)
    });
    d.sampled_ms = now;
    let global = stats(t, c, s);
    if let Ok((totals, stale)) = &global {
        d.totals = Some(totals.clone());
        d.total_label = "全站统计".into();
        d.server_stale = *stale;
    }
    if d.accounts.is_empty() {
        global?;
    } else {
        let batch = today_account_totals(t, c, &d.accounts);
        match batch {
            Ok(totals) => {
                d.totals = Some(totals);
                d.total_label = "当前账号汇总".into();
                // A failed secondary endpoint cannot discard the successful
                // authoritative account headline.
                if let Err(e) = global {
                    d.server_stale = false;
                    warn(&mut d, "全站统计", Err(e));
                }
            }
            Err(e) => {
                // Only explicitly marked global fallback may replace the
                // account aggregate; otherwise retain the entire old snapshot.
                if global.is_err() {
                    return Err(e);
                }
                warn(&mut d, "今日账号汇总", Err(e));
            }
        }
    }
    match points(t, c, s, now) {
        Ok(p) => d.trend = p,
        Err(e) => warn(&mut d, "趋势", Err(e)),
    }
    Ok(d)
}
fn today_account_totals(
    t: &mut impl Transport,
    c: &Config,
    accounts: &[Account],
) -> Result<Totals, Error> {
    if accounts.is_empty()
        || accounts.len() > MAX_ACCOUNTS
        || accounts.iter().any(|a| a.id <= 0)
        || accounts
            .iter()
            .enumerate()
            .any(|(i, a)| accounts[..i].iter().any(|b| b.id == a.id))
    {
        return Err(Error::Format);
    }
    let body = serde_json::json!({
        "account_ids": accounts.iter().map(|a| a.id).collect::<Vec<_>>()
    })
    .to_string();
    let v = request(t, c, "/admin/accounts/today-stats/batch", Some(&body))?;
    let rows = v["stats"].as_object().ok_or(Error::Format)?;
    let mut totals_rows = Vec::with_capacity(accounts.len());
    for a in accounts {
        let r = rows.get(&a.id.to_string()).ok_or(Error::Format)?;
        totals_rows.push(Totals {
            tokens: uint(r, "tokens")?,
            cost: money(r, "user_cost")?,
            requests: Some(uint(r, "requests")?),
        });
    }
    sum(totals_rows.iter())
}

pub fn fetch(t: &mut impl Transport, c: &Config, s: &Scope, now: i64) -> Result<Data, Error> {
    let mut d = Data {
        sampled_ms: now,
        total_label: "全站统计".into(),
        trend_label: if s.period == Period::Total {
            "近 90 天趋势"
        } else {
            "用量趋势"
        }
        .into(),
        ..Default::default()
    };
    if s.page == Page::Connection {
        return Err(Error::Format);
    }
    if s.detail.is_some() {
        d.models = models(t, c, s)?;
        d.totals = Some(sum(d.models.iter().map(|m| &m.totals))?);
        d.total_label = if s.page == Page::Users {
            "当前用户"
        } else {
            "账号模型汇总"
        }
        .into();
        match points(t, c, s, now) {
            Ok(p) => d.trend = p,
            Err(e) => warn(&mut d, "趋势", Err(e)),
        };
        return Ok(d);
    }
    match s.page {
        Page::Overview => {
            if s.period != Period::Month {
                let (a, b) = stats(t, c, s)?;
                d.totals = Some(a);
                d.server_stale = b;
            }
            // Quota is useful on every overview range. Failure here must not
            // discard the separately obtained live headline or month totals.
            match accounts(t, c) {
                Ok(accounts) => d.accounts = accounts,
                Err(e) => warn(&mut d, "额度", Err(e)),
            }
            // Mac's latest DAY headline uses the authoritative per-account batch.
            if s.period == Period::Day && !d.accounts.is_empty() {
                let result = today_account_totals(t, c, &d.accounts).map(|totals| {
                    d.totals = Some(totals);
                    d.total_label = "当前账号汇总".into();
                });
                // Do not silently label fallback global numbers as account-scoped data.
                warn(&mut d, "今日账号汇总", result);
            }
            match points(t, c, s, now) {
                Ok(p) => {
                    if s.period == Period::Month {
                        d.totals = Some(sum(p.iter().map(|p| &p.totals))?);
                    }
                    d.trend = p;
                }
                Err(e) => {
                    if d.totals.is_none() {
                        return Err(e);
                    }
                    warn(&mut d, "趋势", Err(e));
                }
            }
            match models(t, c, s) {
                Ok(m) => d.models = m,
                Err(e) => warn(&mut d, "模型", Err(e)),
            }
            if s.period == Period::Day {
                let mut y = s.clone();
                y.date = (NaiveDate::parse_from_str(&s.date, "%Y-%m-%d")
                    .map_err(|_| Error::Time)?
                    - Duration::days(1))
                .to_string();
                // Yesterday's headline and comparison curve cover all 24
                // hours; today's observed buckets share that fixed day axis.
                match points(t, c, &y, now) {
                    Ok(p) => {
                        d.yesterday = Some(sum(p.iter().map(|p| &p.totals))?);
                        d.yesterday_trend = p;
                    }
                    Err(e) => warn(&mut d, "昨日", Err(e)),
                }
            }
        }
        Page::Accounts => {
            d.accounts = accounts(t, c)?;
            for a in &mut d.accounts {
                if a.platform == "anthropic" && matches!(a.kind.as_str(), "oauth" | "setup-token") {
                    match request(
                        t,
                        c,
                        &format!("/admin/accounts/{}/usage?source=passive", a.id),
                        None,
                    ) {
                        Ok(v) if v["error"].is_null() => {
                            a.five = window(
                                &v["five_hour"],
                                "utilization",
                                "resets_at",
                                "window_minutes",
                            );
                            a.seven = window(
                                &v["seven_day"],
                                "utilization",
                                "resets_at",
                                "window_minutes",
                            );
                            a.usage_updated_ms = timestamp(&v, "updated_at");
                        }
                        Ok(_) => {
                            d.warning
                                .get_or_insert_with(|| "部分账号额度暂不可用".into());
                        }
                        Err(e) => {
                            // Mutable account borrow cannot overlap all of Data.
                            let label = format!("部分账号额度未更新：{}", e.message());
                            match &mut d.warning {
                                Some(message) => message.push_str(" · 部分账号额度未更新"),
                                None => d.warning = Some(label),
                            }
                            break;
                        }
                    }
                }
            }
        }
        Page::Models => {
            d.models = models(t, c, s)?;
            d.totals = Some(sum(d.models.iter().map(|m| &m.totals))?);
        }
        Page::Users => {
            let v = request(
                t,
                c,
                &format!(
                    "/admin/dashboard/user-breakdown?{}&limit=200",
                    query(s, false)?
                ),
                None,
            )?;
            for r in array(&v, "users", 200)? {
                let id = r["user_id"]
                    .as_i64()
                    .filter(|n| *n > 0)
                    .ok_or(Error::Format)?;
                if d.users.iter().any(|u| u.id == id) {
                    return Err(Error::Format);
                }
                d.users.push(User {
                    id,
                    label: masked(&string(r, "email", 80), id),
                    totals: totals(r)?,
                });
            }
            d.users
                .sort_by(|a, b| b.totals.tokens.cmp(&a.totals.tokens));
            d.totals = Some(sum(d.users.iter().map(|u| &u.totals))?);
            d.total_label = "当前范围用户汇总".into();
            if d.users.len() == 200 {
                d.warning = Some("显示前 200 位用户".into());
                d.total_label = "前 200 位用户汇总".into();
            }
        }
        Page::Connection => unreachable!(),
    }
    Ok(d)
}
pub fn validate(t: &mut impl Transport, c: &Config, s: &Scope) -> Result<(), Error> {
    stats(t, c, s)?;
    accounts(t, c)?;
    Ok(())
}
