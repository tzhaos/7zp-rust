use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn help_view(&self, cx: &Context<Self>) -> AnyElement {
        let p = crate::theme::palette(cx);
        let sections = [
            ("help-browse-title", "help-browse-body"),
            ("help-archive-title", "help-archive-body"),
            ("help-entries-title", "help-entries-body"),
            ("help-options-title", "help-options-body"),
            ("help-shortcuts-title", "help-shortcuts-body"),
        ];
        v_flex()
            .id("help-body")
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .px(px(24.))
            .py(px(20.))
            .gap(px(20.))
            .children(sections.map(|(title, body)| {
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(tr(title)),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(20.))
                            .text_color(rgb(p.text))
                            .whitespace_normal()
                            .child(tr(body)),
                    )
            }))
            .into_any_element()
    }
}
