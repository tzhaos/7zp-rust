use super::*;
use gpui_kit::base::{Align, ElementExt, FocusTrapElement, Placement, Positioner};

#[derive(Clone, Copy)]
enum AssociationSelection {
    All,
    None,
    Invert,
}

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
                settings_frame(cx)
                    .p(px(crate::theme::metrics::settings::GROUP_PADDING))
                    .gap(px(16.))
                    .child(
                        div()
                            .text_color(rgb(crate::theme::palette(cx).muted))
                            .whitespace_normal()
                            .child(tr("settings-associations-note")),
                    )
                    .child(settings_row(
                        tr("association-formats"),
                        menu_choice("association-picker", &summary, cx)
                            .w(px(crate::theme::metrics::settings::SELECTOR_WIDTH))
                            .disabled(busy)
                            .on_prepaint(move |rect, _, _| bounds.set(rect))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.association_popup.update(cx, |state, cx| {
                                    if state.is_open() {
                                        state.dismiss(window, cx);
                                    } else {
                                        state.show(window, cx);
                                    }
                                });
                            })),
                        cx,
                    ))
                    .child(
                        h_flex().justify_end().child(
                            command("association-defaults", tr("association-settings"))
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let _ = this
                                        .owner
                                        .update(cx, |owner, cx| owner.open_default_apps(cx));
                                })),
                        ),
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
        let mut extensions = cardo_7zp_commands::EXTENSIONS.to_vec();
        extensions.sort_unstable_by_key(|extension| {
            extension.trim_start_matches('.').to_ascii_lowercase()
        });
        let content = popup_surface(cx)
            .id("association-dropdown")
            .role(Role::Dialog)
            .aria_label(tr("settings-associations"))
            .w(bounds.size.width)
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
                h_flex().gap(px(4.)).pb(px(6.)).children(
                    [
                        ("association-all", "select-all", AssociationSelection::All),
                        (
                            "association-none",
                            "select-none",
                            AssociationSelection::None,
                        ),
                        (
                            "association-invert",
                            "select-invert",
                            AssociationSelection::Invert,
                        ),
                    ]
                    .into_iter()
                    .map(|(id, label, mode)| {
                        command(id, tr(label))
                            .px(px(6.))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.value.associations = cardo_7zp_commands::EXTENSIONS
                                    .iter()
                                    .filter(|extension| match mode {
                                        AssociationSelection::All => true,
                                        AssociationSelection::None => false,
                                        AssociationSelection::Invert => !this
                                            .value
                                            .associations
                                            .iter()
                                            .any(|value| value == **extension),
                                    })
                                    .map(|extension| (*extension).to_owned())
                                    .collect();
                                this.save(window, cx);
                            }))
                    }),
                ),
            )
            .child(
                v_flex()
                    .id("association-options")
                    .max_h((height - px(52.)).max(px(64.)))
                    .overflow_y_scrollbar()
                    .gap(px(2.))
                    .children(extensions.into_iter().map(|extension| {
                        let selected = self
                            .value
                            .associations
                            .iter()
                            .any(|value| value == extension);
                        checkbox(
                            SharedString::from(format!("association:{extension}")),
                            extension.trim_start_matches('.'),
                        )
                        .checked(selected)
                        .disabled(busy)
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .on_click(cx.listener(
                            move |this, checked: &bool, window, cx| {
                                this.value.associations.retain(|value| value != extension);
                                if *checked {
                                    this.value.associations.push(extension.into());
                                }
                                this.save(window, cx);
                            },
                        ))
                    })),
            )
            .focus_trap("association-focus", &focus);
        Some(
            deferred(
                Positioner::side(bounds)
                    .placement(Placement::Bottom)
                    .align(Align::Start)
                    .offset(px(6.))
                    .margin(px(8.))
                    .child(content),
            )
            .with_priority(gpui_kit::base::POPUP_PRIORITY)
            .into_any_element(),
        )
    }
}
