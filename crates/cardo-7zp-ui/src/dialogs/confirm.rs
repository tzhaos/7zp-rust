use crate::*;

impl Workspace {
    pub(super) fn confirm_run_view(&self, entry: &str, cx: &mut Context<Self>) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("confirm-run-body")
                    .child(panel_notice(
                        "Info",
                        tr("file-run-question"),
                        crate::theme::palette(cx).accent,
                    ))
                    .child(path_strip("confirm-run-path", entry.to_owned(), cx)),
            )
            .child(self.footer(
                tr("file-run-action"),
                false,
                cx.listener(|this, _, _, cx| this.confirm_run(cx)),
                cx,
            ))
            .into_any_element()
    }

    pub(super) fn confirm_delete_view(
        &self,
        count: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let notice_icon = artwork(ToolIcon::Delete, window, cx).into_any_element();
        panel_layout(cx)
            .child(
                panel_body("confirm-delete-body").child(
                    gpui_kit::component::h_flex()
                        .w_full()
                        .min_w_0()
                        .flex_shrink_0()
                        .gap(px(12.))
                        .child(notice_icon)
                        .child(
                            body_text(tf("archive-delete-confirm", &[("count", count.into())]))
                                .flex_1()
                                .text_size(px(16.))
                                .font_weight(FontWeight::SEMIBOLD),
                        ),
                ),
            )
            .child(
                panel_actions(cx)
                    .child(
                        panel_button("cancel-delete", tr("cancel"))
                            .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                    )
                    .child(
                        panel_danger("confirm-delete", tr("dialog-delete"))
                            .on_click(cx.listener(|this, _, _, cx| this.confirm_delete(cx))),
                    ),
            )
            .into_any_element()
    }
}
