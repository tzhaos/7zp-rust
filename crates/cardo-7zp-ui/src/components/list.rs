use gpui_kit::prelude::FluentBuilder;
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
        .h(px(38.))
        .flex_shrink_0()
        .w_full()
        .items_center()
        .text_size(px(12.))
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
        .text_size(px(12.))
        .line_height(px(16.))
        .text_left()
        .child(file_icon)
        .child(
            gpui_kit::component::v_flex()
                .flex_1()
                .min_w_0()
                .child(div().truncate().child(name.to_owned()))
                .when_some(detail, |el, detail| {
                    el.child(
                        div()
                            .truncate()
                            .text_size(px(11.))
                            .text_color(rgb(p.muted))
                            .child(detail),
                    )
                }),
        )
}
