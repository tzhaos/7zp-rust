use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn confirm_run_view(&self, entry: &str, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .child(
                div()
                    .px(px(24.))
                    .py(px(22.))
                    .child(tf("file-run-confirm", &[("path", entry.into())])),
            )
            .child(self.footer(
                tr("yes"),
                false,
                cx.listener(|this, _, _, cx| {
                    let Some((directory, target)) = this.pending_run.take() else {
                        return;
                    };
                    this.tasks.retain_file(directory);
                    this.execute(Request::Launch(target), String::new(), cx);
                }),
                cx,
            ))
            .into_any_element()
    }
}
