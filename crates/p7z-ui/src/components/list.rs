use cardo_ui::ConditionalBuilder;

use gpui_kit::*;

pub fn list_row(
    id: impl Into<ElementId>,
    selected: bool,
    disabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let p = crate::theme::palette(cx);
    div()
        .id(id)
        .relative()
        .flex()
        .h(px(crate::theme::ui_font_size(cx).as_f32() + 26.))
        .flex_shrink_0()
        .w_full()
        .items_center()
        .text_size(crate::theme::ui_font_size(cx))
        .rounded(px(4.))
        .when(selected, |el| el.bg(rgb(p.selected)))
        .when(!disabled, |el| {
            el.hover(move |el| el.bg(rgb(if selected { p.selected_hover } else { p.hover })))
        })
        .when(selected, |el| {
            el.child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(10.))
                    .bottom(px(10.))
                    .w(px(3.))
                    .rounded(px(2.))
                    .bg(rgb(p.accent)),
            )
        })
}

pub fn list_entry(
    name: &str,
    file_icon: impl IntoElement,
    detail: Option<String>,
    cx: &App,
) -> Div {
    let p = crate::theme::palette(cx);
    gpui_kit::component::h_flex()
        .flex_1()
        .min_w_0()
        .gap(px(9.))
        .pr(px(12.))
        .items_center()
        .text_size(crate::theme::ui_font_size(cx))
        .line_height(px(crate::theme::ui_font_size(cx).as_f32() + 4.))
        .text_left()
        .child(file_icon)
        .child(
            gpui_kit::component::v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    super::compact_text("entry-name", name.to_owned())
                        .w_full()
                        .text_ellipsis_middle(),
                )
                .when_some(detail, |el, detail| {
                    el.child(
                        super::compact_text("entry-detail", detail)
                            .w_full()
                            .text_size(px((crate::theme::ui_font_size(cx).as_f32() - 1.).max(11.)))
                            .text_color(rgb(p.muted)),
                    )
                }),
        )
}
