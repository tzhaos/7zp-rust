use crate::*;
use gpui_kit::component::v_flex;

impl Workspace {
    pub(super) fn password_view(
        &self,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .child(
                v_flex()
                    .px(px(24.))
                    .py(px(22.))
                    .gap_3()
                    .child(tr("password-prompt"))
                    .child(text_input(input)),
            )
            .child(self.footer(
                tr("continue"),
                input.read(cx).value().is_empty(),
                cx.listener(|this, _, _, cx| {
                    if let Some(Modal::Password { input, request }) = this.dialogs.take() {
                        let password = input.read(cx).value().to_string();
                        this.browser.set_password(password.clone());
                        this.execute(request, password, cx);
                    }
                }),
                cx,
            ))
            .into_any_element()
    }
}
