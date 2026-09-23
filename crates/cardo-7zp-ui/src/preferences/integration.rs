use super::*;
use gpui_kit::base::{Align, FocusTrapElement, Placement, Positioner};

impl PreferencesForm {
    pub(super) fn integration_page(&self, cx: &mut Context<Self>) -> Div {
        let busy = self.is_busy();
        let bounds = self.association_bounds.clone();
        let summary = tf(
            "association-selected-count",
            &[
                ("count", self.value.associations.len().to_string().into()),
                (
                    "total",
                    cardo_7zp_commands::EXTENSIONS.len().to_string().into(),
                ),
            ],
        );
        v_flex()
            .gap(px(crate::theme::metrics::settings::SECTION_GAP))
            .child(settings_section(
                tr("settings-explorer"),
                settings_group(
                    [self
                        .toggle_row("settings-shell", Toggle::Shell, self.value.shell_menu, cx)
                        .into_any_element()],
                    cx,
                ),
                cx,
            ))
            .child(settings_section(
                tr("settings-associations"),
                settings_group(
                    [
                        settings_detail(
                            tr("association-formats"),
                            tr("settings-associations-note"),
                            settings_choice("association-picker", &summary, cx)
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.association_popup.update(cx, |state, cx| {
                                        if state.is_open() {
                                            state.dismiss(window, cx);
                                        } else {
                                            state.show(window, cx);
                                        }
                                    });
                                    if this.association_popup.read(cx).is_open() {
                                        this.association_search
                                            .update(cx, |input, cx| input.focus(window, cx));
                                    }
                                }))
                                .measure_anchor(bounds),
                            cx,
                        )
                        .into_any_element(),
                        settings_detail(
                            tr("settings-default-app-title"),
                            tr("settings-default-app-description"),
                            settings_action("association-defaults", tr("association-settings"), cx)
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let _ = this
                                        .owner
                                        .update(cx, |owner, cx| owner.open_default_apps(cx));
                                })),
                            cx,
                        )
                        .into_any_element(),
                    ],
                    cx,
                ),
                cx,
            ))
    }

    pub(crate) fn dismiss_associations(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.association_popup
            .update(cx, |state, cx| state.dismiss(window, cx));
    }

    pub(super) fn association_dropdown(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if self.tab != Tab::Integration || !self.association_popup.read(cx).is_open() {
            return None;
        }
        let bounds = self.association_bounds.get();
        let focus = self.association_popup.read(cx).focus_handle(cx);
        let busy = self.is_busy();
        let available =
            (window.viewport_size().height - bounds.bottom()).max(bounds.top()) - px(28.);
        let height = available.min(px(340.));
        let query = self
            .association_search
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase();
        let mut extensions = cardo_7zp_commands::EXTENSIONS.to_vec();
        extensions.sort_unstable_by_key(|extension| {
            extension.trim_start_matches('.').to_ascii_lowercase()
        });
        extensions.retain(|extension| extension.contains(&query));
        let empty = extensions.is_empty();
        let content = cardo_ui::settings::picker_surface(cx)
            .id("association-dropdown")
            .role(Role::Dialog)
            .aria_label(tr("settings-associations"))
            .w((bounds.size.width + px(40.)).max(px(280.)))
            .max_w(window.viewport_size().width - px(24.))
            .track_focus(&focus)
            .occlude()
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if !this.association_bounds.get().contains(&event.position) {
                    this.dismiss_associations(window, cx);
                }
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    this.dismiss_associations(window, cx);
                    cx.stop_propagation();
                }
            }))
            .child(
                cardo_ui::settings::picker_list(
                    "association-options",
                    (px(36. * extensions.len().max(1) as f32 + 42.))
                        .min((height - px(12.)).max(px(76.))),
                )
                .child(
                    div().flex_shrink_0().p(px(4.)).child(
                        settings_input(&self.association_search, tr("association-search"))
                            .prefix(icon("Search", 14.))
                            .bordered(false),
                    ),
                )
                .children(extensions.into_iter().map(|extension| {
                    let selected = self
                        .value
                        .associations
                        .iter()
                        .any(|value| value == extension);
                    cardo_ui::settings::picker_option(
                        SharedString::from(format!("association:{extension}")),
                        extension.trim_start_matches('.'),
                        cx,
                    )
                    .checked(selected)
                    .disabled(busy)
                    .on_click(cx.listener(
                        move |this, checked: &bool, window, cx| {
                            this.value.associations.retain(|value| value != extension);
                            if *checked {
                                this.value.associations.push(extension.into());
                            }
                            this.save(window, cx);
                        },
                    ))
                }))
                .when(empty, |el| {
                    el.child(
                        body_text(tr("association-no-matches"))
                            .p(px(12.))
                            .text_color(rgb(crate::theme::palette(cx).muted)),
                    )
                }),
            )
            .focus_trap("association-focus", &focus);
        Some(
            deferred(
                Positioner::side(bounds)
                    .placement(Placement::Bottom)
                    .align(Align::End)
                    .offset(px(6.))
                    .margin(px(8.))
                    .child(content),
            )
            .with_priority(gpui_kit::base::POPUP_PRIORITY)
            .into_any_element(),
        )
    }
}
