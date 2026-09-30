use app_launcher::radio::SettingsSection;
use app_launcher::{
    build_launcher_ui, headless::HeadlessBackend, LauncherState, LocalSettings, UiCommand,
};
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Point, Size, TouchEvent};
const SIZE: Size = Size::new(1024.0, 600.0);
fn render(s: LauncherState) -> Vec<u16> {
    let mut backend = HeadlessBackend::new(1024, 600);
    App::new(build_launcher_ui(Arc::new(Mutex::new(s)), SIZE), SIZE).step(&mut backend);
    backend.pixels
}
#[test]
fn legacy_settings_keep_dark_default_and_invalid_glass_amount_is_rejected() {
    let s: LocalSettings =
        serde_json::from_str(r#"{"brightness":75,"screen_on":true,"timezone_minutes":480}"#)
            .unwrap();
    assert!(!s.light_appearance);
    assert_eq!(s.glass_amount, 65);
    assert!(s.validate());
    let mut invalid = s;
    invalid.glass_amount = 101;
    assert!(!invalid.validate());
}
#[test]
fn appearance_controls_route_real_save_requests_without_losing_app_state() {
    let mut s = LauncherState::new();
    s.open_app("calculator");
    s.open_app("settings");
    s.settings_view.section = SettingsSection::Appearance;
    let state = Arc::new(Mutex::new(s));
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(build_launcher_ui(state.clone(), SIZE), SIZE);
    app.step(&mut backend);
    for (x, y, light, glass) in [
        (710.0, 160.0, true, 65),
        (352.0, 418.0, false, 25),
        (950.0, 307.0, false, 0),
    ] {
        for event in [
            TouchEvent::Down(Point::new(x, y)),
            TouchEvent::Up(Point::new(x, y)),
        ] {
            backend.event(event);
            app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
        }
        assert!(
            matches!(state.lock().unwrap().take_commands().as_slice(),[UiCommand::Appearance {light:l,glass:g}] if *l==light && *g==glass)
        );
    }
    assert!(state
        .lock()
        .unwrap()
        .running_apps
        .contains_key("calculator"));
}
#[test]
fn local_apps_change_background_and_foreground_in_light_mode() {
    for id in ["clock", "timer", "calculator", "notes", "mac", "settings"] {
        let scene = |light| {
            let mut s = LauncherState::new();
            s.settings.light_appearance = light;
            s.open_app(id);
            s.tick(100, 1_790_712_418_000);
            s
        };
        let dark = render(scene(false));
        let light = render(scene(true));
        assert_ne!(dark, light, "theme ignored by {id}");
        let brightness =
            |p: u16| ((p >> 11) & 31) as u32 * 2 + ((p >> 5) & 63) as u32 + (p & 31) as u32 * 2;
        assert!(
            brightness(light[599 * 1024 + 1023]) > brightness(dark[599 * 1024 + 1023]),
            "light background missing in {id}"
        );
    }
}
#[test]
fn desktop_hides_mac_connection_notice_but_keeps_storage_failure_visible() {
    let scene = |notice: &str| {
        let mut s = LauncherState::new();
        s.notice = notice.into();
        s
    };
    let normal = render(scene(""));
    assert_eq!(normal, render(scene("Mac 未连接，请连接 USB 和 Mac 应用")));
    assert_ne!(normal, render(scene("保存失败，原数据已保留")));
}

#[test]
fn application_glass_is_independent_of_desktop_wallpaper_but_cards_sample_it() {
    use tiny_flutter::theme::{Folio, GlassBackdrop};
    fn with_wallpaper(id: Option<&str>, light: bool, pixels: &'static [u8]) -> Vec<u16> {
        let mut state = LauncherState::new();
        state.settings.light_appearance = light;
        if let Some(id) = id {
            state.open_app(id);
        }
        let widget = build_launcher_ui(Arc::new(Mutex::new(state)), SIZE);
        // Replace only the glass source after building the real application.
        // Neither source may leak through an application's navigation or sidebar.
        Folio::set_backdrop(GlassBackdrop {
            pixels,
            width: 1,
            height: 1,
        });
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(widget, SIZE).step(&mut backend);
        backend.pixels
    }
    for light in [false, true] {
        for id in [
            "settings",
            "clock",
            "timer",
            "calculator",
            "notes",
            "mac",
            "display",
        ] {
            assert_eq!(
                with_wallpaper(Some(id), light, &[255, 0, 0]),
                with_wallpaper(Some(id), light, &[0, 0, 255]),
                "desktop image leaked into {id}"
            );
        }
        let red = with_wallpaper(None, light, &[255, 0, 0]);
        let blue = with_wallpaper(None, light, &[0, 0, 255]);
        for x in [300, 750] {
            assert_ne!(
                red[100 * 1024 + x],
                blue[100 * 1024 + x],
                "desktop card lost its backdrop"
            );
        }
    }
}
