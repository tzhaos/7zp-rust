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
use cardo_7zp_core::i18n::{tf, tr};
use cardo_7zp_engine::{Cancellation, Catalog, CreateOptions, Edit, Entry, Overwrite};
use cardo_7zp_requests::{Outcome, Request, browser::Browser};
use cardo_ui::menu::{Menu, MenuItem, MenuTrigger};
use components::*;
use dialogs::{CreateForm, Modal};
use format::{file_kind, size_text};
pub(crate) use gpui_kit::component::scroll::{ScrollableElement, Scrollbar};
use gpui_kit::{
    component::input::{InputEvent, InputState, TextareaState},
    *,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

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
    menu_host: Entity<cardo_ui::menu::MenuHost>,
    main_window: AnyWindowHandle,
    content_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    active_tab_center: std::rc::Rc<std::cell::Cell<Pixels>>,
    preferences: cardo_7zp_core::settings::Preferences,
    appearance: cardo_7zp_core::settings::Appearance,
    tasks: task::TaskState,
    browser: Browser,
    address: Entity<InputState>,
    address_dirty: bool,
    history: Vec<cardo_7zp_core::settings::recent::Entry>,
    address_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    history_popup: Entity<gpui_kit::base::PopoverState>,
    history_cursor: usize,
    _history_popup_subscription: Subscription,
    recent_task: Option<Task<()>>,
    search: Entity<InputState>,
    dialogs: dialogs::DialogState,
    settings_form: Option<Entity<preferences::PreferencesForm>>,
    settings_page: Option<views::SettingsPage>,
    update_status: Option<cardo_7zp_requests::update::Status>,
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
    extract_follow: std::collections::VecDeque<(Request, String)>,
    pending_close_archive: bool,
    theme_save: Option<Task<()>>,
    update_task: Option<Task<()>>,
    language_task: Option<Task<()>>,
    launches: std::collections::VecDeque<Vec<String>>,
    after_open: Option<cardo_7zp_platform::ExplorerAction>,
    dragging: bool,
    hint_anchor: Option<Point<Pixels>>,
    message_since: Option<(String, std::time::Instant)>,
    _search_subscription: Subscription,
    _address_subscription: Subscription,
    _activation_subscription: Subscription,
}

impl Workspace {
    pub fn new(
        startup: cardo_7zp_core::settings::StartupSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let address = cx.new(|cx| InputState::new(window, cx).placeholder(tr("browser-address")));
        let address_subscription =
            cx.subscribe_in(&address, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.dismiss_history(window, cx);
                    this.open_address(window, cx);
                } else if matches!(event, InputEvent::Change) {
                    this.dismiss_history(window, cx);
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
        let history_popup = cx.new(|cx| gpui_kit::base::PopoverState::new(false, cx));
        let history_popup_subscription = cx.observe(&history_popup, |_, _, cx| cx.notify());
        if let Some(path) = dirs::download_dir() {
            destinations.push((DestinationKind::Downloads, path));
        }
        if let Some(path) = dirs::desktop_dir() {
            destinations.push((DestinationKind::Desktop, path));
        }
        let saved = startup.destination;
        let destination = destinations
            .iter()
            .position(|(_, path)| saved.as_deref() == Some(path.to_string_lossy().as_ref()))
            .unwrap_or(0);
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let weak = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            let allow = weak
                .read_with(cx, |this, cx| {
                    !this.tasks.is_busy() && !this.settings_busy(cx)
                })
                .unwrap_or(true);
            if allow {
                let _ = weak.update(cx, |this, cx| this.dialogs.close_prompt(cx));
            }
            allow
        });
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            this.dismiss_history(window, cx);
            if window.is_window_active() {
                this.reload_history(cx);
            }
        });
        let mut workspace = Self {
            menu_host: cardo_ui::menu::MenuHost::install(window, cx, || tr("menu-more").into()),
            main_window: window.window_handle(),
            content_bounds: std::rc::Rc::new(std::cell::Cell::new(Bounds::default())),
            active_tab_center: std::rc::Rc::new(std::cell::Cell::new(px(0.))),
            preferences: cardo_7zp_core::settings::Preferences::default(),
            appearance: startup.appearance,
            tasks: task::TaskState::default(),
            browser: Browser::default(),
            address,
            address_dirty: false,
            history: Vec::new(),
            address_bounds: std::rc::Rc::new(std::cell::Cell::new(Bounds::default())),
            history_popup,
            history_cursor: 0,
            _history_popup_subscription: history_popup_subscription,
            recent_task: None,
            search,
            dialogs: dialogs::DialogState::new(cx),
            settings_form: None,
            settings_page: None,
            update_status: None,
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
            extract_follow: std::collections::VecDeque::new(),
            pending_close_archive: false,
            theme_save: None,
            update_task: None,
            language_task: None,
            launches: std::collections::VecDeque::new(),
            after_open: None,
            dragging: false,
            hint_anchor: None,
            message_since: None,
            _search_subscription: subscription,
            _address_subscription: address_subscription,
            _activation_subscription: activation_subscription,
        };
        let check_updates = startup.preferences.check_updates;
        workspace.apply_preferences(startup.preferences, cx);
        workspace.reload_history(cx);
        if check_updates {
            workspace.check_updates_quietly(cx);
        }
        workspace
    }

    pub(crate) fn allow_hint(&self, enabled: bool) -> bool {
        enabled && self.hint_anchor.is_none() && !self.dialogs.is_open() && !self.dragging
    }
}

impl Workspace {
    fn sync_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if (self.tasks.is_busy() || self.dialogs.is_open()) && self.history_popup.read(cx).is_open()
        {
            self.dismiss_history(window, cx);
        }
        if self.pending_close_archive {
            self.pending_close_archive = false;
            if let Some(parent) = self
                .browser
                .view()
                .catalog
                .as_ref()
                .and_then(|catalog| catalog.path.parent().map(Path::to_owned))
            {
                self.visit(browser::Location::Directory(parent), window, cx);
            }
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
        self.dialogs.sync(window, cx);
        if !self.dialogs.is_open()
            && !self.tasks.is_busy()
            && let Some((label, path)) = self.completion.take()
        {
            let notice = self.message.take();
            self.show_dialog(
                tr("operation-result-title"),
                Modal::Completion {
                    label,
                    path,
                    notice,
                },
                cx,
            );
        }
        if (self.dialogs.is_open() || self.dialogs.pending_create.is_some())
            && !self.dialogs.has_prompt()
        {
            self.schedule_prompt(cx);
        }
    }
}
