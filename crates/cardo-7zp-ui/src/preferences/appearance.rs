use super::*;

impl PreferencesForm {
    pub(super) fn appearance_page(&self, cx: &mut Context<Self>) -> Div {
        let p = crate::theme::palette(cx);
        let busy = self.is_busy();
        let themes = cx.entity().downgrade();
        let selected_theme = self.theme;
        let size_label = format!("{}", self.font_size as u32);
        let minimum_size = cardo_7zp_core::settings::Appearance::MIN_FONT_SIZE;
        let maximum_size = cardo_7zp_core::settings::Appearance::MAX_FONT_SIZE;
        let size_step = cardo_7zp_core::settings::Appearance::FONT_SIZE_STEP;
        v_flex().child(settings_group(
            [
                settings_row(
                    tr("theme-menu"),
                    menu_choice("settings-theme", self.theme.label(), cx)
                        .w(px(crate::theme::metrics::settings::SELECTOR_WIDTH))
                        .disabled(busy)
                        .choice_menu(move |_, _| {
                            let mut menu = Menu::new();
                            for theme in crate::theme::THEMES {
                                let owner = themes.clone();
                                menu = menu.item(
                                    MenuItem::new(theme.label())
                                        .checked(theme == selected_theme)
                                        .on_select(move |window, cx| {
                                            let _ = owner.update(cx, |this, cx| {
                                                this.theme = theme;
                                                cx.notify();
                                            });
                                            let _ =
                                                owner.update(cx, |this, cx| this.save(window, cx));
                                        }),
                                );
                            }
                            menu
                        }),
                    cx,
                )
                .into_any_element(),
                settings_row(
                    tr("appearance-font"),
                    v_flex()
                        .w(px(280.))
                        .gap(px(8.))
                        .child(text_input(&self.font_family).disabled(busy))
                        .when_some(self.font_error.clone(), |el, error| {
                            el.child(
                                div()
                                    .text_size(px(12.))
                                    .line_height(px(18.))
                                    .text_color(rgb(p.danger))
                                    .whitespace_normal()
                                    .child(error),
                            )
                        }),
                    cx,
                )
                .into_any_element(),
                settings_row(
                    tr("appearance-font-size"),
                    h_flex()
                        .w(px(140.))
                        .h(px(crate::theme::metrics::CONTROL_HEIGHT))
                        .overflow_hidden()
                        .rounded(px(crate::theme::metrics::CONTROL_RADIUS))
                        .border_1()
                        .border_color(rgb(p.border))
                        .child(
                            icon_button(
                                "font-size-decrease",
                                "Subtract",
                                tr("appearance-font-decrease"),
                                true,
                                cx,
                            )
                            .disabled(busy || self.font_size <= minimum_size)
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.step_font_size(-size_step, window, cx)
                                },
                            )),
                        )
                        .child(
                            h_flex()
                                .h_full()
                                .flex_1()
                                .justify_center()
                                .border_l_1()
                                .border_r_1()
                                .border_color(rgb(p.border))
                                .text_size(px(12.))
                                .child(size_label),
                        )
                        .child(
                            icon_button(
                                "font-size-increase",
                                "Add",
                                tr("appearance-font-increase"),
                                true,
                                cx,
                            )
                            .disabled(busy || self.font_size >= maximum_size)
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.step_font_size(size_step, window, cx)
                                },
                            )),
                        ),
                    cx,
                )
                .into_any_element(),
                settings_row(
                    tr("appearance-font-hint"),
                    command("appearance-reset", tr("appearance-reset"))
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
                self.toggle_row(
                    "settings-hide-tool-labels",
                    Toggle::ToolLabels,
                    self.value.hide_tool_labels,
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        ))
    }
}
