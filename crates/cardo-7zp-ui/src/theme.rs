pub(super) mod metrics;

use anyhow::{Context, Result};
use cardo_7zp_core::i18n::{tf, tr};
use cardo_ui::fonts::{FontError, FontStack};
pub use cardo_ui::theme::Palette;
use cardo_ui::theme::ThemeStyle;
use gpui_kit::{component::ThemeMode, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    #[default]
    Light,
    OneDark,
}

pub const THEMES: [ThemeId; 2] = [ThemeId::Light, ThemeId::OneDark];

struct ThemeState {
    id: ThemeId,
    appearance: cardo_7zp_core::settings::Appearance,
    font: FontStack,
}
impl Global for ThemeState {}

impl ThemeId {
    pub fn label(self) -> &'static str {
        tr(match self {
            Self::Light => "theme-light",
            Self::OneDark => "theme-one-dark",
        })
    }

    fn mode(self) -> ThemeMode {
        match self {
            Self::Light => ThemeMode::Light,
            Self::OneDark => ThemeMode::Dark,
        }
    }

    fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                surface: 0xffffff,
                panel: 0xf0f1f3,
                title: 0xf5f6f8,
                text: 0x202123,
                muted: 0x777777,
                border: 0xe9e9e9,
                hover: 0xf3f3f3,
                selected: 0xe4edf9,
                selected_hover: 0xd7e5f7,
                accent: 0x3399ff,
                accent_hover: 0x2585e6,
                on_accent: 0xffffff,
                success: 0x267052,
                danger: 0xb42318,
            },
            Self::OneDark => Palette {
                surface: 0x1f1f1f,
                panel: 0x141414,
                title: 0x141414,
                text: 0xe8eaeb,
                muted: 0xa6abad,
                border: 0x393c3e,
                hover: 0x2a2a2a,
                selected: 0x2c2e30,
                selected_hover: 0x393c3e,
                accent: 0x79aff0,
                accent_hover: 0x95bff3,
                on_accent: 0x141414,
                success: 0x7dc9a1,
                danger: 0xf38d91,
            },
        }
    }
}

pub fn current(cx: &App) -> ThemeId {
    cx.global::<ThemeState>().id
}

pub fn appearance(cx: &App) -> cardo_7zp_core::settings::Appearance {
    cx.global::<ThemeState>().appearance.clone()
}

pub fn ui_font_size(cx: &App) -> Pixels {
    px(appearance(cx).font_size)
}

pub fn palette(cx: &App) -> Palette {
    cx.global::<ThemeState>().id.palette()
}

pub fn interface_font(cx: &App) -> Font {
    cx.global::<ThemeState>().font.font()
}

pub(crate) fn resolve_font(requested: &str, cx: &App) -> Result<FontStack> {
    FontStack::resolve(requested, &cx.text_system().all_font_names()).map_err(|error| {
        anyhow::anyhow!(match error {
            FontError::Empty => tr("appearance-font-required").to_owned(),
            FontError::Missing(names) => tf(
                "appearance-font-missing",
                &[("names", names.join(", ").into())]
            ),
        })
    })
}

pub fn load() -> Result<ThemeId> {
    match cardo_7zp_core::settings::read_theme()? {
        Some(bytes) => Ok(serde_json::from_slice(&bytes).context(tr("theme-settings-invalid"))?),
        None => Ok(ThemeId::default()),
    }
}

pub fn save(id: ThemeId) -> Result<()> {
    cardo_7zp_core::settings::write_theme(&serde_json::to_vec(&id)?)
}

pub fn apply(
    id: ThemeId,
    appearance: &cardo_7zp_core::settings::Appearance,
    window: Option<&mut Window>,
    cx: &mut App,
) -> Result<()> {
    let font = resolve_font(&appearance.font_family, cx)?;
    let primary = font.family();
    cx.set_global(ThemeState {
        id,
        appearance: appearance.clone(),
        font,
    });
    ThemeStyle {
        mode: id.mode(),
        palette: id.palette(),
        font_family: primary,
        // List text at 12px corresponds to the component library's 16px rem base.
        font_size: px(appearance.font_size * (16. / 12.)),
        radius: px(16.),
        radius_lg: px(8.),
    }
    .apply(window, cx);
    Ok(())
}
