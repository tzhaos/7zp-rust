use crate::*;
use gpui_kit::component::{h_flex, input::Textarea, v_flex};

impl Workspace {
    pub(super) fn comment_view(
        &self,
        input: &Entity<TextareaState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        {
            v_flex()
                .p(px(24.))
                .gap(px(20.))
                .child(Textarea::new(input).h(px(210.)).text_size(px(13.)))
                .child(
                    h_flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            command("comment-cancel", tr("cancel"))
                                .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                        )
                        .child(
                            primary("comment-save", tr("settings-save")).on_click(cx.listener(
                                |this, _, _, cx| {
                                    if let (Some(Modal::Comment(input)), Some(catalog)) =
                                        (this.dialogs.current(), &this.browser.view().catalog)
                                    {
                                        let request = Request::Comment(
                                            catalog.clone(),
                                            input.read(cx).value().to_string(),
                                        );
                                        this.execute(
                                            request,
                                            this.browser.view().password.clone(),
                                            cx,
                                        );
                                    }
                                },
                            )),
                        ),
                )
                .into_any_element()
        }
    }
}
