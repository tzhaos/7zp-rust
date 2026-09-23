use super::*;

impl PreferencesForm {
    pub(super) fn application_page(&self, cx: &mut Context<Self>) -> Div {
        let busy = self.controls_disabled();
        let languages = cx.entity().downgrade();
        let selected_language = self.language;
        v_flex()
            .gap(px(crate::theme::metrics::settings::SECTION_GAP))
            .child(settings_section(
                tr("settings-language-history"),
                settings_group(
                    [
                        settings_detail(
                            tr("language-menu"),
                            tr("settings-language-description"),
                            settings_choice("settings-language", self.language.label(), cx)
                                .disabled(busy)
                                .choice_menu(move |_, _| {
                                    let mut menu = Menu::new();
                                    for language in zip_core::i18n::LANGUAGES {
                                        let owner = languages.clone();
                                        menu = menu.item(
                                            MenuItem::new(language.label())
                                                .checked(language == selected_language)
                                                .on_select(move |window, cx| {
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
                    ],
                    cx,
                ),
                cx,
            ))
            .child(settings_section(
                tr("settings-appearance"),
                self.appearance_page(cx),
                cx,
            ))
            .child(settings_section(
                tr("settings-completion"),
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
                            "settings-close-after",
                            Toggle::CloseAfter,
                            self.value.close_after_quick,
                            cx,
                        )
                        .into_any_element(),
                        self.toggle_row(
                            "settings-close-archive",
                            Toggle::CloseArchive,
                            self.value.close_archive_after_quick,
                            cx,
                        )
                        .into_any_element(),
                    ],
                    cx,
                ),
                cx,
            ))
            .child(self.shortcuts_section(cx))
    }
}
