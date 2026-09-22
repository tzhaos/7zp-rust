use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn report_view(&self, text: &String, _window: &mut Window, cx: &Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .id("checksum-report")
                    .flex_1()
                    .min_h_0()
                    .px(px(24.))
                    .py(px(20.))
                    .overflow_y_scrollbar()
                    .text_size(px(12.))
                    .child(div().whitespace_nowrap().child(text.clone())),
            )
            .child(self.action_row(cx).child(
                command("copy-checksum-report", tr("checksum-copy")).on_click({
                    let text = text.clone();
                    move |_, _, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))
                }),
            ))
            .into_any_element()
    }
}
