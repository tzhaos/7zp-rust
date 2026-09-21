mod actions;
pub(crate) mod assets;
mod browser;
mod commands;
mod components;
mod dialogs;
mod format;
mod manage;
mod menu;
mod preferences;
mod progress;
mod recent;
mod system;
pub(crate) mod theme;
mod views;

use crate::{
    application::{Outcome, Request},
    archive::{Cancellation, Catalog, CreateOptions, Edit, Engine, Entry, Overwrite},
    i18n::{tf, tr},
};
use anyhow::{Result, bail};
use components::*;
use dialogs::CreateForm;
use format::{file_kind, size_text};
use gpui_kit::{
    component::{
        input::{InputEvent, InputState, TextareaState},
        menu::{PopupMenu, PopupMenuItem},
    },
    *,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

enum Modal {
    Error(dialogs::ErrorDialog),
    Progress,
    ExtractionResult(String),
    Comment(Entity<TextareaState>),
    Create(Entity<CreateForm>),
    Extract {
        folder: Entity<InputState>,
        selected: bool,
        destination: usize,
        overwrite: Overwrite,
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
    Update(crate::application::update::Status),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Files,
    Settings(preferences::Tab),
}

impl Page {
    fn order(self) -> u8 {
        match self {
            Self::Files => 0,
            Self::Settings(tab) => tab.order(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DestinationKind {
    Archive,
    Custom,
    Downloads,
    Desktop,
}

impl DestinationKind {
    fn label(self) -> &'static str {
        tr(match self {
            Self::Archive => "archive-directory",
            Self::Custom => "custom-directory",
            Self::Downloads => "downloads",
            Self::Desktop => "desktop",
        })
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Archive | Self::Custom => "FolderOpen",
            Self::Downloads => "ArrowDownload",
            Self::Desktop => "Desktop",
        }
    }
}

pub struct Workspace {
    preferences: crate::settings::Preferences,
    preferences_task: Option<Task<()>>,
    close_after_task: bool,
    temporary_files: Vec<tempfile::TempDir>,
    catalog: Option<Catalog>,
    password: String,
    folder: String,
    history: Vec<browser::Location>,
    directory: Option<browser::Directory>,
    returning: bool,
    pending_folder: Option<String>,
    address: Entity<InputState>,
    address_dirty: bool,
    recent_folders: Vec<PathBuf>,
    recent_archives: Vec<PathBuf>,
    recent_task: Option<Task<()>>,
    search: Entity<InputState>,
    tools_expanded: bool,
    selected: BTreeSet<String>,
    anchor: Option<String>,
    cursor: Option<String>,
    rows: Vec<Entry>,
    sort: usize,
    descending: bool,
    modal: Option<Modal>,
    modal_title: String,
    page: Page,
    page_direction: f32,
    page_revision: usize,
    settings_form: Option<Entity<preferences::PreferencesForm>>,
    settings_subscription: Option<Subscription>,
    destinations: Vec<(DestinationKind, PathBuf)>,
    destination: usize,
    completion: Option<(String, PathBuf)>,
    message: Option<String>,
    busy: bool,
    status: String,
    cancel: Cancellation,
    extraction: Option<progress::ExtractionProgress>,
    task: Option<Task<()>>,
    focus: FocusHandle,
    scroll: UniformListScrollHandle,
    pending_password: Option<Request>,
    pending_comment: Option<String>,
    clear_search: bool,
    system_task: Option<Task<()>>,
    shell_read_task: Option<Task<()>>,
    theme_save: Option<Task<()>>,
    update_task: Option<Task<()>>,
    language_task: Option<Task<()>>,
    launches: std::collections::VecDeque<Vec<String>>,
    after_open: Option<crate::platform::ShellAction>,
    modal_input: Option<Subscription>,
    context_menu: Option<Entity<PopupMenu>>,
    context_menu_position: Point<Pixels>,
    menu_dismiss: Option<Subscription>,
    dragging: bool,
    message_since: Option<(String, std::time::Instant)>,
    modal_focus: FocusHandle,
    modal_active: bool,
    previous_focus: Option<FocusHandle>,
    _search_subscription: Subscription,
    _address_subscription: Subscription,
    _activation_subscription: Subscription,
}

impl Workspace {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let address = cx.new(|cx| InputState::new(window, cx).placeholder(tr("browser-address")));
        let address_subscription =
            cx.subscribe_in(&address, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.open_address(window, cx);
                }
            });
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(tr("search-placeholder")));
        let subscription = cx.subscribe(&search, |this, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.selected.clear();
                this.anchor = None;
                this.refresh(cx);
            }
        });
        let mut destinations = Vec::new();
        if let Some(path) = dirs::download_dir() {
            destinations.push((DestinationKind::Downloads, path));
        }
        if let Some(path) = dirs::desktop_dir() {
            destinations.push((DestinationKind::Desktop, path));
        }
        let saved = crate::settings::directory()
            .ok()
            .and_then(|root| std::fs::read_to_string(root.join("destination")).ok());
        let destination = destinations
            .iter()
            .position(|(_, path)| saved.as_deref() == Some(path.to_string_lossy().as_ref()))
            .unwrap_or(0);
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let weak = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            weak.read_with(cx, |this, cx| !this.busy && !this.settings_busy(cx))
                .unwrap_or(true)
        });
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                this.reload_history(cx);
            }
        });
        let mut workspace = Self {
            preferences: crate::settings::Preferences::default(),
            preferences_task: None,
            close_after_task: false,
            temporary_files: Vec::new(),
            catalog: None,
            password: String::new(),
            folder: String::new(),
            history: Vec::new(),
            directory: None,
            returning: false,
            pending_folder: None,
            address,
            address_dirty: false,
            recent_folders: Vec::new(),
            recent_archives: Vec::new(),
            recent_task: None,
            search,
            tools_expanded: true,
            selected: BTreeSet::new(),
            anchor: None,
            cursor: None,
            rows: Vec::new(),
            sort: 0,
            descending: false,
            modal: None,
            modal_title: String::new(),
            page: Page::Files,
            page_direction: 1.0,
            page_revision: 0,
            settings_form: None,
            settings_subscription: None,
            destinations,
            destination,
            completion: None,
            message: None,
            busy: false,
            status: String::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            extraction: None,
            task: None,
            focus,
            scroll: UniformListScrollHandle::default(),
            pending_password: None,
            pending_comment: None,
            clear_search: false,
            system_task: None,
            shell_read_task: None,
            theme_save: None,
            update_task: None,
            language_task: None,
            launches: std::collections::VecDeque::new(),
            after_open: None,
            modal_input: None,
            context_menu: None,
            context_menu_position: point(px(32.), px(196.)),
            menu_dismiss: None,
            dragging: false,
            message_since: None,
            modal_focus: cx.focus_handle(),
            modal_active: false,
            previous_focus: None,
            _search_subscription: subscription,
            _address_subscription: address_subscription,
            _activation_subscription: activation_subscription,
        };
        workspace.reload_history(cx);
        workspace.load_preferences(cx);
        workspace
    }

    pub(in crate::ui) fn close_modal(&mut self, cx: &mut Context<Self>) {
        if matches!(self.modal, Some(Modal::Progress)) {
            return;
        }
        if matches!(self.modal, Some(Modal::Password { .. })) {
            self.close_after_task = false;
            self.after_open = None;
            self.returning = false;
            self.pending_folder = None;
        }
        self.update_task = None;
        self.modal = None;
        self.modal_input = None;
        cx.notify();
    }
}

impl Workspace {
    fn sync_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.pending_comment.take() {
            self.modal_title = tr("archive-comment").into();
            self.modal =
                Some(Modal::Comment(cx.new(|cx| {
                    TextareaState::new(window, cx).default_value(text)
                })));
        }
        if self.address_dirty {
            self.address_dirty = false;
            let address = self.address_text();
            self.address
                .update(cx, |input, cx| input.set_value(address, window, cx));
        }
        if self.clear_search {
            self.clear_search = false;
            self.search
                .update(cx, |input, cx| input.set_value("", window, cx));
            self.refresh(cx);
        }
        if let Some(request) = self.pending_password.take() {
            let input = cx.new(|cx| InputState::new(window, cx).masked(true));
            self.modal_input = Some(cx.observe(&input, |_, _, cx| cx.notify()));
            self.modal = Some(Modal::Password { input, request });
            self.modal_title = tr("password-input").into();
        }
        if self.modal.is_some() && !self.modal_active {
            self.previous_focus = window.focused(cx);
            self.modal_focus.focus(window, cx);
            if let Some(Modal::Password { input, .. } | Modal::Rename { input, .. }) = &self.modal {
                input.update(cx, |input, cx| input.focus(window, cx));
            }
        } else if self.modal.is_none()
            && self.modal_active
            && let Some(focus) = self.previous_focus.take()
        {
            focus.focus(window, cx);
        }
        self.modal_active = self.modal.is_some();
    }
}
