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

    pub(super) fn confirm_delete_view(&self, count: usize, cx: &mut Context<Self>) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("confirm-delete-body")
                    .justify_center()
                    .items_center()
                    .gap(px(20.))
                    .child(icon("Delete", 48.).text_color(rgb(crate::theme::palette(cx).danger)))
                    .child(
                        body_text(tf("archive-delete-confirm", &[("count", count.into())]))
                            .max_w(px(480.))
                            .text_center()
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD),
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
