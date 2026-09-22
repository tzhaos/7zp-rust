use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub enum Change {
    Load,
    Remember(PathBuf),
    Remove(PathBuf),
    Clear,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Archives,
    Folders,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: Kind,
}

pub fn apply(change: Change, kind: Kind) -> Result<Vec<Entry>> {
    let store = super::store()?;
    let file = "history.json";
    let saved: Option<Vec<Entry>> = store.read_json(file)?;
    let migrated = saved.is_none();
    let mut entries = match saved {
        Some(entries) => entries,
        None => {
            // Old lists have no timestamps; preserve each list's existing order on import.
            let mut entries = Vec::new();
            for (file, kind) in [
                ("recent-archives.json", Kind::Archives),
                ("recent-folders.json", Kind::Folders),
            ] {
                let paths: Vec<PathBuf> = store.read_json(file)?.unwrap_or_default();
                for path in paths {
                    if !entries.iter().any(|entry: &Entry| entry.path == path) {
                        entries.push(Entry { path, kind });
                    }
                }
            }
            entries
        }
    };
    let previous = entries.clone();
    match change {
        Change::Load => {}
        Change::Remember(path) => {
            entries.retain(|previous| previous.path != path);
            entries.insert(0, Entry { path, kind });
        }
        Change::Remove(path) => entries.retain(|previous| previous.path != path),
        Change::Clear => entries.clear(),
    }
    entries.retain(|entry| !matches!(std::fs::metadata(&entry.path), Err(error) if error.kind() == std::io::ErrorKind::NotFound));
    entries.truncate(20);
    if migrated || entries != previous {
        store.write_json(file, &entries)?;
    }
    Ok(entries)
}
