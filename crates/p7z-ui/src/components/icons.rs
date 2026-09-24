use gpui_kit::component::Icon;
use gpui_kit::*;


pub fn icon(name: &str, size: f32) -> Icon {
    Icon::default()
        .path(format!("fluent/{name}Regular.svg"))
        .size(px(size))
        .flex_shrink_0()
        .min_w(px(size))
        .min_h(px(size))
}

#[derive(Clone, Copy)]
pub enum ToolIcon {
    Browser,
    Open,
    Extract,
    QuickExtract,
    Check,
    General,
    Advanced,
    Associations,
    History,
    About,
    Warning,
    Error,
    Delete,
}

impl ToolIcon {
    fn asset(self) -> &'static str {
        match self {
            Self::Browser => "brand/logo.svg",
            Self::Open => "toolbar/open.svg",
            Self::Extract => "toolbar/extract.svg",
            Self::QuickExtract => "toolbar/quick-extract.svg",
            Self::Check => "toolbar/verify.svg",
            Self::General => "toolbar/general.svg",
            Self::Advanced => "toolbar/bipyramid.svg",
            Self::Associations => "toolbar/chain.svg",
            Self::History => "toolbar/history.svg",
            Self::About => "toolbar/about.svg",
            Self::Warning => "toolbar/warning.svg",
            Self::Error => "toolbar/error.svg",
            Self::Delete => "toolbar/delete.svg",
        }
    }
}

pub fn artwork(name: ToolIcon, window: &mut Window, cx: &mut App) -> impl IntoElement {
    cardo_ui::artwork::artwork(name.asset(), crate::theme::metrics::ARTWORK_SIZE, window, cx)
}
