use super::*;
use gpui_kit::{
    base::FocusTrapElement,
    component::{Root, h_flex, v_flex},
};
use std::cell::Cell;

thread_local! {
    static OPENING_PROMPT: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct PromptWindow {
    menu_host: Entity<cardo_ui::menu::MenuHost>,
    owner: WeakEntity<Workspace>,
    _watch: Subscription,
}

impl PromptWindow {
    fn new(
        owner: Entity<Workspace>,
        focus: FocusHandle,
        generation: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        focus.focus(window, cx);
        let weak = owner.downgrade();
        window.on_window_should_close(cx, {
            let weak = weak.clone();
            move |_, cx| {
                weak.update(cx, |workspace, cx| {
                    if workspace.dialogs.generation() != generation {
                        return true;
                    }
                    if matches!(workspace.dialogs.current(), Some(Modal::Progress))
                        || workspace.settings_busy(cx)
                    {
                        return false;
                    }
                    workspace.dialogs.detach_prompt();
                    workspace.close_modal(cx);
                    true
                })
                .unwrap_or(true)
            }
        });
        Self {
            menu_host: cardo_ui::menu::MenuHost::install(window, cx, || tr("menu-more").into()),
            owner: weak,
            _watch: cx.observe(&owner, |_, _, cx| cx.notify()),
        }
    }
}

impl Render for PromptWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if OPENING_PROMPT.get() {
            return div().size_full().into_any_element();
        }
        let content = self
            .owner
            .update(cx, |workspace, cx| workspace.prompt_content(window, cx))
            .ok()
            .flatten();
        if content.is_none() {
            cx.defer_in(window, |_, window, _| window.remove_window());
        }
        div()
            .relative()
            .size_full()
            .children(content)
            .child(self.menu_host.clone())
            .into_any_element()
    }
}

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
        self.dialogs.close_prompt(cx);
        let title = SharedString::from(self.dialogs.title().to_owned());
        let panel_size = self.dialogs.panel_size();
        let owner = cx.entity();
        let focus = self.dialogs.focus().clone();
        let generation = self.dialogs.generation();
        let content = self.content_bounds.get();
        let positioned = self.main_window.update(cx, |_, window, cx| {
            // Use live client bounds, not restored bounds or the active prompt.
            let client = window.bounds();
            let panel = if content.size.width > px(0.) && content.size.height > px(0.) {
                Bounds::new(client.origin + content.origin, content.size)
            } else {
                client
            };
            let display = window.display(cx);
            let screen = display
                .as_ref()
                .map(|display| display.visible_bounds())
                .unwrap_or(client);
            let available = size(
                (screen.size.width - px(40.)).max(px(1.)),
                (screen.size.height - px(40.)).max(px(1.)),
            );
            let preferred = panel_size.size();
            let actual = size(
                preferred.width.min(available.width),
                preferred.height.min(available.height),
            );
            let mut bounds = Bounds::centered_at(panel.center(), actual);
            bounds.origin.x = bounds
                .origin
                .x
                .max(screen.origin.x + px(20.))
                .min(screen.origin.x + screen.size.width - actual.width - px(20.));
            bounds.origin.y = bounds
                .origin
                .y
                .max(screen.origin.y + px(20.))
                .min(screen.origin.y + screen.size.height - actual.height - px(20.));
            (bounds, display.map(|d| d.id()))
        });
        let (bounds, display_id) = match positioned {
            Ok(position) => position,
            Err(error) => {
                self.prompt_failed(error.context("Cannot measure dialog owner"), cx);
                return;
            }
        };
        OPENING_PROMPT.set(true);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                display_id,
                titlebar: Some(TitlebarOptions {
                    title: Some(title),
                    appears_transparent: true,
                    ..TitlebarOptions::default()
                }),
                kind: WindowKind::Normal,
                is_movable: true,
                is_resizable: true,
                is_minimizable: true,
                focus: true,
                inactive_frame_interval: None,
                window_min_size: Some(size(
                    panel_size.minimum().width.min(bounds.size.width),
                    panel_size.minimum().height.min(bounds.size.height),
                )),
                ..WindowOptions::default()
            },
            move |window, cx| {
                let prompt = cx.new(|cx| PromptWindow::new(owner, focus, generation, window, cx));
                cx.new(|cx| Root::new(prompt, window, cx))
            },
        );
        OPENING_PROMPT.set(false);
        let handle = match opened {
            Ok(handle) => handle,
            Err(error) => {
                self.prompt_failed(error.context("Cannot create native dialog"), cx);
                return;
            }
        };
        let prompt = handle.into();
        self.dialogs.set_prompt(prompt);
        self.bind_dialog(prompt, cx);
    }

    fn prompt_failed(&mut self, error: anyhow::Error, cx: &mut Context<Self>) {
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
        let body = {
            h_flex()
                .flex_1()
                .min_h_0()
                .pb(px(12.))
                .pr(px(12.))
                .child(
                    v_flex()
                        .w(px(100.))
                        .h_full()
                        .flex_shrink_0()
                        .items_center()
                        .pt(px(28.))
                        .gap(px(12.))
                        .child(img("brand/logo.png").size(px(56.)))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(crate::theme::palette(cx).muted))
                                .child("7zplus"),
                        ),
                )
                .child(panel_surface(cx).h_full().child(body))
                .into_any_element()
        };
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
