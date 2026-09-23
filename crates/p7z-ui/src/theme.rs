pub(super) mod metrics;

use anyhow::Result;
use p7z_core::i18n::{tf, tr};
pub use p7z_core::settings::ThemeId;
use cardo_ui::fonts::{FontError, FontStack};
pub use cardo_ui::theme::Palette;
use cardo_ui::theme::ThemeStyle;
use gpui_kit::{component::ThemeMode, *};

pub const THEMES: [ThemeId; 3] = [ThemeId::System, ThemeId::Light, ThemeId::OneDark];

struct ThemeState {
    id: ThemeId,
    resolved: ThemeMode,
    appearance: p7z_core::settings::Appearance,
    font: FontStack,
}
impl Global for ThemeState {}

pub fn label(id: ThemeId) -> &'static str {
    tr(match id {
        ThemeId::System => "theme-system",
        ThemeId::Light => "theme-light",
        ThemeId::OneDark => "theme-one-dark",
    })
}

fn mode(id: ThemeId, cx: &App) -> ThemeMode {
    match id {
        ThemeId::System => match cx.window_appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
            _ => ThemeMode::Light,
        },
        ThemeId::Light => ThemeMode::Light,
        ThemeId::OneDark => ThemeMode::Dark,
    }
}

fn palette_for(id: ThemeMode) -> Palette {
    match id {
        ThemeMode::Light => Palette {
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
        ThemeMode::Dark => Palette {
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
pub fn current(cx: &App) -> ThemeId {
    cx.global::<ThemeState>().id
}

pub fn appearance(cx: &App) -> p7z_core::settings::Appearance {
    cx.global::<ThemeState>().appearance.clone()
}

pub fn ui_font_size(cx: &App) -> Pixels {
    px(appearance(cx).font_size)
}

pub fn palette(cx: &App) -> Palette {
    palette_for(cx.global::<ThemeState>().resolved)
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
    p7z_core::settings::load_theme()
}

pub fn apply(
    id: ThemeId,
    appearance: &p7z_core::settings::Appearance,
    window: Option<&mut Window>,
    cx: &mut App,
) -> Result<()> {
    let font = resolve_font(&appearance.font_family, cx)?;
    let primary = font.family();
    let resolved = mode(id, cx);
    cx.set_global(ThemeState {
        id,
        resolved,
        appearance: appearance.clone(),
        font,
    });
    ThemeStyle {
        mode: resolved,
        palette: palette_for(resolved),
        font_family: primary,
        // List text at 12px corresponds to the component library's 16px rem base.
        font_size: px(appearance.font_size * (16. / 12.)),
        radius: px(16.),
        radius_lg: px(8.),
    }
    .apply(window, cx);
    Ok(())
}
