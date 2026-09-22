use anyhow::{Context, Result, bail};
use fluent_bundle::{FluentArgs, FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_syntax::ast::{Entry, PatternElement};
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

pub struct Catalog {
    locale: String,
    bundle: FluentBundle<FluentResource>,
    labels: HashMap<String, String>,
}

impl Catalog {
    pub fn new(locale: &str, source: &str) -> Result<Self> {
        let resource = FluentResource::try_new(source.to_owned())
            .map_err(|(_, errors)| anyhow::anyhow!("Invalid {locale} messages: {errors:?}"))?;
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
        let language: LanguageIdentifier = locale.parse().context("Invalid language identifier")?;
        let mut bundle = FluentBundle::new_concurrent(vec![language]);
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .map_err(|errors| anyhow::anyhow!("Duplicate {locale} messages: {errors:?}"))?;
        Ok(Self {
            locale: locale.to_owned(),
            bundle,
            labels,
        })
    }

    pub fn text(&self, key: &str) -> Result<&str> {
        self.labels
            .get(key)
            .map(String::as_str)
            .with_context(|| format!("Missing plain message {}:{key}", self.locale))
    }

    pub fn format(&self, key: &str, values: &[(&str, FluentValue<'_>)]) -> Result<String> {
        let pattern = self
            .bundle
            .get_message(key)
            .and_then(|message| message.value())
            .with_context(|| format!("Missing message {}:{key}", self.locale))?;
        let mut args = FluentArgs::new();
        for (name, value) in values {
            args.set(*name, value.clone());
        }
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, Some(&args), &mut errors)
            .into_owned();
        if !errors.is_empty() {
            bail!("Cannot format {}:{key}: {errors:?}", self.locale);
        }
        Ok(text)
    }
}
