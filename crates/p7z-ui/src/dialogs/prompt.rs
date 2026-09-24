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
        let focus = self.dialogs.focus().clone();
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
            |workspace, window, cx| workspace.prompt_content(window, cx),
            |workspace, _, cx| {
                if matches!(workspace.dialogs.current(), Some(Modal::Progress))
                    || workspace.settings_busy(cx)
                {
                    return false;
                }
                workspace.dialogs.host.detach();
                workspace.close_modal(cx);
                true
            },
            || tr("menu-more").into(),
            cx,
        );
        match opened {
            Ok(handle) => self.bind_dialog(handle, cx),
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

    fn bind_dialog(&mut self, prompt: gpui_kit::AnyWindowHandle, cx: &mut Context<Self>) {
        if let Some(files) = self.dialogs.pending_create.take() {
            let email = self.dialogs.pending_email;
            let name = self.dialogs.pending_name.take();
            self.dialogs.pending_email = false;
            let owner = cx.entity().downgrade();
            let form = prompt.update(cx, move |_, window, cx| {
                cx.new(|cx| {
                    let mut form = CreateForm::new(owner, files, window, cx);
                    form.email = email;
                    if let Some(name) = name {
                        form.suggest_name(name, window, cx);
                    }
                    form
                })
            });
            match form {
                Ok(form) => {
                    let title = self.dialogs.title().to_owned();
                    self.dialogs.show(title, Modal::Create(form));
                }
                Err(error) => {
                    self.prompt_failed(error.context("Cannot initialize dialog form"), cx)
                }
            }
            return;
        }
        let prepared = match self.dialogs.current() {
            Some(Modal::Extract { folder, .. }) => {
                Prepared::Text(folder.read(cx).value().to_string(), false)
            }
            Some(Modal::Password { input, .. }) => {
                Prepared::Text(input.read(cx).value().to_string(), true)
            }
            Some(Modal::Rename { input, .. }) => {
                Prepared::Text(input.read(cx).value().to_string(), false)
            }
            Some(Modal::Comment(input)) => Prepared::Comment(input.read(cx).value().to_string()),
            _ => return,
        };
        let created = prompt.update(cx, |_, window, cx| match &prepared {
            Prepared::Text(value, masked) => {
                let input = cx.new(|cx| InputState::new(window, cx).masked(*masked));
                let text = value.clone();
                input.update(cx, |input, cx| input.set_value(text, window, cx));
                input.update(cx, |input, cx| input.focus(window, cx));
                Bound::Text(input)
            }
            Prepared::Comment(value) => {
                let input =
                    cx.new(|cx| TextareaState::new(window, cx).default_value(value.clone()));
                input.update(cx, |input, cx| input.focus(window, cx));
                Bound::Comment(input)
            }
        });
        let bound = match created {
            Ok(bound) => bound,
            Err(error) => {
                self.prompt_failed(error.context("Cannot initialize dialog input"), cx);
                return;
            }
        };
        match bound {
            Bound::Text(input) => {
                self.dialogs.watch_input(&input, cx);
                match self.dialogs.current_mut() {
                    Some(
                        Modal::Extract { folder, .. }
                        | Modal::Password { input: folder, .. }
                        | Modal::Rename { input: folder, .. },
                    ) => {
                        *folder = input;
                    }
                    _ => {}
                }
            }
            Bound::Comment(input) => {
                if let Some(Modal::Comment(slot)) = self.dialogs.current_mut() {
                    *slot = input;
                }
            }
        }
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
                        this.close_modal(cx);
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

enum Prepared {
    Text(String, bool),
    Comment(String),
}

enum Bound {
    Text(Entity<InputState>),
    Comment(Entity<TextareaState>),
}
