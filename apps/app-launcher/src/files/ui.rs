//! Asynchronous Folio file browser. File operations are queued for the storage worker.
use super::*;
use crate::{LauncherState, UiCommand};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

pub const ROW_HEIGHT: f32 = 54.0;
const SIDEBAR: f32 = 176.0;
const ROW_CACHE_LIMIT: usize = 12;
const TEXT_LINES: usize = 14;
const TEXT_COLUMNS: usize = 84;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dialog {
    Search,
    CreateFolder,
    Rename { path: String },
    Trash { path: String, name: String },
    DeletePermanently { id: String, name: String },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clipboard {
    pub source: String,
    pub name: String,
    pub cut: bool,
}
#[derive(Clone)]
pub struct State {
    pub revision: u64,
    pub busy: bool,
    pub directory: String,
    pub query: String,
    pub sort: SortOrder,
    pub listing: Option<Arc<DirectoryListing>>,
    pub preview: Option<Preview>,
    pub trash: Arc<Vec<TrashEntry>>,
    pub trash_mode: bool,
    pub selected: Option<usize>,
    pub scroll: ScrollController,
    pub preview_scroll: ScrollController,
    pub preview_text_pages: Arc<Vec<String>>,
    pub error: Option<FileError>,
    pub dialog: Option<Dialog>,
    pub input: String,
    pub uppercase: bool,
    pub symbols: bool,
    pub clipboard: Option<Clipboard>,
    pending_move: bool,
    silent_revision: Option<u64>,
    list_content_revision: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            revision: 0,
            busy: false,
            directory: String::new(),
            query: String::new(),
            sort: SortOrder::Name,
            listing: None,
            preview: None,
            trash: Arc::new(Vec::new()),
            trash_mode: false,
            selected: None,
            scroll: ScrollController::new(),
            preview_scroll: ScrollController::new(),
            preview_text_pages: Arc::new(Vec::new()),
            error: None,
            dialog: None,
            input: String::new(),
            uppercase: false,
            symbols: false,
            clipboard: None,
            pending_move: false,
            silent_revision: None,
            list_content_revision: 0,
        }
    }
}
impl State {
    pub fn request(&mut self, command: Command) -> Request {
        self.silent_revision = None;
        self.revision = self.revision.wrapping_add(1);
        self.busy = true;
        self.error = None;
        self.dialog = None;
        self.pending_move = matches!(command, Command::Move { .. });
        match &command {
            Command::List {
                directory,
                query,
                sort,
            } => {
                if self.directory != *directory
                    || self.query != *query
                    || self.sort != *sort
                    || self.trash_mode
                {
                    self.scroll.set_offset(0.0);
                    self.selected = None;
                    self.listing = None;
                }
                self.directory.clone_from(directory);
                self.query.clone_from(query);
                self.sort = *sort;
                self.trash_mode = false;
                self.preview = None;
                self.preview_scroll = ScrollController::new();
                self.preview_text_pages = Arc::new(Vec::new());
            }
            Command::ListTrash => {
                if !self.trash_mode {
                    self.scroll.set_offset(0.0);
                    self.selected = None;
                }
                self.trash_mode = true;
                self.preview = None;
                self.preview_scroll = ScrollController::new();
                self.preview_text_pages = Arc::new(Vec::new());
            }
            Command::Preview { .. } => {
                self.selected = None;
                self.preview = None;
                self.preview_scroll = ScrollController::new();
                self.preview_text_pages = Arc::new(Vec::new());
            }
            _ => {}
        }
        Request {
            revision: self.revision,
            command,
        }
    }
    pub fn refresh_request(&mut self) -> Request {
        let command = if self.trash_mode {
            Command::ListTrash
        } else {
            Command::List {
                directory: self.directory.clone(),
                query: self.query.clone(),
                sort: self.sort,
            }
        };
        self.request(command)
    }
    /// Automatic refreshes update the page state without creating repeated
    /// operation-result popups. A later explicit request clears this marker.
    pub fn refresh_request_silent(&mut self) -> Request {
        let request = self.refresh_request();
        self.silent_revision = Some(request.revision);
        request
    }
    pub fn is_silent_response(&self, revision: u64) -> bool {
        self.silent_revision == Some(revision)
    }
    pub fn apply(&mut self, response: Response) -> bool {
        if response.revision != self.revision {
            return false;
        }
        self.busy = false;
        match response.result {
            Ok(Outcome::Listing(listing)) => {
                if self
                    .listing
                    .as_ref()
                    .is_none_or(|old| old.as_ref() != &listing)
                {
                    self.list_content_revision = self.list_content_revision.wrapping_add(1);
                }
                let selected_path = self.selected.and_then(|index| {
                    self.listing
                        .as_ref()
                        .and_then(|listing| listing.entries.get(index))
                        .map(|entry| entry.path.clone())
                });
                self.selected = selected_path
                    .and_then(|path| listing.entries.iter().position(|entry| entry.path == path));
                self.directory.clone_from(&listing.directory);
                self.listing = Some(Arc::new(listing));
                self.trash_mode = false;
                self.clamp();
            }
            Ok(Outcome::Preview(preview)) => {
                self.preview_text_pages = Arc::new(match &preview {
                    Preview::Text { text, .. } => text_pages(text),
                    _ => Vec::new(),
                });
                self.preview = Some(preview);
                self.preview_scroll = ScrollController::new();
            }
            Ok(Outcome::TrashListing(entries)) => {
                if self.trash.as_ref() != &entries {
                    self.list_content_revision = self.list_content_revision.wrapping_add(1);
                }
                let selected_id = self
                    .selected
                    .and_then(|index| self.trash.get(index).map(|entry| entry.id.clone()));
                self.selected =
                    selected_id.and_then(|id| entries.iter().position(|entry| entry.id == id));
                self.trash = Arc::new(entries);
                self.trash_mode = true;
                self.clamp();
            }
            Ok(Outcome::Changed { directory }) => {
                if !self.trash_mode {
                    self.directory = directory;
                }
                self.selected = None;
                self.preview = None;
                self.preview_scroll = ScrollController::new();
                self.preview_text_pages = Arc::new(Vec::new());
                if self.pending_move {
                    self.clipboard = None;
                }
            }
            Err(error) => {
                self.error = Some(error);
                if error == FileError::StorageUnavailable {
                    self.listing = None;
                    self.preview = None;
                    self.preview_scroll = ScrollController::new();
                    self.preview_text_pages = Arc::new(Vec::new());
                    self.trash = Arc::new(Vec::new());
                    self.selected = None;
                }
            }
        }
        true
    }
    /// Drop bounded image/text payloads and invalidate an in-flight preview on close.
    pub fn close(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.busy = false;
        self.selected = None;
        self.preview = None;
        self.preview_text_pages = Arc::new(Vec::new());
        self.preview_scroll = ScrollController::new();
        self.dialog = None;
        self.input.clear();
        self.pending_move = false;
        self.silent_revision = None;
        self.scroll = ScrollController::with_offset(self.scroll.offset());
    }
    pub fn item_count(&self) -> usize {
        if self.trash_mode {
            self.trash.len()
        } else {
            self.listing.as_ref().map_or(0, |v| v.entries.len())
        }
    }
    pub fn clamp(&mut self) {
        if self
            .selected
            .is_some_and(|index| index >= self.item_count())
        {
            self.selected = None;
        }
    }
    pub fn begin_dialog(&mut self, dialog: Dialog, input: String) {
        if self.busy {
            return;
        }
        self.dialog = Some(dialog);
        self.input = input;
        self.uppercase = false;
        self.symbols = false;
        self.error = None;
    }
    pub fn submit_dialog(&mut self) -> Option<Request> {
        let command = match self.dialog.as_ref()? {
            Dialog::Search => Command::List {
                directory: self.directory.clone(),
                query: self.input.trim().into(),
                sort: self.sort,
            },
            Dialog::CreateFolder => Command::CreateFolder {
                directory: self.directory.clone(),
                name: self.input.trim().into(),
            },
            Dialog::Rename { path } => Command::Rename {
                path: path.clone(),
                name: self.input.trim().into(),
            },
            Dialog::Trash { path, .. } => Command::Trash { path: path.clone() },
            Dialog::DeletePermanently { id, .. } => Command::DeletePermanently { id: id.clone() },
        };
        if matches!(&command, Command::CreateFolder { name, .. } | Command::Rename { name, .. } if name.is_empty())
        {
            self.error = Some(FileError::InvalidName);
            return None;
        }
        Some(self.request(command))
    }
    pub fn paste_request(&mut self) -> Option<Request> {
        if self.busy || self.trash_mode || protected(&self.directory) {
            return None;
        }
        let clipboard = self.clipboard.as_ref()?;
        let destination = self.directory.clone();
        let command = if clipboard.cut {
            Command::Move {
                source: clipboard.source.clone(),
                destination,
            }
        } else {
            Command::Copy {
                source: clipboard.source.clone(),
                destination,
            }
        };
        Some(self.request(command))
    }
}
fn protected(path: &str) -> bool {
    path.split('/')
        .next()
        .is_some_and(|segment| segment.eq_ignore_ascii_case("p4desk"))
}
fn edit(shared: &Arc<Mutex<LauncherState>>, edit: impl FnOnce(&mut LauncherState)) {
    let mut state = shared.lock().unwrap();
    edit(&mut state);
    state.changed();
}
fn command(shared: &Arc<Mutex<LauncherState>>, command: Command) {
    edit(shared, |state| {
        if state.files.busy {
            return;
        }
        let request = state.files.request(command);
        state.queue(UiCommand::Files(request));
    });
}
fn at(widget: impl Widget + 'static, x: f32, y: f32) -> Positioned {
    Positioned::new(widget).left(x).top(y)
}
fn text(label: impl Into<String>, size: f32, color: Color) -> Text {
    let size = nearest_size(
        size,
        &[14.0, 16.0, 18.0, 22.0, 24.0, 26.0, 28.0, 34.0, 36.0],
    );
    Text::new(label).font_size(size).color(color)
}
fn file_text(label: impl Into<String>, size: f32, color: Color) -> Text {
    text(label, nearest_size(size, &[14.0, 18.0, 22.0]), color).font(Font::file_font().clone())
}
fn nearest_size(size: f32, sizes: &[f32]) -> f32 {
    sizes
        .iter()
        .copied()
        .min_by(|a, b| (a - size).abs().total_cmp(&(b - size).abs()))
        .unwrap()
}
fn panel(w: f32, h: f32, color: Color) -> Container {
    Container::new()
        .width(w)
        .height(h)
        .color(color)
        .border_radius(Folio::CONTROL_RADIUS)
}
fn button(
    label: impl Into<String>,
    w: f32,
    h: f32,
    selected: bool,
    enabled: bool,
    action: impl Fn() + Send + Sync + 'static,
) -> ElevatedButton {
    let fill = if selected {
        Folio::accent()
    } else {
        Folio::raised()
    };
    ElevatedButton::new(text(
        label,
        17.0,
        if enabled {
            Folio::ink()
        } else {
            Folio::disabled()
        },
    ))
    .style(Folio::control_style(w, h, fill).padding(EdgeInsets::all(6.0)))
    .on_pressed(move || {
        if enabled {
            action();
        }
    })
}
fn file_short(label: &str, maximum_width: f32, size: f32) -> String {
    let label = if label.chars().any(char::is_control) {
        std::borrow::Cow::Owned(
            label
                .chars()
                .map(|character| {
                    if character.is_control() {
                        ' '
                    } else {
                        character
                    }
                })
                .collect::<String>(),
        )
    } else {
        std::borrow::Cow::Borrowed(label)
    };
    let label = label.as_ref();
    let size = nearest_size(size, &[14.0, 18.0, 22.0]);
    let font = Font::file_font();
    if label
        .chars()
        .map(|character| font.advance(character, size))
        .sum::<f32>()
        <= maximum_width
    {
        return label.into();
    }
    let mut result = String::new();
    let mut width = 0.0;
    let ellipsis = font.advance('…', size);
    for character in label.chars() {
        let advance = font.advance(character, size);
        if width + advance + ellipsis > maximum_width {
            break;
        }
        result.push(character);
        width += advance;
    }
    result.push('…');
    result
}
pub fn human_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
fn kind_label(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Directory => "文件夹",
        FileKind::Text => "文本",
        FileKind::Image => "图片",
        FileKind::Other => "文件",
    }
}

/// Antialiased code vectors for document types; folder artwork follows the active icon pack.
struct FileGlyph(FileKind);
impl CustomPainter for FileGlyph {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        if self.0 == FileKind::Directory {
            if let Some(icon) = crate::app_icons::get_app_icon_asset("file-manager") {
                icon.paint(
                    canvas,
                    Rect::from_ltwh(0.0, 0.0, size.width, size.height),
                    Folio::blue(),
                );
            }
            return;
        }
        let x = size.width * 0.21;
        let y = size.height * 0.08;
        let w = size.width * 0.58;
        let h = size.height * 0.82;
        let color = match self.0 {
            FileKind::Text => Folio::accent_ink(),
            FileKind::Image => Folio::orange(),
            _ => Folio::muted(),
        };
        canvas.draw_rrect_aa(
            RRect::from_rect_circular(Rect::from_ltwh(x, y, w, h), 4.0),
            color,
        );
        let mut path = tiny_gfx::PathBuilder::new();
        if self.0 == FileKind::Image {
            path.move_to(x + 4.0, y + h - 8.0);
            path.line_to(x + w * 0.42, y + h * 0.50);
            path.line_to(x + w * 0.63, y + h * 0.72);
            path.line_to(x + w - 4.0, y + h * 0.48);
            canvas.draw_circle(
                Point::new(x + w * 0.67, y + h * 0.29),
                2.4,
                Folio::surface(),
            );
        } else {
            for fraction in [0.36, 0.52, 0.68] {
                path.move_to(x + 5.0, y + h * fraction);
                path.line_to(x + w - 5.0, y + h * fraction);
            }
        }
        if let Some(path) = path.finish() {
            canvas.stroke_path(
                &path,
                &tiny_gfx::Paint::new(Folio::surface().to_gfx()),
                &tiny_gfx::Stroke {
                    width: 1.7,
                    line_cap: tiny_gfx::LineCap::Round,
                    ..Default::default()
                },
            );
        }
    }
}
fn glyph(kind: FileKind, size: f32) -> CustomPaint {
    CustomPaint::new(FileGlyph(kind)).size(Size::new(size, size))
}

pub fn build(shared: Arc<Mutex<LauncherState>>, size: Size) -> impl Widget {
    let state = {
        let state = shared.lock().unwrap();
        state.files.clone()
    };
    let main_x = SIDEBAR + 16.0;
    let list_width = size.width - main_x;
    let list_height = size.height - 134.0;
    let mut view = Stack::new()
        .push(
            Container::new()
                .width(size.width)
                .height(size.height)
                .color(Folio::bg()),
        )
        .push(at(panel(SIDEBAR, size.height, Folio::surface()), 0.0, 0.0))
        .push(at(text("位置", 16.0, Folio::muted()), 18.0, 18.0));
    for (index, (label, directory)) in [
        ("TF 卡", ""),
        ("我的文件", "Files"),
        ("Documents", "Documents"),
        ("Pictures", "Pictures"),
        ("Downloads", "Downloads"),
    ]
    .into_iter()
    .enumerate()
    {
        let target = directory.to_owned();
        let selected = !state.trash_mode && state.directory == directory;
        let shared = shared.clone();
        let child = Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .push(glyph(FileKind::Directory, 28.0))
            .push(SizedBox::from_size(Size::new(8.0, 0.0)))
            .push(text(label, 18.0, Folio::ink()));
        view = view.push(at(
            ElevatedButton::new(child)
                .style(
                    Folio::control_style(
                        SIDEBAR - 20.0,
                        48.0,
                        if selected {
                            Folio::accent()
                        } else {
                            Folio::surface()
                        },
                    )
                    .padding(EdgeInsets::symmetric(8.0, 6.0)),
                )
                .on_pressed(move || {
                    command(
                        &shared,
                        Command::List {
                            directory: target.clone(),
                            query: String::new(),
                            sort: SortOrder::Name,
                        },
                    )
                }),
            10.0,
            46.0 + index as f32 * 52.0,
        ));
    }
    let trash = shared.clone();
    view = view
        .push(at(
            button(
                "回收站",
                SIDEBAR - 20.0,
                48.0,
                state.trash_mode,
                !state.busy,
                move || command(&trash, Command::ListTrash),
            ),
            10.0,
            322.0,
        ))
        .push(at(
            text("Mac 可通过 USB\n传入文件", 14.0, Folio::muted()),
            18.0,
            size.height - 88.0,
        ));
    let parent = parent_directory(&state.directory);
    let parent_shared = shared.clone();
    view = view.push(at(
        button(
            "‹",
            42.0,
            40.0,
            false,
            !state.busy && !state.trash_mode && !state.directory.is_empty(),
            move || {
                command(
                    &parent_shared,
                    Command::List {
                        directory: parent.clone(),
                        query: String::new(),
                        sort: SortOrder::Name,
                    },
                )
            },
        ),
        main_x,
        0.0,
    ));
    let heading = if state.trash_mode {
        text("回收站", 22.0, Folio::ink())
    } else if state.directory.is_empty() {
        text("TF 卡", 22.0, Folio::ink())
    } else {
        file_text(
            file_short(&state.directory, list_width - 140.0, 22.0),
            22.0,
            Folio::ink(),
        )
    };
    view = view.push(at(heading, main_x + 54.0, 9.0));
    let refresh = shared.clone();
    view = view.push(at(
        button("刷新", 70.0, 40.0, false, !state.busy, move || {
            edit(&refresh, |s| {
                let request = s.files.refresh_request();
                s.queue(UiCommand::Files(request));
            })
        }),
        size.width - 76.0,
        0.0,
    ));
    if !state.trash_mode {
        let search = shared.clone();
        let search_label = if state.query.is_empty() {
            text("搜索", 18.0, Folio::ink())
        } else {
            file_text(file_short(&state.query, 110.0, 18.0), 18.0, Folio::ink())
        };
        let search_enabled = !state.busy;
        view = view.push(at(
            ElevatedButton::new(search_label)
                .style(
                    Folio::control_style(
                        130.0,
                        40.0,
                        if state.query.is_empty() {
                            Folio::raised()
                        } else {
                            Folio::accent()
                        },
                    )
                    .padding(EdgeInsets::all(6.0)),
                )
                .on_pressed(move || {
                    if search_enabled {
                        edit(&search, |s| {
                            s.files.begin_dialog(Dialog::Search, s.files.query.clone())
                        });
                    }
                }),
            main_x,
            52.0,
        ));
        let next_sort = match state.sort {
            SortOrder::Name => SortOrder::Modified,
            SortOrder::Modified => SortOrder::Size,
            SortOrder::Size => SortOrder::Name,
        };
        let sort_label = match state.sort {
            SortOrder::Name => "名称 ↑",
            SortOrder::Modified => "日期 ↓",
            SortOrder::Size => "大小 ↓",
        };
        let sort = shared.clone();
        let directory = state.directory.clone();
        let query = state.query.clone();
        view = view.push(at(
            button(sort_label, 98.0, 40.0, false, !state.busy, move || {
                command(
                    &sort,
                    Command::List {
                        directory: directory.clone(),
                        query: query.clone(),
                        sort: next_sort,
                    },
                )
            }),
            main_x + 138.0,
            52.0,
        ));
        let create = shared.clone();
        view = view.push(at(
            button(
                "新建文件夹",
                132.0,
                40.0,
                false,
                !state.busy && !protected(&state.directory),
                move || {
                    edit(&create, |s| {
                        s.files.begin_dialog(Dialog::CreateFolder, String::new())
                    })
                },
            ),
            size.width - 132.0,
            52.0,
        ));
        if let Some(clipboard) = &state.clipboard {
            let paste = shared.clone();
            view = view.push(at(
                button(
                    if clipboard.cut {
                        "移动到此文件夹"
                    } else {
                        "粘贴到此文件夹"
                    },
                    174.0,
                    40.0,
                    true,
                    !state.busy
                        && !protected(&state.directory)
                        && state.error != Some(FileError::StorageUnavailable),
                    move || {
                        edit(&paste, |s| {
                            if let Some(request) = s.files.paste_request() {
                                s.queue(UiCommand::Files(request));
                            }
                        })
                    },
                ),
                size.width - 318.0,
                52.0,
            ));
        }
    } else {
        view = view.push(at(
            text("删除的文件可在这里恢复", 18.0, Folio::muted()),
            main_x + 4.0,
            61.0,
        ));
    }
    view = view.push(at(
        panel(list_width, list_height, Folio::surface()),
        main_x,
        102.0,
    ));
    if state.item_count() > 0 && state.error != Some(FileError::StorageUnavailable) {
        let cache_key = state.list_content_revision
            ^ (tiny_flutter::graphics::font::file_fontpack_revision() as u64).rotate_left(17)
            ^ (tiny_flutter::graphics::font::typeface_revision() as u64).rotate_left(37)
            ^ ((state.trash_mode as u64) << 63)
            ^ ((crate::icon_theme::raster_key() as u64) << 40);
        view = view.push(at(
            Container::new()
                .width(list_width - 16.0)
                .height(list_height - 16.0)
                .child(
                    SingleChildScrollView::new(CachedScrollBody {
                        state: state.clone(),
                        shared: shared.clone(),
                        minimum_height: list_height - 16.0,
                    })
                    .controller(state.scroll.clone())
                    .raster_cache(cache_key, Folio::surface()),
                ),
            main_x + 8.0,
            110.0,
        ));
    } else {
        let empty = if state.error == Some(FileError::StorageUnavailable) {
            "TF 卡未就绪，请检查后刷新"
        } else if state.busy {
            "正在读取…"
        } else if state.trash_mode {
            "回收站为空"
        } else if !state.query.is_empty() {
            "没有找到文件"
        } else {
            "此文件夹为空"
        };
        view = view.push(at(text(empty, 22.0, Folio::muted()), main_x + 24.0, 175.0));
    }
    view = view.push(at(
        text(
            format!(
                "{} 项{}",
                state.item_count(),
                if state.listing.as_ref().is_some_and(|v| v.truncated) && !state.trash_mode {
                    " · 列表已达上限"
                } else {
                    ""
                }
            ),
            14.0,
            Folio::muted(),
        ),
        main_x + 4.0,
        size.height - 24.0,
    ));
    if state.busy {
        view = view.push(at(
            text("处理中…", 14.0, Folio::muted()),
            size.width - 108.0,
            size.height - 24.0,
        ));
    }
    if state.selected.is_some() {
        view = view.push(details_view(shared.clone(), &state, size));
    }
    if state.preview.is_some() {
        view = view.push(preview_view(shared.clone(), &state, size));
    }
    if state.dialog.is_some() {
        view = view.push(dialog_view(shared, &state, size));
    }
    view
}

/// The scroll renderer retains at most 2 MiB. Long lists use this virtual body and
/// only materialize visible rows, so 256 entries never become one tall bitmap.
struct CachedScrollBody {
    state: State,
    shared: Arc<Mutex<LauncherState>>,
    minimum_height: f32,
}
impl Widget for CachedScrollBody {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderCachedScrollBody {
            state: self.state.clone(),
            shared: self.shared.clone(),
            minimum_height: self.minimum_height,
            size: Size::ZERO,
            offset: Offset::ZERO,
            rows: Mutex::new(RowCache::default()),
        })
    }
}
#[derive(Default)]
struct RowCache {
    rows: std::collections::VecDeque<(usize, Box<dyn RenderBox>)>,
    pressed: Option<usize>,
}
struct RenderCachedScrollBody {
    state: State,
    shared: Arc<Mutex<LauncherState>>,
    minimum_height: f32,
    size: Size,
    offset: Offset,
    rows: Mutex<RowCache>,
}
struct MorePainter;
impl CustomPainter for MorePainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        for dx in [-6.0, 0.0, 6.0] {
            canvas.draw_circle(
                Point::new(size.width * 0.5 + dx, size.height * 0.5),
                1.8,
                Folio::ink(),
            );
        }
    }
}
impl RenderCachedScrollBody {
    fn row<'a>(&self, cache: &'a mut RowCache, index: usize) -> &'a mut Box<dyn RenderBox> {
        if let Some(at) = cache.rows.iter().position(|(id, _)| *id == index) {
            return &mut cache.rows.get_mut(at).unwrap().1;
        }
        if cache.rows.len() >= ROW_CACHE_LIMIT {
            let oldest = cache
                .rows
                .iter()
                .position(|(id, _)| Some(*id) != cache.pressed)
                .unwrap_or(0);
            cache.rows.remove(oldest);
        }
        let (name, kind, size, read_only, path) = if self.state.trash_mode {
            let e = &self.state.trash[index];
            (
                e.name.clone(),
                e.kind,
                e.size,
                false,
                e.original_path.clone(),
            )
        } else {
            let e = &self.state.listing.as_ref().unwrap().entries[index];
            (e.name.clone(), e.kind, e.size, e.read_only, e.path.clone())
        };
        let shared = self.shared.clone();
        let trash = self.state.trash_mode;
        let enabled = !self.state.busy;
        let has_more = !trash && kind == FileKind::Directory;
        let main_width = self.size.width - if has_more { 52.0 } else { 0.0 };
        let child = Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .push(glyph(kind, 32.0))
            .push(SizedBox::from_size(Size::new(12.0, 0.0)))
            .push(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .push(file_text(
                        file_short(&name, main_width - 64.0, 18.0),
                        18.0,
                        Folio::ink(),
                    ))
                    .push(text(
                        if read_only {
                            "系统管理 · 只读".into()
                        } else if kind == FileKind::Directory {
                            "文件夹".into()
                        } else {
                            format!("{} · {}", kind_label(kind), human_size(size))
                        },
                        14.0,
                        Folio::muted(),
                    )),
            );
        let main = ElevatedButton::new(child)
            .style(
                Folio::control_style(main_width, ROW_HEIGHT - 4.0, Folio::surface())
                    .padding(EdgeInsets::symmetric(8.0, 4.0)),
            )
            .on_pressed(move || {
                if enabled {
                    if !trash && kind == FileKind::Directory {
                        command(
                            &shared,
                            Command::List {
                                directory: path.clone(),
                                query: String::new(),
                                sort: SortOrder::Name,
                            },
                        );
                    } else {
                        edit(&shared, |s| {
                            if !s.files.busy {
                                s.files.selected = Some(index);
                            }
                        });
                    }
                }
            });
        let mut row = if has_more {
            let more = self.shared.clone();
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .push(main)
                .push(SizedBox::from_size(Size::new(8.0, 0.0)))
                .push(
                    ElevatedButton::new(CustomPaint::new(MorePainter).size(Size::new(24.0, 24.0)))
                        .style(
                            Folio::control_style(44.0, 44.0, Folio::raised())
                                .padding(EdgeInsets::all(6.0)),
                        )
                        .on_pressed(move || {
                            if enabled {
                                edit(&more, |s| {
                                    if !s.files.busy {
                                        s.files.selected = Some(index);
                                    }
                                });
                            }
                        }),
                )
                .create_render_object()
        } else {
            main.create_render_object()
        };
        row.layout(&BoxConstraints::tight(Size::new(
            self.size.width,
            ROW_HEIGHT - 4.0,
        )));
        cache.rows.push_back((index, row));
        &mut cache.rows.back_mut().unwrap().1
    }
}
impl RenderBox for RenderCachedScrollBody {
    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = constraints.constrain(Size::new(
            constraints.max_width,
            (self.state.item_count() as f32 * ROW_HEIGHT).max(self.minimum_height),
        ));
        self.size
    }
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        canvas.draw_rect(
            Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height),
            Folio::surface(),
        );
        let clip = canvas.current_clip().unwrap_or(Rect::from_ltwh(
            offset.dx,
            offset.dy,
            self.size.width,
            self.size.height,
        ));
        let start = ((clip.y - offset.dy).max(0.0) / ROW_HEIGHT).floor() as usize;
        let end = (((clip.bottom() - offset.dy).max(0.0) / ROW_HEIGHT).ceil() as usize)
            .min(self.state.item_count());
        let mut cache = self.rows.lock().unwrap();
        for index in start.min(end)..end {
            self.row(&mut cache, index).paint(
                canvas,
                Offset::new(offset.dx, offset.dy + index as f32 * ROW_HEIGHT),
            );
        }
    }
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let mut cache = self.rows.lock().unwrap();
        let point = event.point();
        if matches!(event, TouchEvent::Cancel) {
            if let Some(index) = cache.pressed.take() {
                self.row(&mut cache, index).dispatch_touch(event);
            }
            return false;
        }
        let index = if let Some(index) = cache.pressed {
            index
        } else {
            (point.y.max(0.0) / ROW_HEIGHT).floor() as usize
        };
        if index >= self.state.item_count() {
            return false;
        }
        if matches!(event, TouchEvent::Down(_)) {
            cache.pressed = Some(index);
        }
        let local = Point::new(point.x, point.y - index as f32 * ROW_HEIGHT);
        let handled = self
            .row(&mut cache, index)
            .dispatch_touch(&event.transform(local));
        if matches!(event, TouchEvent::Up(_)) {
            cache.pressed = None;
        }
        handled
    }
    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        let index = (point.y.max(0.0) / ROW_HEIGHT).floor() as usize;
        if index >= self.state.item_count() {
            return;
        }
        let mut cache = self.rows.lock().unwrap();
        self.row(&mut cache, index).set_pressed_at(
            Point::new(point.x, point.y - index as f32 * ROW_HEIGHT),
            pressed,
        );
    }
    fn hit_rect(&self, point: Point) -> Option<Rect> {
        let index = (point.y.max(0.0) / ROW_HEIGHT).floor() as usize;
        (index < self.state.item_count()).then(|| {
            Rect::from_ltwh(
                0.0,
                index as f32 * ROW_HEIGHT,
                self.size.width,
                ROW_HEIGHT - 4.0,
            )
        })
    }
}

pub fn details_bounds(size: Size) -> Rect {
    let w = 560.0_f32.min(size.width - 32.0);
    let h = 430.0_f32.min(size.height - 32.0);
    // The shell reserves 78 px above this content and 16 px below it.
    // Center the dialog on the physical display, including the navigation bar.
    let content_top = crate::flip_clock::CLOCK_CONTENT_ORIGIN.dy;
    let center_y = (size.height + content_top + 16.0) * 0.5 - content_top;
    Rect::from_ltwh(
        (size.width - w) * 0.5,
        (center_y - h * 0.5).clamp(0.0, size.height - h),
        w,
        h,
    )
}
fn details_view(shared: Arc<Mutex<LauncherState>>, state: &State, size: Size) -> Stack {
    let bounds = details_bounds(size);
    let dismiss = shared.clone();
    let close = shared.clone();
    let mut content = Stack::new()
        .push(at(text("文件信息", 22.0, Folio::ink()), 24.0, 20.0))
        .push(at(
            button("×", 40.0, 40.0, false, true, move || {
                edit(&close, |s| s.files.selected = None)
            }),
            bounds.width - 56.0,
            12.0,
        ));
    let index = state.selected.unwrap();
    if state.trash_mode {
        if let Some(entry) = state.trash.get(index) {
            content = content
                .push(at(glyph(entry.kind, 64.0), bounds.width * 0.5 - 32.0, 67.0))
                .push(at(
                    file_text(
                        file_short(&entry.name, bounds.width - 48.0, 22.0),
                        22.0,
                        Folio::ink(),
                    ),
                    24.0,
                    149.0,
                ))
                .push(at(
                    file_text(
                        file_short(&entry.original_path, bounds.width - 48.0, 14.0),
                        14.0,
                        Folio::muted(),
                    ),
                    24.0,
                    191.0,
                ))
                .push(at(
                    text(
                        format!("{} · {}", kind_label(entry.kind), human_size(entry.size)),
                        18.0,
                        Folio::muted(),
                    ),
                    24.0,
                    224.0,
                ));
            let restore = shared.clone();
            let id = entry.id.clone();
            content = content.push(at(
                button(
                    "恢复到原位置",
                    bounds.width - 48.0,
                    44.0,
                    true,
                    !state.busy,
                    move || command(&restore, Command::Restore { id: id.clone() }),
                ),
                24.0,
                276.0,
            ));
            let dialog = Dialog::DeletePermanently {
                id: entry.id.clone(),
                name: entry.name.clone(),
            };
            content = content.push(at(
                button(
                    "永久删除…",
                    bounds.width - 48.0,
                    44.0,
                    false,
                    !state.busy,
                    move || {
                        edit(&shared, |s| {
                            s.files.begin_dialog(dialog.clone(), String::new())
                        })
                    },
                )
                .style(
                    Folio::control_style(bounds.width - 48.0, 44.0, Folio::destructive_fill())
                        .padding(EdgeInsets::all(6.0)),
                ),
                24.0,
                332.0,
            ));
        }
    } else if let Some(entry) = state
        .listing
        .as_ref()
        .and_then(|listing| listing.entries.get(index))
    {
        content = content
            .push(at(glyph(entry.kind, 64.0), bounds.width * 0.5 - 32.0, 67.0))
            .push(at(
                file_text(
                    file_short(&entry.name, bounds.width - 48.0, 22.0),
                    22.0,
                    Folio::ink(),
                ),
                24.0,
                149.0,
            ))
            .push(at(
                text(
                    format!("{} · {}", kind_label(entry.kind), human_size(entry.size)),
                    18.0,
                    Folio::muted(),
                ),
                24.0,
                191.0,
            ));
        if entry.read_only {
            content = content.push(at(
                text("系统管理 · 只读", 14.0, Folio::muted()),
                24.0,
                222.0,
            ));
        } else if let Some(time) = entry
            .modified_seconds
            .and_then(|value| i64::try_from(value).ok())
            .and_then(|value| chrono::DateTime::from_timestamp(value, 0))
        {
            let time = time.with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap());
            content = content.push(at(
                text(
                    format!("修改于 {}", time.format("%m-%d %H:%M")),
                    14.0,
                    Folio::muted(),
                ),
                24.0,
                222.0,
            ));
        }
        let width = (bounds.width - 60.0) * 0.5;
        let enabled = !state.busy && !entry.read_only;
        let open = shared.clone();
        let path = entry.path.clone();
        let directory = entry.kind == FileKind::Directory;
        content = content.push(at(
            button(
                if directory { "打开" } else { "预览" },
                width,
                42.0,
                true,
                !state.busy && (directory || !entry.read_only),
                move || {
                    command(
                        &open,
                        if directory {
                            Command::List {
                                directory: path.clone(),
                                query: String::new(),
                                sort: SortOrder::Name,
                            }
                        } else {
                            Command::Preview { path: path.clone() }
                        },
                    )
                },
            ),
            24.0,
            254.0,
        ));
        let rename = shared.clone();
        let dialog = Dialog::Rename {
            path: entry.path.clone(),
        };
        let name = entry.name.clone();
        content = content.push(at(
            button("重命名", width, 42.0, false, enabled, move || {
                edit(&rename, |s| {
                    s.files.begin_dialog(dialog.clone(), name.clone())
                })
            }),
            36.0 + width,
            254.0,
        ));
        for (cut, label, x) in [(false, "复制", 24.0), (true, "剪切", 36.0 + width)] {
            let copy = shared.clone();
            let clipboard = Clipboard {
                source: entry.path.clone(),
                name: entry.name.clone(),
                cut,
            };
            content = content.push(at(
                button(label, width, 42.0, false, enabled, move || {
                    edit(&copy, |s| {
                        s.files.clipboard = Some(clipboard.clone());
                        s.files.selected = None;
                    })
                }),
                x,
                308.0,
            ));
        }
        let dialog = Dialog::Trash {
            path: entry.path.clone(),
            name: entry.name.clone(),
        };
        content = content.push(at(
            button(
                "移到回收站…",
                bounds.width - 48.0,
                42.0,
                false,
                enabled,
                move || {
                    edit(&shared, |s| {
                        s.files.begin_dialog(dialog.clone(), String::new())
                    })
                },
            ),
            24.0,
            362.0,
        ));
    }
    Stack::new()
        .push(
            GestureDetector::new(
                Container::new()
                    .width(size.width)
                    .height(size.height)
                    .color(Color::BLACK.with_opacity(0.45)),
            )
            .on_tap(move || edit(&dismiss, |s| s.files.selected = None)),
        )
        .push(at(
            GestureDetector::new(
                panel(bounds.width, bounds.height, Folio::surface()).child(content),
            )
            .on_tap(|| {}),
            bounds.x,
            bounds.y,
        ))
}

/// Wrap a bounded UTF-8 preview into screen-sized chunks for virtualized vertical scrolling.
pub fn text_pages(text: &str) -> Vec<String> {
    let mut pages = Vec::new();
    let mut page = String::new();
    let mut columns = 0;
    let mut line_width = 0.0;
    let mut lines = 1;
    for character in text.chars() {
        if character == '\r' {
            continue;
        }
        let character = if character == '\t' { ' ' } else { character };
        let width = if character.is_ascii() { 1 } else { 2 };
        let advance = Font::file_font().advance(character, 18.0);
        if character == '\n' || columns + width > TEXT_COLUMNS || line_width + advance > 840.0 {
            if lines == TEXT_LINES {
                pages.push(std::mem::take(&mut page));
                lines = 1;
            } else {
                page.push('\n');
                lines += 1;
            }
            columns = 0;
            line_width = 0.0;
            if character == '\n' {
                continue;
            }
        }
        if !character.is_control() {
            page.push(character);
            columns += width;
            line_width += advance;
        }
    }
    if !page.is_empty() || pages.is_empty() {
        pages.push(page);
    }
    pages
}
const TEXT_BLOCK_HEIGHT: f32 = 350.0;
struct VisibleTextBody {
    pages: Arc<Vec<String>>,
    minimum_height: f32,
}
impl Widget for VisibleTextBody {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderVisibleTextBody {
            pages: self.pages.clone(),
            minimum_height: self.minimum_height,
            size: Size::ZERO,
            offset: Offset::ZERO,
            blocks: Mutex::new(std::collections::VecDeque::new()),
        })
    }
}
struct RenderVisibleTextBody {
    pages: Arc<Vec<String>>,
    minimum_height: f32,
    size: Size,
    offset: Offset,
    blocks: Mutex<std::collections::VecDeque<(usize, Box<dyn RenderBox>)>>,
}
impl RenderBox for RenderVisibleTextBody {
    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = constraints.constrain(Size::new(
            constraints.max_width,
            (self.pages.len() as f32 * TEXT_BLOCK_HEIGHT).max(self.minimum_height),
        ));
        self.size
    }
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        canvas.draw_rect(
            Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height),
            Folio::surface(),
        );
        let clip = canvas.current_clip().unwrap_or(Rect::from_ltwh(
            offset.dx,
            offset.dy,
            self.size.width,
            self.size.height,
        ));
        let start = ((clip.y - offset.dy).max(0.0) / TEXT_BLOCK_HEIGHT).floor() as usize;
        let end = (((clip.bottom() - offset.dy).max(0.0) / TEXT_BLOCK_HEIGHT).ceil() as usize)
            .min(self.pages.len());
        let mut blocks = self.blocks.lock().unwrap();
        for index in start.min(end)..end {
            let at = if let Some(at) = blocks.iter().position(|(id, _)| *id == index) {
                at
            } else {
                if blocks.len() >= 4 {
                    blocks.pop_front();
                }
                let mut block =
                    file_text(self.pages[index].clone(), 18.0, Folio::ink()).create_render_object();
                block.layout(&BoxConstraints::loose(Size::new(
                    self.size.width,
                    TEXT_BLOCK_HEIGHT,
                )));
                blocks.push_back((index, block));
                blocks.len() - 1
            };
            blocks[at].1.paint(
                canvas,
                Offset::new(offset.dx, offset.dy + index as f32 * TEXT_BLOCK_HEIGHT),
            );
        }
    }
}

struct ImagePainter {
    width: u32,
    height: u32,
    pixels: Arc<[u16]>,
}
impl CustomPainter for ImagePainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        canvas.save();
        canvas.clip_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height));
        canvas.blit_image_565(
            ((size.width - self.width as f32) * 0.5) as i32,
            ((size.height - self.height as f32) * 0.5) as i32,
            self.width,
            self.height,
            &self.pixels,
        );
        canvas.restore();
    }
}
fn preview_view(shared: Arc<Mutex<LauncherState>>, state: &State, size: Size) -> Stack {
    let close = shared.clone();
    let mut view = Stack::new()
        .push(
            Container::new()
                .width(size.width)
                .height(size.height)
                .color(Folio::bg()),
        )
        .push(at(text("文件预览", 24.0, Folio::ink()), 18.0, 8.0))
        .push(at(
            button("返回列表", 114.0, 40.0, false, true, move || {
                edit(&close, |s| {
                    s.files.preview = None;
                    s.files.preview_scroll = ScrollController::new();
                    s.files.preview_text_pages = Arc::new(Vec::new());
                })
            }),
            size.width - 132.0,
            0.0,
        ));
    match state.preview.as_ref().unwrap() {
        Preview::Text {
            text: content,
            truncated,
        } => {
            let pages = if state.preview_text_pages.is_empty() {
                Arc::new(text_pages(content))
            } else {
                state.preview_text_pages.clone()
            };
            let cache_key = state.revision
                ^ (tiny_flutter::graphics::font::file_fontpack_revision() as u64).rotate_left(29)
                ^ (tiny_flutter::graphics::font::typeface_revision() as u64).rotate_left(43);
            view = view.push(at(
                panel(size.width - 36.0, 390.0, Folio::surface()).child(Padding::new(
                    EdgeInsets::all(18.0),
                    Container::new()
                        .width(size.width - 72.0)
                        .height(354.0)
                        .child(
                            SingleChildScrollView::new(VisibleTextBody {
                                pages,
                                minimum_height: 354.0,
                            })
                            .controller(state.preview_scroll.clone())
                            .raster_cache(cache_key, Folio::surface()),
                        ),
                )),
                18.0,
                52.0,
            ));
            if *truncated {
                view = view.push(at(
                    text("文件较大，仅预览前一部分", 14.0, Folio::muted()),
                    20.0,
                    size.height - 39.0,
                ));
            }
        }
        Preview::Image {
            width,
            height,
            original_width,
            original_height,
            pixels,
        } => {
            view = view
                .push(at(
                    CustomPaint::new(ImagePainter {
                        width: *width,
                        height: *height,
                        pixels: pixels.clone(),
                    })
                    .size(Size::new(size.width - 36.0, 400.0)),
                    18.0,
                    52.0,
                ))
                .push(at(
                    text(
                        format!("{original_width} × {original_height}"),
                        16.0,
                        Folio::muted(),
                    ),
                    20.0,
                    size.height - 28.0,
                ));
        }
        Preview::Unsupported => {
            view = view
                .push(at(
                    crate::widgets::app_badge("file-manager", 100.0),
                    size.width * 0.5 - 50.0,
                    138.0,
                ))
                .push(at(
                    text("暂不支持预览此文件格式", 22.0, Folio::muted()),
                    size.width * 0.5 - 143.0,
                    268.0,
                ));
        }
    }
    view
}
fn dialog_view(shared: Arc<Mutex<LauncherState>>, state: &State, size: Size) -> Stack {
    let dialog = state.dialog.as_ref().unwrap();
    let keyboard = matches!(
        dialog,
        Dialog::Search | Dialog::CreateFolder | Dialog::Rename { .. }
    );
    let title = match dialog {
        Dialog::Search => "搜索当前文件夹",
        Dialog::CreateFolder => "新建文件夹",
        Dialog::Rename { .. } => "重命名",
        Dialog::Trash { .. } => "移到回收站?",
        Dialog::DeletePermanently { .. } => "永久删除?",
    };
    let mut view = Stack::new()
        .push(
            Container::new()
                .width(size.width)
                .height(size.height)
                .color(Folio::bg()),
        )
        .push(at(text(title, 27.0, Folio::ink()), 36.0, 24.0));
    if keyboard {
        let input = if state.input.is_empty() {
            text("请输入…", 22.0, Folio::muted())
        } else {
            file_text(
                file_short(&state.input, size.width - 100.0, 22.0),
                22.0,
                Folio::ink(),
            )
        };
        view = view.push(at(
            panel(size.width - 72.0, 54.0, Folio::surface())
                .child(Padding::new(EdgeInsets::all(14.0), input)),
            36.0,
            77.0,
        ));
        let rows = if state.symbols {
            vec!["1234567890", "!@#$%&*()-", "_+=[]{}.,", "~`'\";:?"]
        } else {
            vec!["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"]
        };
        let key_width = (size.width - 90.0) / 10.0 - 4.0;
        for (row_index, row) in rows.into_iter().enumerate() {
            let offset = (10 - row.chars().count()) as f32 * (key_width + 4.0) * 0.5;
            for (column, key) in row.chars().enumerate() {
                let key = if state.uppercase {
                    key.to_ascii_uppercase()
                } else {
                    key
                };
                let shared = shared.clone();
                view = view.push(at(
                    button(key.to_string(), key_width, 47.0, false, true, move || {
                        edit(&shared, |s| {
                            if s.files.input.len() + key.len_utf8() <= 120 {
                                s.files.input.push(key);
                            }
                        })
                    }),
                    45.0 + offset + column as f32 * (key_width + 4.0),
                    153.0 + row_index as f32 * 54.0,
                ));
            }
        }
        let upper = shared.clone();
        let symbols = shared.clone();
        let space = shared.clone();
        let delete = shared.clone();
        view = view
            .push(at(
                button(
                    if state.uppercase { "小写" } else { "大写" },
                    106.0,
                    42.0,
                    state.uppercase,
                    true,
                    move || edit(&upper, |s| s.files.uppercase = !s.files.uppercase),
                ),
                45.0,
                374.0,
            ))
            .push(at(
                button(
                    if state.symbols { "ABC" } else { "符号" },
                    106.0,
                    42.0,
                    state.symbols,
                    true,
                    move || edit(&symbols, |s| s.files.symbols = !s.files.symbols),
                ),
                161.0,
                374.0,
            ))
            .push(at(
                button("空格", size.width - 550.0, 42.0, false, true, move || {
                    edit(&space, |s| {
                        if s.files.input.len() < 120 {
                            s.files.input.push(' ');
                        }
                    })
                }),
                277.0,
                374.0,
            ))
            .push(at(
                button("退格", 128.0, 42.0, false, true, move || {
                    edit(&delete, |s| {
                        s.files.input.pop();
                    })
                }),
                size.width - 253.0,
                374.0,
            ));
        let clear = shared.clone();
        view = view.push(at(
            button("清空", 70.0, 42.0, false, true, move || {
                edit(&clear, |s| s.files.input.clear())
            }),
            size.width - 115.0,
            374.0,
        ));
    } else {
        let (name, message) = match dialog {
            Dialog::Trash { name, .. } => (name, "文件会移到回收站，可随时恢复。"),
            Dialog::DeletePermanently { name, .. } => {
                (name, "此操作无法撤销，文件将从 TF 卡永久移除。")
            }
            _ => unreachable!(),
        };
        view = view
            .push(at(glyph(FileKind::Other, 84.0), 48.0, 110.0))
            .push(at(
                file_text(
                    file_short(name, size.width - 192.0, 22.0),
                    24.0,
                    Folio::ink(),
                ),
                156.0,
                124.0,
            ))
            .push(at(text(message, 22.0, Folio::muted()), 48.0, 247.0));
    }
    let cancel = shared.clone();
    let label = match dialog {
        Dialog::Search => "搜索",
        Dialog::CreateFolder => "创建",
        Dialog::Rename { .. } => "保存",
        Dialog::Trash { .. } => "移到回收站",
        Dialog::DeletePermanently { .. } => "确认永久删除",
    };
    let destructive = matches!(dialog, Dialog::DeletePermanently { .. });
    let confirm = button(label, 164.0, 44.0, true, true, move || {
        edit(&shared, |s| {
            if let Some(request) = s.files.submit_dialog() {
                s.queue(UiCommand::Files(request));
            }
        })
    });
    let confirm = if destructive {
        confirm.style(
            Folio::control_style(164.0, 44.0, Folio::destructive_fill())
                .padding(EdgeInsets::all(6.0)),
        )
    } else {
        confirm
    };
    view = view
        .push(at(
            button("取消", 132.0, 44.0, false, true, move || {
                edit(&cancel, |s| {
                    s.files.dialog = None;
                    s.files.error = None;
                })
            }),
            size.width - 346.0,
            size.height - 60.0,
        ))
        .push(at(confirm, size.width - 202.0, size.height - 60.0));
    if let Some(error) = state.error {
        view = view.push(at(
            text(error.message(), 16.0, Folio::destructive()),
            36.0,
            size.height - 49.0,
        ));
    }
    view
}
