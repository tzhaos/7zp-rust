use crate::*;

impl Workspace {
    pub(super) fn password_view(
        &self,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("password-body")
                    .child(panel_notice(
                        "LockClosed",
                        tr("password-prompt"),
                        crate::theme::palette(cx).accent,
                    ))
                    .child(panel_field(
                        tr("password"),
                        settings_input(input, tr("password")),
                    )),
            )
            .child(self.footer(
                tr("continue"),
                input.read(cx).value().is_empty(),
                cx.listener(|this, _, _, cx| this.submit_password(cx)),
                cx,
            ))
            .into_any_element()
    }
}
