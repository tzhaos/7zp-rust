use anyhow::{Context, Result, bail};
pub mod recent;
pub mod shortcuts;
mod config;
pub mod state;
pub use config::{initialize, load_language, save_all};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

pub static LOW_PRIORITY: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    #[serde(with = "shortcuts::config_map")]
    pub shortcuts: shortcuts::Shortcuts,
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
            shortcuts: shortcuts::Shortcuts::default(),
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
    Ok(config::current()?.preferences)
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    System,
    #[default]
    Light,
    OneDark,
}

pub fn load_theme() -> Result<ThemeId> {
    Ok(config::current()?.theme)
}

pub fn load_destination() -> Result<Option<String>> {
    store()?.read_json("destination")
}

pub fn save_destination(path: &std::path::Path) -> Result<()> {
    store()?.write_json("destination", &path.to_string_lossy().as_ref())
}

pub fn prepare_temp_directory(path: &str) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        bail!(crate::i18n::tr("settings-temp-absolute"));
    }
    std::fs::create_dir_all(&path)
        .with_context(|| format!("Cannot create temporary directory {}", path.display()))?;
    Ok(())
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
    Ok(config::current()?.appearance)
}

pub fn directory() -> Result<PathBuf> {
    Ok(cardo_runtime::storage::Store::for_app("p7z")?.directory().to_owned())
}

pub fn store() -> Result<&'static state::StateStore> {
    config::state()
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
