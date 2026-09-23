use super::{icon, icon_button};
use zip_core::i18n::tr;
use gpui_kit::*;

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

pub fn path_strip(id: impl Into<ElementId>, path: String, cx: &App) -> Stateful<Div> {
    let p = crate::theme::palette(cx);
    gpui_kit::component::h_flex()
        .id(id)
        .min_w_0()
        .gap(px(6.))
        .child(
            gpui_kit::component::h_flex()
                .flex_1()
                .min_w_0()
                .h(px(crate::theme::metrics::CONTROL_HEIGHT))
                .px(px(8.))
                .gap(px(7.))
                .rounded(px(cardo_ui::settings::metrics::CONTROL_RADIUS))
                .bg(rgb(p.surface))
                .border_1()
                .border_color(rgb(p.border))
                .text_color(rgb(p.muted))
                .text_size(px(12.))
                .child(icon("Folder", 14.))
                .child(
                    super::compact_text("path-value", path.clone())
                        .flex_1()
                        .text_ellipsis_middle(),
                ),
        )
        .child(
            icon_button("copy-path", "Copy", tr("browser-copy-path"), true, cx)
                .w(px(crate::theme::metrics::CONTROL_HEIGHT))
                .h(px(crate::theme::metrics::CONTROL_HEIGHT))
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(path.clone()))
                }),
        )
}
