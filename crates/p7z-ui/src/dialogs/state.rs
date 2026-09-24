use crate::*;

pub(crate) enum Modal {
    Error(super::ErrorDialog),
    Progress,
    ExtractionResult(String),
    Completion {
        label: String,
        path: PathBuf,
        notice: Option<String>,
    },
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
        error: Option<String>,
    },
    ConfirmRun {
        entry: String,
    },
    ConfirmDelete {
        request: Request,
        count: usize,
        password: String,
    },
    Conflict {
        plan: p7z_requests::extraction::ExtractionPlan,
        password: String,
        index: usize,
        decisions: Vec<(String, Overwrite)>,
        repeat: bool,
    },
}

pub(crate) struct DialogState {
    pub(super) host: cardo_ui::dialog::DialogHost,
    active: Option<Modal>,
    title: String,
    input_subscription: Option<Subscription>,
    input_events: Option<Subscription>,
    focus: FocusHandle,
    pending_password: Option<Request>,
    pending_comment: Option<String>,
    pub(crate) pending_create: Option<Vec<PathBuf>>,
    pub(crate) pending_name: Option<String>,
    pub(crate) pending_email: bool,
}

impl DialogState {
    pub(crate) fn panel_size(&self) -> PanelSize {
        if self.pending_create.is_some() {
            return PanelSize::Form;
        }
        match self.active.as_ref() {
            Some(Modal::Create(_)) => PanelSize::Form,
            Some(Modal::Extract { .. }) => PanelSize::Extract,
            Some(Modal::Conflict { .. }) => PanelSize::Comparison,
            Some(Modal::Report(_)) => PanelSize::Wide,
            Some(Modal::Info(fields)) => PanelSize::Properties(fields.len()),
            Some(
                Modal::Password { .. }
                | Modal::Rename { .. }
                | Modal::ConfirmRun { .. }
                | Modal::ConfirmDelete { .. },
            ) => PanelSize::Compact,
            Some(Modal::Completion { .. }) => PanelSize::Compact,
            _ => PanelSize::Standard,
        }
    }

    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            host: Default::default(),
            active: None,
            title: String::new(),
            input_subscription: None,
            input_events: None,
            focus: cx.focus_handle(),
            pending_password: None,
            pending_comment: None,
            pending_create: None,
            pending_name: None,
            pending_email: false,
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

    pub(crate) fn watch_input(&mut self, input: &Entity<InputState>, cx: &mut Context<Workspace>) {
        self.input_subscription = Some(cx.observe(input, |_, _, cx| cx.notify()));
        self.input_events = Some(cx.subscribe(input, |workspace, _, event, cx| match event {
            InputEvent::Change => {
                if let Some(Modal::Rename { error, .. }) = workspace.dialogs.current_mut() {
                    *error = None;
                }
                cx.notify();
            }
            InputEvent::PressEnter {
                secondary: false,
                shift: false,
            } => match workspace.dialogs.current() {
                Some(Modal::Password { .. }) => workspace.submit_password(cx),
                Some(Modal::Rename { .. }) => workspace.submit_rename(cx),
                Some(Modal::Extract { .. }) if !workspace.destinations.is_empty() => {
                    workspace.submit_extract(cx)
                }
                _ => {}
            },
            _ => {}
        }));
    }

    pub(crate) fn has_prompt(&self) -> bool { self.host.handle().is_some() }
    pub(crate) fn prompt_handle(&self) -> Option<AnyWindowHandle> { self.host.handle() }
    pub(crate) fn bump_generation(&mut self) { self.host.invalidate(); }
    pub(crate) fn close_prompt(&mut self, cx: &mut App) { self.host.close(cardo_ui::dialog::CloseReason::Cancel, cx); }

    pub(crate) fn take(&mut self) -> Option<Modal> {
        self.input_subscription = None;
        self.input_events = None;
        self.pending_create = None;
        self.pending_name = None;
        self.pending_email = false;
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
            self.watch_input(&input, cx);
            self.show(tr("password-input"), Modal::Password { input, request });
        }
        self.host.sync_focus(self.active.is_some(), window, cx);
    }
}
