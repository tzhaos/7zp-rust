use crate::*;
use gpui_kit::component::{h_flex, v_flex};

impl Workspace {
    pub(super) fn extraction_result_view(
        &self,
        text: &String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .p(px(24.))
            .gap(px(20.))
            .child(
                div()
                    .id("extraction-result")
                    .max_h(window.viewport_size().height - px(220.))
                    .overflow_scroll()
                    .text_size(px(13.))
                    .child(div().whitespace_normal().child(text.clone())),
            )
            .child(
                h_flex().justify_end().child(
                    command("close-extraction-result", tr("dialog-close"))
                        .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                ),
            )
            .into_any_element()
    }
}
