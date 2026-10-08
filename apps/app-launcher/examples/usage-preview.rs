//! Synthetic data only; renders the production Pad monitor UI in both appearances.
use app_launcher::{build_launcher_ui, headless::HeadlessBackend, usage::*, LauncherState};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tiny_flutter::{App, Size};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or("artifacts/usage".into()));
    std::fs::create_dir_all(&out)?;
    for light in [false, true] {
        for (page, period, detail) in [
            (Page::Overview, Period::Day, None),
            (Page::Accounts, Period::Day, None),
            (Page::Accounts, Period::Month, None),
            (Page::Models, Period::Day, None),
            (Page::Users, Period::Day, None),
            (Page::Connection, Period::Day, None),
            (Page::Overview, Period::Month, None),
            (Page::Overview, Period::Total, None),
            (Page::Users, Period::Day, Some(1)),
            (Page::Accounts, Period::Month, Some(1)),
        ] {
            let mut s = LauncherState::new();
            s.settings.light_appearance = light;
            s.open_app("sub2api-monitor");
            s.tick(10000, 1_791_287_400_000);
            let mut d = Data {
                totals: Some(Totals {
                    tokens: 19_280_000,
                    cost: 8.76,
                    requests: Some(428),
                }),
                yesterday: Some(Totals {
                    tokens: 16_320_000,
                    cost: 7.2,
                    requests: Some(394),
                }),
                sampled_ms: s.unix_ms,
                total_label: "当前账号汇总".into(),
                trend_label: "今日用量趋势".into(),
                ..Default::default()
            };
            for hour in 0..14 {
                d.trend.push(Point {
                    date: format!("2026-10-06 {hour:02}:00"),
                    totals: Totals {
                        tokens: (hour * 923_471 + 1423) % 3_050_000,
                        cost: ((hour * 371) % 980) as f64 / 1000.0,
                        requests: Some(hour * 6),
                    },
                });
            }
            for hour in 0..24 {
                d.yesterday_trend.push(Point {
                    date: format!("2026-10-05 {hour:02}:00"),
                    totals: Totals {
                        tokens: (hour * 723_411 + 1643) % 2_750_000,
                        cost: ((hour * 171) % 680) as f64 / 1000.0,
                        requests: Some(hour * 4),
                    },
                });
            }
            for (i, name) in [
                "gpt-6.1-sol",
                "gpt-6-astra",
                "gpt-5.6-terra",
                "gpt-5.6-luna",
                "gpt-6-luna",
            ]
            .iter()
            .enumerate()
            {
                d.models.push(Model {
                    name: name.to_string(),
                    totals: Totals {
                        tokens: 8_600_000 / (i as u64 + 1),
                        cost: 3.25 / (i as f64 + 1.0),
                        requests: None,
                    },
                    input: 2_020_000,
                    output: 530_000,
                    cache: 6_050_000,
                });
            }
            for id in 1..=7 {
                d.users.push(User {
                    id,
                    name: [
                        Some("Workspace"), Some("Alice"), None, Some("测试账号"),
                        Some("Bob"), None, Some("Design team"),
                    ][id as usize - 1].map(str::to_owned),
                    label: format!("u{id}********"),
                    totals: Totals {
                        tokens: 5_400_000 / id as u64,
                        cost: 2.3 / id as f64,
                        requests: None,
                    },
                });
            }
            for id in 1..=if page == Page::Accounts && period == Period::Day {
                1
            } else {
                3
            } {
                d.accounts.push(Account {
                    id,
                    label: format!("p{id}********"),
                    platform: "openai".into(),
                    kind: "oauth".into(),
                    plan_label: "Pro 200".into(),
                    usage_updated_ms: Some(s.unix_ms),
                    five: Some(Window {
                        used: 24.0,
                        window_minutes: Some(300),
                        reset_ms: Some(s.unix_ms + 4 * 3600000),
                    }),
                    seven: Some(Window {
                        used: 61.0,
                        window_minutes: Some(10080),
                        reset_ms: Some(s.unix_ms + 3 * 86400000),
                    }),
                });
            }
            if period != Period::Day {
                d.yesterday = None;
                d.yesterday_trend.clear();
                d.trend.clear();
                let start = chrono::NaiveDate::from_ymd_opt(2026, 10, 6).unwrap()
                    - chrono::Duration::days(if period == Period::Total { 89 } else { 5 });
                for i in 0..if period == Period::Total { 90 } else { 6 } {
                    d.trend.push(Point {
                        date: (start + chrono::Duration::days(i)).to_string(),
                        totals: Totals {
                            tokens: ((i as u64 * 72941 + 37955) % 3070014),
                            cost: ((i * 43) % 790) as f64 / 1000.0,
                            requests: Some(i as u64 * 3),
                        },
                    });
                }
                d.trend_label = if period == Period::Total {
                    "近 90 天趋势"
                } else {
                    "本月用量趋势"
                }
                .into();
                d.total_label = "全站统计".into();
            }
            if detail.is_some() {
                if page == Page::Users {
                    s.usage.config = Some(Config::new("monitor.example", "synthetic-key")?);
                    let scope = api::scope(period, Page::Users, None, s.unix_ms, 480).unwrap();
                    s.usage.remember_page(&scope, Arc::new(d.clone()));
                }
                d.accounts.clear();
                d.users.clear();
                d.yesterday = None;
                d.yesterday_trend.clear();
                d.total_label = if page == Page::Users {
                    "当前用户"
                } else {
                    "账号模型汇总"
                }
                .into();
            }
            s.usage.period = period;
            s.usage.detail = detail;
            s.usage.config = Some(Config::new("monitor.example", "synthetic-key")?);
            s.usage.page = page;
            s.usage.data = Some(Arc::new(d));
            s.usage.status = if page == Page::Overview && period == Period::Day && detail.is_none()
            {
                "更新于 14:30:00 · 实时采样 5 秒 / 看板 60 秒"
            } else {
                "更新于 14:30:00 · 自动刷新 60 秒"
            }
            .into();
            let state = Arc::new(Mutex::new(s));
            let size = Size::new(1024.0, 600.0);
            let mut b = HeadlessBackend::new(1024, 600);
            let mut app = App::new(build_launcher_ui(state, size), size);
            app.step(&mut b);
            let suffix = if detail.is_some() {
                format!("-{:?}-detail", period)
            } else if period != Period::Day {
                format!("-{:?}", period)
            } else {
                String::new()
            };
            let file = std::fs::File::create(out.join(format!(
                "{}-{:?}{suffix}.png",
                if light { "light" } else { "dark" },
                page
            )))?;
            let mut enc = png::Encoder::new(file, 1024, 600);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut writer = enc.write_header()?;
            let mut pixels = Vec::with_capacity(1024 * 600 * 3);
            for p in b.pixels {
                let (r, g, b) = tiny_flutter::tiny_gfx::rgb565_to_rgb888(p);
                pixels.extend([r, g, b]);
            }
            writer.write_image_data(&pixels)?;
        }
    }
    Ok(())
}
