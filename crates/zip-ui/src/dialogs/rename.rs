use crate::*;
use gpui_kit::prelude::FluentBuilder;

impl Workspace {
    pub(super) fn rename_view(
        &self,
        source: &String,
        input: &Entity<InputState>,
        error: &Option<String>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("rename-body")
                    .child(path_strip("rename-source", source.clone(), cx))
                    .child(panel_field(
                        tr("rename-new-name"),
                        settings_input(input, tr("rename-new-name")),
                    ))
                    .when_some(error.clone(), |el, error| {
                        el.child(panel_notice(
                            "Info",
                            error,
                            crate::theme::palette(cx).danger,
                        ))
                    }),
            )
            .child(self.footer(
                tr("rename-action"),
                input.read(cx).value().trim().is_empty(),
                cx.listener(|this, _, _, cx| this.submit_rename(cx)),
                cx,
            ))
            .into_any_element()
    }
}
