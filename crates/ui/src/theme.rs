pub(super) mod metrics;

use anyhow::{Context, Result};
use gpui_kit::{
    component::{Theme, ThemeMode},
    *,
};
use serde::{Deserialize, Serialize};
use sevenzip_core::i18n::tr;

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    #[default]
    FluentLight,
    GithubLight,
    OneDark,
    Dracula,
}

pub const THEMES: [ThemeId; 2] = [ThemeId::FluentLight, ThemeId::OneDark];

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

struct ThemeState(ThemeId);
impl Global for ThemeState {}

impl ThemeId {
    pub fn label(self) -> &'static str {
        tr(match self {
            Self::FluentLight => "theme-fluent-light",
            Self::GithubLight => "theme-github-light",
            Self::OneDark => "theme-one-dark",
            Self::Dracula => "theme-dracula",
        })
    }

    fn mode(self) -> ThemeMode {
        match self {
            Self::FluentLight | Self::GithubLight => ThemeMode::Light,
            Self::OneDark | Self::Dracula => ThemeMode::Dark,
        }
    }

    fn palette(self) -> Palette {
        match self {
            Self::FluentLight => Palette {
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
    cx.global::<ThemeState>().0
}
pub fn palette(cx: &App) -> Palette {
    current(cx).palette()
}

fn settings_path() -> Result<std::path::PathBuf> {
    Ok(sevenzip_core::settings::directory()?.join("theme.json"))
}

pub fn load() -> Result<ThemeId> {
    match std::fs::read(settings_path()?) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes).context(tr("theme-settings-invalid"))?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ThemeId::default()),
        Err(error) => Err(error).context(tr("theme-settings-invalid")),
    }
}

pub fn save(id: ThemeId) -> Result<()> {
    let path = settings_path()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, serde_json::to_vec(&id)?)?;
    Ok(())
}

pub fn apply(id: ThemeId, window: Option<&mut Window>, cx: &mut App) {
    Theme::change(id.mode(), window, cx);
    let p = id.palette();
    let theme = Theme::global_mut(cx);
    theme.font_family = "Microsoft YaHei UI".into();
    // GPUI Kit's text_sm uses 0.875rem; a 16px base yields a 14px menu label.
    theme.font_size = px(16.);
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
    // Component backgrounds read resolved tokens separately from the color palette.
    theme.tokens = theme.colors.into();
    Theme::sync_base(cx);
    cx.set_global(ThemeState(id));
    cx.refresh_windows();
}
