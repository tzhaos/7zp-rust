use anyhow::{Context, Result, bail};
pub mod recent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

pub static LOW_PRIORITY: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub remember_recent: bool,
    pub open_after: bool,
    pub close_after_quick: bool,
    pub low_priority: bool,
    pub check_updates: bool,
    pub shell_menu: bool,
    pub associations: Vec<String>,
    pub temp_directory: String,
    pub extract_all: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            remember_recent: true,
            open_after: true,
            close_after_quick: false,
            low_priority: true,
            check_updates: false,
            shell_menu: true,
            associations: [
                ".7z", ".zip", ".rar", ".tar", ".gz", ".bz2", ".xz", ".wim", ".iso", ".cab", ".001",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            temp_directory: String::new(),
            extract_all: "*.exe;*.com;*.msi;*.htm;*.html".into(),
        }
    }
}

pub fn load_preferences() -> Result<Preferences> {
    match std::fs::read(directory()?.join("preferences.json")) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(error) => Err(error.into()),
    }
}

pub fn save_preferences(preferences: &Preferences) -> Result<()> {
    let directory = directory()?;
    std::fs::create_dir_all(&directory)?;
    std::fs::write(
        directory.join("preferences.json"),
        serde_json::to_vec(preferences)?,
    )?;
    Ok(())
}

pub fn prepare_temp_directory(path: &str) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        bail!(crate::i18n::tr("settings-temp-absolute"));
    }
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn directory() -> Result<PathBuf> {
    Ok(dirs::config_local_dir()
        .context(crate::i18n::tr("theme-storage-unavailable"))?
        .join("7zplus-rust"))
}
