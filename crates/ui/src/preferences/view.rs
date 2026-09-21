use super::*;

impl Render for PreferencesForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = crate::theme::palette(cx);
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
                                            for theme in crate::theme::THEMES {
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
                                            for language in sevenzip_core::i18n::LANGUAGES {
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
                                            cx.open_url(sevenzip_platform::DEFAULT_APPS_URI)
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
