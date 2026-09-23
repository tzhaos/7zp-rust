use crate::*;
use gpui_kit::component::input::Textarea;

impl Workspace {
    pub(super) fn comment_view(
        &self,
        input: &Entity<TextareaState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        panel_layout(cx)
            .child(
                body_container("comment-body")
                    .child(
                        body_text(tr("archive-comment"))
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(Textarea::new(input).h_full().min_h_0().text_size(px(13.))),
            )
            .child(self.footer(
                tr("settings-save"),
                false,
                cx.listener(|this, _, _, cx| this.submit_comment(cx)),
                cx,
            ))
            .into_any_element()
    }
}
