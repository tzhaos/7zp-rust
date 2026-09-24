use super::*;
use gpui_kit::{base::FocusTrapElement, component::v_flex};
impl Workspace {
    pub(crate) fn schedule_prompt(&mut self, cx: &mut Context<Self>) {
        if self.dialogs.has_prompt() {
            return;
        }
        if !self.dialogs.is_open() && self.dialogs.pending_create.is_none() {
            return;
        }
        let owner = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = owner.update(cx, |this, cx| {
                if this.dialogs.has_prompt() {
                    return;
                }
                if this.dialogs.is_open() || this.dialogs.pending_create.is_some() {
                    this.open_prompt_window(cx);
                    cx.notify();
                }
            });
        });
    }

    pub(crate) fn open_prompt_window(&mut self, cx: &mut Context<Self>) {
        let panel = self.dialogs.panel_size();
        let title = self.dialogs.title().to_owned().into();
        let owner = cx.entity();
        let owner_id = owner.entity_id();
        let focus = self.dialogs.focus().clone();
        let pending = self.dialogs.pending.take();
        let files = self.dialogs.pending_create.take();
        let email = std::mem::take(&mut self.dialogs.pending_email);
        let name = self.dialogs.pending_name.take();
        let form_owner = owner.downgrade();
        let opened = self.dialogs.host.open(
            owner,
            self.main_window,
            self.content_bounds.get(),
            title,
            cardo_ui::dialog::DialogSpec {
                preferred: panel.size(),
                minimum: panel.minimum(),
            },
            focus,
            move |window, cx| {
                if let Some(files) = files {
                    return Some(Modal::Create(cx.new(|cx| {
                        let mut form = CreateForm::new(form_owner, files, window, cx);
                        form.email = email;
                        if let Some(name) = name {
                            form.suggest_name(name, window, cx);
                        }
                        form
                    })));
                }
                pending.map(|pending| match pending {
                    PendingModal::Comment(text) => {
                        let input = cx.new(|cx| TextareaState::new(window, cx).default_value(text));
                        input.update(cx, |input, cx| input.focus(window, cx));
                        Modal::Comment(input)
                    }
                    PendingModal::Password(request) => Modal::Password {
                        input: create_input(String::new(), true, window, cx),
                        request,
                    },
                    PendingModal::Rename { source, name } => Modal::Rename {
                        source,
                        input: create_input(name, false, window, cx),
                        error: None,
                    },
                    PendingModal::Extract {
                        folder,
                        selected,
                        destination,
                        open_after,
                    } => Modal::Extract {
                        folder: create_input(folder, false, window, cx),
                        selected,
                        destination,
                        open_after,
                    },
                })
            },
            |workspace, window, cx| workspace.prompt_content(window, cx),
            |workspace, cx| {
                !matches!(workspace.dialogs.current(), Some(Modal::Progress))
                    && !workspace.settings_busy(cx)
            },
            move |workspace, reason, cx| {
                workspace.dialogs.host.detach();
                if !matches!(reason, cardo_ui::dialog::CloseReason::Submit) {
                    workspace.cleanup_modal();
                }
                cx.notify(owner_id);
                cx.refresh_windows();
            },
            || tr("menu-more").into(),
            cx,
        );
        match opened {
            Ok((_, Some(modal))) => {
                match &modal {
                    Modal::Extract { folder: input, .. }
                    | Modal::Password { input, .. }
                    | Modal::Rename { input, .. } => self.dialogs.watch_input(input, cx),
                    _ => {}
                }
                self.dialogs.show(self.dialogs.title().to_owned(), modal);
            }
            Ok((_, None)) => {}
            Err(error) => self.prompt_failed(error.context("Cannot create native dialog"), cx),
        }
    }

    pub(crate) fn prompt_failed(&mut self, error: anyhow::Error, cx: &mut Context<Self>) {
        let mut details = tf(
            "dialog-open-failed-detail",
            &[
                ("title", self.dialogs.title().into()),
                ("error", format!("{error:#}").into()),
            ],
        );
        if let Some(Modal::Error(original)) = self.dialogs.current() {
            details.push_str("\n\n");
            details.push_str(&original.details);
        }
        tracing::error!(error = %details, "Dialog presentation failed");
        if self.tasks.is_busy() {
            self.tasks.cancel(tr("cancelling"));
        }
        self.tasks.set_close_after(false);
        self.tasks.set_close_archive(false);
        self.extract_follow.clear();
        self.after_open = None;
        self.pending_run = None;
        self.browser.cancel_navigation();
        self.dialogs.take();
        self.dialogs.close_prompt(cx);
        // A failed presentation is reported explicitly; the business form is never rehosted.
        cx.background_executor()
            .spawn(async move {
                rfd::MessageDialog::new()
                    .set_title(tr("dialog-open-failed"))
                    .set_description(details)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            })
            .detach();
        cx.notify();
    }

    fn prompt_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let body = self.dialog_body(window, cx)?;
        let dismissible =
            !matches!(self.dialogs.current(), Some(Modal::Progress)) && !self.settings_busy(cx);
        let body = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .px(px(crate::theme::metrics::popup::OUTER_INSET))
            .pb(px(crate::theme::metrics::popup::OUTER_INSET))
            .child(panel_surface(cx).child(body));
        Some(
            panel_frame(cx)
                .id("prompt-panel")
                .track_focus(self.dialogs.focus())
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if cardo_ui::menu::MenuHost::is_open(window, cx) {
                        return;
                    }
                    if event.keystroke.key == "escape" {
                        this.dismiss_modal(cardo_ui::dialog::CloseReason::Escape, cx);
                        cx.stop_propagation();
                    }
                }))
                .child(self.dialog_title("close-prompt", true, dismissible, cx))
                .child(body)
                .focus_trap("prompt-trap", self.dialogs.focus())
                .into_any_element(),
        )
    }
}

fn create_input(
    value: String,
    masked: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    let input = cx.new(|cx| InputState::new(window, cx).masked(masked));
    input.update(cx, |input, cx| {
        input.set_value(value, window, cx);
        input.focus(window, cx);
    });
    input
}
