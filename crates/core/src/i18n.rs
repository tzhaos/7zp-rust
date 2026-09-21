use anyhow::{Context, Result, bail};
use fluent_bundle::{FluentArgs, FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_syntax::ast::{Entry, PatternElement};
use std::{
    collections::HashMap,
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use unic_langid::LanguageIdentifier;

struct Localizer {
    bundle: FluentBundle<FluentResource>,
    labels: HashMap<String, String>,
}

static LOCALIZERS: OnceLock<Vec<Localizer>> = OnceLock::new();
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

pub fn save(language: Language) -> Result<()> {
    let root = crate::settings::directory()?;
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("language"), language.code())?;
    Ok(())
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
    let saved = if language.is_none() {
        match std::fs::read_to_string(crate::settings::directory()?.join("language")) {
            Ok(value) => Some(value),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        }
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

fn load(language: &str) -> Result<Localizer> {
    let source = match language {
        "zh-CN" => include_str!("../../../locales/zh-CN.ftl"),
        "zh-TW" => include_str!("../../../locales/zh-TW.ftl"),
        "en-US" => include_str!("../../../locales/en-US.ftl"),
        other => bail!("Unsupported language: {other}"),
    };
    let resource = FluentResource::try_new(source.to_owned())
        .map_err(|(_, errors)| anyhow::anyhow!("Invalid Fluent resource: {errors:?}"))?;
    let mut labels = HashMap::new();
    for entry in resource.entries() {
        if let Entry::Message(message) = entry
            && let Some(pattern) = &message.value
        {
            let text: Option<String> = pattern
                .elements
                .iter()
                .map(|element| match element {
                    PatternElement::TextElement { value } => Some(*value),
                    _ => None,
                })
                .collect();
            if let Some(text) = text {
                labels.insert(message.id.name.to_owned(), text);
            }
        }
    }
    let locale: LanguageIdentifier = language.parse().context("Invalid language identifier")?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errors| anyhow::anyhow!("Duplicate Fluent keys: {errors:?}"))?;
    Ok(Localizer { bundle, labels })
}

pub fn tr(key: &str) -> &'static str {
    &LOCALIZERS
        .get()
        .expect("localization initialized before UI")[LANGUAGE.load(Ordering::Relaxed)]
    .labels[key]
}

pub fn tf(key: &str, values: &[(&str, FluentValue<'_>)]) -> String {
    let localizer = &LOCALIZERS
        .get()
        .expect("localization initialized before UI")[LANGUAGE.load(Ordering::Relaxed)];
    let pattern = localizer
        .bundle
        .get_message(key)
        .and_then(|message| message.value())
        .expect("registered message key");
    let mut args = FluentArgs::new();
    for (key, value) in values {
        args.set(*key, value.clone());
    }
    let mut errors = Vec::new();
    localizer
        .bundle
        .format_pattern(pattern, Some(&args), &mut errors)
        .into_owned()
}
