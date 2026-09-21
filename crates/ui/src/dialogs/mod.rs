mod comment;
mod create;
mod error;
mod extract;
mod info;
mod password;
mod rename;
mod report;
mod result;
mod state;
pub(crate) use state::{DialogState, Modal};
mod update;
pub(super) use create::CreateForm;
pub(super) use error::ErrorDialog;

use crate::*;
use gpui_kit::base::FocusTrapElement;
use gpui_kit::{
    component::{Disableable, h_flex, v_flex},
    prelude::FluentBuilder,
};
use sevenzip_core::i18n::tr;

impl Workspace {
    pub(crate) fn footer(
        &self,
        label: &str,
        disabled: bool,
        submit: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        cx: &Context<Self>,
    ) -> Div {
        let p = crate::theme::palette(cx);
        h_flex()
            .flex_shrink_0()
            .px(px(24.))
            .py(px(16.))
            .gap_2()
            .justify_end()
            .bg(rgb(p.panel))
            .rounded_b(px(7.))
            .border_t_1()
            .border_color(rgb(p.border))
            .child(
                command("cancel-modal", tr("cancel"))
                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
            )
            .child(
                primary("submit-modal", label)
                    .disabled(disabled)
                    .on_click(submit),
            )
    }

    pub(crate) fn modal(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let p = crate::theme::palette(cx);
        let modal = self.dialogs.current()?;
        let body = match modal {
            Modal::Error(error) => self.error_view(error, window, cx).into_any_element(),
            Modal::Progress => self.extraction_progress_view(cx).into_any_element(),
            Modal::ExtractionResult(text) => self.extraction_result_view(text, window, cx),
            Modal::Comment(input) => self.comment_view(input, cx),
            Modal::Update(status) => self.update_view(status, cx),
            Modal::Create(form) => form.clone().into_any_element(),
            Modal::Rename { source, input } => self.rename_view(source, input, cx),
            Modal::Report(text) => self.report_view(text, window),
            Modal::Info(fields) => self.info_view(fields, window, cx),
            Modal::Password { input, .. } => self.password_view(input, cx),
            Modal::Extract {
                folder,
                selected,
                destination,
                overwrite,
                open_after,
            } => self.extract_view(
                folder,
                selected,
                destination,
                overwrite,
                open_after,
                window,
                cx,
            ),
        };
        Some(
            div()
                .absolute()
                .inset_0()
                .occlude()
                .bg(rgba(p.scrim))
                .flex()
                .items_center()
                .justify_center()
                .p(px(24.))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.close_modal(cx);
                        cx.stop_propagation();
                    }),
                )
                .child(
                    v_flex()
                        .id("modal")
                        .track_focus(self.dialogs.focus())
                        .w(px(match modal {
                            Modal::Report(_) => 680.,
                            Modal::Progress => 540.,
                            Modal::Error(_) => 520.,
                            _ => 464.,
                        }))
                        .max_w_full()
                        .max_h(window.viewport_size().height - px(48.))
                        .bg(rgb(p.surface))
                        .border_1()
                        .border_color(rgb(p.border))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .shadow_lg()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(
                            h_flex()
                                .flex_shrink_0()
                                .px(px(20.))
                                .py(px(14.))
                                .gap_3()
                                .items_center()
                                .border_b_1()
                                .border_color(rgb(p.border))
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(16.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(self.dialogs.title().to_owned()),
                                )
                                .when(!matches!(modal, Modal::Progress), |el| {
                                    el.child(
                                        icon_button(
                                            "close-modal",
                                            "Dismiss",
                                            tr("dialog-close"),
                                            cx,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.close_modal(cx)),
                                        ),
                                    )
                                }),
                        )
                        .child(body)
                        .focus_trap("modal-trap", self.dialogs.focus()),
                )
                .into_any_element(),
        )
    }
}
