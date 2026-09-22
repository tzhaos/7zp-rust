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
    Ok(store()?.read_json("preferences.json")?.unwrap_or_default())
}

pub fn save_preferences(preferences: &Preferences) -> Result<()> {
    store()?.write_json("preferences.json", preferences)
}

pub fn read_theme() -> Result<Option<Vec<u8>>> {
    store()?
        .read("theme.json")
        .context(crate::i18n::tr("theme-settings-invalid"))
}

pub fn write_theme(bytes: &[u8]) -> Result<()> {
    store()?.write("theme.json", bytes)
}

pub fn load_destination() -> Result<Option<String>> {
    store()?.read_text("destination")
}

pub fn save_destination(path: &std::path::Path) -> Result<()> {
    store()?.write("destination", path.to_string_lossy().as_bytes())
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
    pub const MIN_FONT_SIZE: f32 = 12.;
    pub const MAX_FONT_SIZE: f32 = 20.;
    pub const FONT_SIZE_STEP: f32 = 1.;

    pub fn clamp_font_size(value: f32) -> f32 {
        value
            .round()
            .clamp(Self::MIN_FONT_SIZE, Self::MAX_FONT_SIZE)
    }
}

pub fn load_appearance() -> Result<Appearance> {
    let mut appearance: Appearance = store()?.read_json("appearance.json")?.unwrap_or_default();
    appearance.font_size = Appearance::clamp_font_size(appearance.font_size);
    Ok(appearance)
}

pub fn save_appearance(appearance: &Appearance) -> Result<()> {
    store()?.write_json("appearance.json", appearance)
}

pub fn directory() -> Result<PathBuf> {
    Ok(store()?.directory().to_owned())
}

pub fn store() -> Result<cardo_runtime::storage::Store> {
    cardo_runtime::storage::Store::for_app("7zplus-rust")
}

pub struct StartupSettings {
    pub preferences: Preferences,
    pub appearance: Appearance,
    pub destination: Option<String>,
}

impl StartupSettings {
    pub fn load() -> Result<Self> {
        Ok(Self {
            preferences: load_preferences()?,
            appearance: load_appearance()?,
            destination: load_destination()?,
        })
    }
}
