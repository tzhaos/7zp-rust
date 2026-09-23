use gpui_kit::*;

pub fn compact_text(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Stateful<Div> {
    let text = text.into();
    crate::tooltip::bubble_tooltip(
        div()
            .id(id)
            .min_w_0()
            .truncate()
            .aria_label(text.clone())
            .child(text.clone()),
        text,
    )
}

pub fn body_text(text: impl Into<SharedString>) -> Div {
    div().min_w_0().whitespace_normal().child(text.into())
}
