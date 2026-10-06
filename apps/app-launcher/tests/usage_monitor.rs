use app_launcher::usage::{
    api::{self, Error, Transport},
    *,
};
use serde_json::{json, Value};
#[derive(Default)]
struct Mock {
    paths: Vec<String>,
    bad: Option<&'static str>,
    bad_second: Option<&'static str>,
    yesterday_bad: bool,
    trend_override: Option<Vec<Value>>,
    page_duplicate: bool,
}
impl Transport for Mock {
    fn request(&mut self, _: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        self.paths.push(path.into());
        if self.bad.is_some_and(|p| path.starts_with(p))
            || self.bad_second.is_some_and(|p| path.starts_with(p))
            || (self.yesterday_bad && path.contains("start_date=2026-10-05"))
        {
            return Err(Error::Authentication);
        }
        let data = if path == "/admin/dashboard/stats" {
            json!({"today_tokens":100,"total_tokens":900,"today_actual_cost":1.0,"total_actual_cost":9.0,"today_requests":4,"total_requests":40})
        } else if path.starts_with("/admin/accounts?") {
            json!({"total":if self.page_duplicate{2}else{1},"items":[{"id":1,"name":"secret@example.com","platform":"openai","type":"oauth","credentials":{"access_token":"NEVER_RETAIN","plan_type":"pro"},"extra":{"codex_5h_used_percent":25,"codex_5h_window_minutes":300,"codex_7d_used_percent":40,"codex_7d_window_minutes":10080,"codex_7d_reset_at":"2026-10-08T00:00:00Z","codex_usage_updated_at":"2026-10-06T02:00:00Z","codex_primary_used_percent":99}}]})
        } else if path == "/admin/accounts/today-stats/batch" {
            assert_eq!(
                serde_json::from_str::<Value>(body.unwrap()).unwrap(),
                json!({"account_ids":[1]})
            );
            json!({"stats":{"1":{"tokens":120,"user_cost":2.5,"requests":8}}})
        } else if path.starts_with("/admin/dashboard/trend?") {
            if let Some(rows) = &self.trend_override {
                json!({"trend": rows})
            } else {
                json!({"trend":[{"date":if path.contains("start_date=2026-10-05"){"2026-10-05 10:00"}else{"2026-10-06 10:00"},"total_tokens":50,"actual_cost":0.5,"requests":2}]})
            }
        } else if path.starts_with("/admin/dashboard/models?") {
            json!({"models":[{"model":"model-a","total_tokens":50,"actual_cost":0.5,"input_tokens":20,"output_tokens":10,"cache_read_tokens":20}]})
        } else if path.starts_with("/admin/dashboard/user-breakdown?") {
            json!({"users":[{"user_id":2,"email":"user@example.com","total_tokens":50,"actual_cost":0.5}]})
        } else {
            panic!("unexpected API path")
        };
        Ok(serde_json::to_vec(&json!({"code":0,"data":data})).unwrap())
    }
}
fn c() -> Config {
    Config::new("https://monitor.example/api/v1/", "test-key").unwrap()
}
fn observed_ms() -> i64 {
    // 2026-10-06 12:30 Asia/Shanghai: all mocked 10:00 data is elapsed.
    1_791_261_000_000
}
fn scope(page: Page, period: Period) -> Scope {
    Scope {
        page,
        period,
        detail: None,
        date: "2026-10-06".into(),
        timezone_minutes: 480,
    }
}
#[test]
fn configuration_is_https_bounded_and_redacted() {
    assert_eq!(c().site, "https://monitor.example");
    for s in [
        "http://example.com",
        "https://me:password@example.com",
        "https://example.com?key=1",
        "https://example.com/a/../b",
        "https://a\r\nx-api-key: fake",
        "https://example.com:0",
    ] {
        assert!(Config::new(s, "key").is_err());
    }
    for key in ["", "abc\r\ninject", "a b"] {
        assert!(Config::new("example.com", key).is_err());
    }
    assert!(!format!("{:?}", c()).contains("test-key"));
    let message = p4desk_protocol::HostMessage::MonitorConfigure {
        request_id: 4,
        site: c().site,
        key: p4desk_protocol::MonitorKey(c().key),
    };
    assert!(!format!("{message:?}").contains("test-key"));
}
#[test]
fn headline_uses_authoritative_account_batch_and_actual_cost() {
    let mut t = Mock::default();
    let d = api::fetch(
        &mut t,
        &c(),
        &scope(Page::Overview, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert_eq!(d.totals.as_ref().unwrap().tokens, 120);
    assert_eq!(d.totals.as_ref().unwrap().cost, 2.5);
    assert_eq!(d.total_label, "当前账号汇总");
    assert_eq!(d.yesterday.as_ref().unwrap().tokens, 50);
    assert_eq!(d.yesterday_trend.len(), 24);
    assert_eq!(d.trend.len(), 13);
    assert!(!serde_json::to_string(&d.accounts)
        .unwrap()
        .contains("NEVER_RETAIN"));
    assert!(!serde_json::to_string(&d.accounts)
        .unwrap()
        .contains("example.com"));
    assert_eq!(d.accounts[0].five.as_ref().unwrap().remaining(), 75.0);
    assert_eq!(d.accounts[0].plan_label, "Pro 200");
    assert_eq!(
        d.accounts[0].five.as_ref().unwrap().window_minutes,
        Some(300)
    );
    assert!(d.accounts[0].usage_updated_ms.is_some());
}
#[test]
fn range_scope_is_explicit_and_total_chart_is_bounded() {
    let day = api::query(&scope(Page::Overview, Period::Day), true).unwrap();
    assert!(day.contains("granularity=hour"));
    assert!(day.contains("timezone=Asia%2FShanghai"));
    assert!(api::query(&scope(Page::Overview, Period::Month), false)
        .unwrap()
        .contains("start_date=2026-10-01"));
    assert!(api::query(&scope(Page::Models, Period::Total), false)
        .unwrap()
        .contains("start_date=1970-01-01"));
    assert!(!api::query(&scope(Page::Overview, Period::Total), true)
        .unwrap()
        .contains("1970"));
    let s = api::scope(Period::Day, Page::Users, Some(2), 1_791_216_000_000, 480).unwrap();
    assert!(api::query(&s, true).unwrap().contains("user_id=2"));
    assert!(api::scope(Period::Day, Page::Overview, None, 0, 480).is_err());
}
#[test]
fn secondary_error_keeps_headline_and_flags_stale_data() {
    let mut t = Mock {
        bad: Some("/admin/dashboard/models"),
        ..Default::default()
    };
    let d = api::fetch(
        &mut t,
        &c(),
        &scope(Page::Overview, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert!(d.totals.is_some());
    assert!(d.models.is_empty());
    assert!(d.warning.is_some());
    t.bad = Some("/admin/dashboard/stats");
    assert!(api::fetch(&mut t, &c(), &scope(Page::Overview, Period::Day), 1).is_err());
}
#[test]
fn rejected_today_batch_keeps_verified_quota_and_labels_global_fallback() {
    let mut t = Mock {
        bad: Some("/admin/accounts/today-stats/batch"),
        ..Default::default()
    };
    let data = api::fetch(
        &mut t,
        &c(),
        &scope(Page::Overview, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert_eq!(data.totals.as_ref().unwrap().tokens, 100);
    assert_eq!(data.total_label, "全站统计");
    assert_eq!(data.accounts.len(), 1);
    assert_eq!(data.accounts[0].five.as_ref().unwrap().remaining(), 75.0);
    assert!(data
        .warning
        .as_deref()
        .unwrap()
        .contains("今日账号汇总未更新"));
}
#[test]
fn duplicate_account_pages_cannot_double_count() {
    assert_eq!(
        api::accounts(
            &mut Mock {
                page_duplicate: true,
                ..Default::default()
            },
            &c()
        )
        .unwrap_err(),
        Error::Format
    );
}
#[test]
fn no_primary_quota_guess_or_expiry_reset() {
    let a = api::accounts(&mut Mock::default(), &c()).unwrap();
    assert_eq!(a[0].seven.as_ref().unwrap().remaining(), 60.0);
    let w = Window {
        used: 70.0,
        reset_ms: Some(1),
        window_minutes: None,
    };
    assert_eq!(w.caption(2), "等待额度更新");
    assert_eq!(w.remaining(), 30.0);
    assert_eq!(w.remaining_time_ratio(2, 300), Some(0.0));
}
fn point(date: &str, tokens: u64) -> Point {
    Point {
        date: date.into(),
        totals: Totals {
            tokens,
            cost: tokens as f64 / 100.0,
            requests: Some(1),
        },
    }
}
#[test]
fn hourly_timeline_is_local_time_aligned_and_validates_every_bucket() {
    let s = scope(Page::Overview, Period::Day);
    let points = api::normalize_points(
        vec![
            point("2026-10-06T02:00:00Z", 50),
            point("2026-10-06 00:00", 10),
            point("2026-10-06T12:00:00+08:00", 20),
        ],
        &s,
        observed_ms(),
    )
    .unwrap();
    assert_eq!(points.len(), 13);
    assert_eq!(points[0].date, "2026-10-06 00:00");
    assert_eq!(points[0].totals.tokens, 10);
    assert_eq!(points[1].totals.tokens, 0);
    assert_eq!(points[1].totals.requests, Some(0));
    assert_eq!(points[10].totals.tokens, 50);
    assert_eq!(points[12].totals.tokens, 20);
    for invalid in [
        "2026-10-05 10:00",
        "2026-10-07 10:00",
        "2026-10-06 25:00",
        "2026-10-06 10:30",
        "2026-10-06 10:00:01",
        "2026-10-06",
        "invalid",
    ] {
        assert_eq!(
            api::normalize_points(vec![point(invalid, 1)], &s, observed_ms()).unwrap_err(),
            Error::Format
        );
    }
    // Equivalent timezone/local spellings identify one bucket, not two samples.
    assert_eq!(
        api::normalize_points(
            vec![
                point("2026-10-06T02:00:00Z", 1),
                point("2026-10-06 10:00", 2)
            ],
            &s,
            observed_ms()
        )
        .unwrap_err(),
        Error::Format
    );
}
#[test]
fn future_usage_is_rejected_but_empty_future_template_buckets_are_ignored() {
    let s = scope(Page::Overview, Period::Day);
    assert_eq!(
        api::normalize_points(vec![point("2026-10-06 13:00", 1)], &s, observed_ms()).unwrap_err(),
        Error::Format
    );
    let future = Point {
        date: "2026-10-06 13:00".into(),
        totals: Totals {
            requests: Some(0),
            ..Default::default()
        },
    };
    let normalized = api::normalize_points(vec![future], &s, observed_ms()).unwrap();
    assert_eq!(normalized.len(), 13);
    assert!(normalized.iter().all(|p| p.totals.tokens == 0));
    let mut t = Mock {
        trend_override: Some(vec![
            json!({"date":"2026-10-06 13:00","total_tokens":1,"actual_cost":0.1,"requests":1}),
        ]),
        ..Default::default()
    };
    let data = api::fetch(&mut t, &c(), &s, observed_ms()).unwrap();
    assert_eq!(data.totals.as_ref().unwrap().tokens, 120);
    assert!(data.trend.is_empty());
    assert!(data.warning.as_deref().unwrap().contains("趋势未更新"));
}
#[test]
fn month_start_accepts_a_timestamped_daily_bucket_without_changing_scope() {
    let mut s = scope(Page::Overview, Period::Month);
    s.date = "2026-10-01".into();
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-01T12:30:00+08:00")
        .unwrap()
        .timestamp_millis();
    let data = api::normalize_points(vec![point("2026-10-01 10:00", 42)], &s, now).unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].date, "2026-10-01");
    assert_eq!(data[0].totals.tokens, 42);
}
#[test]
fn users_summary_covers_returned_rows_and_has_an_explicit_scope_label() {
    let data = api::fetch(
        &mut Mock::default(),
        &c(),
        &scope(Page::Users, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert_eq!(data.totals.as_ref().unwrap().tokens, 50);
    assert_eq!(data.totals.as_ref().unwrap().cost, 0.5);
    assert_eq!(data.total_label, "当前范围用户汇总");
}
#[test]
fn yesterday_has_all_hours_while_comparison_uses_only_elapsed_hours() {
    let mut s = scope(Page::Overview, Period::Day);
    s.date = "2026-10-05".into();
    let points = api::normalize_points(
        vec![point("2026-10-05 10:00", 50), point("2026-10-05 23:00", 25)],
        &s,
        observed_ms(),
    )
    .unwrap();
    assert_eq!(points.len(), 24);
    assert_eq!(points.iter().map(|p| p.totals.tokens).sum::<u64>(), 75);
    assert_eq!(
        points.iter().take(13).map(|p| p.totals.tokens).sum::<u64>(),
        50
    );
    assert_eq!(points[23].date, "2026-10-05 23:00");
}
#[test]
fn completed_empty_day_is_zero_but_failed_yesterday_stays_missing() {
    let mut empty = Mock {
        trend_override: Some(vec![]),
        ..Default::default()
    };
    let data = api::fetch(
        &mut empty,
        &c(),
        &scope(Page::Overview, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert_eq!(data.trend.len(), 13);
    assert_eq!(data.yesterday_trend.len(), 24);
    assert_eq!(data.yesterday.as_ref().unwrap().tokens, 0);
    assert_eq!(data.yesterday.as_ref().unwrap().requests, Some(0));
    assert!(data.warning.is_none());
    let mut failed = Mock {
        yesterday_bad: true,
        ..Default::default()
    };
    let data = api::fetch(
        &mut failed,
        &c(),
        &scope(Page::Overview, Period::Day),
        observed_ms(),
    )
    .unwrap();
    assert!(data.yesterday.is_none());
    assert!(data.yesterday_trend.is_empty());
    assert_eq!(data.totals.as_ref().unwrap().tokens, 120);
    assert!(data.warning.as_deref().unwrap().contains("昨日未更新"));
}
#[test]
fn month_and_total_keep_headline_if_multiple_auxiliary_sections_fail() {
    for period in [Period::Month, Period::Total] {
        let mut t = Mock {
            bad: Some("/admin/accounts?"),
            bad_second: Some("/admin/dashboard/models?"),
            ..Default::default()
        };
        let d = api::fetch(&mut t, &c(), &scope(Page::Overview, period), observed_ms()).unwrap();
        assert!(d.totals.is_some());
        assert!(d.accounts.is_empty());
        assert!(d.models.is_empty());
        assert_eq!(d.trend.len(), if period == Period::Month { 6 } else { 90 });
        let warning = d.warning.as_deref().unwrap();
        assert!(warning.contains("额度未更新"));
        assert!(warning.contains("模型未更新"));
        assert!(!warning.contains("test-key"));
        assert!(t.paths.iter().any(|p| p.starts_with("/admin/accounts?")));
    }
    let d = api::fetch(
        &mut Mock::default(),
        &c(),
        &scope(Page::Overview, Period::Month),
        observed_ms(),
    )
    .unwrap();
    assert_eq!(d.accounts.len(), 1);
    assert_eq!(d.totals.as_ref().unwrap().tokens, 50);
}
#[test]
fn window_time_ratio_does_not_reset_expired_quota_or_guess_missing_reset() {
    let w = Window {
        used: 90.0,
        reset_ms: Some(120_000),
        window_minutes: Some(2),
    };
    assert_eq!(w.remaining_time_ratio(60_000, 300), Some(0.5));
    assert_eq!(w.remaining_time_ratio(120_001, 300), Some(0.0));
    assert_eq!(w.remaining(), 10.0);
    let mut w = w;
    w.window_minutes = Some(0);
    assert_eq!(w.remaining_time_ratio(60_000, 300), None);
    w.window_minutes = None;
    assert_eq!(w.remaining_time_ratio(60_000, 2), Some(0.5));
    w.reset_ms = None;
    assert_eq!(w.remaining_time_ratio(60_000, 300), None);
}
#[test]
fn old_cached_fields_load_with_safe_defaults() {
    let account: Account = serde_json::from_value(json!({"id":1,"label":"ab********","platform":"openai","kind":"oauth","five":{"used":25.0,"reset_ms":null},"seven":null})).unwrap();
    assert!(account.plan_label.is_empty());
    assert!(account.usage_updated_ms.is_none());
    assert!(account.five.as_ref().unwrap().window_minutes.is_none());
    let mut old_data = serde_json::to_value(Data::default()).unwrap();
    old_data.as_object_mut().unwrap().remove("yesterday_trend");
    old_data["accounts"] = json!([account]);
    let restored: Data = serde_json::from_value(old_data).unwrap();
    assert!(restored.yesterday_trend.is_empty());
    assert_eq!(restored.accounts.len(), 1);
}
#[test]
fn account_detail_is_explicitly_model_sum() {
    let mut s = scope(Page::Accounts, Period::Day);
    s.detail = Some(1);
    let data = api::fetch(&mut Mock::default(), &c(), &s, observed_ms()).unwrap();
    assert_eq!(data.total_label, "账号模型汇总");
    assert_eq!(data.totals.as_ref().unwrap().tokens, 50);
}
#[test]
fn input_size_and_missing_fields_do_not_become_zero() {
    struct T(Vec<u8>);
    impl Transport for T {
        fn request(&mut self, _: &Config, _: &str, _: Option<&str>) -> Result<Vec<u8>, Error> {
            Ok(self.0.clone())
        }
    }
    let mut t = T(vec![b' '; api::MAX_RESPONSE + 1]);
    assert_eq!(
        api::fetch(&mut t, &c(), &scope(Page::Models, Period::Day), 1).unwrap_err(),
        Error::TooLarge
    );
    t.0 = br#"{"data":{"models":[{"model":"a","total_tokens":1,"cost":12}]}}"#.to_vec();
    assert_eq!(
        api::fetch(&mut t, &c(), &scope(Page::Models, Period::Day), 1).unwrap_err(),
        Error::Format
    );
}
#[test]
fn persistence_separates_credentials_cache_and_recovers_after_restart() {
    let root = std::env::temp_dir().join(format!("p4-monitor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let local = root.join("flash");
    let sd = root.join("sd");
    std::fs::create_dir_all(&sd).unwrap();
    let store = app_launcher::GenerationStore::new(&sd, &local).monitor_store();
    store.save_config(Some(&c())).unwrap();
    assert_eq!(store.load_config().unwrap(), Some(c()));
    let s = scope(Page::Users, Period::Day);
    let d = api::fetch(&mut Mock::default(), &c(), &s, 1).unwrap();
    store.save_cache(&c(), &s, &d).unwrap();
    assert!(store.load_cache(&c(), &s).is_some());
    assert!(store
        .load_cache(&Config::new("other.example", "key").unwrap(), &s)
        .is_none());
    assert!(store
        .load_cache(&c(), &scope(Page::Users, Period::Month))
        .is_none());
    for file in std::fs::read_dir(&sd).unwrap() {
        let text = std::fs::read_to_string(file.unwrap().path()).unwrap();
        assert!(!text.contains("test-key"));
        assert!(!text.contains("user@example.com"));
    }
    store.save_config(None).unwrap();
    assert!(store.load_config().unwrap().is_none());
    for file in std::fs::read_dir(&local).unwrap() {
        assert!(!std::fs::read_to_string(file.unwrap().path())
            .unwrap()
            .contains("test-key"));
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn period_navigation_drops_old_scope_and_legacy_usage_session_opens_real_app() {
    let mut s = app_launcher::LauncherState::new();
    s.open_app("sub2api-monitor");
    assert!(matches!(s.active_app, app_launcher::ActiveApp::Usage));
    s.usage.data = Some(std::sync::Arc::new(Data::default()));
    s.usage.navigate(Page::Models, Period::Month, None);
    assert!(s.usage.data.is_none());
    let mut session = app_launcher::session::Session::capture(&s);
    session.foreground = Some(app_launcher::session::SavedApp::Planned(
        app_launcher::planned_apps::PlannedApp::Usage,
    ));
    let mut restored = app_launcher::LauncherState::new();
    assert!(session.restore(&mut restored));
    assert!(matches!(
        restored.active_app,
        app_launcher::ActiveApp::Usage
    ));
}
fn snapshot(tokens: u64) -> std::sync::Arc<Data> {
    std::sync::Arc::new(Data {
        totals: Some(Totals {
            tokens,
            ..Default::default()
        }),
        ..Default::default()
    })
}
#[test]
fn navigation_restores_exact_seen_page_immediately_and_still_requests_fresh_data() {
    let mut state = State::default();
    state.config = Some(c());
    state.set_clock_context(observed_ms(), 480);
    let overview = scope(Page::Overview, Period::Day);
    let remembered = snapshot(123);
    state.remember_page(&overview, remembered.clone());
    state.navigate(Page::Models, Period::Day, None);
    assert!(state.data.is_none());
    state.navigate(Page::Overview, Period::Day, None);
    assert!(std::sync::Arc::ptr_eq(
        state.data.as_ref().unwrap(),
        &remembered
    ));
    assert!(state.stale);
    assert!(state.refresh);
    assert!(!state.busy);
    assert_eq!(state.status, "上次数据，正在刷新");
    assert_eq!(state.scope.as_ref(), Some(&overview));
}
#[test]
fn page_snapshots_are_bounded_and_evict_the_least_recently_used_page() {
    let mut state = State::default();
    state.config = Some(c());
    state.set_clock_context(observed_ms(), 480);
    for (i, page) in [Page::Overview, Page::Accounts, Page::Models, Page::Users]
        .into_iter()
        .enumerate()
    {
        state.remember_page(&scope(page, Period::Day), snapshot(i as u64));
    }
    assert!(state.load_cached_page(&scope(Page::Overview, Period::Day)));
    state.remember_page(&scope(Page::Overview, Period::Month), snapshot(4));
    assert_eq!(state.cached_page_count(), 4);
    assert!(!state.load_cached_page(&scope(Page::Accounts, Period::Day)));
    assert!(state.data.is_none());
    assert!(state.load_cached_page(&scope(Page::Overview, Period::Day)));
    assert_eq!(
        state.data.as_ref().unwrap().totals.as_ref().unwrap().tokens,
        0
    );
    state.clear_page_cache();
    assert_eq!(state.cached_page_count(), 0);
}
#[test]
fn cached_page_cannot_cross_site_key_date_timezone_period_or_detail() {
    let mut state = State::default();
    state.config = Some(c());
    state.set_clock_context(observed_ms(), 480);
    let mut user = scope(Page::Users, Period::Day);
    user.detail = Some(2);
    state.remember_page(&user, snapshot(42));
    assert!(state.load_cached_page(&user));
    state.config = Some(Config::new("other.example", "test-key").unwrap());
    assert!(!state.load_cached_page(&user));
    assert!(state.data.is_none());
    state.config = Some(Config::new("monitor.example", "different-key").unwrap());
    assert!(!state.load_cached_page(&user));
    state.config = Some(c());
    let mut different = user.clone();
    different.period = Period::Month;
    assert!(!state.load_cached_page(&different));
    different = user.clone();
    different.detail = Some(3);
    assert!(!state.load_cached_page(&different));
    different = user.clone();
    different.timezone_minutes = 0;
    assert!(!state.load_cached_page(&different));
    state.set_clock_context(observed_ms() + 86_400_000, 480);
    assert!(!state.load_cached_page(&user));
    different = user.clone();
    different.date = "2026-10-07".into();
    assert!(!state.load_cached_page(&different));
    state.config = None;
    assert!(!state.load_cached_page(&user));
    assert_eq!(state.cached_page_count(), 0);
}
#[test]
fn refreshed_list_and_detail_clamp_selection_after_data_shrinks() {
    let mut state = State::default();
    state.page = Page::Models;
    state.list_page = 10;
    state.model_selected = Some(50);
    let mut data = Data::default();
    for i in 0..7 {
        data.models.push(Model {
            name: format!("model-{i}"),
            totals: Totals::default(),
            input: 0,
            output: 0,
            cache: 0,
        });
    }
    state.data = Some(std::sync::Arc::new(data.clone()));
    state.clamp_selection();
    assert_eq!(state.list_page, 1);
    assert_eq!(state.model_selected, None);
    state.model_selected = Some(6);
    state.clamp_selection();
    assert_eq!(state.model_selected, Some(6));
    data.models.truncate(1);
    state.data = Some(std::sync::Arc::new(data));
    state.clamp_selection();
    assert_eq!(state.list_page, 0);
    assert_eq!(state.model_selected, None);
    state.page = Page::Users;
    state.detail = Some(2);
    state.model_selected = Some(6);
    state.clamp_selection();
    assert_eq!(state.model_selected, None);
}
#[test]
fn real_ui_navigation_keyboard_and_home_are_clickable() {
    use app_launcher::{build_launcher_ui, headless::HeadlessBackend, ActiveApp};
    use std::sync::{Arc, Mutex};
    use tiny_flutter::{App, Point as P, Size, TouchEvent};
    let mut state = app_launcher::LauncherState::new();
    state.open_app("sub2api-monitor");
    state.usage.config = Some(c());
    let state = Arc::new(Mutex::new(state));
    let size = Size::new(1024.0, 600.0);
    let mut b = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut b);
    let mut tap = |x: f32, y: f32| {
        b.event(TouchEvent::Down(P::new(x, y)));
        app.step_with_builder(&mut b, |s| build_launcher_ui(state.clone(), s));
        b.event(TouchEvent::Up(P::new(x, y)));
        app.step_with_builder(&mut b, |s| build_launcher_ui(state.clone(), s));
    };
    tap(308.0, 100.0);
    assert_eq!(state.lock().unwrap().usage.page, Page::Models);
    tap(803.0, 100.0);
    assert_eq!(state.lock().unwrap().usage.period, Period::Month);
    tap(509.0, 100.0);
    assert_eq!(state.lock().unwrap().usage.page, Page::Connection);
    tap(490.0, 292.0);
    assert!(state.lock().unwrap().usage.editor.keyboard);
    tap(72.0, 346.0);
    assert_eq!(state.lock().unwrap().usage.editor.key, "1");
    tap(48.0, 30.0);
    assert!(matches!(
        state.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
}
