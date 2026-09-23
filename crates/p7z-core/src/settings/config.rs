use super::{Appearance, Preferences, ThemeId, state::StateStore};
use crate::i18n::{tf, tr};
use anyhow::{Context, Result, ensure};
use cardo_runtime::config::Snapshot;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

static CONFIG: OnceLock<Mutex<Snapshot<Configuration>>> = OnceLock::new();
static STATE: OnceLock<StateStore> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Configuration {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default)]
    pub theme: ThemeId,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default)]
    pub preferences: Preferences,
}

impl Configuration {
    fn defaults() -> Self {
        Self {
            schema_version: 1,
            language: None,
            theme: ThemeId::default(),
            appearance: Appearance::default(),
            preferences: Preferences::default(),
        }
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "{}",
            tf(
                "settings-schema-unsupported",
                &[("version", self.schema_version.to_string().into())]
            )
        );
        if let Some(language) = &self.language {
            ensure!(
                crate::i18n::LANGUAGES
                    .iter()
                    .any(|item| item.code() == language),
                "{}",
                tf(
                    "language-unsupported",
                    &[("language", language.as_str().into())]
                )
            );
        }
        ensure!(
            self.appearance.font_size.is_finite()
                && (Appearance::MIN_FONT_SIZE..=Appearance::MAX_FONT_SIZE)
                    .contains(&self.appearance.font_size),
            "{}",
            tr("appearance-font-size-invalid")
        );
        ensure!(
            !self.appearance.font_family.trim().is_empty(),
            "{}",
            tr("appearance-font-required")
        );
        ensure!(
            self.preferences.temp_directory.is_empty()
                || std::path::Path::new(&self.preferences.temp_directory).is_absolute(),
            "{}",
            tr("settings-temp-absolute")
        );
        super::shortcuts::validate(&self.preferences.shortcuts)
    }
}

/// Called after localization catalogs are initialized, before any application or helper work.
pub fn initialize() -> Result<()> {
    let directory = super::directory()?;
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("Cannot create {}", directory.display()))?;
    let _lock = cardo_runtime::storage::lock_file(&directory.join("initialize.lock"))?;
    let mut snapshot = Snapshot::load(directory.join("settings.toml"), || {
        Ok(Configuration::defaults())
    })?;
    snapshot
        .value()
        .validate()
        .with_context(|| format!("Invalid configuration {}", snapshot.path().display()))?;
    let state = StateStore::open()?;
    if !snapshot.exists() {
        snapshot.save(snapshot.value().clone())?;
    }
    CONFIG
        .set(Mutex::new(snapshot))
        .map_err(|_| anyhow::anyhow!("Configuration already initialized"))?;
    STATE
        .set(state)
        .map_err(|_| anyhow::anyhow!("State already initialized"))?;
    Ok(())
}

pub(super) fn current() -> Result<Configuration> {
    let config = CONFIG
        .get()
        .context("Configuration not initialized")?
        .lock()
        .map_err(|_| anyhow::anyhow!("Configuration lock poisoned"))?;
    Ok(config.value().clone())
}

pub(super) fn state() -> Result<&'static StateStore> {
    STATE.get().context("State not initialized")
}

pub fn load_language() -> Result<Option<String>> {
    Ok(current()?.language)
}

pub fn save_all(
    preferences: &Preferences,
    appearance: &Appearance,
    theme: ThemeId,
    language: crate::i18n::Language,
) -> Result<()> {
    let mut config = CONFIG
        .get()
        .context("Configuration not initialized")?
        .lock()
        .map_err(|_| anyhow::anyhow!("Configuration lock poisoned"))?;
    let next = Configuration {
        schema_version: 1,
        preferences: preferences.clone(),
        appearance: appearance.clone(),
        theme,
        language: Some(language.code().into()),
    };
    next.validate()
        .with_context(|| format!("Invalid configuration {}", config.path().display()))?;
    config.save(next)
}
