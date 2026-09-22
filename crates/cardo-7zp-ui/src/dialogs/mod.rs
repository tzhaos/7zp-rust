mod comment;
mod confirm;
mod conflict;
mod create;
mod error;
mod extract;
mod help;
mod info;
mod password;
mod prompt;
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
use cardo_7zp_core::i18n::tr;

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
        self.dialogs.set_open_failed(false);
        self.schedule_prompt(cx);
        cx.notify();
    }

    pub(crate) fn action_row(&self, cx: &Context<Self>) -> Div {
        self.action_chrome(h_flex(), cx).justify_end()
    }

    pub(crate) fn action_stack(&self, cx: &Context<Self>) -> Div {
        self.action_chrome(v_flex(), cx)
            .gap(px(8.))
            .items_stretch()
    }

    fn action_chrome(&self, row: Div, cx: &Context<Self>) -> Div {
        let p = crate::theme::palette(cx);
        row.flex_shrink_0()
            .px(px(24.))
            .py(px(16.))
            .gap(px(8.))
            .items_center()
            .bg(rgb(p.panel))
            .border_t_1()
            .border_color(rgb(p.border))
    }

    pub(crate) fn dialog_title(
        &self,
        id: &'static str,
        drag: bool,
        dismissible: bool,
        cx: &Context<Self>,
    ) -> Div {
        let p = crate::theme::palette(cx);
        h_flex()
            .flex_shrink_0()
            .h(px(40.))
            .px(px(8.))
            .gap(px(8.))
            .items_center()
            .bg(rgb(p.surface))
            .border_b_1()
            .border_color(rgb(p.border))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .when(drag, |el| el.window_control_area(WindowControlArea::Drag))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.dialogs.title().to_owned()),
            )
            .when(dismissible, |el| {
                el.child(
                    icon_button(id, "Dismiss", tr("dialog-close"), false, cx).on_click(
                        cx.listener(|this, _, _, cx| this.close_modal(cx)),
                    ),
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
                command("cancel-modal", tr("cancel"))
                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
            )
            .child(
                primary("submit-modal", label)
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
            Modal::Comment(input) => self.comment_view(input, cx),
            Modal::Update(status) => self.update_view(status, cx),
            Modal::Create(form) => form.clone().into_any_element(),
            Modal::Rename { source, input } => self.rename_view(source, input, cx),
            Modal::Report(text) => self.report_view(text, window, cx),
            Modal::Info(fields) => self.info_view(fields, window, cx),
            Modal::Password { input, .. } => self.password_view(input, cx),
            Modal::ConfirmRun { entry } => self.confirm_run_view(entry, cx),
            Modal::Conflict { .. } => self.conflict_view(cx),
            Modal::Help => self.help_view(cx),
            Modal::Extract {
                folder,
                selected,
                destination,
                open_after,
            } => self.extract_view(folder, selected, destination, open_after, window, cx),
        })
    }

    pub(crate) fn modal(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let p = crate::theme::palette(cx);
        if self.dialogs.has_prompt() {
            let prompt = self.dialogs.prompt_handle();
            return Some(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(rgba(p.scrim))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |_, _, _, cx| {
                            if let Some(prompt) = prompt {
                                let _ = prompt.update(cx, |_, window, _| window.activate_window());
                            }
                            cx.stop_propagation();
                        }),
                    )
                    .into_any_element(),
            );
        }
        if !self.dialogs.open_failed() {
            return None;
        }
        let modal = self.dialogs.current()?;
        let body = self.dialog_body(window, cx)?;
        let dismissible = !matches!(modal, Modal::Progress);
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
                            Modal::Conflict { .. } => 640.,
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
                        .child(self.dialog_title("close-modal", false, dismissible, cx))
                        .child(body)
                        .focus_trap("modal-trap", self.dialogs.focus()),
                )
                .into_any_element(),
        )
    }
}
