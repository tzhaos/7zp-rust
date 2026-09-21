use super::*;
use crate::settings::Preferences;
use gpui_kit::{
    component::{
        Disableable, checkbox::Checkbox, h_flex, menu::DropdownMenu, switch::Switch, v_flex,
    },
    prelude::FluentBuilder,
};

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
    theme: crate::ui::theme::ThemeId,
    language: crate::i18n::Language,
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
            theme: crate::ui::theme::current(cx),
            language: crate::i18n::current(),
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
                .color(rgb(crate::ui::theme::palette(cx).accent))
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

    pub(super) fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_busy() {
            return;
        }
        self.value.temp_directory = self.temporary.read(cx).value().trim().to_owned();
        self.value.extract_all = self.patterns.read(cx).value().trim().to_owned();
        let value = self.value.clone();
        let owner = self.owner.clone();
        let handle = window.window_handle();
        let theme = self.theme;
        let language = self.language;
        let registration_changed = owner
            .read_with(cx, |workspace, _| {
                workspace.preferences.associations != value.associations
                    || workspace.preferences.shell_menu != value.shell_menu
            })
            .unwrap_or(false);
        let job = cx.background_executor().spawn(async move {
            if !value.temp_directory.is_empty() {
                let path = PathBuf::from(&value.temp_directory);
                if !path.is_absolute() {
                    bail!(tr("settings-temp-absolute"));
                }
                std::fs::create_dir_all(&path)?;
            }
            if registration_changed {
                crate::platform::configure(&std::env::current_exe()?, &value)?;
            }
            crate::settings::save_preferences(&value)?;
            Ok::<_, anyhow::Error>(value)
        });
        self.task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                this.task = None;
                match result {
                    Ok(value) => {
                        App::defer(cx, move |cx| {
                            let _ = handle.update(cx, |_, window, cx| {
                                let _ = owner.update(cx, |owner, cx| {
                                    owner.apply_preferences(value, cx);
                                    if theme != crate::ui::theme::current(cx) {
                                        owner.change_theme(theme, window, cx);
                                    }
                                    if language != crate::i18n::current() {
                                        owner.change_language(language, window, cx);
                                    }
                                });
                            });
                        });
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = window.window_handle();
        let job = cx
            .background_executor()
            .spawn(async { rfd::FileDialog::new().pick_folder() });
        self.task = Some(cx.spawn(async move |view, cx| {
            let path = job.await;
            let _ = handle.update(cx, |_, window, cx| {
                let _ = view.update(cx, |this, cx| {
                    this.task = None;
                    if let Some(path) = path {
                        this.temporary.update(cx, |input, cx| {
                            input.set_value(path.display().to_string(), window, cx)
                        });
                    }
                    cx.notify();
                });
            });
        }));
        cx.notify();
    }
}

impl Render for PreferencesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = crate::ui::theme::palette(cx);
        let busy = self.task.is_some();
        v_flex()
            .size_full()
            .min_h_0()
            .child(
                v_flex()
                    .id(("settings-content", self.tab.order() as usize))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(24.))
                    .py(px(20.))
                    .gap(px(24.))
                    .when(self.tab == Tab::Appearance, |el| {
                        let themes = cx.entity().downgrade();
                        let languages = cx.entity().downgrade();
                        let selected_theme = self.theme;
                        let selected_language = self.language;
                        el.child(settings_group(
                            [
                                settings_row(
                                    tr("theme-menu"),
                                    command("settings-theme", self.theme.label())
                                        .w(px(190.))
                                        .dropdown_caret(true)
                                        .disabled(busy)
                                        .dropdown_menu(move |menu, _, _| {
                                            let mut menu = menu_style(menu);
                                            for theme in crate::ui::theme::THEMES {
                                                let owner = themes.clone();
                                                menu = menu.item(
                                                    PopupMenuItem::new(theme.label())
                                                        .checked(theme == selected_theme)
                                                        .on_click(move |_, window, cx| {
                                                            let _ = owner.update(cx, |this, cx| {
                                                                this.theme = theme;
                                                                cx.notify();
                                                            });
                                                            let _ = owner.update(cx, |this, cx| {
                                                                this.save(window, cx)
                                                            });
                                                        }),
                                                );
                                            }
                                            menu
                                        }),
                                )
                                .into_any_element(),
                                settings_row(
                                    tr("language-menu"),
                                    command("settings-language", self.language.label())
                                        .w(px(190.))
                                        .dropdown_caret(true)
                                        .disabled(busy)
                                        .dropdown_menu(move |menu, _, _| {
                                            let mut menu = menu_style(menu);
                                            for language in crate::i18n::LANGUAGES {
                                                let owner = languages.clone();
                                                menu = menu.item(
                                                    PopupMenuItem::new(language.label())
                                                        .checked(language == selected_language)
                                                        .on_click(move |_, window, cx| {
                                                            let _ = owner.update(cx, |this, cx| {
                                                                this.language = language;
                                                                cx.notify();
                                                            });
                                                            let _ = owner.update(cx, |this, cx| {
                                                                this.save(window, cx)
                                                            });
                                                        }),
                                                );
                                            }
                                            menu
                                        }),
                                )
                                .into_any_element(),
                            ],
                            cx,
                        ))
                    })
                    .when(self.tab == Tab::General, |el| {
                        el.child(settings_section(
                            tr("settings-application"),
                            settings_group(
                                [
                                    self.toggle_row(
                                        "settings-updates",
                                        Toggle::Updates,
                                        self.value.check_updates,
                                        cx,
                                    )
                                    .into_any_element(),
                                    self.toggle_row(
                                        "settings-history",
                                        Toggle::History,
                                        self.value.remember_recent,
                                        cx,
                                    )
                                    .into_any_element(),
                                    self.toggle_row(
                                        "settings-shell",
                                        Toggle::Shell,
                                        self.value.shell_menu,
                                        cx,
                                    )
                                    .into_any_element(),
                                ],
                                cx,
                            ),
                        ))
                    })
                    .when(self.tab == Tab::Advanced, |el| {
                        el.child(settings_section(
                            tr("settings-extraction"),
                            settings_group(
                                [
                                    self.toggle_row(
                                        "settings-open-after",
                                        Toggle::OpenAfter,
                                        self.value.open_after,
                                        cx,
                                    )
                                    .into_any_element(),
                                    self.toggle_row(
                                        "settings-priority",
                                        Toggle::Priority,
                                        self.value.low_priority,
                                        cx,
                                    )
                                    .into_any_element(),
                                    self.toggle_row(
                                        "settings-close-after",
                                        Toggle::CloseAfter,
                                        self.value.close_after_quick,
                                        cx,
                                    )
                                    .into_any_element(),
                                ],
                                cx,
                            ),
                        ))
                    })
                    .when(self.tab == Tab::Associations, |el| {
                        el.child(
                            div()
                                .id("association-formats")
                                .min_h(px(180.))
                                .flex_shrink_0()
                                .py(px(8.))
                                .child(div().flex().flex_wrap().gap_y(px(10.)).children(
                                    sevenzip_shell_api::EXTENSIONS.iter().map(|extension| {
                                        let extension = *extension;
                                        div().w(relative(0.25)).child(
                                            Checkbox::new(SharedString::from(format!(
                                                "assoc:{extension}"
                                            )))
                                            .label(extension.trim_start_matches('.'))
                                            .text_size(px(13.))
                                            .checked(
                                                self.value
                                                    .associations
                                                    .iter()
                                                    .any(|e| e == extension),
                                            )
                                            .disabled(busy)
                                            .on_click(cx.listener(
                                                move |this, checked: &bool, window, cx| {
                                                    this.value
                                                        .associations
                                                        .retain(|e| e != extension);
                                                    if *checked {
                                                        this.value
                                                            .associations
                                                            .push(extension.into());
                                                    }
                                                    this.save(window, cx);
                                                    cx.notify();
                                                },
                                            )),
                                        )
                                    }),
                                )),
                        )
                        .child(
                            h_flex()
                                .gap(px(8.))
                                .child(
                                    command("association-all", tr("select-all"))
                                        .disabled(busy)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.value.associations =
                                                sevenzip_shell_api::EXTENSIONS
                                                    .iter()
                                                    .map(|e| (*e).to_owned())
                                                    .collect();
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    command("association-invert", tr("select-invert"))
                                        .disabled(busy)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.value.associations =
                                                sevenzip_shell_api::EXTENSIONS
                                                    .iter()
                                                    .filter(|e| {
                                                        !this
                                                            .value
                                                            .associations
                                                            .iter()
                                                            .any(|s| s == **e)
                                                    })
                                                    .map(|e| (*e).to_owned())
                                                    .collect();
                                            cx.notify();
                                        })),
                                )
                                .child(div().flex_1())
                                .child(
                                    command("association-defaults", tr("association-settings"))
                                        .disabled(busy)
                                        .on_click(|_, _, cx| {
                                            cx.open_url(crate::platform::DEFAULT_APPS_URI)
                                        }),
                                ),
                        )
                    })
                    .when(self.tab == Tab::Advanced, |el| {
                        el.child(settings_section(
                            tr("settings-opening"),
                            settings_group(
                                [
                                    settings_row(
                                        tr("settings-temp"),
                                        h_flex()
                                            .w(px(300.))
                                            .gap(px(8.))
                                            .child(
                                                div().flex_1().min_w_0().child(
                                                    text_input(&self.temporary).disabled(busy),
                                                ),
                                            )
                                            .child(
                                                icon_button(
                                                    "settings-temp-browse",
                                                    "FolderOpen",
                                                    tr("browser-browse"),
                                                    cx,
                                                )
                                                .disabled(busy)
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.browse(window, cx)
                                                })),
                                            ),
                                    )
                                    .into_any_element(),
                                    settings_row(
                                        tr("settings-extract-all"),
                                        div()
                                            .w(px(300.))
                                            .child(text_input(&self.patterns).disabled(busy)),
                                    )
                                    .into_any_element(),
                                ],
                                cx,
                            ),
                        ))
                    }),
            )
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .px(px(24.))
                        .pb(px(12.))
                        .text_color(rgb(p.danger))
                        .child(error),
                )
            })
    }
}
