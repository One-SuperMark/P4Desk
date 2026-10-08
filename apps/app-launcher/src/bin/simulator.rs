#[cfg(any(feature = "simulator", feature = "screenshots"))]
use app_launcher::build_launcher_ui;
use app_launcher::LauncherState;
use p4desk_protocol::{Action, Button, Note, Snapshot};
use std::sync::{Arc, Mutex};
#[cfg(feature = "screenshots")]
use tiny_flutter::{App, Size};

#[cfg(feature = "simulator")]
fn simulator_commands(
    state: &Arc<Mutex<LauncherState>>,
    store: &app_launcher::GenerationStore,
    wall_offset: &mut i64,
) {
    let commands = state.lock().unwrap().take_commands();
    for command in commands {
        match command {
            app_launcher::UiCommand::DeleteNote(id) => {
                let current = state.lock().unwrap().snapshot.clone();
                match store.delete_note(&current, &id) {
                    Ok(next) => state.lock().unwrap().apply_snapshot(next),
                    Err(_) => {
                        state
                            .lock()
                            .unwrap()
                            .show_error(app_launcher::launcher_state::SystemError::SaveFailed);
                    }
                }
            }
            app_launcher::UiCommand::Brightness(value) => {
                let mut settings = state.lock().unwrap().settings.clone();
                settings.brightness = value;
                if store.save_settings(&settings).is_ok() {
                    state.lock().unwrap().settings = settings;
                }
            }
            app_launcher::UiCommand::SetTime(unix_ms, timezone) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;
                *wall_offset = unix_ms - now;
                let mut s = state.lock().unwrap();
                s.settings.timezone_minutes = timezone;
                s.last_time_refresh();
                let _ = store.save_settings(&s.settings);
            }
            app_launcher::UiCommand::RequestTimeSync => {
                *wall_offset = 0;
            }
            app_launcher::UiCommand::Screen(_) => {
                state
                    .lock()
                    .unwrap()
                    .show_notice(app_launcher::launcher_state::SystemNotice::SimulatorScreenHelp);
            }
            app_launcher::UiCommand::DismissError { id } => {
                state.lock().unwrap().dismiss_dialog(id);
            }
            _ => {
                state
                    .lock()
                    .unwrap()
                    .show_notice(app_launcher::launcher_state::SystemNotice::SimulatorUSBHelp);
            }
        }
    }
}

fn demo() -> Snapshot {
    Snapshot {generation:1,notes:vec![Note{id:"demo_1".into(),title:"今天的安排".into(),body:"上午：整理项目资料\n下午：完成桌面工具\n\n专注一件事，完成后稍作休息。\n便签在 Mac 上编辑，同步后立即显示。".into(),updated_ms:0}],buttons:vec![
    Button{id:"copy".into(),label:"复制".into(),action:Action::Shortcut{key_code:8,modifiers:0x100000}},
    Button{id:"paste".into(),label:"粘贴".into(),action:Action::Shortcut{key_code:9,modifiers:0x100000}},
    Button{id:"finder".into(),label:"打开访达".into(),action:Action::Application{bundle_path:"/System/Library/CoreServices/Finder.app".into()}},
],deleted_note_ids:vec![]}
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let get = |key: &str| {
        args.iter()
            .position(|s| s == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let state = Arc::new(Mutex::new(LauncherState::new()));
    {
        let mut s = state.lock().unwrap();
        if args.iter().any(|a| a == "--demo") {
            s.snapshot = demo();
            s.connected = true;
            s.usb_connected = true;
            s.sd_ready = true;
        }
        if let Some(p) = get("--snapshot") {
            s.snapshot = serde_json::from_slice(&std::fs::read(p)?)?;
            s.snapshot.validate()?;
        }
        s.tick(
            0,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as i64,
        );
    }
    if let Some(p) = get("--fontpack") {
        tiny_flutter::graphics::font::install_fontpack(Some(Arc::new(
            tiny_flutter::graphics::fontpack::FontPack::from_file(p)?,
        )));
    }
    if args.iter().any(|a| a == "--headless") {
        #[cfg(feature = "screenshots")]
        {
            let out = std::path::PathBuf::from(get("--output").unwrap_or_else(|| "pad.png".into()));
            let screen = get("--screen").unwrap_or_else(|| "home".into());
            let screens: Vec<&str> = if screen == "all" {
                vec![
                    "home",
                    "clock",
                    "timer",
                    "notes",
                    "calculator",
                    "mac",
                    "settings",
                    "manual",
                ]
            } else {
                vec![&screen]
            };
            if screen == "all" {
                std::fs::create_dir_all(&out)?;
            }
            for name in screens {
                {
                    let mut s = state.lock().unwrap();
                    if name == "home" {
                        s.background_active_app();
                        s.page_controller.set_page(0);
                    } else if name == "manual" {
                        s.open_app("settings");
                        s.manual_time_open = true;
                    } else {
                        s.open_app(name);
                    }
                }
                let size = Size::new(1024.0, 600.0);
                let mut backend = app_launcher::headless::HeadlessBackend::new(1024, 600);
                let mut app = App::new(build_launcher_ui(state.clone(), size), size);
                app.step(&mut backend);
                let path = if screen == "all" {
                    out.join(format!("{name}.png"))
                } else {
                    out.clone()
                };
                backend.screenshot(&path)?;
            }
            println!("{{\"rendered\":true,\"width\":1024,\"height\":600}}");
            return Ok(());
        }
        #[cfg(not(feature = "screenshots"))]
        return Err("build with --features screenshots for PNG output".into());
    }
    #[cfg(feature = "simulator")]
    {
        let started = std::time::Instant::now();
        let state_copy = state.clone();
        let store =
            app_launcher::GenerationStore::new(".cache/simulator/sd", ".cache/simulator/flash");
        let mut wall_offset = 0;
        tiny_flutter::run_simulator_animated("P4Desk · Rust Pad", 1024, 600, move |size| {
            simulator_commands(&state_copy, &store, &mut wall_offset);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            state_copy
                .lock()
                .unwrap()
                .tick(started.elapsed().as_millis() as u64, now + wall_offset);
            build_launcher_ui(state_copy.clone(), size)
        })?;
    }
    #[cfg(not(feature = "simulator"))]
    return Err("use --headless or build with --features simulator".into());
    #[allow(unreachable_code)]
    Ok(())
}
