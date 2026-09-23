use super::*;

impl PreferencesForm {
    pub(super) fn advanced_page(&self, cx: &mut Context<Self>) -> Div {
        let busy = self.is_busy();
        let path = self.temporary.read(cx).value();
        let custom_path = !path.is_empty();
        let path_label = if custom_path {
            path.to_string()
        } else {
            self.system_temporary.to_string()
        };
        v_flex()
            .gap(px(crate::theme::metrics::settings::SECTION_GAP))
            .child(settings_section(
                tr("settings-opening"),
                settings_group(
                    [
                        settings_detail(
                            tr("settings-temp"),
                            tr("settings-temp-description"),
                            h_flex()
                                .w(px(280.))
                                .max_w_full()
                                .gap(px(8.))
                                .child(
                                    cardo_ui::settings::path_value(
                                        "temporary-path",
                                        path_label,
                                        cx,
                                    )
                                    .flex_1(),
                                )
                                .child(
                                    settings_action(
                                        "settings-temp-browse",
                                        tr("settings-change"),
                                        cx,
                                    )
                                    .flex_shrink_0()
                                    .disabled(busy)
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.browse(window, cx)),
                                    ),
                                )
                                .when(custom_path, |el| {
                                    el.child(
                                        icon_button(
                                            "settings-temp-reset",
                                            "Dismiss",
                                            tr("settings-temp-reset"),
                                            true,
                                            cx,
                                        )
                                        .disabled(busy)
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.temporary.update(cx, |input, cx| {
                                                    input.set_value("", window, cx)
                                                });
                                                this.save(window, cx);
                                            }),
                                        ),
                                    )
                                }),
                            cx,
                        )
                        .into_any_element(),
                        cardo_ui::settings::editor(
                            tr("settings-extract-all-title"),
                            tr("settings-patterns-description"),
                            settings_action("settings-patterns-save", tr("settings-save"), cx)
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                            cardo_ui::settings::textarea(
                                &self.patterns,
                                tr("settings-extract-all-title"),
                            )
                            .disabled(busy),
                            cx,
                        )
                        .into_any_element(),
                    ],
                    cx,
                ),
                cx,
            ))
            .child(settings_section(
                tr("settings-performance"),
                settings_group(
                    [self
                        .toggle_row(
                            "settings-priority",
                            Toggle::Priority,
                            self.value.low_priority,
                            cx,
                        )
                        .into_any_element()],
                    cx,
                ),
                cx,
            ))
    }
}
