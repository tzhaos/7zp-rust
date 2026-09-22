use super::*;
use gpui_kit::component::{Root, v_flex};
use std::cell::Cell;

thread_local! {
    static OPENING_PROMPT: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct PromptWindow {
    owner: WeakEntity<Workspace>,
    _watch: Subscription,
}

impl PromptWindow {
    fn new(
        owner: Entity<Workspace>,
        generation: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let weak = owner.downgrade();
        window.on_window_should_close(cx, {
            let weak = weak.clone();
            move |_, cx| {
                weak.update(cx, |workspace, cx| {
                    if workspace.dialogs.generation() != generation {
                        return true;
                    }
                    if matches!(workspace.dialogs.current(), Some(Modal::Progress)) {
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
        content.unwrap_or_else(|| div().size_full().into_any_element())
    }
}

impl Workspace {
    pub(crate) fn schedule_prompt(&mut self, cx: &mut Context<Self>) {
        if self.dialogs.has_prompt() || self.dialogs.open_failed() {
            return;
        }
        if !self.dialogs.is_open() && self.dialogs.pending_create.is_none() {
            return;
        }
        let owner = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = owner.update(cx, |this, cx| {
                if this.dialogs.has_prompt() || this.dialogs.open_failed() {
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
        let (width, height) = dialog_size(self.dialogs.current(), self.dialogs.pending_create.is_some());
        let owner = cx.entity();
        let generation = self.dialogs.generation();
        let bounds = WindowBounds::centered(size(px(width), px(height)), cx);
        OPENING_PROMPT.set(true);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(bounds),
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
                window_min_size: Some(size(px(420.), px(320.))),
                ..WindowOptions::default()
            },
            move |window, cx| {
                let prompt = cx.new(|cx| PromptWindow::new(owner, generation, window, cx));
                cx.new(|cx| Root::new(prompt, window, cx))
            },
        );
        OPENING_PROMPT.set(false);
        let Ok(handle) = opened else {
            self.dialogs.set_open_failed(true);
            self.fallback_create(cx);
            return;
        };
        let prompt = handle.into();
        self.dialogs.set_prompt(prompt);
        self.bind_dialog(prompt, cx);
    }

    fn fallback_create(&mut self, cx: &mut Context<Self>) {
        let Some(files) = self.dialogs.pending_create.take() else {
            return;
        };
        let email = self.dialogs.pending_email;
        let name = self.dialogs.pending_name.take();
        self.dialogs.pending_email = false;
        let Some(window) = cx.active_window() else {
            return;
        };
        let owner = cx.entity().downgrade();
        let Ok(form) = window.update(cx, move |_, window, cx| {
            cx.new(|cx| {
                let mut form = CreateForm::new(owner, files, window, cx);
                form.email = email;
                if let Some(name) = name {
                    form.suggest_name(name, window, cx);
                }
                form
            })
        }) else {
            return;
        };
        let title = self.dialogs.title().to_owned();
        self.dialogs.show(title, Modal::Create(form));
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
            if let Ok(form) = form {
                let title = self.dialogs.title().to_owned();
                self.dialogs.show(title, Modal::Create(form));
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
        let Ok(bound) = created else {
            return;
        };
        match bound {
            Bound::Text(input) => {
                self.dialogs
                    .observe_input(cx.observe(&input, |_, _, cx| cx.notify()));
                match self.dialogs.current_mut() {
                    Some(Modal::Extract { folder, .. } | Modal::Password { input: folder, .. } | Modal::Rename { input: folder, .. }) => {
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

    fn prompt_content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let body = self.dialog_body(window, cx)?;
        let dismissible = !matches!(self.dialogs.current(), Some(Modal::Progress));
        let p = crate::theme::palette(cx);
        let appearance = crate::theme::appearance(cx);
        Some(
            v_flex()
                .size_full()
                .bg(rgb(p.surface))
                .text_color(rgb(p.text))
                .font(crate::theme::interface_font(cx))
                .text_size(px(appearance.font_size))
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                    if event.keystroke.key == "escape" {
                        this.close_modal(cx);
                        cx.stop_propagation();
                    }
                }))
                .child(self.dialog_title("close-prompt", true, dismissible, cx))
                .child(div().flex_1().min_h_0().overflow_hidden().child(body))
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

fn dialog_size(modal: Option<&Modal>, creating: bool) -> (f32, f32) {
    if creating {
        return (560., 680.);
    }
    match modal {
        Some(Modal::Conflict { .. }) => (680., 520.),
        Some(Modal::Extract { .. } | Modal::Create(_)) => (560., 680.),
        Some(Modal::Report(_) | Modal::Help) => (640., 520.),
        Some(Modal::Error(_)) => (520., 440.),
        Some(Modal::Progress) => (520., 360.),
        Some(Modal::Comment(_)) => (520., 420.),
        _ => (480., 320.),
    }
}
