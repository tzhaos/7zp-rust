use crate::*;
use gpui_kit::prelude::FluentBuilder;

impl Workspace {
    pub(super) fn completion_view(
        &self,
        label: &str,
        path: &Path,
        notice: &Option<String>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let p = crate::theme::palette(cx);
        let destination = path.to_owned();
        panel_layout(cx)
            .child(
                panel_body("completion-body")
                    .child(panel_notice("CheckmarkCircle", label.to_owned(), p.success))
                    .child(path_strip(
                        "copy-completion-path",
                        path.display().to_string(),
                        cx,
                    ))
                    .when_some(notice.clone(), |el, text| {
                        el.child(panel_notice("Info", text, p.danger))
                    }),
            )
            .child(
                panel_actions(cx).child(
                    panel_primary("reveal-completion", tr("folder-open"))
                        .icon(icon("FolderOpen", 16.))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.reveal_completion(destination.clone(), cx);
                        })),
                ),
            )
            .into_any_element()
    }

    pub(super) fn extraction_result_view(
        &self,
        text: &String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("extraction-result")
                    .child(panel_card(cx).child(body_text(text.clone()))),
            )
            .into_any_element()
    }
}
