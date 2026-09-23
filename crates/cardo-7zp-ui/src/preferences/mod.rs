mod advanced;
mod appearance;
mod application;
mod controller;
mod integration;
mod shortcuts;
mod view;

use super::*;
use cardo_7zp_core::settings::Preferences;
use gpui_kit::{
    component::{Disableable, h_flex, v_flex},
    prelude::FluentBuilder,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tab {
    Application,
    Advanced,
    Integration,
    Updates,
}

impl Tab {
    pub(super) fn order(self) -> u8 {
        match self {
            Self::Application => 1,
            Self::Integration => 2,
            Self::Advanced => 3,
            Self::Updates => 4,
        }
    }

    pub(super) fn label(self) -> &'static str {
        tr(match self {
            Self::Application => "settings-general",
            Self::Advanced => "settings-advanced",
            Self::Integration => "settings-integration",
            Self::Updates => "settings-update-page",
        })
    }
}

#[derive(Clone, Copy)]
enum Toggle {
    History,
    OpenAfter,
    CloseAfter,
    CloseArchive,
    Priority,
    Updates,
    Shell,
    ToolLabels,
}

impl Toggle {
    fn title(self) -> &'static str {
        tr(match self {
            Self::History => "settings-history-title",
            Self::OpenAfter => "settings-open-after-title",
            Self::CloseAfter => "settings-close-after-title",
            Self::CloseArchive => "settings-close-archive-title",
            Self::Priority => "settings-priority-title",
            Self::Updates => "settings-updates-title",
            Self::Shell => "settings-shell-title",
            Self::ToolLabels => "settings-tool-labels-title",
        })
    }
}

pub(super) struct PreferencesForm {
    owner: WeakEntity<Workspace>,
    value: Preferences,
    tab: Tab,
    theme: crate::theme::ThemeId,
    language: cardo_7zp_core::i18n::Language,
    font_family: Entity<InputState>,
    font_size: f32,
    association_popup: Entity<gpui_kit::base::PopoverState>,
    association_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    association_search: Entity<InputState>,
    temporary: Entity<InputState>,
    system_temporary: SharedString,
    patterns: Entity<TextareaState>,
    task: Option<Task<()>>,
    saving: bool,
    shortcut_expanded: bool,
    shortcut_search: Entity<InputState>,
    shortcut_recording: Option<cardo_7zp_core::settings::shortcuts::ShortcutAction>,
    shortcut_focus: FocusHandle,
    shortcut_error: Option<String>,
    error: Option<String>,
    font_error: Option<String>,
    _watch: Vec<Subscription>,
}

impl PreferencesForm {
    pub(super) fn set_tab(&mut self, tab: Tab, window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_associations(window, cx);
        self.shortcut_recording = None;
        self.tab = tab;
        cx.notify();
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    pub fn controls_disabled(&self) -> bool {
        self.is_busy() && !self.saving
    }

    pub fn is_saving(&self) -> bool {
        self.saving
    }
    pub fn new(
        owner: WeakEntity<Workspace>,
        value: Preferences,
        appearance: cardo_7zp_core::settings::Appearance,
        tab: Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let system_temporary: SharedString = std::env::temp_dir().display().to_string().into();
        let temporary = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(value.temp_directory.clone())
                .placeholder(system_temporary.clone())
        });
        let patterns = cx.new(|cx| {
            TextareaState::new(window, cx).default_value(value.extract_all.replace(';', "\n"))
        });
        let font_family = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(appearance.font_family.clone())
                .placeholder(cardo_7zp_core::settings::Appearance::default().font_family)
        });
        let mut watch: Vec<Subscription> = [&font_family]
            .into_iter()
            .map(|input| {
                cx.subscribe_in(input, window, |this, _, event, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.save(window, cx);
                    }
                })
            })
            .collect();
        let association_popup = cx.new(|cx| gpui_kit::base::PopoverState::new(false, cx));
        let shortcut_search =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr("shortcuts-search")));
        watch.push(cx.subscribe(&shortcut_search, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        }));
        let association_search =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr("association-search")));
        watch.push(cx.subscribe(&association_search, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        }));
        watch.push(cx.observe(&association_popup, |_, _, cx| cx.notify()));
        watch.push(cx.observe_window_bounds(window, |this, window, cx| {
            this.dismiss_associations(window, cx);
        }));
        watch.push(cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.shortcut_recording = None;
                cx.notify();
                this.dismiss_associations(window, cx);
            }
        }));
        Self {
            owner,
            value,
            tab,
            theme: crate::theme::current(cx),
            language: cardo_7zp_core::i18n::current(),
            font_family,
            font_size: cardo_7zp_core::settings::Appearance::clamp_font_size(appearance.font_size),
            association_popup,
            association_bounds: std::rc::Rc::new(std::cell::Cell::new(Bounds::default())),
            association_search,
            temporary,
            system_temporary,
            patterns,
            task: None,
            saving: false,
            shortcut_expanded: false,
            shortcut_search,
            shortcut_recording: None,
            shortcut_focus: cx.focus_handle(),
            shortcut_error: None,
            error: None,
            font_error: None,
            _watch: watch,
        }
    }

    fn toggle_row(
        &self,
        id: &'static str,
        toggle: Toggle,
        checked: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        settings_detail(
            toggle.title(),
            tr(id),
            SettingsSwitch::new(id, toggle.title(), checked, self.controls_disabled()).on_click(
                cx.listener(move |this, checked: &bool, window, cx| {
                    if this.is_busy() {
                        return;
                    }
                    match toggle {
                        Toggle::History => this.value.remember_recent = *checked,
                        Toggle::OpenAfter => this.value.open_after = *checked,
                        Toggle::CloseAfter => this.value.close_after_quick = *checked,
                        Toggle::CloseArchive => this.value.close_archive_after_quick = *checked,
                        Toggle::Priority => this.value.low_priority = *checked,
                        Toggle::Updates => this.value.check_updates = *checked,
                        Toggle::Shell => this.value.shell_menu = *checked,
                        Toggle::ToolLabels => this.value.hide_tool_labels = *checked,
                    }
                    this.save(window, cx);
                    cx.notify();
                }),
            ),
            cx,
        )
    }
}
