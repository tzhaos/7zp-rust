use crate::*;
use gpui_kit::component::{h_flex, v_flex};

impl Workspace {
    pub(super) fn info_view(
        &self,
        fields: &[(String, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let p = crate::theme::palette(cx);
        v_flex()
            .id("properties-body")
            .max_h(window.viewport_size().height - px(180.))
            .overflow_y_scrollbar()
            .flex_1()
            .min_h_0()
            .px(px(24.))
            .py(px(20.))
            .gap(px(16.))
            .children(fields.iter().map(|(key, value)| {
                h_flex()
                    .gap(px(16.))
                    .child(
                        div()
                            .w(px(88.))
                            .flex_shrink_0()
                            .text_color(rgb(p.muted))
                            .child(key.clone()),
                    )
                    .child(div().flex_1().min_w_0().child(value.clone()))
            }))
            .into_any_element()
    }
}
