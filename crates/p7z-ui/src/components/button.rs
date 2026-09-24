use cardo_ui::ConditionalBuilder;
use super::{ToolIcon, artwork, compact_text, bubble_tooltip, icon};
use crate::theme::metrics::*;
use gpui_kit::{component::{Disableable, button::{Button, ButtonVariants}}, *};
pub use cardo_ui::controls::{subtle_variant, header_variant, command, checkbox, text_input, primary};
pub fn tool(
    id: impl Into<ElementId>,
    name: ToolIcon,
    label: &str,
    disabled: bool,
    show_label: bool,
    window: &mut Window,
    cx: &mut App,
) -> Button {
    let p = crate::theme::palette(cx);
    let height = if show_label {
        TOOL_HEIGHT
    } else {
        TOOL_ICON_HEIGHT
    };
    let button = Button::new(id)
        .custom(subtle_variant(cx))
        .disabled(disabled)
        .accessibility_label(label.to_owned())
        .w(px(TOOL_MAX_WIDTH))
        .h(px(height))
        .flex_1()
        .min_w(px(TOOL_MIN_WIDTH))
        .max_w(px(TOOL_MAX_WIDTH))
        .px(px(2.))
        .py(px(2.))
        .rounded(px(6.))
        .shadow_none()
        .child(
            gpui_kit::component::v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap(px(2.))
                .child(
                    div()
                        .size(px(ARTWORK_SIZE))
                        .flex_shrink_0()
                        .opacity(if disabled { 0.45 } else { 1.0 })
                        .child(artwork(name, window, cx)),
                )
                .when(show_label, |el| {
                    el.child(
                        gpui_kit::component::v_flex()
                            .w_full()
                            .h(px(26.))
                            .flex_shrink_0()
                            .justify_center()
                            .child(
                                compact_text("tool-label", label.to_owned())
                                    .w_full()
                                    .text_size(px(11.))
                                    .line_height(px(13.))
                                    .text_center()
                                    .whitespace_normal()
                                    .line_clamp(2)
                                    .text_ellipsis()
                                    .text_color(rgb(if disabled { p.muted } else { p.text })),
                            ),
                    )
                }),
        );
    if show_label {
        button
    } else {
        bubble_tooltip(button, label.to_owned())
    }
}

pub fn icon_button(id: impl Into<ElementId>, name: &str, title: &str, hint: bool, cx: &App) -> Button {
    cardo_ui::controls::icon_button(id, icon(name, 16.), title, hint, cx)
}
