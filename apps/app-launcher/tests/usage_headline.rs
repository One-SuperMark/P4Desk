use app_launcher::usage::{
    api,
    headline::{HeadlineState, FRAME_INTERVAL_MS, PERIOD_DURATION_MS, UPDATE_DURATION_MS},
    Config, Data, Page, Period, State, Totals,
};
use app_launcher::LauncherState;
use p4desk_protocol::Mode;
use std::sync::Arc;

const NOW: i64 = 1_791_287_400_000;

fn model() -> State {
    let mut state = State::default();
    state.config = Some(Config::new("monitor.example", "synthetic-key").unwrap());
    state.set_clock_context(NOW, 480);
    state
}

fn data(state: &mut State, tokens: Option<u64>) {
    state.scope = Some(api::scope(state.period, state.page, state.detail, NOW, 480).unwrap());
    state.data = Some(Arc::new(Data {
        totals: tokens.map(|tokens| Totals {
            tokens,
            cost: 1.25,
            requests: Some(5),
        }),
        ..Default::default()
    }));
}

#[test]
fn first_value_is_direct_and_missing_value_cancels() {
    let mut headline = HeadlineState::default();
    headline.update(Some(270_879_573), 100, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(100).value, Some(270_879_573));
    assert!(!headline.sample(100).active);
    headline.update(Some(280_000_000), 200, UPDATE_DURATION_MS);
    assert!(headline.sample(200).active);
    headline.update(None, 300, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(300).value, None);
    assert_eq!(headline.sample(300).width_chars, 1);
    assert!(!headline.take_dirty(1_300));
    headline.update(Some(42), 1_400, PERIOD_DURATION_MS);
    assert_eq!(headline.sample(1_400).value, Some(42));
    assert!(!headline.sample(1_400).active);
}

#[test]
fn quartic_easing_is_interruptible_and_target_remains_authoritative() {
    let mut headline = HeadlineState::default();
    headline.snap(Some(0));
    headline.update(Some(160_000), 0, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(500).value, Some(150_000));
    assert_eq!(headline.sample(500).target, Some(160_000));
    headline.update(Some(310_000), 500, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(500).from, Some(150_000));
    assert_eq!(headline.sample(500).value, Some(150_000));
    assert_eq!(headline.sample(1_000).value, Some(300_000));
    assert_eq!(headline.sample(1_500).value, Some(310_000));
    assert!(!headline.sample(1_500).active);
}

#[test]
fn unchanged_target_does_not_restart_and_reverse_updates_start_on_screen() {
    let mut headline = HeadlineState::default();
    headline.snap(Some(0));
    headline.update(Some(160_000), 100, UPDATE_DURATION_MS);
    headline.update(Some(160_000), 600, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(600).value, Some(150_000));
    headline.update(Some(10_000), 600, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(600).from, Some(150_000));
    assert_eq!(headline.sample(1_100).value, Some(18_750));
    assert_eq!(headline.sample(1_600).value, Some(10_000));
}

#[test]
fn u64_extremes_and_adjacent_large_integers_finish_exactly() {
    let mut headline = HeadlineState::default();
    headline.snap(Some(0));
    headline.update(Some(u64::MAX), 0, UPDATE_DURATION_MS);
    let mut before = 0;
    for ms in 0..=1_000 {
        let current = headline.sample(ms).value.unwrap();
        assert!(current >= before);
        before = current;
    }
    assert_eq!(before, u64::MAX);
    headline.update(Some(0), 1_000, UPDATE_DURATION_MS);
    let mut before = u64::MAX;
    for ms in 1_000..=2_000 {
        let current = headline.sample(ms).value.unwrap();
        assert!(current <= before);
        before = current;
    }
    assert_eq!(before, 0);
    headline.snap(Some(u64::MAX - 1));
    headline.update(Some(u64::MAX), 2_000, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(2_000).value, Some(u64::MAX - 1));
    assert_eq!(headline.sample(3_000).value, Some(u64::MAX));
}

#[test]
fn grouping_width_is_stable_until_settlement() {
    let mut headline = HeadlineState::default();
    headline.snap(Some(1_000));
    headline.update(Some(999), 0, UPDATE_DURATION_MS);
    for ms in [0, 200, 500, 999] {
        assert_eq!(headline.sample(ms).width_digits, 4);
        assert_eq!(headline.sample(ms).width_chars, 5);
    }
    assert_eq!(headline.sample(1_000).width_chars, 3);
    headline.update(Some(u64::MAX), 1_000, UPDATE_DURATION_MS);
    assert_eq!(headline.sample(1_000).width_digits, 20);
    assert_eq!(headline.sample(1_000).width_chars, 26);
}

#[test]
fn frame_gate_skips_backlog_and_always_returns_exact_final_frame() {
    let mut headline = HeadlineState::default();
    headline.snap(Some(100));
    headline.update(Some(200), 100, UPDATE_DURATION_MS);
    assert!(!headline.take_dirty(100));
    assert!(!headline.take_dirty(100 + FRAME_INTERVAL_MS - 1));
    assert!(headline.take_dirty(100 + FRAME_INTERVAL_MS));
    assert!(!headline.take_dirty(100 + FRAME_INTERVAL_MS));
    assert!(headline.take_dirty(850));
    assert!(!headline.take_dirty(850));
    assert!(headline.take_dirty(5_000));
    assert_eq!(headline.sample(5_000).value, Some(200));
    assert!(!headline.take_dirty(5_000));
    assert!(!headline.take_dirty(5_100));
}

#[test]
fn state_uses_one_second_updates_and_never_changes_real_totals() {
    let mut state = model();
    data(&mut state, Some(0));
    state.sync_headline(0, true);
    data(&mut state, Some(160_000));
    state.sync_headline(100, true);
    assert_eq!(state.headline.sample(600).value, Some(150_000));
    assert_eq!(
        state.data.as_ref().unwrap().totals.as_ref().unwrap().tokens,
        160_000
    );
    state.sync_headline(600, true);
    assert_eq!(state.headline.sample(1_100).value, Some(160_000));
    assert!(!state.headline.sample(1_100).active);
}

#[test]
fn repeated_sampling_and_cost_only_updates_do_not_restart_a_counter() {
    let mut state = model();
    data(&mut state, Some(0));
    state.sync_headline(0, true);
    // A snapshot pointer is only a fast observation hint. Updating a uniquely
    // owned Arc must still detect a changed Token value at the same address.
    Arc::get_mut(state.data.as_mut().unwrap())
        .unwrap()
        .totals
        .as_mut()
        .unwrap()
        .tokens = 160_000;
    state.sync_headline(100, true);
    assert_eq!(state.headline.sample(600).value, Some(150_000));
    for now in [200, 300, 400, 500] {
        state.sync_headline(now, true);
    }
    let mut next = (**state.data.as_ref().unwrap()).clone();
    next.totals.as_mut().unwrap().cost += 1.0;
    state.data = Some(Arc::new(next));
    state.sync_headline(600, true);
    assert_eq!(state.headline.sample(600).value, Some(150_000));
    assert_eq!(state.headline.sample(1_100).value, Some(160_000));
    assert!(!state.headline.sample(1_100).active);
}

#[test]
fn cached_period_switch_animates_for_eight_hundred_milliseconds() {
    let mut state = model();
    data(&mut state, Some(0));
    state.sync_headline(0, true);
    state.period = Period::Month;
    data(&mut state, Some(160_000));
    state.sync_headline(100, true);
    assert_eq!(state.headline.sample(500).value, Some(150_000));
    assert_eq!(state.headline.sample(900).value, Some(160_000));
    assert!(!state.headline.sample(900).active);
}

#[test]
fn missing_period_snapshot_shows_dash_then_direct_first_result() {
    let mut state = model();
    data(&mut state, Some(160_000));
    state.sync_headline(0, true);
    state.navigate(Page::Overview, Period::Month, None);
    state.sync_headline(100, true);
    assert_eq!(state.headline.sample(100).value, None);
    data(&mut state, Some(320_000));
    state.sync_headline(200, true);
    assert_eq!(state.headline.sample(200).value, Some(320_000));
    assert!(!state.headline.sample(200).active);
}

#[test]
fn site_key_page_detail_and_calendar_changes_cannot_cross_animate() {
    for change in 0..6 {
        let mut state = model();
        data(&mut state, Some(100));
        state.sync_headline(0, true);
        match change {
            0 => state.config = Some(Config::new("other.example", "synthetic-key").unwrap()),
            1 => state.config = Some(Config::new("monitor.example", "other-key").unwrap()),
            2 => state.page = Page::Users,
            3 => state.detail = Some(7),
            4 => {
                state.set_clock_context(NOW + 86_400_000, 480);
                state.scope = Some(
                    api::scope(
                        state.period,
                        state.page,
                        state.detail,
                        NOW + 86_400_000,
                        480,
                    )
                    .unwrap(),
                );
            }
            5 => {
                state.set_clock_context(NOW, 0);
                state.scope =
                    Some(api::scope(state.period, state.page, state.detail, NOW, 0).unwrap());
            }
            _ => unreachable!(),
        }
        if change < 4 {
            data(&mut state, Some(200));
        } else {
            state.data = Some(Arc::new(Data {
                totals: Some(Totals {
                    tokens: 200,
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
        state.sync_headline(100, true);
        assert_eq!(
            state.headline.sample(100).value,
            Some(200),
            "change {change}"
        );
        assert!(!state.headline.sample(100).active, "change {change}");
    }
}

#[test]
fn stale_scope_is_hidden_at_navigation_and_midnight() {
    let mut state = model();
    data(&mut state, Some(100));
    state.sync_headline(0, true);
    state.page = Page::Users;
    state.sync_headline(100, true);
    assert_eq!(state.headline.sample(100).value, None);
    state.page = Page::Overview;
    state.sync_headline(200, true);
    assert_eq!(state.headline.sample(200).value, Some(100));
    state.set_clock_context(NOW + 86_400_000, 480);
    state.sync_headline(300, true);
    assert_eq!(state.headline.sample(300).value, None);
}

#[test]
fn launcher_keeps_animation_local_and_stops_when_hidden() {
    let mut state = LauncherState::new();
    state.open_app("sub2api-monitor");
    state.usage = model();
    data(&mut state.usage, Some(0));
    state.tick(0, NOW);
    data(&mut state.usage, Some(160_000));
    let revision = state.revision;
    assert!(!state.tick(100, NOW + 100));
    assert_eq!(state.revision, revision);
    assert!(state.usage.headline.sample(100).active);
    assert!(!state.tick(500, NOW + 500));
    assert_eq!(state.revision, revision);
    assert_eq!(state.usage.headline.sample(600).value, Some(150_000));
    state.background_active_app();
    state.tick(610, NOW + 610);
    assert!(!state.usage.headline.sample(610).active);
    assert!(!state.usage.headline.take_dirty(700));
    state.open_app("sub2api-monitor");
    data(&mut state.usage, Some(320_000));
    state.tick(710, NOW + 710);
    assert!(state.usage.headline.sample(710).active);
    state.settings.screen_on = false;
    state.tick(720, NOW + 720);
    assert!(!state.usage.headline.sample(720).active);
    state.settings.screen_on = true;
    data(&mut state.usage, Some(640_000));
    state.tick(730, NOW + 730);
    assert!(state.usage.headline.sample(730).active);
    state.mode = Mode::Display;
    state.tick(740, NOW + 740);
    assert!(!state.usage.headline.sample(740).active);
}

#[test]
fn overlay_or_connection_settings_settles_counter_without_rebuilds() {
    let mut state = LauncherState::new();
    state.open_app("sub2api-monitor");
    state.usage = model();
    data(&mut state.usage, Some(0));
    state.tick(0, NOW);
    data(&mut state.usage, Some(100));
    state.tick(100, NOW + 100);
    assert!(state.usage.headline.sample(100).active);
    state.status_panel_open = true;
    state.tick(110, NOW + 110);
    assert!(!state.usage.headline.sample(110).active);
    state.status_panel_open = false;
    data(&mut state.usage, Some(200));
    state.tick(120, NOW + 120);
    assert!(state.usage.headline.sample(120).active);
    state.usage.navigate(Page::Connection, Period::Day, None);
    state.tick(130, NOW + 130);
    assert!(!state.usage.headline.sample(130).active);
    assert_eq!(state.usage.headline.sample(130).value, None);
}
