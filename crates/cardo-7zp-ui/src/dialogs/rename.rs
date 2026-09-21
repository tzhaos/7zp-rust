use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn rename_view(
        &self,
        source: &String,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .child(
                v_flex()
                    .px(px(24.))
                    .py(px(22.))
                    .gap_3()
                    .child(source.clone())
                    .child(text_input(input)),
            )
            .child(self.footer(
                tr("continue"),
                input.read(cx).value().trim().is_empty(),
                cx.listener(|this, _, _, cx| {
                    if let Some(Modal::Rename { source, input }) = this.dialogs.take() {
                        let name = input.read(cx).value().trim().to_owned();
                        if name.contains(['/', '\\']) {
                            this.message = Some(tr("archive-name-invalid").into());
                            cx.notify();
                            return;
                        }
                        let destination = source
                            .rsplit_once('/')
                            .map(|(parent, _)| format!("{parent}/{name}"))
                            .unwrap_or(name);
                        if let Some(catalog) = this.browser.view().catalog.clone() {
                            this.execute(
                                Request::Edit(
                                    catalog,
                                    Edit::Rename {
                                        source,
                                        destination,
                                    },
                                ),
                                this.browser.view().password.clone(),
                                cx,
                            );
                        }
                    }
                }),
                cx,
            ))
            .into_any_element()
    }
}
