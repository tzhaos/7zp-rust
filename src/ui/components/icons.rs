use gpui_kit::component::Icon;
use gpui_kit::*;

pub fn icon(name: &str, size: f32) -> Icon {
    Icon::default()
        .path(format!("fluent/{name}Regular.svg"))
        .size(px(size))
        .min_w(px(size))
        .min_h(px(size))
}

pub fn filled_icon(name: &str, size: f32) -> Icon {
    Icon::default()
        .path(format!("fluent/{name}Filled.svg"))
        .size(px(size))
}

pub enum ToolIcon {
    Files,
    Open,
    Extract,
    QuickExtract,
    Check,
    General,
    Advanced,
    Appearance,
    Associations,
    History,
}

impl ToolIcon {
    fn asset(self) -> &'static str {
        match self {
            Self::Files => "cursor",
            Self::Open => "open",
            Self::Extract => "extract",
            Self::QuickExtract => "quick-extract",
            Self::Check => "verify",
            Self::General => "general",
            Self::Advanced => "bipyramid",
            Self::Appearance => "palette",
            Self::Associations => "chain",
            Self::History => "history",
        }
    }
}

pub fn artwork(name: ToolIcon) -> impl IntoElement {
    img(SharedString::from(format!("toolbar/{}.svg", name.asset())))
        .size(px(crate::ui::theme::metrics::ARTWORK_SIZE))
        .flex_shrink_0()
}
