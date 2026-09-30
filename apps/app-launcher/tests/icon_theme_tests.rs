use app_launcher::{
    app_icons, build_launcher_ui, headless::HeadlessBackend, icon_theme::IconTheme,
    radio::SettingsSection, LauncherState, LocalSettings, UiCommand,
};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};

#[test]
fn theme_page_routes_four_choices_and_keeps_appearance_and_background_apps() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    s.open_app("settings");
    let state = Arc::new(Mutex::new(s));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    let tap = |app: &mut App, backend: &mut HeadlessBackend, x, y| {
        for event in [
            TouchEvent::Down(Point::new(x, y)),
            TouchEvent::Up(Point::new(x, y)),
        ] {
            backend.event(event);
            app.step_with_builder(backend, |size| build_launcher_ui(state.clone(), size));
        }
    };
    tap(&mut app, &mut backend, 100.0, 250.0);
    assert_eq!(
        state.lock().unwrap().settings_view.section,
        SettingsSection::Appearance
    );
    for event in [
        TouchEvent::Down(Point::new(720.0, 540.0)),
        TouchEvent::Move(Point::new(720.0, 120.0)),
        TouchEvent::Up(Point::new(720.0, 120.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(
        state.lock().unwrap().take_commands().is_empty(),
        "scrolling must not activate appearance controls"
    );
    for (i, theme) in IconTheme::ALL.into_iter().enumerate() {
        tap(&mut app, &mut backend, 370.5 + i as f32 * 174.0, 330.0);
        let mut s = state.lock().unwrap();
        assert!(matches!(s.take_commands().as_slice(),[UiCommand::IconTheme(t)] if *t==theme));
        // Runtime commits these only after its durable journal write succeeds.
        s.settings.icon_theme = theme;
        assert!(!s.settings.light_appearance);
        assert!(s.running_apps.contains_key("calculator"));
        drop(s);
        app.request_rebuild();
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
}

#[test]
fn all_entries_have_four_distinct_packs_and_old_settings_migrate() {
    for &(id, _) in app_launcher::launcher_ui::DESKTOP_ENTRIES {
        let icons = IconTheme::ALL.map(|theme| app_icons::get_app_icon_for(theme, id).unwrap());
        assert!(!std::ptr::eq(icons[0], icons[1]));
        assert!(!std::ptr::eq(icons[1], icons[2]));
        assert!(!std::ptr::eq(icons[2], icons[3]));
    }
    assert_eq!(
        serde_json::from_str::<SettingsSection>("\"Theme\"").unwrap(),
        SettingsSection::Appearance
    );
    let old = r#"{"brightness":75,"screen_on":true,"timezone_minutes":480}"#;
    assert_eq!(
        serde_json::from_str::<LocalSettings>(old)
            .unwrap()
            .icon_theme,
        IconTheme::Colloid
    );
    for theme in IconTheme::ALL {
        let settings = LocalSettings {
            icon_theme: theme,
            ..Default::default()
        };
        let restored: LocalSettings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert_eq!(restored.icon_theme, theme);
    }
}

#[test]
fn display_transition_obeys_both_theme_and_appearance_on_a_different_thread() {
    for theme in IconTheme::ALL {
        for light in [false, true] {
            let pixels = std::thread::spawn(move || {
                let mut p = tiny_flutter::tiny_gfx::Pixmap565::new(1024, 600).unwrap();
                app_launcher::app_launch::paint_usb_display_reveal(
                    &mut tiny_flutter::Canvas::new(p.as_mut()),
                    Size::new(1024.0, 600.0),
                    0,
                    600,
                    light,
                    theme,
                );
                p
            })
            .join()
            .unwrap();
            let color = app_icons::launch_color_for_theme("display", light, theme).to_rgb565();
            assert!(pixels.data().iter().all(|p| *p == color));
        }
    }
}

#[test]
fn icon_colors_are_independent_of_appearance_for_all_four_packs() {
    for theme in IconTheme::ALL {
        for id in [
            "clock",
            "timer",
            "settings",
            "file-manager",
            "office-viewer",
            "sub2api-monitor",
        ] {
            let mut seen = [0usize; 2];
            for light_icons in [false, true] {
                for light_appearance in [false, true] {
                    let settings = LocalSettings {
                        icon_theme: theme,
                        light_appearance,
                        icon_light_override: Some(light_icons),
                        ..Default::default()
                    };
                    app_launcher::appearance::configure(&settings);
                    assert_eq!(tiny_flutter::theme::Folio::is_light(), light_appearance);
                    assert_eq!(app_launcher::icon_theme::is_light(), light_icons);
                    let ptr = app_icons::get_app_icon_asset(id).unwrap() as *const _ as usize;
                    if light_appearance {
                        assert_eq!(seen[light_icons as usize], ptr);
                    } else {
                        seen[light_icons as usize] = ptr;
                    }
                    assert_eq!(
                        app_icons::launch_color(id),
                        app_icons::launch_color_for_theme(id, light_icons, theme)
                    );
                }
            }
            assert_ne!(
                seen[0], seen[1],
                "missing light/dark variant for {theme:?}/{id}"
            );
        }
    }
}

#[test]
fn palette_switch_toggles_on_release_and_vertical_scroll_cancels() {
    let mut s = LauncherState::new();
    s.open_app("settings");
    s.settings_view.section = SettingsSection::Appearance;
    s.settings_view.appearance_scroll.set_offset(420.0);
    let state = Arc::new(Mutex::new(s));
    let size = Size::new(1024.0, 600.0);
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), size), size);
    app.step(&mut backend);
    for light in [true, false] {
        backend.event(TouchEvent::Down(Point::new(950.0, 225.0)));
        app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
        assert!(state.lock().unwrap().take_commands().is_empty());
        backend.event(TouchEvent::Up(Point::new(950.0, 225.0)));
        app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
        let mut s = state.lock().unwrap();
        assert!(matches!(s.take_commands().as_slice(),[UiCommand::IconLight(v)] if *v==light));
        assert_eq!(s.settings_view.appearance_scroll.offset(), 420.0);
        s.settings.icon_light_override = Some(light);
        drop(s);
        app.request_rebuild();
        app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
    }
    for e in [
        TouchEvent::Down(Point::new(950.0, 225.0)),
        TouchEvent::Move(Point::new(950.0, 325.0)),
        TouchEvent::Up(Point::new(950.0, 325.0)),
    ] {
        backend.event(e);
        app.step_with_builder(&mut backend, |sz| build_launcher_ui(state.clone(), sz));
    }
    let mut s = state.lock().unwrap();
    assert!(s.take_commands().is_empty());
    assert!(s.settings_view.appearance_scroll.offset() < 420.0);
}
