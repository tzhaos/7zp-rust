use gpui_kit::{Font, FontFallbacks, SharedString, font};

pub enum FontError {
    Empty,
    Missing(Vec<String>),
}

#[derive(Clone)]
pub struct FontStack {
    names: Vec<String>,
}

impl FontStack {
    pub fn resolve(requested: &str, installed: &[String]) -> Result<Self, FontError> {
        let mut names = Vec::new();
        let mut missing = Vec::new();
        for name in requested
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match installed
                .iter()
                .find(|item| item.eq_ignore_ascii_case(name))
            {
                Some(canonical) => {
                    if !names.contains(canonical) {
                        names.push(canonical.clone());
                    }
                }
                None => {
                    if !missing
                        .iter()
                        .any(|item: &String| item.eq_ignore_ascii_case(name))
                    {
                        missing.push(name.to_owned());
                    }
                }
            }
        }
        if !missing.is_empty() {
            return Err(FontError::Missing(missing));
        }
        if names.is_empty() {
            return Err(FontError::Empty);
        }
        Ok(Self { names })
    }

    pub fn setting(&self) -> String {
        self.names.join(", ")
    }

    pub fn family(&self) -> SharedString {
        self.names[0].clone().into()
    }

    pub fn font(&self) -> Font {
        let mut face = font(self.family());
        if self.names.len() > 1 {
            face.fallbacks = Some(FontFallbacks::from_fonts(self.names[1..].to_vec()));
        }
        face
    }
}
