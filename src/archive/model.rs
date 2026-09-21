use sevenzip_shell_api::OpenType;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub const FORMATS: [&str; 7] = ["7z", "zip", "tar", "gzip", "bzip2", "xz", "wim"];
pub type Cancellation = Arc<AtomicBool>;

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
    pub format: String,
    pub level: String,
    pub method: String,
    pub threads: String,
    pub split: String,
    pub solid: bool,
    pub password: String,
    pub encrypt_names: bool,
}
