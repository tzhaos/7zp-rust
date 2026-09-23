use p7z_commands::OpenType;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub type Cancellation = Arc<AtomicBool>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    SevenZip,
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz,
    Wim,
}

impl Format {
    pub const ALL: [Self; 7] = [
        Self::SevenZip,
        Self::Zip,
        Self::Tar,
        Self::Gzip,
        Self::Bzip2,
        Self::Xz,
        Self::Wim,
    ];

    pub fn from_command(format: p7z_commands::ArchiveFormat) -> Self {
        match format {
            p7z_commands::ArchiveFormat::SevenZip => Self::SevenZip,
            p7z_commands::ArchiveFormat::Zip => Self::Zip,
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            Self::SevenZip => "7z",
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::Gzip => "gzip",
            Self::Bzip2 => "bzip2",
            Self::Xz => "xz",
            Self::Wim => "wim",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Gzip => "gz",
            Self::Bzip2 => "bz2",
            other => other.value(),
        }
    }

    pub fn title(self) -> String {
        self.value().to_uppercase()
    }

    pub fn supports_password(self) -> bool {
        matches!(self, Self::SevenZip | Self::Zip)
    }

    pub fn supports_method(self) -> bool {
        self.supports_password()
    }

    pub fn supports_solid(self) -> bool {
        matches!(self, Self::SevenZip)
    }

    pub fn supports_header_encryption(self) -> bool {
        matches!(self, Self::SevenZip)
    }

    pub fn single_file(self) -> bool {
        matches!(self, Self::Gzip | Self::Bzip2 | Self::Xz)
    }

    pub fn default_method(self) -> Method {
        match self {
            Self::SevenZip => Method::Lzma2,
            Self::Zip => Method::Deflate,
            _ => Method::Default,
        }
    }

    pub fn methods(self) -> &'static [Method] {
        match self {
            Self::SevenZip => &[Method::Lzma2, Method::Lzma, Method::Ppmd, Method::Bzip2],
            Self::Zip => &[
                Method::Deflate,
                Method::Deflate64,
                Method::Bzip2,
                Method::Lzma,
                Method::Ppmd,
            ],
            _ => &[Method::Default],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Lzma2,
    Lzma,
    Ppmd,
    Bzip2,
    Deflate,
    Deflate64,
    Default,
}

impl Method {
    pub fn argument(self) -> &'static str {
        match self {
            Self::Lzma2 => "LZMA2",
            Self::Lzma => "LZMA",
            Self::Ppmd => "PPMd",
            Self::Bzip2 => "BZip2",
            Self::Deflate => "Deflate",
            Self::Deflate64 => "Deflate64",
            Self::Default => "default",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Store,
    Fastest,
    Fast,
    Normal,
    Maximum,
    Ultra,
}

impl Level {
    pub const ALL: [Self; 6] = [
        Self::Store,
        Self::Fastest,
        Self::Fast,
        Self::Normal,
        Self::Maximum,
        Self::Ultra,
    ];

    pub fn argument(self) -> &'static str {
        match self {
            Self::Store => "0",
            Self::Fastest => "1",
            Self::Fast => "3",
            Self::Normal => "5",
            Self::Maximum => "7",
            Self::Ultra => "9",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Store => "level-store",
            Self::Fastest => "level-fastest",
            Self::Fast => "level-fast",
            Self::Normal => "level-normal",
            Self::Maximum => "level-maximum",
            Self::Ultra => "level-ultra",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Threads {
    Auto,
    One,
    Two,
    Four,
    Eight,
}

impl Threads {
    pub const ALL: [Self; 5] = [Self::Auto, Self::One, Self::Two, Self::Four, Self::Eight];

    pub fn argument(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::One => Some("1"),
            Self::Two => Some("2"),
            Self::Four => Some("4"),
            Self::Eight => Some("8"),
        }
    }

    pub fn label_key(self) -> Option<&'static str> {
        match self {
            Self::Auto => Some("automatic"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Volume {
    None,
    Megabytes10,
    Megabytes100,
    Megabytes650,
    Gigabyte,
}

impl Volume {
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Megabytes10,
        Self::Megabytes100,
        Self::Megabytes650,
        Self::Gigabyte,
    ];

    pub fn argument(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Megabytes10 => Some("10m"),
            Self::Megabytes100 => Some("100m"),
            Self::Megabytes650 => Some("650m"),
            Self::Gigabyte => Some("1g"),
        }
    }

    pub fn label_key(self) -> Option<&'static str> {
        match self {
            Self::None => Some("volume-none"),
            _ => None,
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Megabytes10 => "10 MB",
            Self::Megabytes100 => "100 MB",
            Self::Megabytes650 => "650 MB",
            Self::Gigabyte => "1 GB",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    RenameIncoming,
    Skip,
    Replace,
    RenameExisting,
}
impl Overwrite {
    pub const ALL: [Self; 4] = [
        Self::RenameIncoming,
        Self::Skip,
        Self::Replace,
        Self::RenameExisting,
    ];
    pub fn label_key(self) -> &'static str {
        match self {
            Self::RenameIncoming => "overwrite-rename-incoming",
            Self::Skip => "overwrite-skip",
            Self::Replace => "overwrite-replace",
            Self::RenameExisting => "overwrite-rename-existing",
        }
    }
    pub(super) fn argument(self) -> &'static str {
        match self {
            Self::RenameIncoming => "-aou",
            Self::Skip => "-aos",
            Self::Replace => "-aoa",
            Self::RenameExisting => "-aot",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: String,
    pub name: String,
    pub directory: bool,
    pub size: Option<u64>,
    pub modified: String,
    pub link: bool,
    pub encrypted: bool,
}

#[derive(Clone)]
pub struct Catalog {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    pub warning: bool,
    pub format: String,
    pub size: u64,
    pub open_type: Option<OpenType>,
}

impl Catalog {
    pub fn editable(&self) -> bool {
        matches!(self.format.as_str(), "7Z" | "ZIP" | "TAR" | "WIM")
            && !matches!(
                self.open_type,
                Some(OpenType::Parser | OpenType::ParserEach)
            )
    }

    pub fn accepts_comment(&self) -> bool {
        self.format == "ZIP" && self.editable()
    }
    pub fn children(&self, folder: &str, search: &str) -> Vec<Entry> {
        let prefix = if folder.is_empty() {
            String::new()
        } else {
            format!("{folder}/")
        };
        let query = search.to_lowercase();
        let mut rows = BTreeMap::new();
        for entry in &self.entries {
            let Some(relative) = entry.path.strip_prefix(&prefix) else {
                continue;
            };
            if relative.is_empty() {
                continue;
            }
            if let Some((name, _)) = relative.split_once('/') {
                let path = format!("{prefix}{name}");
                rows.entry(path.clone()).or_insert(Entry {
                    path,
                    name: name.into(),
                    directory: true,
                    size: None,
                    modified: String::new(),
                    link: false,
                    encrypted: false,
                });
            } else {
                rows.insert(entry.path.clone(), entry.clone());
            }
        }
        if !query.is_empty() {
            for entry in &self.entries {
                if entry.path.starts_with(&prefix) {
                    rows.insert(entry.path.clone(), entry.clone());
                }
            }
        }
        rows.into_values()
            .filter(|e| e.name.to_lowercase().contains(&query))
            .collect()
    }
}

#[derive(Clone)]
pub struct CreateOptions {
    pub format: Format,
    pub level: Level,
    pub method: Method,
    pub threads: Threads,
    pub volume: Volume,
    pub solid: bool,
    pub password: String,
    pub encrypt_names: bool,
}

impl CreateOptions {
    pub fn archive_file_name(&self, name: &str) -> String {
        let extension = self.format.extension();
        if self.volume == Volume::None {
            format!("{name}.{extension}")
        } else {
            format!("{name}.{extension}-volumes.zip")
        }
    }

    pub fn list_password(&self) -> &str {
        if self.volume == Volume::None {
            &self.password
        } else {
            ""
        }
    }
}
