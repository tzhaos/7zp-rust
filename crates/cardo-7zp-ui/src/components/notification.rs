use super::icon;
use crate::ScrollableElement;
use gpui_kit::{
    component::{h_flex, v_flex},
    *,
};

pub(crate) fn notification(
    id: &'static str,
    body: impl IntoElement,
    close: impl IntoElement,
    cx: &App,
) -> Div {
    let p = crate::theme::palette(cx);
    h_flex()
        .w(px(460.))
        .max_w_full()
        .p(px(12.))
        .gap(px(12.))
        .items_start()
        .bg(rgb(p.surface))
        .border_1()
        .border_color(rgb(p.border))
        .rounded(px(14.))
        .shadow_md()
        .child(
            div()
                .flex_shrink_0()
                .size(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(14.))
                .bg(rgb(p.selected))
                .text_color(rgb(p.accent))
                .child(icon("Info", 18.)),
        )
        .child(
            v_flex().flex_1().min_w_0().gap(px(8.)).child(
                div()
                    .id(id)
                    .max_h(px(96.))
                    .w_full()
                    .min_w_0()
                    .whitespace_normal()
                    .text_size(px(12.))
                    .line_height(px(18.))
                    .overflow_y_scrollbar()
                    .child(body),
            ),
        )
        .child(close)
}
