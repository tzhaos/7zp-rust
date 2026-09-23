use crate::theme::metrics::settings as metrics;
use gpui_kit::component::scroll::Scrollable;
use gpui_kit::component::v_flex;
use gpui_kit::*;

pub fn settings_page(cx: &App) -> Div {
    v_flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .text_size(crate::theme::ui_font_size(cx))
        .whitespace_normal()
        .line_height(relative(1.5))
}

pub fn settings_frame() -> Div {
    v_flex()
        .w_full()
        .min_w(px(metrics::CONTENT_MIN_WIDTH))
        .max_w(px(metrics::CONTENT_MAX_WIDTH))
        .mx_auto()
        .flex_shrink_0()
        .gap(px(metrics::SECTION_GAP))
}

pub fn settings_content(id: &'static str, content: Div) -> Scrollable<Stateful<Div>> {
    super::panel_body(id)
        .py(px(metrics::CONTENT_PADDING))
        .px(px(metrics::OUTER_PADDING))
        .child(content)
}

pub fn settings_row(label: &str, control: impl IntoElement, cx: &App) -> Div {
    cardo_ui::settings::row(label.to_owned(), None, control, cx)
}

pub fn settings_detail(label: &str, description: &str, control: impl IntoElement, cx: &App) -> Div {
    cardo_ui::settings::row(
        label.to_owned(),
        Some(description.to_owned().into()),
        control,
        cx,
    )
}

pub fn settings_group(rows: impl IntoIterator<Item = AnyElement>, cx: &App) -> Div {
    cardo_ui::settings::group(rows, cx)
}

pub fn settings_section(title: &str, group: Div, cx: &App) -> Div {
    v_flex()
        .flex_shrink_0()
        .gap(px(metrics::HEADING_GAP))
        .child(
            super::body_text(title.to_owned())
                .w_full()
                .text_size(px(metrics::SECTION_TITLE_SIZE).max(crate::theme::ui_font_size(cx)))
                .font_weight(FontWeight::SEMIBOLD),
        )
        .child(group)
}
