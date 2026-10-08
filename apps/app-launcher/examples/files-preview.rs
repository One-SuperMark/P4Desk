//! Render synthetic file fixtures through the production application shell.
#[cfg(feature = "screenshots")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use app_launcher::{build_launcher_ui, files::*, headless::HeadlessBackend, LauncherState};
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use tiny_flutter::{App, Size};
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or("artifacts/files".into()));
    std::fs::create_dir_all(&out)?;
    if let Some(path) = std::env::args().nth(2) {
        tiny_flutter::graphics::font::install_file_fontpack(Some(Arc::new(
            tiny_flutter::graphics::fontpack::FontPack::from_file(path)?,
        )));
    }
    for light in [false, true] {
        for mode in [
            "list",
            "details",
            "folder-details",
            "details-error",
            "sync-error",
            "sync-success",
            "information",
            "text",
            "image",
            "trash",
            "keyboard",
            "missing-storage",
            "delete-confirm",
            "delete-permanent",
        ] {
            let mut state = LauncherState::new();
            state.settings.light_appearance = light;
            state.open_app("file-manager");
            let listing = DirectoryListing {
                directory: "Documents".into(),
                scanned: 12,
                truncated: false,
                entries: [
                    ("Projects", FileKind::Directory, 0),
                    ("工作计划.txt", FileKind::Text, 1926),
                    ("Welcome.md", FileKind::Text, 4281),
                    ("Holiday.jpg", FileKind::Image, 184020),
                    ("Background.png", FileKind::Image, 272001),
                    ("Report.pdf", FileKind::Other, 930225),
                    ("Notes.json", FileKind::Text, 2802),
                    ("Archive", FileKind::Directory, 0),
                    ("Design.txt", FileKind::Text, 2784),
                    ("Cover.png", FileKind::Image, 184200),
                    ("Schedule.csv", FileKind::Text, 6722),
                    ("Links.txt", FileKind::Text, 790),
                ]
                .into_iter()
                .map(|(name, kind, size)| FileEntry {
                    path: format!("Documents/{name}"),
                    name: name.into(),
                    kind,
                    size,
                    modified_seconds: Some(1791331200),
                    read_only: false,
                })
                .collect(),
            };
            let revision = state.files.revision;
            state.files.apply(Response {
                revision,
                result: Ok(Outcome::Listing(listing)),
            });
            state.files.selected = None;
            match mode {
                "details" => state.files.selected = Some(1),
                "folder-details" => state.files.selected = Some(0),
                "details-error" => {
                    state.files.selected = Some(1);
                    let request = state.files.request(Command::Rename {
                        path: "Documents/工作计划.txt".into(),
                        name: "Welcome.md".into(),
                    });
                    state.apply_files_response(Response {
                        revision: request.revision,
                        result: Err(FileError::AlreadyExists),
                    });
                }
                "sync-error" => {
                    state.show_error(app_launcher::launcher_state::SystemError::SyncFailed);
                }
                "sync-success" => {
                    state.show_notice(app_launcher::launcher_state::SystemNotice::NotesSynced);
                }
                "information" => {
                    state.show_notice(
                        app_launcher::launcher_state::SystemNotice::UnsupportedWifiSecurity,
                    );
                }
                "text" => {
                    let request = state.files.request(Command::Preview {
                        path: "Documents/Welcome.md".into(),
                    });
                    state.files.apply(Response { revision: request.revision, result: Ok(Outcome::Preview(Preview::Text {
                        text: Arc::from("欢迎使用 P4Desk 文件管理\n\n在 TF 卡上浏览文件、预览文字和图片。\n\n可以创建文件夹、重命名、复制和移动。\n文件移入回收站后仍然可以恢复。\n\nMac 应用可通过 USB 传入文件。\n\nDocuments / Pictures / Downloads\n".repeat(6)), truncated: false,
                    })) });
                }
                "image" => {
                    let pixels = (0..640 * 400)
                        .map(|i| {
                            let x = i % 640;
                            let y = i / 640;
                            tiny_flutter::tiny_gfx::rgb888_to_rgb565(
                                (x * 255 / 640) as u8,
                                (y * 255 / 400) as u8,
                                if ((x / 48) + (y / 48)) % 2 == 0 {
                                    150
                                } else {
                                    80
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                    let request = state.files.request(Command::Preview {
                        path: "Documents/Background.png".into(),
                    });
                    state.files.apply(Response {
                        revision: request.revision,
                        result: Ok(Outcome::Preview(Preview::Image {
                            width: 640,
                            height: 400,
                            original_width: 1024,
                            original_height: 600,
                            pixels: pixels.into(),
                        })),
                    });
                }
                "trash" => {
                    let request = state.files.request(Command::ListTrash);
                    state.files.apply(Response {
                        revision: request.revision,
                        result: Ok(Outcome::TrashListing(vec![TrashEntry {
                            id: "synthetic-trash-entry".into(),
                            original_path: "Documents/Old.txt".into(),
                            name: "Old.txt".into(),
                            kind: FileKind::Text,
                            size: 926,
                            deleted_seconds: 1791331200,
                        }])),
                    });
                    state.files.selected = Some(0);
                }
                "keyboard" => state
                    .files
                    .begin_dialog(ui::Dialog::CreateFolder, "New Folder".into()),
                "delete-confirm" => state.files.begin_dialog(
                    ui::Dialog::Trash {
                        path: "Documents/工作计划.txt".into(),
                        name: "工作计划.txt".into(),
                    },
                    String::new(),
                ),
                "delete-permanent" => state.files.begin_dialog(
                    ui::Dialog::DeletePermanently {
                        id: "synthetic-trash-entry".into(),
                        name: "工作计划.txt".into(),
                    },
                    String::new(),
                ),
                "missing-storage" => {
                    let revision = state.files.revision;
                    state.files.apply(Response {
                        revision,
                        result: Err(FileError::StorageUnavailable),
                    });
                }
                _ => (),
            }
            let shared = Arc::new(Mutex::new(state));
            let mut backend = HeadlessBackend::new(1024, 600);
            let size = Size::new(1024.0, 600.0);
            let mut app = App::new(build_launcher_ui(shared, size), size);
            app.step(&mut backend);
            backend.screenshot(&out.join(format!(
                "{}-{mode}.png",
                if light { "light" } else { "dark" }
            )))?;
        }
    }
    Ok(())
}
#[cfg(not(feature = "screenshots"))]
fn main() {
    eprintln!("Use --features screenshots to render the file browser preview.");
}
