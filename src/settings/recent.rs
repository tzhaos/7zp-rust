use anyhow::Result;
use std::path::PathBuf;

pub enum Change {
    Load,
    Remember(PathBuf),
    Remove(PathBuf),
    Clear,
}

#[derive(Clone, Copy)]
pub enum Kind {
    Archives,
    Folders,
}

pub fn apply(change: Change, kind: Kind) -> Result<Vec<PathBuf>> {
    let root = super::directory()?;
    let file = root.join(match kind {
        Kind::Archives => "recent-archives.json",
        Kind::Folders => "recent-folders.json",
    });
    let mut paths: Vec<PathBuf> = match std::fs::read(&file) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    let previous = paths.clone();
    match change {
        Change::Load => {}
        Change::Remember(path) => {
            paths.retain(|previous| previous != &path);
            paths.insert(0, path);
            paths.truncate(20);
        }
        Change::Remove(path) => paths.retain(|previous| previous != &path),
        Change::Clear => paths.clear(),
    }
    paths.retain(|path| !matches!(std::fs::metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound));
    if paths != previous {
        std::fs::create_dir_all(root)?;
        std::fs::write(file, serde_json::to_vec(&paths)?)?;
    }
    Ok(paths)
}
