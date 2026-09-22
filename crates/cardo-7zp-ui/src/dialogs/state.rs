use crate::*;

pub(crate) enum Modal {
    Error(super::ErrorDialog),
    Progress,
    ExtractionResult(String),
    Comment(Entity<TextareaState>),
    Create(Entity<CreateForm>),
    Extract {
        folder: Entity<InputState>,
        selected: bool,
        destination: usize,
        open_after: bool,
    },
    Password {
        input: Entity<InputState>,
        request: Request,
    },
    Info(Vec<(String, String)>),
    Report(String),
    Rename {
        source: String,
        input: Entity<InputState>,
    },
    Update(cardo_7zp_requests::update::Status),
    Help,
    ConfirmRun {
        entry: String,
    },
    Conflict {
        catalog: Catalog,
        selected: Vec<String>,
        parent: PathBuf,
        folder: String,
        open_after: bool,
        password: String,
        conflicts: Vec<cardo_7zp_requests::filesystem::NameConflict>,
        index: usize,
        decisions: Vec<(String, Overwrite)>,
        repeat: bool,
    },
}

pub(crate) struct DialogState {
    active: Option<Modal>,
    title: String,
    input_subscription: Option<Subscription>,
    focus: FocusHandle,
    was_active: bool,
    previous_focus: Option<FocusHandle>,
    pending_password: Option<Request>,
    pending_comment: Option<String>,
    prompt: Option<gpui_kit::AnyWindowHandle>,
    prompt_generation: u64,
    pub(crate) pending_create: Option<Vec<PathBuf>>,
    pub(crate) pending_name: Option<String>,
    pub(crate) pending_email: bool,
    open_failed: bool,
}

impl DialogState {
    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            active: None,
            title: String::new(),
            input_subscription: None,
            focus: cx.focus_handle(),
            was_active: false,
            previous_focus: None,
            pending_password: None,
            pending_comment: None,
            prompt: None,
            prompt_generation: 0,
            pending_create: None,
            pending_name: None,
            pending_email: false,
            open_failed: false,
        }
    }

    pub(crate) fn current(&self) -> Option<&Modal> {
        self.active.as_ref()
    }
    pub(crate) fn current_mut(&mut self) -> Option<&mut Modal> {
        self.active.as_mut()
    }
    pub(crate) fn is_open(&self) -> bool {
        self.active.is_some()
    }
    pub(crate) fn title(&self) -> &str {
        &self.title
    }
    pub(crate) fn focus(&self) -> &FocusHandle {
        &self.focus
    }
    pub(crate) fn awaiting_password(&self) -> bool {
        self.pending_password.is_some()
    }
    pub(crate) fn set_title(&mut self, title: &str) {
        self.title = title.to_owned();
    }

    pub(crate) fn show(&mut self, title: impl Into<String>, modal: Modal) {
        self.title = title.into();
        self.active = Some(modal);
    }

    pub(crate) fn replace(&mut self, modal: Modal) {
        self.active = Some(modal);
    }

    pub(crate) fn observe_input(&mut self, subscription: Subscription) {
        self.input_subscription = Some(subscription);
    }

    pub(crate) fn has_prompt(&self) -> bool {
        self.prompt.is_some()
    }

    pub(crate) fn prompt_handle(&self) -> Option<gpui_kit::AnyWindowHandle> {
        self.prompt
    }

    pub(crate) fn generation(&self) -> u64 {
        self.prompt_generation
    }

    pub(crate) fn bump_generation(&mut self) {
        self.prompt_generation = self.prompt_generation.wrapping_add(1);
    }

    pub(crate) fn open_failed(&self) -> bool {
        self.open_failed
    }

    pub(crate) fn set_open_failed(&mut self, failed: bool) {
        self.open_failed = failed;
    }

    pub(crate) fn set_prompt(&mut self, window: gpui_kit::AnyWindowHandle) {
        self.open_failed = false;
        self.prompt = Some(window);
    }

    pub(crate) fn detach_prompt(&mut self) {
        self.prompt = None;
    }

    pub(crate) fn close_prompt(&mut self, cx: &mut App) {
        if let Some(window) = self.prompt.take() {
            let _ = window.update(cx, |_, window, _| window.remove_window());
        }
    }

    pub(crate) fn take(&mut self) -> Option<Modal> {
        self.input_subscription = None;
        self.pending_create = None;
        self.pending_name = None;
        self.pending_email = false;
        self.open_failed = false;
        self.active.take()
    }

    pub(crate) fn request_password(&mut self, request: Request) {
        self.pending_password = Some(request);
    }

    pub(crate) fn request_comment(&mut self, text: String) {
        self.pending_comment = Some(text);
    }

    pub(crate) fn sync(&mut self, window: &mut Window, cx: &mut Context<Workspace>) {
        if let Some(text) = self.pending_comment.take() {
            let input = cx.new(|cx| TextareaState::new(window, cx).default_value(text));
            self.show(tr("archive-comment"), Modal::Comment(input));
        }
        if let Some(request) = self.pending_password.take() {
            let input = cx.new(|cx| InputState::new(window, cx).masked(true));
            self.observe_input(cx.observe(&input, |_, _, cx| cx.notify()));
            self.show(tr("password-input"), Modal::Password { input, request });
        }
        if self.active.is_some() && !self.was_active && self.prompt.is_none() && self.open_failed {
            self.previous_focus = window.focused(cx);
            self.focus.focus(window, cx);
            if let Some(Modal::Password { input, .. } | Modal::Rename { input, .. }) = &self.active
            {
                input.update(cx, |input, cx| input.focus(window, cx));
            }
        } else if self.active.is_none()
            && self.was_active
            && let Some(focus) = self.previous_focus.take()
        {
            focus.focus(window, cx);
        }
        self.was_active = self.active.is_some();
    }
}
