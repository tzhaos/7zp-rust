use cardo_ui::ConditionalBuilder;
use crate::*;


pub(crate) struct ErrorDialog {
    pub(crate) summary: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) details: String,
    pub(crate) expanded: bool,
}

impl Workspace {
    pub(crate) fn error_view(
        &self,
        error: &ErrorDialog,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let notice = panel_artwork_notice(ToolIcon::Error, error.summary.clone(), window, cx);
        let details = error.details.clone();
        panel_layout(cx)
            .child(
                panel_document("error-body")
                    .child(notice)
                    .when_some(error.path.clone(), |el, path| {
                        el.child(path_strip(
                            "error-copy-path",
                            path.display().to_string(),
                            cx,
                        ))
                    })
                    .child(
                        command(
                            "error-details-toggle",
                            tr(if error.expanded {
                                "error-hide-details"
                            } else {
                                "error-show-details"
                            }),
                        )
                        .border_0()
                        .self_start()
                        .icon(icon(
                            if error.expanded {
                                "ChevronDown"
                            } else {
                                "ChevronRight"
                            },
                            14.,
                        ))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(Modal::Error(error)) = this.dialogs.current_mut() {
                                error.expanded = !error.expanded;
                                cx.notify();
                            }
                        })),
                    )
                    .when(error.expanded, |el| {
                        el.child(
                            panel_card(cx)
                                .id("error-details")
                                .w_full()
                                .font_family("Consolas")
                                .text_size(px(11.))
                                .line_height(px(17.))
                                .child(div().whitespace_nowrap().child(error.details.clone())),
                        )
                    }),
            )
            .child(
                self.action_row(cx).child(
                    panel_button("error-copy-details", tr("error-copy-details"))
                        .icon(icon("Copy", 14.))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(details.clone()))
                        }),
                ),
            )
    }
}
