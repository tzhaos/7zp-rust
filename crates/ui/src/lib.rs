mod actions;
pub mod assets;
mod browser;
mod commands;
mod components;
mod dialogs;
mod format;
mod menus;
mod preferences;
mod progress;
mod recent;
mod system;
mod task;
pub mod theme;
mod views;

use anyhow::{Result, bail};
use components::*;
use dialogs::{CreateForm, Modal};
use format::{file_kind, size_text};
use gpui_kit::{
    component::{
        input::{InputEvent, InputState, TextareaState},
        menu::{PopupMenu, PopupMenuItem},
    },
    *,
};
use sevenzip_application::{Outcome, Request, browser::Browser};
use sevenzip_archive::{Cancellation, Catalog, CreateOptions, Edit, Entry, Overwrite};
use sevenzip_core::i18n::{tf, tr};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

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
    preferences: sevenzip_core::settings::Preferences,
    preferences_task: Option<Task<()>>,
    tasks: task::TaskState,
    browser: Browser,
    address: Entity<InputState>,
    address_dirty: bool,
    recent_folders: Vec<PathBuf>,
    recent_archives: Vec<PathBuf>,
    recent_task: Option<Task<()>>,
    search: Entity<InputState>,
    tools_expanded: bool,
    dialogs: dialogs::DialogState,
    page: Page,
    page_direction: f32,
    page_revision: usize,
    settings_form: Option<Entity<preferences::PreferencesForm>>,
    settings_subscription: Option<Subscription>,
    destinations: Vec<(DestinationKind, PathBuf)>,
    destination: usize,
    completion: Option<(String, PathBuf)>,
    message: Option<String>,
    focus: FocusHandle,
    scroll: UniformListScrollHandle,
    clear_search: bool,
    system_task: Option<Task<()>>,
    shell_read_task: Option<Task<()>>,
    pending_run: Option<(tempfile::TempDir, PathBuf)>,
    theme_save: Option<Task<()>>,
    update_task: Option<Task<()>>,
    language_task: Option<Task<()>>,
    launches: std::collections::VecDeque<Vec<String>>,
    after_open: Option<sevenzip_platform::ShellAction>,
    context_menu: Option<Entity<PopupMenu>>,
    context_menu_position: Point<Pixels>,
    menu_dismiss: Option<Subscription>,
    dragging: bool,
    message_since: Option<(String, std::time::Instant)>,
    _search_subscription: Subscription,
    _address_subscription: Subscription,
    _activation_subscription: Subscription,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
                this.browser.clear_selection(true);
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
        let saved = sevenzip_core::settings::load_destination();
        let destination = destinations
            .iter()
            .position(|(_, path)| saved.as_deref() == Some(path.to_string_lossy().as_ref()))
            .unwrap_or(0);
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let weak = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            weak.read_with(cx, |this, cx| {
                !this.tasks.is_busy() && !this.settings_busy(cx)
            })
            .unwrap_or(true)
        });
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                this.reload_history(cx);
            }
        });
        let mut workspace = Self {
            preferences: sevenzip_core::settings::Preferences::default(),
            preferences_task: None,
            tasks: task::TaskState::default(),
            browser: Browser::default(),
            address,
            address_dirty: false,
            recent_folders: Vec::new(),
            recent_archives: Vec::new(),
            recent_task: None,
            search,
            tools_expanded: true,
            dialogs: dialogs::DialogState::new(cx),
            page: Page::Files,
            page_direction: 1.0,
            page_revision: 0,
            settings_form: None,
            settings_subscription: None,
            destinations,
            destination,
            completion: None,
            message: None,
            focus,
            scroll: UniformListScrollHandle::default(),
            clear_search: false,
            system_task: None,
            shell_read_task: None,
            pending_run: None,
            theme_save: None,
            update_task: None,
            language_task: None,
            launches: std::collections::VecDeque::new(),
            after_open: None,
            context_menu: None,
            context_menu_position: point(px(32.), px(196.)),
            menu_dismiss: None,
            dragging: false,
            message_since: None,
            _search_subscription: subscription,
            _address_subscription: address_subscription,
            _activation_subscription: activation_subscription,
        };
        workspace.reload_history(cx);
        workspace.load_preferences(cx);
        workspace
    }

    pub(crate) fn close_modal(&mut self, cx: &mut Context<Self>) {
        if matches!(self.dialogs.current(), Some(Modal::Progress)) {
            return;
        }
        if matches!(self.dialogs.current(), Some(Modal::Password { .. })) {
            self.tasks.set_close_after(false);
            self.after_open = None;
            self.browser.cancel_navigation();
        }
        if matches!(self.dialogs.current(), Some(Modal::ConfirmRun { .. })) {
            self.pending_run = None;
        }
        self.update_task = None;
        self.dialogs.take();
        cx.notify();
    }
}

impl Workspace {
    fn sync_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        self.dialogs.sync(window, cx);
    }
}
