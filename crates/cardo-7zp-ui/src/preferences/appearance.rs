use super::*;

impl PreferencesForm {
    pub(super) fn appearance_page(&self, cx: &mut Context<Self>) -> Div {
        let p = crate::theme::palette(cx);
        let busy = self.is_busy();
        let minimum_size = cardo_7zp_core::settings::Appearance::MIN_FONT_SIZE;
        let maximum_size = cardo_7zp_core::settings::Appearance::MAX_FONT_SIZE;
        let size_step = cardo_7zp_core::settings::Appearance::FONT_SIZE_STEP;
        settings_group(
            [
                settings_detail(
                    tr("theme-menu"),
                    tr("settings-theme-description"),
                    settings_segments("settings-theme", tr("theme-menu")).children(
                        crate::theme::THEMES
                            .into_iter()
                            .enumerate()
                            .map(|(index, theme)| {
                                let owner = cx.entity().downgrade();
                                settings_segment(
                                    ("theme-option", index),
                                    theme.label(),
                                    self.theme == theme,
                                    busy,
                                    cx,
                                )
                                .set_position(index + 1, crate::theme::THEMES.len())
                                .on_change(
                                    move |_, _, window, cx| {
                                        let _ = owner.update(cx, |this, cx| {
                                            this.theme = theme;
                                            this.save(window, cx);
                                        });
                                    },
                                )
                            }),
                    ),
                    cx,
                )
                .into_any_element(),
                settings_detail(
                    tr("appearance-font"),
                    tr("appearance-font-hint"),
                    v_flex()
                        .w(px(cardo_ui::settings::metrics::INPUT_WIDTH))
                        .max_w_full()
                        .min_w_0()
                        .gap(px(8.))
                        .child(
                            settings_input(&self.font_family, tr("appearance-font")).disabled(busy),
                        )
                        .when_some(self.font_error.clone(), |el, error| {
                            el.child(
                                body_text(error)
                                    .text_size(px(12.))
                                    .text_color(rgb(p.danger)),
                            )
                        }),
                    cx,
                )
                .into_any_element(),
                settings_detail(
                    tr("appearance-font-size"),
                    &tf(
                        "settings-font-size-description",
                        &[
                            ("min", (minimum_size as u32).to_string().into()),
                            ("max", (maximum_size as u32).to_string().into()),
                        ],
                    ),
                    settings_stepper(
                        format!("{}", self.font_size as u32),
                        icon_button(
                            "font-size-decrease",
                            "Subtract",
                            tr("appearance-font-decrease"),
                            true,
                            cx,
                        )
                        .disabled(busy || self.font_size <= minimum_size)
                        .on_click(cx.listener(
                            move |this, _, window, cx| this.step_font_size(-size_step, window, cx),
                        )),
                        icon_button(
                            "font-size-increase",
                            "Add",
                            tr("appearance-font-increase"),
                            true,
                            cx,
                        )
                        .disabled(busy || self.font_size >= maximum_size)
                        .on_click(cx.listener(
                            move |this, _, window, cx| this.step_font_size(size_step, window, cx),
                        )),
                        cx,
                    ),
                    cx,
                )
                .into_any_element(),
                self.toggle_row(
                    "settings-hide-tool-labels",
                    Toggle::ToolLabels,
                    self.value.hide_tool_labels,
                    cx,
                )
                .into_any_element(),
                settings_detail(
                    tr("settings-font-reset-title"),
                    tr("settings-font-reset-description"),
                    settings_action("appearance-reset", tr("appearance-reset"), cx)
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, window, cx| {
                            let default = cardo_7zp_core::settings::Appearance::default();
                            this.font_family.update(cx, |input, cx| {
                                input.set_value(default.font_family, window, cx)
                            });
                            this.font_size = default.font_size;
                            this.save(window, cx);
                        })),
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        )
    }
}
