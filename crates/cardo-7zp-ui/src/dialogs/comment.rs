use crate::*;
use gpui_kit::component::{input::Textarea, v_flex};

impl Workspace {
    pub(super) fn comment_view(
        &self,
        input: &Entity<TextareaState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        {
            v_flex()
                .flex_1()
                .min_h_0()
                .child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .px(px(24.))
                        .py(px(20.))
                        .child(Textarea::new(input).h(px(210.)).text_size(px(13.))),
                )
                .child(self.footer(
                    tr("settings-save"),
                    false,
                    cx.listener(|this, _, _, cx| {
                        if let (Some(Modal::Comment(input)), Some(catalog)) =
                            (this.dialogs.current(), &this.browser.view().catalog)
                        {
                            let request = Request::Comment(
                                catalog.clone(),
                                input.read(cx).value().to_string(),
                            );
                            this.execute(request, this.browser.view().password.clone(), cx);
                        }
                    }),
                    cx,
                ))
                .into_any_element()
        }
    }
}
