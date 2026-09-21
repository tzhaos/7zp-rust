mod controller;
mod view;

use super::*;
use gpui_kit::{
    component::{
        Disableable, checkbox::Checkbox, h_flex, menu::DropdownMenu, switch::Switch, v_flex,
    },
    prelude::FluentBuilder,
};
use sevenzip_core::settings::Preferences;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tab {
    General,
    Advanced,
    Appearance,
    Associations,
}

impl Tab {
    pub(super) fn order(self) -> u8 {
        match self {
            Self::General => 1,
            Self::Appearance => 2,
            Self::Associations => 3,
            Self::Advanced => 4,
        }
    }

    pub(super) fn label(self) -> &'static str {
        tr(match self {
            Self::General => "settings-general",
            Self::Advanced => "settings-advanced",
            Self::Appearance => "settings-appearance",
            Self::Associations => "settings-associations",
        })
    }
}

#[derive(Clone, Copy)]
enum Toggle {
    History,
    OpenAfter,
    CloseAfter,
    Priority,
    Updates,
    Shell,
}

pub(super) struct PreferencesForm {
    owner: WeakEntity<Workspace>,
    value: Preferences,
    tab: Tab,
    theme: crate::theme::ThemeId,
    language: sevenzip_core::i18n::Language,
    temporary: Entity<InputState>,
    patterns: Entity<InputState>,
    task: Option<Task<()>>,
    error: Option<String>,
}

impl PreferencesForm {
    pub(super) fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }
    pub fn new(
        owner: WeakEntity<Workspace>,
        value: Preferences,
        tab: Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let temporary = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(value.temp_directory.clone())
                .placeholder(tr("settings-temp-default"))
        });
        let patterns =
            cx.new(|cx| InputState::new(window, cx).default_value(value.extract_all.clone()));
        Self {
            owner,
            value,
            tab,
            theme: crate::theme::current(cx),
            language: sevenzip_core::i18n::current(),
            temporary,
            patterns,
            task: None,
            error: None,
        }
    }

    fn toggle_row(
        &self,
        id: &'static str,
        toggle: Toggle,
        checked: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        settings_row(
            tr(id),
            Switch::new(id)
                .accessibility_label(tr(id))
                .color(rgb(crate::theme::palette(cx).accent))
                .checked(checked)
                .disabled(self.task.is_some())
                .on_click(cx.listener(move |this, checked: &bool, window, cx| {
                    match toggle {
                        Toggle::History => this.value.remember_recent = *checked,
                        Toggle::OpenAfter => this.value.open_after = *checked,
                        Toggle::CloseAfter => this.value.close_after_quick = *checked,
                        Toggle::Priority => this.value.low_priority = *checked,
                        Toggle::Updates => this.value.check_updates = *checked,
                        Toggle::Shell => this.value.shell_menu = *checked,
                    }
                    this.save(window, cx);
                    cx.notify();
                })),
        )
    }
}
