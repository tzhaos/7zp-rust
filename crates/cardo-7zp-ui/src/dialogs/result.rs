use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn extraction_result_view(
        &self,
        text: &String,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .flex_1()
            .min_h_0()
            .px(px(24.))
            .py(px(20.))
            .child(
                div()
                    .id("extraction-result")
                    .flex_1()
                    .min_h_0()
                    .max_h(window.viewport_size().height - px(120.))
                    .overflow_y_scrollbar()
                    .text_size(px(13.))
                    .child(div().whitespace_normal().child(text.clone())),
            )
            .into_any_element()
    }
}
