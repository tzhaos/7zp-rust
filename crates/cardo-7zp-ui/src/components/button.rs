use super::{ToolIcon, artwork, bubble_tooltip, icon};
use crate::theme::metrics::*;
use gpui_kit::component::{
    Disableable, Sizable,
    button::{Button, ButtonCustomVariant, ButtonVariants},
    input::{Input, InputState},
};
use gpui_kit::*;

pub fn subtle_variant(cx: &App) -> ButtonCustomVariant {
    let p = crate::theme::palette(cx);
    ButtonCustomVariant::new(cx)
        .hover(rgb(p.hover).into())
        .active(rgb(p.selected).into())
}

pub fn tool(
    id: impl Into<ElementId>,
    name: ToolIcon,
    label: &str,
    disabled: bool,
    cx: &App,
) -> Button {
    let p = crate::theme::palette(cx);
    bubble_tooltip(Button::new(id), label.to_owned())
        .custom(subtle_variant(cx))
        .disabled(disabled)
        .accessibility_label(label.to_owned())
        .w(px(TOOL_MAX_WIDTH))
        .h(px(TOOL_HEIGHT))
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
                        .child(artwork(name)),
                )
                .child(
                    gpui_kit::component::v_flex()
                        .w_full()
                        .h(px(26.))
                        .flex_shrink_0()
                        .justify_center()
                        .child(
                            div()
                                .w_full()
                                .text_size(px(11.))
                                .line_height(px(13.))
                                .text_center()
                                .whitespace_normal()
                                .text_color(rgb(if disabled { p.muted } else { p.text }))
                                .child(label.to_owned()),
                        ),
                ),
        )
}

pub fn command(id: impl Into<ElementId>, label: &str) -> Button {
    // Button sizes control the inner label; an outer text_size is overridden.
    Button::new(id)
        .xsmall()
        .label(label.to_owned())
        .h(px(CONTROL_HEIGHT))
        .px(px(12.))
        .rounded(px(CONTROL_RADIUS))
        .border_1()
        .shadow_none()
}

pub fn text_input(state: &Entity<InputState>) -> Input {
    Styled::h(Input::new(state).xsmall(), px(CONTROL_HEIGHT))
}

pub fn primary(id: impl Into<ElementId>, label: &str) -> Button {
    command(id, label).primary()
}

pub fn icon_button(id: impl Into<ElementId>, name: &str, title: &str, cx: &App) -> Button {
    bubble_tooltip(Button::new(id), title.to_owned())
        .custom(subtle_variant(cx))
        .compact()
        .icon(icon(name, 16.))
        .accessibility_label(title.to_owned())
        .w(px(ICON_BUTTON_SIZE))
        .h(px(ICON_BUTTON_SIZE))
        .p_0()
        .flex_shrink_0()
        .rounded(px(CONTROL_RADIUS))
        .border_0()
        .shadow_none()
}
