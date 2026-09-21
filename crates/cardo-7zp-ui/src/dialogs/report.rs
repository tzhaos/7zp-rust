use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn report_view(&self, text: &String, window: &mut Window) -> AnyElement {
        v_flex()
            .px(px(24.))
            .py(px(22.))
            .gap(px(16.))
            .child(
                div()
                    .id("checksum-report")
                    .max_h(window.viewport_size().height - px(240.))
                    .overflow_scroll()
                    .text_size(px(12.))
                    .child(div().whitespace_nowrap().child(text.clone())),
            )
            .child(
                command("copy-checksum-report", tr("checksum-copy")).on_click({
                    let text = text.clone();
                    move |_, _, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))
                }),
            )
            .into_any_element()
    }
}
