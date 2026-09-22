pub(super) mod metrics;

use anyhow::{Context, Result};
use gpui_kit::{
    component::{Theme, ThemeMode},
    *,
};
use serde::{Deserialize, Serialize};
use cardo_7zp_core::i18n::tr;

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    #[default]
    #[serde(alias = "fluent-light")]
    Light,
    GithubLight,
    OneDark,
    Dracula,
}

pub const THEMES: [ThemeId; 2] = [ThemeId::Light, ThemeId::OneDark];

#[derive(Clone, Copy)]
pub struct Palette {
    pub surface: u32,
    pub panel: u32,
    pub title: u32,
    pub text: u32,
    pub muted: u32,
    pub border: u32,
    pub hover: u32,
    pub selected: u32,
    pub selected_hover: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub on_accent: u32,
    pub success: u32,
    pub danger: u32,
    pub scrim: u32,
}

struct ThemeState {
    id: ThemeId,
    appearance: cardo_7zp_core::settings::Appearance,
    font_fallbacks: Vec<String>,
}
impl Global for ThemeState {}

impl ThemeId {
    pub fn label(self) -> &'static str {
        tr(match self {
            Self::Light => "theme-light",
            Self::GithubLight => "theme-github-light",
            Self::OneDark => "theme-one-dark",
            Self::Dracula => "theme-dracula",
        })
    }

    fn mode(self) -> ThemeMode {
        match self {
            Self::Light | Self::GithubLight => ThemeMode::Light,
            Self::OneDark | Self::Dracula => ThemeMode::Dark,
        }
    }

    fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                surface: 0xffffff,
                panel: 0xf0f1f3,
                title: 0xf5f6f8,
                text: 0x18181b,
                muted: 0x71717a,
                border: 0xe1e4e9,
                hover: 0xe9edf2,
                selected: 0xe4edf9,
                selected_hover: 0xd7e5f7,
                accent: 0x3479cf,
                accent_hover: 0x2866b3,
                on_accent: 0xffffff,
                success: 0x267052,
                danger: 0xb42318,
                scrim: 0x18181b45,
            },
            Self::GithubLight => Palette {
                surface: 0xffffff,
                panel: 0xf6f8fa,
                title: 0xf0f3f6,
                text: 0x1f2328,
                muted: 0x59636e,
                border: 0xd1d9e0,
                hover: 0xeaeef2,
                selected: 0xddf4ff,
                selected_hover: 0xc8e9fc,
                accent: 0x0969da,
                accent_hover: 0x0550ae,
                on_accent: 0xffffff,
                success: 0x1a7f37,
                danger: 0xcf222e,
                scrim: 0x1f232840,
            },
            Self::OneDark => Palette {
                surface: 0x282c34,
                panel: 0x242830,
                title: 0x21252b,
                text: 0xabb2bf,
                muted: 0x939cab,
                border: 0x414754,
                hover: 0x323842,
                selected: 0x35465b,
                selected_hover: 0x405571,
                accent: 0x61afef,
                accent_hover: 0x82c1f4,
                on_accent: 0x18212b,
                success: 0x98c379,
                danger: 0xe06c75,
                scrim: 0x00000088,
            },
            Self::Dracula => Palette {
                surface: 0x282a36,
                panel: 0x242631,
                title: 0x21222c,
                text: 0xf8f8f2,
                muted: 0xb0b4c9,
                border: 0x484b61,
                hover: 0x343746,
                selected: 0x44475a,
                selected_hover: 0x51556c,
                accent: 0xbd93f9,
                accent_hover: 0xd0afff,
                on_accent: 0x282a36,
                success: 0x50fa7b,
                danger: 0xff7979,
                scrim: 0x00000088,
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

/// The face actually used for interface text.
///
/// The first installed name in the comma-separated stack is the family. The
/// remaining installed names are glyph fallbacks. A name the system does not
/// list is not passed to GPUI, because a missing family panics on layout.
pub fn interface_font(cx: &App) -> Font {
    let fallbacks = cx.global::<ThemeState>().font_fallbacks.clone();
    let mut face = font(Theme::global(cx).font_family.clone());
    if !fallbacks.is_empty() {
        face.fallbacks = Some(FontFallbacks::from_fonts(fallbacks));
    }
    face
}

fn font_choice(requested: &str, installed: &[String]) -> (SharedString, Vec<String>) {
    let mut chosen = Vec::new();
    for name in cardo_7zp_core::settings::font_names(requested) {
        let Some(canonical) = installed
            .iter()
            .find(|item| item.eq_ignore_ascii_case(&name))
        else {
            continue;
        };
        if chosen
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(canonical))
        {
            continue;
        }
        chosen.push(canonical.clone());
    }
    if chosen.is_empty() {
        let family = installed
            .iter()
            .find(|item| item.eq_ignore_ascii_case("Microsoft YaHei UI"))
            .cloned()
            .unwrap_or_else(|| ".SystemUIFont".to_string());
        return (family.into(), Vec::new());
    }
    let primary = chosen.remove(0);
    (primary.into(), chosen)
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
) {
    let installed = cx.text_system().all_font_names();
    let (primary, font_fallbacks) = font_choice(&appearance.font_family, &installed);
    cx.set_global(ThemeState {
        id,
        appearance: appearance.clone(),
        font_fallbacks,
    });
    Theme::change(id.mode(), window, cx);
    let p = palette(cx);
    let theme = Theme::global_mut(cx);
    theme.font_family = primary;
    // A 12px list keeps the previous 16px rem base, so menu labels stay 14px.
    theme.font_size = px(appearance.font_size * (16. / 12.));
    theme.radius = px(metrics::CONTROL_RADIUS);
    theme.radius_lg = px(8.);
    let c = &mut theme.colors;
    c.background = rgb(p.surface).into();
    c.foreground = rgb(p.text).into();
    c.border = rgb(p.border).into();
    c.input = c.border;
    c.accent = rgb(p.hover).into();
    c.accent_foreground = c.foreground;
    c.muted = rgb(p.panel).into();
    c.muted_foreground = rgb(p.muted).into();
    c.popover = c.background;
    c.popover_foreground = c.foreground;
    c.button = c.background;
    c.button_foreground = c.foreground;
    c.button_hover = rgb(p.hover).into();
    c.button_active = rgb(p.selected).into();
    c.primary = rgb(p.accent).into();
    c.primary_hover = rgb(p.accent_hover).into();
    c.primary_active = c.primary_hover;
    c.primary_foreground = rgb(p.on_accent).into();
    c.button_primary = c.primary;
    c.button_primary_hover = c.primary_hover;
    c.button_primary_active = c.primary_active;
    c.button_primary_foreground = c.primary_foreground;
    c.secondary = c.muted;
    c.secondary_foreground = c.foreground;
    c.secondary_hover = c.button_hover;
    c.secondary_active = c.button_active;
    c.ring = c.primary;
    c.caret = c.primary;
    c.list = c.background;
    c.list_head = c.muted;
    c.list_hover = c.button_hover;
    c.list_active = c.button_active;
    c.list_active_border = c.primary;
    c.selection = rgb(p.selected).into();
    c.scrollbar = rgba(0x00000000).into();
    c.scrollbar_thumb = rgba((p.muted << 8) | 0xb0).into();
    c.scrollbar_thumb_hover = rgba((p.text << 8) | 0xe6).into();
    theme.scrollbar_mode = gpui_kit::component::scroll::ScrollbarMode::Always;
    // Component backgrounds read resolved tokens separately from the color palette.
    theme.tokens = theme.colors.into();
    Theme::sync_base(cx);
    cx.refresh_windows();
}
