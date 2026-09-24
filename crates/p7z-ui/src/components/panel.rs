use gpui_kit::*;
use super::{ToolIcon, artwork, icon};
pub(crate) use cardo_ui::panel::{TabIndicator, connected_panel_outline, body_container, panel_actions, panel_body, panel_button, panel_card, panel_danger, panel_document, panel_field, panel_frame, panel_header, panel_layout, panel_primary, panel_property, panel_surface, popup_surface};
#[derive(Clone, Copy)]
pub(crate) enum PanelSize {
    Compact,
    Standard,
    Wide,
    Form,
    Comparison,
    Extract,
    Properties(usize),
}

impl PanelSize {
    pub(crate) fn size(self) -> Size<Pixels> {
        let (width, height) = match self {
            Self::Compact => (600., 280.),
            Self::Standard => (700., 420.),
            Self::Wide => (820., 600.),
            Self::Form => (780., 760.),
            Self::Comparison => (840., 560.),
            Self::Extract => (740., 640.),
            Self::Properties(rows) => (660., (144. + rows as f32 * 36.).clamp(240., 560.)),
        };
        size(px(width), px(height))
    }

    pub(crate) fn minimum(self) -> Size<Pixels> {
        size(px(520.), px(240.))
    }
}

pub(crate) fn panel_notice(name: &str, text: impl Into<SharedString>, color: u32) -> Div {
    cardo_ui::panel::panel_notice(icon(name, 20.), text, color)
}
pub(crate) fn panel_artwork_notice(name: ToolIcon, text: impl Into<SharedString>, window: &mut Window, cx: &mut App) -> Div {
    cardo_ui::panel::panel_artwork_notice(artwork(name, window, cx), text)
}
