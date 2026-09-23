use anyhow::{Context, Result, bail};
use cardo_runtime::localization::{Catalog, MessageValue};
use std::sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
};
static LOCALIZERS: OnceLock<Vec<Catalog>> = OnceLock::new();
static LANGUAGE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    SimplifiedChinese,
    TraditionalChinese,
    English,
}

pub const LANGUAGES: [Language; 3] = [
    Language::SimplifiedChinese,
    Language::TraditionalChinese,
    Language::English,
];

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::SimplifiedChinese => "zh-CN",
            Self::TraditionalChinese => "zh-TW",
            Self::English => "en-US",
        }
    }
    pub fn label(self) -> &'static str {
        tr(match self {
            Self::SimplifiedChinese => "language-zh-cn",
            Self::TraditionalChinese => "language-zh-tw",
            Self::English => "language-en-us",
        })
    }
}

pub fn current() -> Language {
    LANGUAGES[LANGUAGE.load(Ordering::Relaxed)]
}

pub fn select(language: Language) {
    LANGUAGE.store(language as usize, Ordering::Relaxed);
}

pub fn init(language: Option<&str>) -> Result<()> {
    let detected = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    let system_language = match detected {
        0x0404 | 0x0c04 | 0x1404 => Language::TraditionalChinese,
        0x0804 | 0x1004 => Language::SimplifiedChinese,
        _ => Language::English,
    };
    let localizers = LANGUAGES
        .iter()
        .map(|language| load(language.code()))
        .collect::<Result<Vec<_>>>()?;
    LOCALIZERS
        .set(localizers)
        .map_err(|_| anyhow::anyhow!("Localization already initialized"))?;
    select(system_language);
    crate::settings::initialize()?;
    let saved = if language.is_none() {
        crate::settings::load_language()?
    } else {
        None
    };
    let requested = language
        .or(saved.as_deref())
        .unwrap_or(system_language.code());
    let selected = LANGUAGES
        .iter()
        .find(|language| language.code() == requested)
        .context(tf(
            "language-unsupported",
            &[("language", requested.into())],
        ))?;
    select(*selected);
    Ok(())
}

fn load(language: &str) -> Result<Catalog> {
    let source = match language {
        "zh-CN" => include_str!("../../../locales/zh-CN.ftl"),
        "zh-TW" => include_str!("../../../locales/zh-TW.ftl"),
        "en-US" => include_str!("../../../locales/en-US.ftl"),
        other => bail!("Unsupported language: {other}"),
    };
    Catalog::new(language, source)
}

pub fn tr(key: &str) -> &'static str {
    LOCALIZERS
        .get()
        .expect("localization initialized before UI")[LANGUAGE.load(Ordering::Relaxed)]
    .text(key)
    .unwrap_or_else(|error| panic!("{error:#}"))
}

pub fn tf(key: &str, values: &[(&str, MessageValue<'_>)]) -> String {
    let localizer = &LOCALIZERS
        .get()
        .expect("localization initialized before UI")[LANGUAGE.load(Ordering::Relaxed)];
    localizer
        .format(key, values)
        .unwrap_or_else(|error| panic!("{error:#}"))
}
