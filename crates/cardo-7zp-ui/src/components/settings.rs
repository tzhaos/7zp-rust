use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub fn settings_row(label: &str, control: impl IntoElement, cx: &App) -> Div {
    h_flex()
        .w_full()
        .min_h(px(52.))
        .py(px(10.))
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
    v_flex()
        .w_full()
        .flex_shrink_0()
        .children(rows.into_iter().enumerate().map(|(index, row)| {
            div()
                .when(index > 0, |el| el.border_t_1().border_color(rgb(p.border)))
                .child(row)
        }))
}

pub fn settings_section(title: &str, group: Div, cx: &App) -> Div {
    v_flex()
        .flex_shrink_0()
        .gap(px(8.))
        .child(
            div()
                .text_size(crate::theme::ui_font_size(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
        .child(group)
}
