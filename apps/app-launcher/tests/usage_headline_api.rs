use app_launcher::usage::{
    api::{self, Error, Transport},
    *,
};
use serde_json::{json, Value};

#[derive(Default)]
struct Online {
    paths: Vec<String>,
    fail: Option<&'static str>,
}
impl Transport for Online {
    fn request(&mut self, _: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        self.paths.push(path.into());
        if self.fail.is_some_and(|s| path.starts_with(s)) {
            return Err(Error::Http(503));
        }
        let data = if path == "/admin/dashboard/stats" {
            json!({"today_tokens":100,"total_tokens":900,"today_actual_cost":1,"total_actual_cost":9,"today_requests":2,"total_requests":20})
        } else if path == "/admin/accounts/today-stats/batch" {
            assert_eq!(
                serde_json::from_str::<Value>(body.unwrap()).unwrap(),
                json!({"account_ids":[7]})
            );
            json!({"stats":{"7":{"tokens":123,"user_cost":1.23,"requests":3}}})
        } else if path.starts_with("/admin/dashboard/trend?") {
            json!({"trend":[{"date":"2026-10-06 12:00","total_tokens":60,"actual_cost":0.6,"requests":1}]})
        } else {
            panic!("fast sample requested a static dashboard endpoint: {path}");
        };
        Ok(serde_json::to_vec(&json!({"code":0,"data":data})).unwrap())
    }
}
fn config() -> Config {
    Config::new("https://test.example", "test-key").unwrap()
}
fn now() -> i64 {
    1_791_261_000_000
}
fn scope() -> Scope {
    api::scope(Period::Day, Page::Overview, None, now(), 480).unwrap()
}
fn base() -> Data {
    Data {
        totals: Some(Totals {
            tokens: 90,
            cost: 0.9,
            requests: Some(2),
        }),
        accounts: vec![Account {
            id: 7,
            label: "account".into(),
            platform: "openai".into(),
            kind: "oauth".into(),
            plan_label: "Pro".into(),
            usage_updated_ms: Some(now() - 1000),
            five: Some(Window {
                used: 25.0,
                reset_ms: None,
                window_minutes: Some(300),
            }),
            seven: None,
        }],
        yesterday: Some(Totals {
            tokens: 456,
            ..Default::default()
        }),
        yesterday_trend: vec![Point {
            date: "2026-10-05 12:00".into(),
            totals: Totals {
                tokens: 44,
                ..Default::default()
            },
        }],
        trend: vec![Point {
            date: "2026-10-06 12:00".into(),
            totals: Totals {
                tokens: 50,
                ..Default::default()
            },
        }],
        models: vec![Model {
            name: "m".into(),
            totals: Totals {
                tokens: 77,
                ..Default::default()
            },
            input: 10,
            output: 20,
            cache: 47,
        }],
        sampled_ms: now(),
        total_label: "当前账号汇总".into(),
        ..Default::default()
    }
}
#[test]
fn three_sequential_live_endpoints_replace_primary_and_trend_preserve_static_sections() {
    let mut t = Online::default();
    let d = api::fetch_headline(&mut t, &config(), &scope(), now() + 5000, &base()).unwrap();
    assert_eq!(t.paths.len(), 3);
    assert_eq!(t.paths[0], "/admin/dashboard/stats");
    assert_eq!(t.paths[1], "/admin/accounts/today-stats/batch");
    assert!(t.paths[2].starts_with("/admin/dashboard/trend?"));
    assert_eq!(d.totals.as_ref().unwrap().tokens, 123);
    assert_eq!(d.total_label, "当前账号汇总");
    assert_eq!(d.trend.last().unwrap().totals.tokens, 60);
    assert_eq!(d.yesterday.as_ref().unwrap().tokens, 456);
    assert_eq!(d.yesterday_trend[0].totals.tokens, 44);
    assert_eq!(d.models[0].totals.tokens, 77);
    assert_eq!(d.accounts[0].five.as_ref().unwrap().used, 25.0);
    assert_eq!(d.accounts[0].usage_updated_ms, Some(now() - 1000));
    assert_eq!(d.sampled_ms, now() + 5000);
}
#[test]
fn no_known_account_uses_online_global_primary_without_discovery_or_batch() {
    let mut t = Online::default();
    let mut b = base();
    b.accounts.clear();
    let d = api::fetch_headline(&mut t, &config(), &scope(), now() + 5000, &b).unwrap();
    assert_eq!(d.totals.as_ref().unwrap().tokens, 100);
    assert_eq!(d.total_label, "全站统计");
    assert_eq!(t.paths.len(), 2);
}
#[test]
fn successful_account_headline_survives_secondary_stats_error() {
    let mut t = Online {
        fail: Some("/admin/dashboard/stats"),
        ..Default::default()
    };
    let d = api::fetch_headline(&mut t, &config(), &scope(), now() + 5000, &base()).unwrap();
    assert_eq!(d.totals.unwrap().tokens, 123);
    assert_eq!(d.total_label, "当前账号汇总");
    assert!(d.warning.unwrap().contains("全站统计未更新"));
}
#[test]
fn successful_headline_survives_trend_failure_with_previous_curve_and_explicit_warning() {
    let mut t = Online {
        fail: Some("/admin/dashboard/trend"),
        ..Default::default()
    };
    let d = api::fetch_headline(&mut t, &config(), &scope(), now() + 5000, &base()).unwrap();
    assert_eq!(d.totals.unwrap().tokens, 123);
    assert_eq!(d.trend.len(), 1);
    assert_eq!(d.trend[0].totals.tokens, 50);
    assert!(d.warning.unwrap().contains("趋势未更新"));
}
#[test]
fn failed_account_batch_fallback_is_explicitly_global_not_account_labeled() {
    let mut t = Online {
        fail: Some("/admin/accounts/today-stats"),
        ..Default::default()
    };
    let d = api::fetch_headline(&mut t, &config(), &scope(), now() + 5000, &base()).unwrap();
    assert_eq!(d.totals.unwrap().tokens, 100);
    assert_eq!(d.total_label, "全站统计");
    assert!(d.warning.unwrap().contains("今日账号汇总未更新"));
}
#[test]
fn non_today_overview_scopes_and_cross_midnight_base_are_rejected_before_network() {
    let mut t = Online::default();
    for (page, period, detail) in [
        (Page::Overview, Period::Month, None),
        (Page::Overview, Period::Total, None),
        (Page::Models, Period::Day, None),
        (Page::Accounts, Period::Day, Some(7)),
    ] {
        let mut s = scope();
        s.page = page;
        s.period = period;
        s.detail = detail;
        assert!(matches!(
            api::fetch_headline(&mut t, &config(), &s, now(), &base()),
            Err(Error::Format)
        ));
    }
    let mut b = base();
    b.sampled_ms -= 86_400_000;
    assert!(matches!(
        api::fetch_headline(&mut t, &config(), &scope(), now(), &b),
        Err(Error::Format)
    ));
    let mut s = scope();
    s.date = "2026-10-05".into();
    assert!(matches!(
        api::fetch_headline(&mut t, &config(), &s, now(), &base()),
        Err(Error::Format)
    ));
    assert!(t.paths.is_empty());
}

#[test]
fn successful_sample_clears_recovered_live_warning_and_retains_static_warning() {
    let mut b = base();
    b.warning = Some("趋势未更新：连接失败 · 昨日未更新 · 今日账号汇总未更新".into());
    let d = api::fetch_headline(
        &mut Online::default(),
        &config(),
        &scope(),
        now() + 5000,
        &b,
    )
    .unwrap();
    assert_eq!(d.warning.as_deref(), Some("昨日未更新"));
    b.warning = Some("趋势未更新：连接失败".into());
    let d = api::fetch_headline(
        &mut Online::default(),
        &config(),
        &scope(),
        now() + 5000,
        &b,
    )
    .unwrap();
    assert!(d.warning.is_none());
}
