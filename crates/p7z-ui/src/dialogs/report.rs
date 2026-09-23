use crate::*;

impl Workspace {
    pub(super) fn report_view(
        &self,
        text: &String,
        _window: &mut Window,
        cx: &Context<Self>,
    ) -> AnyElement {
        let copy = text.clone();
        panel_layout(cx)
            .child(
                panel_document("checksum-report").child(
                    panel_card(cx).child(
                        div()
                            .font_family("Consolas")
                            .text_size(px(12.))
                            .whitespace_nowrap()
                            .child(text.clone()),
                    ),
                ),
            )
            .child(
                panel_actions(cx).child(
                    panel_button("copy-checksum-report", tr("checksum-copy"))
                        .icon(icon("Copy", 16.))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()))
                        }),
                ),
            )
            .into_any_element()
    }
}
