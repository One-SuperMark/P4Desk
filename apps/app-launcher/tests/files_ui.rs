//! Production file browser hit testing and asynchronous response ownership.
use app_launcher::{
    build_launcher_ui,
    files::{
        ui::{details_bounds, text_pages, Clipboard, Dialog, State, ROW_HEIGHT},
        *,
    },
    headless::HeadlessBackend,
    LauncherState, UiCommand,
};
use std::sync::Arc;
use std::sync::Mutex;
use tiny_flutter::{App, Point, Size, TouchEvent};

fn listing(count: usize) -> DirectoryListing {
    DirectoryListing {
        directory: String::new(),
        truncated: false,
        scanned: count,
        entries: (0..count)
            .map(|i| FileEntry {
                path: format!("entry-{i}.txt"),
                name: format!("entry-{i}.txt"),
                kind: if i == 0 {
                    FileKind::Directory
                } else {
                    FileKind::Text
                },
                size: 1234,
                modified_seconds: Some(1791331200),
                read_only: false,
            })
            .collect(),
    }
}
fn fixture(count: usize) -> Arc<Mutex<LauncherState>> {
    let mut state = LauncherState::new();
    state.open_app("file-manager");
    let revision = state.files.revision;
    assert!(state.files.apply(Response {
        revision,
        result: Ok(Outcome::Listing(listing(count)))
    }));
    state.take_commands();
    Arc::new(Mutex::new(state))
}
fn app(state: &Arc<Mutex<LauncherState>>) -> (App, HeadlessBackend) {
    let mut backend = HeadlessBackend::new(1024, 600);
    let mut app = App::new(
        build_launcher_ui(state.clone(), Size::new(1024.0, 600.0)),
        Size::new(1024.0, 600.0),
    );
    app.step(&mut backend);
    (app, backend)
}
fn tap(
    app: &mut App,
    backend: &mut HeadlessBackend,
    state: &Arc<Mutex<LauncherState>>,
    x: f32,
    y: f32,
) {
    for event in [
        TouchEvent::Down(Point::new(x + 24.0, y + 78.0)),
        TouchEvent::Up(Point::new(x + 24.0, y + 78.0)),
    ] {
        backend.event(event);
        app.step_with_builder(backend, |size| build_launcher_ui(state.clone(), size));
    }
}
#[test]
fn file_tap_opens_centered_details_and_preview_queues_a_worker_request() {
    let state = fixture(2);
    let (mut app, mut backend) = app(&state);
    tap(
        &mut app,
        &mut backend,
        &state,
        280.0,
        110.0 + ROW_HEIGHT + 22.0,
    );
    assert_eq!(state.lock().unwrap().files.selected, Some(1));
    assert!(state.lock().unwrap().take_commands().is_empty());
    let bounds = details_bounds(Size::new(976.0, 506.0));
    assert_eq!(bounds.x + bounds.width * 0.5, 488.0);
    assert_eq!(bounds.y + bounds.height * 0.5 + 78.0, 300.0);
    tap(
        &mut app,
        &mut backend,
        &state,
        bounds.x + 140.0,
        bounds.y + 275.0,
    );
    let mut state = state.lock().unwrap();
    assert!(state.files.busy);
    assert!(
        matches!(state.take_commands().as_slice(), [UiCommand::Files(Request { command: Command::Preview { path }, .. })] if path == "entry-1.txt")
    );
}
#[test]
fn directory_tap_enters_it_directly_without_a_details_modal() {
    let state = fixture(2);
    let (mut app, mut backend) = app(&state);
    tap(&mut app, &mut backend, &state, 280.0, 132.0);
    let mut state = state.lock().unwrap();
    assert_eq!(state.files.selected, None);
    assert_eq!(state.files.directory, "entry-0.txt");
    assert!(
        matches!(state.take_commands().as_slice(), [UiCommand::Files(Request { command: Command::List { directory, .. }, .. })] if directory == "entry-0.txt")
    );
}
#[test]
fn folder_more_exposes_actions_while_its_main_area_still_opens_the_directory() {
    let state = fixture(2);
    let (mut app, mut backend) = app(&state);
    tap(&mut app, &mut backend, &state, 946.0, 132.0);
    assert_eq!(state.lock().unwrap().files.selected, Some(0));
    assert!(state.lock().unwrap().take_commands().is_empty());
    let bounds = details_bounds(Size::new(976.0, 506.0));
    tap(
        &mut app,
        &mut backend,
        &state,
        bounds.x + 140.0,
        bounds.y + 329.0,
    );
    let copied = state.lock().unwrap().files.clipboard.clone().unwrap();
    assert_eq!(copied.source, "entry-0.txt");
    assert!(!copied.cut);
    assert_eq!(state.lock().unwrap().files.selected, None);
    tap(&mut app, &mut backend, &state, 946.0, 132.0);
    tap(
        &mut app,
        &mut backend,
        &state,
        bounds.x + 410.0,
        bounds.y + 275.0,
    );
    assert!(matches!(
        state.lock().unwrap().files.dialog,
        Some(Dialog::Rename { ref path }) if path == "entry-0.txt"
    ));
    tap(&mut app, &mut backend, &state, 700.0, 467.0);
    tap(
        &mut app,
        &mut backend,
        &state,
        bounds.x + 140.0,
        bounds.y + 275.0,
    );
    assert!(
        matches!(state.lock().unwrap().take_commands().as_slice(), [UiCommand::Files(Request { command: Command::List { directory, .. }, .. })] if directory == "entry-0.txt")
    );
}
#[test]
fn dragging_from_folder_more_cancels_both_more_and_directory_clicks() {
    let state = fixture(32);
    let (mut app, mut backend) = app(&state);
    for event in [
        TouchEvent::Down(Point::new(970.0, 210.0)),
        TouchEvent::Move(Point::new(970.0, 250.0)),
        TouchEvent::Up(Point::new(970.0, 250.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert_eq!(state.lock().unwrap().files.selected, None);
    assert!(state.lock().unwrap().files.directory.is_empty());
    assert!(state.lock().unwrap().take_commands().is_empty());
}
#[test]
fn a_failed_operation_uses_a_closable_dialog_and_leaves_no_error_banner() {
    use app_launcher::launcher_state::{DialogContent, SystemError};
    let state = fixture(2);
    state.lock().unwrap().files.selected = Some(1);
    let (_, resting) = app(&state);
    let revision = state
        .lock()
        .unwrap()
        .files
        .request(Command::Rename {
            path: "entry-1.txt".into(),
            name: "existing.txt".into(),
        })
        .revision;
    {
        let mut state = state.lock().unwrap();
        assert!(state.apply_files_response(Response {
            revision,
            result: Err(FileError::AlreadyExists)
        }));
        assert_eq!(
            state.active_dialog().unwrap().content,
            DialogContent::Error(SystemError::File(FileError::AlreadyExists))
        );
        assert_eq!(state.files.selected, Some(1));
    }
    let (mut app, mut failed) = app(&state);
    assert!(resting.pixels != failed.pixels);
    tap(&mut app, &mut failed, &state, 488.0, 313.0);
    assert!(state.lock().unwrap().active_dialog().is_none());
    assert_eq!(state.lock().unwrap().files.selected, Some(1));
    let (_, closed) = self::app(&state);
    assert!(
        resting.pixels == closed.pixels,
        "a failed operation left a persistent error banner"
    );
}
#[test]
fn automatic_list_failure_is_silent_but_an_explicit_retry_has_a_dialog() {
    let state = fixture(2);
    let mut state = state.lock().unwrap();
    let silent = state.files.refresh_request_silent();
    assert!(state.apply_files_response(Response {
        revision: silent.revision,
        result: Err(FileError::NoSpace)
    }));
    assert!(state.active_dialog().is_none());
    let explicit = state.files.refresh_request();
    assert!(!state.apply_files_response(Response {
        revision: silent.revision,
        result: Err(FileError::NotFound)
    }));
    assert!(state.active_dialog().is_none());
    assert!(state.apply_files_response(Response {
        revision: explicit.revision,
        result: Err(FileError::NoSpace)
    }));
    assert!(state.active_dialog().is_some());
}
#[test]
fn vertical_drag_is_continuous_and_does_not_select_a_file_then_tap_selects_visible_row() {
    let state = fixture(32);
    let (mut app, mut backend) = app(&state);
    for (kind, y) in [(0, 400.0), (1, 330.0), (1, 250.0), (1, 170.0), (2, 170.0)] {
        let point = Point::new(324.0, y + 78.0);
        backend.event(match kind {
            0 => TouchEvent::Down(point),
            1 => TouchEvent::Move(point),
            _ => TouchEvent::Up(point),
        });
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!((state.lock().unwrap().files.scroll.offset() - 230.0).abs() < 1.0);
    assert_eq!(state.lock().unwrap().files.selected, None);
    assert!(state.lock().unwrap().take_commands().is_empty());
    tap(&mut app, &mut backend, &state, 280.0, 151.0);
    assert_eq!(state.lock().unwrap().files.selected, Some(5));
    // Tapping the dimmed background dismisses the modal without touching the list.
    tap(&mut app, &mut backend, &state, 20.0, 420.0);
    assert_eq!(state.lock().unwrap().files.selected, None);
    assert!((state.lock().unwrap().files.scroll.offset() - 230.0).abs() < 1.0);
}
#[test]
fn drag_starting_on_an_entry_does_not_select_or_open_it() {
    let state = fixture(2);
    let (mut app, mut backend) = app(&state);
    for event in [
        TouchEvent::Down(Point::new(310.0, 210.0)),
        TouchEvent::Move(Point::new(480.0, 260.0)),
        TouchEvent::Up(Point::new(480.0, 260.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert_eq!(state.lock().unwrap().files.selected, None);
    assert!(state.lock().unwrap().take_commands().is_empty());
}
#[test]
fn protected_files_allow_directory_open_but_block_mutation_and_file_preview() {
    let state = fixture(1);
    {
        let mut state = state.lock().unwrap();
        let mut value = listing(1);
        value.entries[0].read_only = true;
        value.entries[0].kind = FileKind::Text;
        state.files.listing = Some(Arc::new(value));
        state.files.selected = Some(0);
    }
    let (mut app, mut backend) = app(&state);
    for (x, y) in [
        (356.0, 313.0),
        (618.0, 313.0),
        (356.0, 367.0),
        (488.0, 421.0),
    ] {
        tap(&mut app, &mut backend, &state, x, y);
    }
    let mut state = state.lock().unwrap();
    assert!(state.take_commands().is_empty());
    assert_eq!(state.files.dialog, None);
    assert_eq!(state.files.clipboard, None);
}
#[test]
fn trash_and_permanent_delete_require_their_own_confirmation() {
    let state = fixture(2);
    state.lock().unwrap().files.selected = Some(1);
    let (mut app, mut backend) = app(&state);
    tap(&mut app, &mut backend, &state, 488.0, 390.0);
    assert!(matches!(
        state.lock().unwrap().files.dialog,
        Some(Dialog::Trash { .. })
    ));
    assert!(state.lock().unwrap().take_commands().is_empty());
    tap(&mut app, &mut backend, &state, 856.0, 467.0);
    assert!(
        matches!(state.lock().unwrap().take_commands().as_slice(), [UiCommand::Files(Request { command: Command::Trash { path }, .. })] if path == "entry-1.txt")
    );
    let mut files = State::default();
    files.begin_dialog(
        Dialog::DeletePermanently {
            id: "owned-id".into(),
            name: "file.txt".into(),
        },
        String::new(),
    );
    assert!(
        matches!(files.submit_dialog().unwrap().command, Command::DeletePermanently { id } if id == "owned-id")
    );
}
#[test]
fn stale_responses_cannot_restore_closed_previews_or_override_a_new_directory() {
    let mut state = State::default();
    let first = state.request(Command::Preview {
        path: "old.txt".into(),
    });
    state.close();
    assert!(!state.apply(Response {
        revision: first.revision,
        result: Ok(Outcome::Preview(Preview::Text {
            text: Arc::from("old"),
            truncated: false
        }))
    }));
    assert!(state.preview.is_none());
    let latest = state.request(Command::List {
        directory: "Pictures".into(),
        query: String::new(),
        sort: SortOrder::Name,
    });
    assert!(!state.apply(Response {
        revision: first.revision,
        result: Ok(Outcome::Listing(listing(2)))
    }));
    assert_eq!(state.directory, "Pictures");
    assert!(state.busy);
    assert!(state.apply(Response {
        revision: latest.revision,
        result: Err(FileError::StorageUnavailable)
    }));
    assert!(!state.busy);
    assert_eq!(state.error, Some(FileError::StorageUnavailable));
}
#[test]
fn refresh_selection_follows_file_identity_instead_of_a_reused_row_index() {
    let mut state = State::default();
    let req = state.refresh_request();
    state.apply(Response {
        revision: req.revision,
        result: Ok(Outcome::Listing(listing(3))),
    });
    state.selected = Some(1);
    let mut reordered = listing(3);
    reordered.entries.swap(1, 2);
    let req = state.refresh_request();
    state.apply(Response {
        revision: req.revision,
        result: Ok(Outcome::Listing(reordered)),
    });
    assert_eq!(state.selected, Some(2));
    let req = state.refresh_request();
    state.apply(Response {
        revision: req.revision,
        result: Ok(Outcome::Listing(listing(1))),
    });
    assert_eq!(state.selected, None);
}
#[test]
fn paste_targets_the_current_directory_and_cannot_target_protected_storage() {
    let mut state = State::default();
    state.directory = "Pictures".into();
    state.clipboard = Some(Clipboard {
        source: "Downloads/picture.png".into(),
        name: "picture.png".into(),
        cut: false,
    });
    assert!(
        matches!(state.paste_request().unwrap().command, Command::Copy { source, destination } if source == "Downloads/picture.png" && destination == "Pictures")
    );
    state.busy = false;
    state.directory = "p4desk/resources".into();
    assert!(state.paste_request().is_none());
    state.directory = String::new();
    state.trash_mode = true;
    assert!(state.paste_request().is_none());
}
#[test]
fn close_releases_large_image_payload_and_text_pages_stay_bounded() {
    let mut state = State::default();
    let pixels: Arc<[u16]> = vec![0x1234; 640 * 400].into();
    let weak = Arc::downgrade(&pixels);
    state.preview = Some(Preview::Image {
        width: 640,
        height: 400,
        original_width: 1024,
        original_height: 600,
        pixels,
    });
    state.close();
    assert!(weak.upgrade().is_none());
    let pages = text_pages(&"中文abc\t\r\n".repeat(1024));
    assert!(pages.len() > 1);
    for page in &pages {
        assert!(page.lines().count() <= 14);
        for line in page.lines() {
            assert!(
                line.chars()
                    .map(|c| if c.is_ascii() { 1 } else { 2 })
                    .sum::<usize>()
                    <= 84
            );
        }
        assert!(!page.contains('\r') && !page.contains('\t'));
    }
    let pages = text_pages(&"长".repeat(3000));
    assert_eq!(pages.concat().replace('\n', ""), "长".repeat(3000));
    let pages = text_pages(&"W".repeat(3000));
    assert_eq!(pages.concat().replace('\n', ""), "W".repeat(3000));
    for page in pages {
        for line in page.lines() {
            assert!(
                tiny_flutter::Font::file_font()
                    .measure_text(line, 18.0)
                    .width
                    <= 840.1
            );
        }
    }
}

#[test]
fn reboot_restores_file_directory_and_sort_without_restoring_a_preview_or_pending_operation() {
    use app_launcher::{
        session::{SavedApp, Session},
        ActiveApp,
    };
    let mut state = LauncherState::new();
    state.open_app("file-manager");
    state.files.directory = "Documents/Projects".into();
    state.files.sort = SortOrder::Size;
    state.files.query = "private-search".into();
    state.files.preview = Some(Preview::Text {
        text: Arc::from("synthetic private preview"),
        truncated: false,
    });
    state.files.clipboard = Some(Clipboard {
        source: "Documents/file.txt".into(),
        name: "file.txt".into(),
        cut: true,
    });
    let checkpoint = Session::capture(&state);
    assert!(
        matches!(checkpoint.foreground, Some(SavedApp::Files { ref directory, sort: 2 }) if directory == "Documents/Projects")
    );
    let bytes = serde_json::to_vec(&checkpoint).unwrap();
    assert!(!std::str::from_utf8(&bytes).unwrap().contains("private"));
    let checkpoint: Session = serde_json::from_slice(&bytes).unwrap();
    let mut restored = LauncherState::new();
    assert!(checkpoint.restore(&mut restored));
    assert!(matches!(restored.active_app, ActiveApp::Files));
    assert_eq!(restored.files.directory, "Documents/Projects");
    assert_eq!(restored.files.sort, SortOrder::Size);
    assert!(restored.files.query.is_empty());
    assert!(!restored.files.busy);
    assert!(restored.files.preview.is_none());
    assert!(restored.files.clipboard.is_none());
}
#[test]
fn oldest_background_file_app_eviction_releases_its_preview_payload() {
    use app_launcher::launcher_state::MAX_BACKGROUND_APPS;
    let mut state = LauncherState::new();
    state.open_app("file-manager");
    let pixels: Arc<[u16]> = vec![0x1234; 640 * 400].into();
    let weak = Arc::downgrade(&pixels);
    state.files.preview = Some(Preview::Image {
        width: 640,
        height: 400,
        original_width: 1024,
        original_height: 600,
        pixels,
    });
    state.background_active_app();
    assert!(weak.upgrade().is_some());
    for name in ["clock", "timer", "calculator", "notes"] {
        state.open_app(name);
        state.background_active_app();
    }
    assert_eq!(state.background_app_ids().count(), MAX_BACKGROUND_APPS);
    assert!(!state.running_apps.contains_key("file-manager"));
    assert!(state.files.preview.is_none());
    assert!(weak.upgrade().is_none());
}
#[test]
fn a_successful_folder_creation_keeps_cut_clipboard_until_its_move_succeeds() {
    let mut state = State::default();
    state.clipboard = Some(Clipboard {
        source: "source.txt".into(),
        name: "source.txt".into(),
        cut: true,
    });
    let request = state.request(Command::CreateFolder {
        directory: String::new(),
        name: "destination".into(),
    });
    state.apply(Response {
        revision: request.revision,
        result: Ok(Outcome::Changed {
            directory: String::new(),
        }),
    });
    assert!(state.clipboard.is_some());
    state.directory = "destination".into();
    let request = state.paste_request().unwrap();
    state.apply(Response {
        revision: request.revision,
        result: Err(FileError::NoSpace),
    });
    assert!(state.clipboard.is_some());
    let request = state.paste_request().unwrap();
    state.apply(Response {
        revision: request.revision,
        result: Ok(Outcome::Changed {
            directory: "destination".into(),
        }),
    });
    assert!(state.clipboard.is_none());
}

#[test]
fn a_long_directory_scroll_uses_no_tall_raster_and_matches_a_fresh_view() {
    use tiny_flutter::widgets::take_scroll_metrics;
    let state = fixture(256);
    take_scroll_metrics();
    let (mut app, mut backend) = app(&state);
    for event in [
        TouchEvent::Down(Point::new(324.0, 478.0)),
        TouchEvent::Move(Point::new(324.0, 208.0)),
        TouchEvent::Up(Point::new(324.0, 208.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!(state.lock().unwrap().files.scroll.offset() > 250.0);
    assert_eq!(state.lock().unwrap().files.selected, None);
    let metrics = take_scroll_metrics();
    assert_eq!(
        metrics.builds, 0,
        "256 rows must exceed the bounded full-content cache and use viewport rows"
    );
    let (_, fresh) = self::app(&state);
    assert!(
        backend.pixels == fresh.pixels,
        "partial scroll output must match a fresh resting frame"
    );
}
#[test]
fn text_previews_scroll_vertically_and_return_releases_the_scroll_surface() {
    let state = fixture(2);
    {
        let mut state = state.lock().unwrap();
        let request = state.files.request(Command::Preview {
            path: "entry-1.txt".into(),
        });
        state.files.apply(Response {
            revision: request.revision,
            result: Ok(Outcome::Preview(Preview::Text {
                text: Arc::from("long synthetic line\n".repeat(400)),
                truncated: false,
            })),
        });
        state.take_commands();
    }
    let (mut app, mut backend) = app(&state);
    for event in [
        TouchEvent::Down(Point::new(324.0, 478.0)),
        TouchEvent::Move(Point::new(324.0, 278.0)),
        TouchEvent::Up(Point::new(324.0, 278.0)),
    ] {
        backend.event(event);
        app.step_with_builder(&mut backend, |size| build_launcher_ui(state.clone(), size));
    }
    assert!((state.lock().unwrap().files.preview_scroll.offset() - 200.0).abs() < 1.0);
    let (_, fresh) = self::app(&state);
    assert!(
        backend.pixels == fresh.pixels,
        "vertical text scroll differs from a fresh frame"
    );
    tap(&mut app, &mut backend, &state, 900.0, 20.0);
    let state = state.lock().unwrap();
    assert!(state.files.preview.is_none());
    assert!(state.files.preview_text_pages.is_empty());
    assert_eq!(state.files.preview_scroll.offset(), 0.0);
}
#[test]
fn a_removed_card_clears_old_rows_and_selected_details() {
    let mut state = State::default();
    let request = state.refresh_request();
    state.apply(Response {
        revision: request.revision,
        result: Ok(Outcome::Listing(listing(2))),
    });
    state.selected = Some(1);
    let request = state.refresh_request();
    state.apply(Response {
        revision: request.revision,
        result: Err(FileError::StorageUnavailable),
    });
    assert_eq!(state.item_count(), 0);
    assert_eq!(state.selected, None);
    assert!(state.preview.is_none());
}
