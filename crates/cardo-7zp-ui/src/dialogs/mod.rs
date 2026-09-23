mod comment;
mod confirm;
mod conflict;
mod create;
mod error;
mod extract;
mod info;
mod password;
mod prompt;
mod rename;
mod report;
mod result;
mod state;
pub(super) use create::CreateForm;
pub(super) use error::ErrorDialog;
pub(crate) use state::{DialogState, Modal};

use crate::*;
use cardo_7zp_core::i18n::tr;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn show_dialog(
        &mut self,
        title: impl Into<String>,
        modal: Modal,
        cx: &mut Context<Self>,
    ) {
        self.dialogs.show(title, modal);
        self.dialogs.bump_generation();
        self.dialogs.close_prompt(cx);
        self.schedule_prompt(cx);
        cx.notify();
    }

    pub(crate) fn action_row(&self, cx: &Context<Self>) -> Div {
        panel_actions(cx)
    }

    pub(crate) fn dialog_title(
        &self,
        id: &'static str,
        drag: bool,
        dismissible: bool,
        cx: &Context<Self>,
    ) -> Div {
        panel_header(self.dialogs.title().to_owned(), drag, cx).when(dismissible, |el| {
            el.child(
                icon_button(id, "Dismiss", tr("dialog-close"), false, cx)
                    .custom(header_variant(cx))
                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
            )
        })
    }

    pub(crate) fn footer(
        &self,
        label: &str,
        disabled: bool,
        submit: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        cx: &Context<Self>,
    ) -> Div {
        self.action_row(cx)
            .child(
                panel_button("cancel-modal", tr("cancel"))
                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
            )
            .child(
                panel_primary("submit-modal", label)
                    .disabled(disabled)
                    .on_click(submit),
            )
    }

    fn dialog_body(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let modal = self.dialogs.current()?;
        Some(match modal {
            Modal::Error(error) => self.error_view(error, window, cx).into_any_element(),
            Modal::Progress => self.extraction_progress_view(window, cx).into_any_element(),
            Modal::ExtractionResult(text) => self.extraction_result_view(text, window, cx),
            Modal::Completion {
                label,
                path,
                notice,
            } => self.completion_view(label, path, notice, cx),
            Modal::Comment(input) => self.comment_view(input, cx),
            Modal::Create(form) => form.clone().into_any_element(),
            Modal::Rename {
                source,
                input,
                error,
            } => self.rename_view(source, input, error, cx),
            Modal::Report(text) => self.report_view(text, window, cx),
            Modal::Info(fields) => self.info_view(fields, window, cx),
            Modal::Password { input, .. } => self.password_view(input, cx),
            Modal::ConfirmRun { entry } => self.confirm_run_view(entry, cx),
            Modal::ConfirmDelete { count, .. } => self.confirm_delete_view(*count, cx),
            Modal::Conflict { .. } => self.conflict_view(window, cx),
            Modal::Extract {
                folder,
                selected,
                destination,
                open_after,
            } => self.extract_view(folder, selected, destination, open_after, window, cx),
        })
    }

    pub(crate) fn modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let prompt = self.dialogs.prompt_handle()?;
        Some(
            div()
                .absolute()
                .inset_0()
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_, _, _, cx| {
                        if let Err(error) =
                            prompt.update(cx, |_, window, _| window.activate_window())
                        {
                            tracing::error!(error = %error, "Cannot activate dialog");
                        }
                        cx.stop_propagation();
                    }),
                )
                .into_any_element(),
        )
    }
}
