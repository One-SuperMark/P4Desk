//! Bounded TF file management. The engine does no UI work and never logs paths or contents.
pub mod fs;
pub mod preview;
pub mod ui;

use std::sync::Arc;

pub use fs::{FileEngine, Limits};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Name,
    Modified,
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Directory,
    Text,
    Image,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub kind: FileKind,
    pub size: u64,
    pub modified_seconds: Option<u64>,
    pub read_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryListing {
    pub directory: String,
    pub entries: Vec<FileEntry>,
    pub truncated: bool,
    pub scanned: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrashEntry {
    pub id: String,
    pub original_path: String,
    pub name: String,
    pub kind: FileKind,
    pub size: u64,
    pub deleted_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Preview {
    Text {
        text: Arc<str>,
        truncated: bool,
    },
    Image {
        width: u32,
        height: u32,
        original_width: u32,
        original_height: u32,
        pixels: Arc<[u16]>,
    },
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    List {
        directory: String,
        query: String,
        sort: SortOrder,
    },
    Preview {
        path: String,
    },
    CreateFolder {
        directory: String,
        name: String,
    },
    Rename {
        path: String,
        name: String,
    },
    Copy {
        source: String,
        destination: String,
    },
    Move {
        source: String,
        destination: String,
    },
    Trash {
        path: String,
    },
    ListTrash,
    Restore {
        id: String,
    },
    DeletePermanently {
        id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub revision: u64,
    pub command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Listing(DirectoryListing),
    Preview(Preview),
    Changed { directory: String },
    TrashListing(Vec<TrashEntry>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub revision: u64,
    pub result: Result<Outcome, FileError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileError {
    StorageUnavailable,
    InvalidPath,
    Protected,
    NotFound,
    AlreadyExists,
    NotDirectory,
    InvalidName,
    Unsupported,
    TooLarge,
    TooManyEntries,
    InvalidText,
    InvalidImage,
    NoSpace,
    PermissionDenied,
    Io,
    Busy,
    Integrity,
    Offset,
    Cancelled,
}

impl FileError {
    pub fn message(self) -> &'static str {
        match self {
            Self::StorageUnavailable => "TF 卡未就绪，请检查存储",
            Self::InvalidPath => "无法访问此路径",
            Self::Protected => "系统管理文件受保护",
            Self::NotFound => "文件已不存在，请刷新",
            Self::AlreadyExists => "目标已存在同名文件",
            Self::NotDirectory => "请选择一个文件夹",
            Self::InvalidName => "名称无效，请使用普通文件名",
            Self::Unsupported => "暂不支持此文件格式",
            Self::TooLarge => "文件过大，超出预览或操作上限",
            Self::TooManyEntries => "文件夹内容过多，超出操作上限",
            Self::InvalidText => "文本不是有效的 UTF-8 编码",
            Self::InvalidImage => "图片损坏或格式暂不支持",
            Self::NoSpace => "TF 卡空间不足",
            Self::PermissionDenied => "存储只读或没有访问权限",
            Self::Io => "存储操作失败，请检查 TF 卡",
            Self::Busy => "文件传输正在进行，请稍候",
            Self::Integrity => "文件校验失败，请重新传输",
            Self::Offset => "文件传输顺序不正确，请重新传输",
            Self::Cancelled => "文件传输已取消",
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::StorageUnavailable => "storage_unavailable",
            Self::InvalidPath => "invalid_path",
            Self::Protected => "protected",
            Self::NotFound => "not_found",
            Self::AlreadyExists => "already_exists",
            Self::NotDirectory => "not_directory",
            Self::InvalidName => "invalid_name",
            Self::Unsupported => "unsupported",
            Self::TooLarge => "too_large",
            Self::TooManyEntries => "too_many_entries",
            Self::InvalidText => "invalid_text",
            Self::InvalidImage => "invalid_image",
            Self::NoSpace => "no_space",
            Self::PermissionDenied => "permission_denied",
            Self::Io => "io",
            Self::Busy => "busy",
            Self::Integrity => "integrity",
            Self::Offset => "offset",
            Self::Cancelled => "cancelled",
        }
    }
}

impl From<std::io::Error> for FileError {
    fn from(error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match error.kind() {
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::AlreadyExists => Self::AlreadyExists,
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            _ if error.raw_os_error() == Some(28) => Self::NoSpace,
            _ => Self::Io,
        }
    }
}

pub fn parent_directory(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default()
}

pub fn file_kind(name: &str, directory: bool) -> FileKind {
    if directory {
        return FileKind::Directory;
    }
    let extension = name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "txt" | "md" | "json" | "csv" | "log" | "rs" | "c" | "h" | "toml" | "yaml" | "yml"
        | "ini" | "xml" | "html" => FileKind::Text,
        "png" | "jpg" | "jpeg" | "bmp" => FileKind::Image,
        _ => FileKind::Other,
    }
}
