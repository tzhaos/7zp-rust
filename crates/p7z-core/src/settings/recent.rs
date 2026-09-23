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
    // Inspect paths before acquiring the SQLite write transaction.
    let previous = store.database.read(super::state::history)?;
    let missing = previous.iter().filter(|entry| matches!(std::fs::metadata(&entry.path), Err(error) if error.kind() == std::io::ErrorKind::NotFound)).collect::<Vec<_>>();
    store.database.write(|transaction| {
        for entry in missing {
            transaction.execute("DELETE FROM history WHERE path = ?1", [entry.path.to_string_lossy().as_ref()])?;
        }
        match change {
            Change::Load => {},
            Change::Remember(path) => super::state::remember(transaction, &Entry { path, kind })?,
            Change::Remove(path) => { transaction.execute("DELETE FROM history WHERE path = ?1", [path.to_string_lossy().as_ref()])?; },
            Change::Clear => { transaction.execute("DELETE FROM history", [])?; },
        }
        transaction.execute("DELETE FROM history WHERE position NOT IN (SELECT position FROM history ORDER BY position DESC LIMIT 20)", [])?;
        super::state::history(transaction)
    })
}
