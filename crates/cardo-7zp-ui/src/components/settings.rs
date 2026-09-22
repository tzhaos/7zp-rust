use crate::theme::metrics::settings as metrics;
use gpui_kit::component::scroll::Scrollable;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub fn settings_page(cx: &App) -> Div {
    v_flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .text_size(crate::theme::ui_font_size(cx))
        .line_height(relative(1.5))
}

pub fn settings_content(id: &'static str) -> Scrollable<Stateful<Div>> {
    super::panel_body(id)
        .p(px(metrics::CONTENT_PADDING))
        .gap(px(metrics::SECTION_GAP))
}

pub fn settings_row(label: &str, control: impl IntoElement, cx: &App) -> Div {
    h_flex()
        .w_full()
        .min_h(px(metrics::ROW_HEIGHT))
        .py(px(metrics::ROW_PADDING))
        .gap(px(20.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(crate::theme::ui_font_size(cx))
                .line_height(relative(1.5))
                .whitespace_normal()
                .font_weight(FontWeight::NORMAL)
                .child(label.to_owned()),
        )
        .child(div().flex_shrink_0().child(control))
}

pub fn settings_group(rows: impl IntoIterator<Item = AnyElement>, cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    settings_frame(cx)
        .px(px(metrics::GROUP_PADDING))
        .children(rows.into_iter().enumerate().map(|(index, row)| {
            div()
                .when(index > 0, |el| el.border_t_1().border_color(rgb(p.border)))
                .child(row)
        }))
}

pub fn settings_frame(cx: &App) -> Div {
    v_flex()
        .w_full()
        .flex_shrink_0()
        .border_1()
        .border_color(rgb(crate::theme::palette(cx).border))
        .rounded(px(metrics::GROUP_RADIUS))
}

pub fn settings_section(title: &str, group: Div, cx: &App) -> Div {
    v_flex()
        .flex_shrink_0()
        .gap(px(metrics::HEADING_GAP))
        .child(
            div()
                .text_size(px(metrics::SECTION_TITLE_SIZE).max(crate::theme::ui_font_size(cx)))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
        .child(group)
}
