use app_launcher::usage::{
    api::{Error, Transport},
    user_names::{clean_name, UserNameCache},
    Config, Data, Totals, User,
};
use serde_json::{json, Value};

#[derive(Default)]
struct Profiles {
    replies: Vec<Result<Value, Error>>,
    paths: Vec<String>,
}
impl Transport for Profiles {
    fn request(&mut self, _: &Config, path: &str, body: Option<&str>) -> Result<Vec<u8>, Error> {
        assert!(path.starts_with("/admin/users?page="));
        assert!(path.ends_with("&page_size=25"));
        assert!(body.is_none());
        self.paths.push(path.into());
        let value = self.replies.remove(0)?;
        Ok(serde_json::to_vec(&json!({"code":0,"data":value})).unwrap())
    }
}
fn config() -> Config {
    Config::new("https://monitor.example", "test-key").unwrap()
}
fn ranked(ids: &[i64]) -> Data {
    Data {
        users: ids
            .iter()
            .map(|id| User {
                id: *id,
                label: "ex********@example.test".into(),
                name: None,
                totals: Totals {
                    tokens: *id as u64 * 100,
                    cost: 1.25,
                    requests: Some(2),
                },
            })
            .collect(),
        ..Data::default()
    }
}
fn page(total: usize, rows: Vec<Value>) -> Result<Value, Error> {
    Ok(json!({"total":total,"items":rows}))
}
fn profile(id: i64, name: Option<&str>) -> Value {
    json!({"id":id,"notes":null,"username":name,"email":"never-retain@example.test"})
}

#[test]
fn name_uses_remark_then_username_and_never_email() {
    assert_eq!(
        clean_name(&json!({"notes":"  备注名字 \n", "username":"用户名"})),
        Some("备注名字".into())
    );
    assert_eq!(
        clean_name(&json!({"notes":" \n\t", "username":"  用户名  "})),
        Some("用户名".into())
    );
    for value in [
        json!({}),
        json!({"notes":null,"username":null}),
        json!({"notes":" ","username":"\n\t","email":"account@example.test"}),
    ] {
        assert_eq!(clean_name(&value), None);
    }
}

#[test]
fn displayed_names_are_bounded_unicode_and_controls_are_removed() {
    assert_eq!(
        clean_name(&json!({"notes":" \u{0}中\r文\n名\u{7} "})),
        Some("中文名".into())
    );
    let name = clean_name(&json!({"notes":format!(" {}尾", "字".repeat(100))})).unwrap();
    assert_eq!(name.chars().count(), 80);
    assert_eq!(name, "字".repeat(80));
    assert_eq!(
        clean_name(&json!({"notes":"\u{0}","username":"备用名"})),
        Some("备用名".into())
    );
}

#[test]
fn profiles_match_real_ids_independently_of_ranking_order() {
    let mut transport = Profiles {
        replies: vec![page(
            3,
            vec![
                profile(1, Some("一号姓名")),
                profile(2, None),
                profile(9, Some("九号姓名")),
            ],
        )],
        ..Profiles::default()
    };
    let mut data = ranked(&[9, 2, 1]);
    UserNameCache::new()
        .enrich(&mut transport, &config(), &mut data, 1_000)
        .unwrap();
    assert_eq!(data.users[0].display_name(), "九号姓名");
    assert_eq!(data.users[1].display_name(), "用户#2");
    assert_eq!(data.users[2].display_name(), "一号姓名");
    assert_eq!(data.users[0].totals.tokens, 900);
    assert_eq!(data.users[0].totals.cost, 1.25);
}

#[test]
fn inline_names_need_no_management_request() {
    let mut data = ranked(&[8]);
    data.users[0].name = Some("  已有名字  ".into());
    let mut transport = Profiles::default();
    let mut cache = UserNameCache::new();
    cache
        .enrich(&mut transport, &config(), &mut data, 100)
        .unwrap();
    assert_eq!(data.users[0].display_name(), "已有名字");
    assert_eq!(cache.lookup(8), Some("已有名字"));
    assert!(transport.paths.is_empty());
}

#[test]
fn named_and_nameless_profiles_have_five_minute_cache() {
    let mut transport = Profiles {
        replies: vec![page(
            2,
            vec![profile(1, Some("初始名字")), profile(7, None)],
        )],
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[1, 7]), 10_000)
        .unwrap();
    let mut fresh = ranked(&[7, 1]);
    cache
        .enrich(&mut transport, &config(), &mut fresh, 309_999)
        .unwrap();
    assert_eq!(transport.paths.len(), 1);
    assert_eq!(fresh.users[0].display_name(), "用户#7");
    assert_eq!(fresh.users[1].display_name(), "初始名字");
    transport.replies = vec![page(2, vec![profile(1, None), profile(7, Some("新名字"))])];
    let mut expired = ranked(&[1, 7]);
    cache
        .enrich(&mut transport, &config(), &mut expired, 310_000)
        .unwrap();
    assert_eq!(transport.paths.len(), 2);
    assert_eq!(expired.users[0].display_name(), "用户#1");
    assert_eq!(expired.users[1].display_name(), "新名字");
}

#[test]
fn transport_failure_keeps_cached_names_and_usage() {
    let mut transport = Profiles {
        replies: vec![page(1, vec![profile(4, Some("有效名字"))])],
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[4]), 1_000)
        .unwrap();
    transport.replies = vec![Err(Error::Authentication)];
    let mut data = ranked(&[4]);
    assert_eq!(
        cache.enrich(&mut transport, &config(), &mut data, 301_000),
        Err(Error::Authentication)
    );
    assert_eq!(data.users[0].display_name(), "有效名字");
    assert_eq!(data.users[0].totals.tokens, 400);
    assert_eq!(data.users[0].totals.cost, 1.25);
    assert_eq!(cache.lookup(4), Some("有效名字"));
}

#[test]
fn profiles_are_isolated_by_both_site_and_key() {
    let mut transport = Profiles {
        replies: vec![page(1, vec![profile(1, Some("原站姓名"))])],
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[1]), 1_000)
        .unwrap();
    let mut other_site = ranked(&[1]);
    transport.replies = vec![Err(Error::Transport)];
    assert_eq!(
        cache.enrich(
            &mut transport,
            &Config::new("https://other.example", "test-key").unwrap(),
            &mut other_site,
            2_000,
        ),
        Err(Error::Transport)
    );
    assert_eq!(other_site.users[0].display_name(), "用户#1");
    transport.replies = vec![page(1, vec![profile(1, Some("其他站名字"))])];
    cache
        .enrich(
            &mut transport,
            &Config::new("https://other.example", "test-key").unwrap(),
            &mut ranked(&[1]),
            3_000,
        )
        .unwrap();
    transport.replies = vec![Err(Error::Authentication)];
    let mut other_key = ranked(&[1]);
    assert_eq!(
        cache.enrich(
            &mut transport,
            &Config::new("https://other.example", "changed-key").unwrap(),
            &mut other_key,
            4_000,
        ),
        Err(Error::Authentication)
    );
    assert_eq!(other_key.users[0].display_name(), "用户#1");
    assert!(cache.is_empty());
}

#[test]
fn monotonic_backstep_does_not_treat_stale_name_as_fresh() {
    let mut cache = UserNameCache::new();
    let mut transport = Profiles {
        replies: vec![page(1, vec![profile(1, Some("先前名字"))])],
        ..Profiles::default()
    };
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[1]), 50_000)
        .unwrap();
    transport.replies = vec![page(1, vec![profile(1, Some("随后名字"))])];
    let mut data = ranked(&[1]);
    cache
        .enrich(&mut transport, &config(), &mut data, 1_000)
        .unwrap();
    assert_eq!(transport.paths.len(), 2);
    assert_eq!(data.users[0].display_name(), "随后名字");
}

#[test]
fn total_above_two_hundred_is_valid_when_ranked_ids_are_found() {
    let mut transport = Profiles {
        replies: vec![page(
            1_000,
            (1..=25).map(|id| profile(id, Some("名字"))).collect(),
        )],
        ..Profiles::default()
    };
    let mut data = ranked(&[25, 1]);
    UserNameCache::new()
        .enrich(&mut transport, &config(), &mut data, 1_000)
        .unwrap();
    assert_eq!(transport.paths.len(), 1);
    assert!(data.users.iter().all(|u| u.display_name() == "名字"));
}

#[test]
fn pagination_is_bounded_and_partial_success_is_retained() {
    let mut transport = Profiles {
        replies: (0..8)
            .map(|page_index| {
                page(
                    1_000,
                    (page_index * 25 + 1..=page_index * 25 + 25)
                        .map(|id| profile(id, Some("找到名字")))
                        .collect(),
                )
            })
            .collect(),
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    let mut data = ranked(&[2, 999]);
    assert_eq!(
        cache.enrich(&mut transport, &config(), &mut data, 1_000),
        Err(Error::TooLarge)
    );
    assert_eq!(transport.paths.len(), 8);
    assert_eq!(data.users[0].display_name(), "找到名字");
    assert_eq!(data.users[1].display_name(), "用户#999");
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.lookup(2), Some("找到名字"));
    assert_eq!(cache.lookup(3), None);
}

#[test]
fn second_page_matches_by_id_and_caches_absent_profiles() {
    let mut transport = Profiles {
        replies: vec![
            page(26, (1..=25).map(|id| profile(id, None)).collect()),
            page(26, vec![profile(100, Some("后一页名字"))]),
        ],
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    let mut data = ranked(&[100, 999]);
    cache
        .enrich(&mut transport, &config(), &mut data, 1_000)
        .unwrap();
    assert_eq!(transport.paths.len(), 2);
    assert_eq!(data.users[0].display_name(), "后一页名字");
    assert_eq!(data.users[1].display_name(), "用户#999");
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[999, 100]), 2_000)
        .unwrap();
    assert_eq!(transport.paths.len(), 2);
}

#[test]
fn duplicate_and_malformed_profiles_do_not_apply_invalid_page() {
    for reply in [
        page(2, vec![profile(1, Some("一号")), profile(1, None)]),
        page(1, vec![json!({"id":1,"notes":42})]),
        page(1, vec![json!({"id":0,"username":"名字"})]),
        page(2, vec![profile(1, Some("缺失行"))]),
        Ok(json!({"total":-1,"items":[]})),
        Ok(json!({"total":1,"items":null})),
    ] {
        let mut cache = UserNameCache::new();
        let mut transport = Profiles {
            replies: vec![page(1, vec![profile(1, Some("有效缓存"))])],
            ..Profiles::default()
        };
        cache
            .enrich(&mut transport, &config(), &mut ranked(&[1]), 1_000)
            .unwrap();
        transport.replies = vec![reply];
        let mut data = ranked(&[1]);
        assert_eq!(
            cache.enrich(&mut transport, &config(), &mut data, 301_000),
            Err(Error::Format)
        );
        assert_eq!(data.users[0].display_name(), "有效缓存");
        assert_eq!(cache.lookup(1), Some("有效缓存"));
    }
}

#[test]
fn changed_total_and_repeated_ids_across_pages_are_rejected() {
    for next_page in [
        page(27, vec![profile(99, None), profile(100, None)]),
        page(26, vec![profile(1, None)]),
        page(26, Vec::new()),
    ] {
        let mut transport = Profiles {
            replies: vec![
                page(
                    26,
                    (1..=25).map(|id| profile(id, Some("前页姓名"))).collect(),
                ),
                next_page,
            ],
            ..Profiles::default()
        };
        let mut data = ranked(&[2, 100]);
        assert_eq!(
            UserNameCache::new().enrich(&mut transport, &config(), &mut data, 1_000),
            Err(Error::Format)
        );
        assert_eq!(data.users[0].display_name(), "前页姓名");
        assert_eq!(data.users[1].display_name(), "用户#100");
    }
}

#[test]
fn cache_retains_only_current_ranked_ids() {
    let mut transport = Profiles::default();
    let mut cache = UserNameCache::new();
    let mut data = ranked(&(1..=200).collect::<Vec<_>>());
    for user in &mut data.users {
        user.name = Some(format!("名字{}", user.id));
    }
    cache
        .enrich(&mut transport, &config(), &mut data, 1_000)
        .unwrap();
    assert_eq!(cache.len(), 200);
    let mut next = ranked(&[999]);
    next.users[0].name = Some("下一范围名字".into());
    cache
        .enrich(&mut transport, &config(), &mut next, 2_000)
        .unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.lookup(1), None);
    assert_eq!(cache.lookup(999), Some("下一范围名字"));
    assert!(transport.paths.is_empty());
}

#[test]
fn forgetting_clears_names_and_requires_a_new_profile_fetch() {
    let mut transport = Profiles {
        replies: vec![page(1, vec![profile(1, Some("保存的姓名"))])],
        ..Profiles::default()
    };
    let mut cache = UserNameCache::new();
    cache
        .enrich(&mut transport, &config(), &mut ranked(&[1]), 1_000)
        .unwrap();
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.lookup(1), None);
    transport.replies = vec![Err(Error::Transport)];
    let mut data = ranked(&[1]);
    assert_eq!(
        cache.enrich(&mut transport, &config(), &mut data, 2_000),
        Err(Error::Transport)
    );
    assert_eq!(transport.paths.len(), 2);
    assert_eq!(data.users[0].display_name(), "用户#1");
}

#[test]
fn legacy_serialized_users_never_display_masked_email_as_name() {
    let user: User = serde_json::from_value(json!({
        "id":7,"label":"ex********@example.test",
        "totals":{"tokens":100,"cost":1.0,"requests":null}
    }))
    .unwrap();
    assert_eq!(user.name, None);
    assert_eq!(user.display_name(), "用户#7");
}
