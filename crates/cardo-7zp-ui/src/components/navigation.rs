use super::{icon, icon_button};
use gpui_kit::*;
use cardo_7zp_core::i18n::tr;
use std::{cell::Cell, rc::Rc};

pub fn navigation_row(cx: &App) -> Div {
    gpui_kit::component::h_flex()
        .h(px(44.))
        .flex_shrink_0()
        .px(px(16.))
        .gap(px(6.))
        .items_center()
        .border_b_1()
        .border_color(rgb(crate::theme::palette(cx).border))
}

pub fn panel_pointer(center: Rc<Cell<Pixels>>, cx: &App) -> impl IntoElement + use<> {
    let p = crate::theme::palette(cx);
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let x = center
                .get()
                .clamp(bounds.left() + px(14.), bounds.right() - px(14.));
            let top = bounds.top();
            let mut triangle = PathBuilder::fill();
            triangle.move_to(point(x - px(14.), top + px(2.)));
            triangle.line_to(point(x + px(14.), top + px(2.)));
            triangle.line_to(point(x + px(14.), top + px(5.)));
            triangle.line_to(point(x - px(14.), top + px(5.)));
            triangle.close();
            if let Ok(path) = triangle.build() {
                window.paint_path(path, rgb(p.accent));
            }
        },
    )
    .absolute()
    .left_0()
    .top(px(-8.))
    .w_full()
    .h(px(7.))
}

pub fn path_strip(id: impl Into<ElementId>, path: String, cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    gpui_kit::component::h_flex()
        .min_w_0()
        .gap(px(6.))
        .child(
            gpui_kit::component::h_flex()
                .flex_1()
                .min_w_0()
                .h(px(28.))
                .px(px(8.))
                .gap(px(7.))
                .rounded(px(6.))
                .bg(rgb(p.title))
                .text_color(rgb(p.muted))
                .text_size(px(12.))
                .child(icon("Folder", 14.))
                .child(div().flex_1().min_w_0().truncate().child(path.clone())),
        )
        .child(
            icon_button(id, "Copy", tr("browser-copy-path"), cx)
                .w(px(28.))
                .h(px(28.))
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(path.clone()))
                }),
        )
}
