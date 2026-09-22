use crate::*;
use gpui_kit::{
    component::{button::ButtonVariants, h_flex, v_flex},
    prelude::FluentBuilder,
};

pub(crate) struct ErrorDialog {
    summary: String,
    path: Option<PathBuf>,
    details: String,
    expanded: bool,
}

impl Workspace {
    pub(crate) fn show_task_error(
        &mut self,
        error: anyhow::Error,
        extracting: bool,
        cx: &mut Context<Self>,
    ) {
        let mut title = tr(if extracting {
            "extract-failed"
        } else {
            "operation-failed"
        });
        let mut summary = tr("operation-failed-summary");
        let mut path = None;
        if let Some(source) = error.downcast_ref::<cardo_7zp_engine::SourceError>() {
            path = Some(source.path.clone());
            match source.source.kind() {
                std::io::ErrorKind::NotFound => {
                    title = tr("archive-missing-title");
                    let in_history = self.recent_archives.contains(&source.path);
                    summary = tr(if in_history {
                        "archive-missing-history"
                    } else {
                        "archive-missing-summary"
                    });
                    self.recent_archives.retain(|path| path != &source.path);
                    self.update_recent(recent::Change::Remove(source.path.clone()), cx);
                }
                std::io::ErrorKind::PermissionDenied => {
                    title = tr("archive-access-title");
                    summary = tr("archive-access-summary");
                }
                _ => {
                    title = tr("archive-access-title");
                    summary = tr("archive-read-summary");
                }
            }
        } else {
            self.reload_history(cx);
        }
        self.message = None;
        self.show_dialog(
            title,
            Modal::Error(ErrorDialog {
                summary: summary.into(),
                path,
                details: format!("{error:#}"),
                expanded: false,
            }),
            cx,
        );
    }

    pub(crate) fn error_view(
        &self,
        error: &ErrorDialog,
        window: &Window,
        cx: &Context<Self>,
    ) -> Div {
        let p = crate::theme::palette(cx);
        let details = error.details.clone();
        v_flex()
            .flex_1()
            .min_h_0()
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .px(px(24.))
                    .py(px(20.))
                    .gap(px(16.))
                    .child(
                h_flex()
                    .items_start()
                    .gap(px(12.))
                    .child(icon("Info", 22.).text_color(rgb(p.danger)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(13.))
                            .line_height(px(20.))
                            .whitespace_normal()
                            .child(error.summary.clone()),
                    ),
            )
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
                .custom(subtle_variant(cx))
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
                    div()
                        .id("error-details")
                        .w_full()
                        .min_h_0()
                        .max_h(window.viewport_size().height - px(360.))
                        .overflow_scrollbar()
                        .p(px(12.))
                        .bg(rgb(p.panel))
                        .border_1()
                        .border_color(rgb(p.border))
                        .rounded(px(4.))
                        .font_family("Consolas")
                        .text_size(px(11.))
                        .line_height(px(17.))
                        .child(div().whitespace_nowrap().child(error.details.clone())),
                )
            })
            )
            .child(
                self.action_row(cx)
                    .child(
                        command("error-copy-details", tr("error-copy-details"))
                            .icon(icon("Copy", 14.))
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(details.clone()))
                            }),
                    )
                    .child(
                        primary("error-dismiss", tr("error-dismiss"))
                            .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                    ),
            )
    }
}
