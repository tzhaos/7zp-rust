mod controller;
mod view;

use super::*;
use gpui_kit::{
    component::{
        Disableable, checkbox::Checkbox, h_flex, menu::DropdownMenu, switch::Switch, v_flex,
    },
    prelude::FluentBuilder,
};
use cardo_7zp_core::settings::Preferences;

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
    CloseArchive,
    Priority,
    Updates,
    Shell,
    ToolLabels,
}

pub(super) struct PreferencesForm {
    owner: WeakEntity<Workspace>,
    value: Preferences,
    tab: Tab,
    theme: crate::theme::ThemeId,
    language: cardo_7zp_core::i18n::Language,
    font_family: Entity<InputState>,
    font_size: f32,
    installed_fonts: Vec<String>,
    temporary: Entity<InputState>,
    patterns: Entity<InputState>,
    task: Option<Task<()>>,
    error: Option<String>,
    _watch: Vec<Subscription>,
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
        appearance: cardo_7zp_core::settings::Appearance,
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
        let font_family = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(appearance.font_family.clone())
                .placeholder(cardo_7zp_core::settings::Appearance::default().font_family)
        });
        let watch = vec![
            cx.subscribe_in(&font_family, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.save(window, cx);
                }
            }),
        ];
        Self {
            owner,
            value,
            tab,
            theme: crate::theme::current(cx),
            language: cardo_7zp_core::i18n::current(),
            font_family,
            font_size: appearance.font_size,
            installed_fonts: cx.text_system().all_font_names(),
            temporary,
            patterns,
            task: None,
            error: None,
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
                        Toggle::CloseArchive => this.value.close_archive_after_quick = *checked,
                        Toggle::Priority => this.value.low_priority = *checked,
                        Toggle::Updates => this.value.check_updates = *checked,
                        Toggle::Shell => this.value.shell_menu = *checked,
                        Toggle::ToolLabels => this.value.hide_tool_labels = *checked,
                    }
                    this.save(window, cx);
                    cx.notify();
                })),
            cx,
        )
    }
}
