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
    pub close_archive_after_quick: bool,
    pub low_priority: bool,
    pub check_updates: bool,
    pub shell_menu: bool,
    pub hide_tool_labels: bool,
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
            close_archive_after_quick: false,
            low_priority: true,
            check_updates: false,
            shell_menu: true,
            hide_tool_labels: false,
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

pub fn read_theme() -> Result<Option<Vec<u8>>> {
    match std::fs::read(directory()?.join("theme.json")) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context(crate::i18n::tr("theme-settings-invalid")),
    }
}

pub fn write_theme(bytes: &[u8]) -> Result<()> {
    let path = directory()?.join("theme.json");
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn load_destination() -> Option<String> {
    let path = directory().ok()?.join("destination");
    std::fs::read_to_string(path).ok()
}

pub fn save_destination(path: &std::path::Path) -> Result<()> {
    let root = directory()?;
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("destination"), path.to_string_lossy().as_bytes())?;
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub font_family: String,
    pub font_size: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            font_family: "Microsoft YaHei UI".into(),
            font_size: 12.,
        }
    }
}

impl Appearance {
    pub const SIZES: [f32; 6] = [12., 13., 14., 16., 18., 20.];
}

/// Splits a font stack on English commas. Empty pieces are dropped.
pub fn font_names(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn join_fonts(names: &[String]) -> String {
    names.join(", ")
}

/// Trims each name, drops empties, and keeps the first spelling of a repeated name.
pub fn normalize_font_stack(value: &str) -> String {
    let mut names = Vec::new();
    for name in font_names(value) {
        if names
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&name))
        {
            continue;
        }
        names.push(name);
    }
    join_fonts(&names)
}

/// Adds `font` to the stack, or removes it when it is already present.
pub fn toggle_font(value: &str, font: &str) -> String {
    let mut names = font_names(value);
    if let Some(index) = names
        .iter()
        .position(|name| name.eq_ignore_ascii_case(font))
    {
        names.remove(index);
    } else {
        names.push(font.to_owned());
    }
    join_fonts(&names)
}

pub fn load_appearance() -> Appearance {
    let Ok(root) = directory() else {
        return Appearance::default();
    };
    match std::fs::read(root.join("appearance.json")) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Appearance::default(),
    }
}

pub fn save_appearance(appearance: &Appearance) -> Result<()> {
    let root = directory()?;
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("appearance.json"), serde_json::to_vec(appearance)?)?;
    Ok(())
}

pub fn directory() -> Result<PathBuf> {
    Ok(dirs::config_local_dir()
        .context(crate::i18n::tr("theme-storage-unavailable"))?
        .join("7zplus-rust"))
}
