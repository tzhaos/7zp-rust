use super::*;

impl PreferencesForm {
    pub(super) fn advanced_page(&self, cx: &mut Context<Self>) -> Div {
        let busy = self.is_busy();
        v_flex()
            .gap(px(crate::theme::metrics::settings::SECTION_GAP))
            .child(settings_section(
                tr("settings-opening"),
                settings_group(
                    [
                        settings_row(
                            tr("settings-temp"),
                            h_flex()
                                .w(px(300.))
                                .gap(px(8.))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .child(text_input(&self.temporary).disabled(busy)),
                                )
                                .child(
                                    icon_button(
                                        "settings-temp-browse",
                                        "FolderOpen",
                                        tr("browser-browse"),
                                        !busy
                                            && self
                                                .owner
                                                .read_with(cx, |workspace, _| {
                                                    workspace.allow_hint(true)
                                                })
                                                .unwrap_or(false),
                                        cx,
                                    )
                                    .disabled(busy)
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.browse(window, cx)),
                                    ),
                                ),
                            cx,
                        )
                        .into_any_element(),
                        settings_row(
                            tr("settings-extract-all"),
                            div()
                                .w(px(300.))
                                .child(text_input(&self.patterns).disabled(busy)),
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
