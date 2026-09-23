use super::*;
use cardo_7zp_core::settings::shortcuts::{Shortcut, ShortcutAction};

impl PreferencesForm {
    pub(super) fn shortcuts_section(&self, cx: &mut Context<Self>) -> Div {
        let query = self.shortcut_search.read(cx).value().to_lowercase();
        let actions = ShortcutAction::ALL
            .iter()
            .copied()
            .filter(|action| {
                tr(action.label()).to_lowercase().contains(&query)
                    || tr(action.description()).to_lowercase().contains(&query)
                    || action
                        .binding(&self.value.shortcuts)
                        .is_some_and(|key| key.display().to_lowercase().contains(&query))
            })
            .collect::<Vec<_>>();
        let busy = self.controls_disabled();
        let mut section = v_flex().gap(px(24.)).child(
            h_flex()
                .w_full()
                .min_w_0()
                .gap(px(12.))
                .child(
                    body_text(tr("shortcuts-title"))
                        .flex_1()
                        .text_size(px(24.))
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    settings_action(
                        "shortcuts-expand",
                        tr(if self.shortcut_expanded {
                            "shortcuts-collapse"
                        } else {
                            "shortcuts-configure"
                        }),
                        cx,
                    )
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if this.is_busy() {
                            return;
                        }
                        this.shortcut_expanded = !this.shortcut_expanded;
                        this.shortcut_recording = None;
                        cx.notify();
                    })),
                ),
        );
        if !self.shortcut_expanded {
            return section;
        }
        section = section.child(
            settings_input(&self.shortcut_search, tr("shortcuts-search"))
                .w_full()
                .rounded(px(20.))
                .prefix(icon("Search", 16.))
                .suffix(
                    icon_button(
                        "shortcuts-reset-all",
                        "ArrowClockwise",
                        tr("shortcuts-reset-all"),
                        true,
                        cx,
                    )
                    .disabled(busy || self.value.shortcuts.is_empty())
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.is_busy() {
                            return;
                        }
                        this.shortcut_recording = None;
                        this.shortcut_error = None;
                        this.value.shortcuts.clear();
                        this.save(window, cx);
                    })),
                ),
        );
        if actions.is_empty() {
            return section.child(body_text(tr("shortcuts-no-matches")));
        }
        section.child(
            div()
                .id("shortcut-recorder")
                .track_focus(&self.shortcut_focus)
                .child(settings_group(
                    actions.into_iter().map(|action| {
                        let recording = self.shortcut_recording == Some(action);
                        let binding = action.binding(&self.value.shortcuts);
                        let label = if recording {
                            tr("shortcuts-recording").to_owned()
                        } else {
                            binding
                                .as_ref()
                                .map(|key| key.display())
                                .unwrap_or_else(|| tr("shortcuts-unassigned").to_owned())
                        };
                        let controls = h_flex()
                            .w_full()
                            .min_w_0()
                            .gap(px(4.))
                            .child(cardo_ui::settings::shortcut_badge(label, recording, cx))
                            .child(
                                icon_button(
                                    SharedString::from(format!("shortcut-edit-{action:?}")),
                                    if recording { "Dismiss" } else { "Edit" },
                                    tr(if recording {
                                        "shortcuts-cancel"
                                    } else {
                                        "shortcuts-edit"
                                    }),
                                    true,
                                    cx,
                                )
                                .disabled(busy)
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        if this.is_busy() {
                                            return;
                                        }
                                        this.shortcut_error = None;
                                        this.shortcut_recording =
                                            if recording { None } else { Some(action) };
                                        if !recording {
                                            this.shortcut_focus.focus(window, cx);
                                        }
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(div().flex_1().min_w_0())
                            .when(self.value.shortcuts.contains_key(&action), |el| {
                                el.child(
                                    icon_button(
                                        SharedString::from(format!("shortcut-reset-{action:?}")),
                                        "ArrowClockwise",
                                        tr("shortcuts-reset-one"),
                                        true,
                                        cx,
                                    )
                                    .disabled(busy)
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            if !this.is_busy() {
                                                this.assign_shortcut(
                                                    action,
                                                    action.default_shortcut(),
                                                    window,
                                                    cx,
                                                );
                                            }
                                        },
                                    )),
                                )
                            })
                            .child(
                                icon_button(
                                    SharedString::from(format!("shortcut-remove-{action:?}")),
                                    "Delete",
                                    tr("shortcuts-remove"),
                                    true,
                                    cx,
                                )
                                .disabled(busy || binding.is_none())
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        if !this.is_busy() {
                                            this.assign_shortcut(action, None, window, cx);
                                        }
                                    },
                                )),
                            );
                        v_flex()
                            .child(cardo_ui::settings::shortcut_row(
                                tr(action.label()),
                                tr(action.description()),
                                controls,
                                cx,
                            ))
                            .when(recording, |el| {
                                el.children(self.shortcut_error.clone().map(|error| {
                                    body_text(error)
                                        .pb(px(12.))
                                        .text_color(rgb(crate::theme::palette(cx).danger))
                                }))
                            })
                            .into_any_element()
                    }),
                    cx,
                )),
        )
    }

    fn assign_shortcut(
        &mut self,
        action: ShortcutAction,
        binding: Option<Shortcut>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(key) = &binding {
            if let Some(other) = ShortcutAction::ALL.iter().copied().find(|&other| {
                other != action && other.binding(&self.value.shortcuts).as_ref() == Some(key)
            }) {
                self.shortcut_error = Some(tf(
                    "shortcuts-conflict",
                    &[
                        ("keys", key.display().into()),
                        ("command", tr(other.label()).into()),
                    ],
                ));
                self.shortcut_recording = Some(action);
                self.shortcut_focus.focus(window, cx);
                cx.notify();
                return;
            }
        }
        if binding == action.default_shortcut() {
            self.value.shortcuts.remove(&action);
        } else {
            self.value.shortcuts.insert(action, binding);
        }
        self.shortcut_recording = None;
        self.shortcut_error = None;
        self.save(window, cx);
    }

    pub(crate) fn record_shortcut(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(action) = self.shortcut_recording else {
            return false;
        };
        if !self.shortcut_focus.is_focused(window) {
            self.shortcut_recording = None;
            cx.notify();
            return false;
        }
        if event.is_held {
            return true;
        }
        if event.keystroke.key == "escape" {
            self.shortcut_recording = None;
            self.shortcut_error = None;
            cx.notify();
            return true;
        }
        if matches!(
            event.keystroke.key.as_str(),
            "control" | "shift" | "alt" | "platform" | "fn"
        ) {
            return true;
        }
        let Some(binding) = crate::shortcuts::chord(event) else {
            self.shortcut_error = Some(tr("shortcuts-reserved").to_owned());
            cx.notify();
            return true;
        };
        if binding.key == "backspace" && !binding.control && !binding.shift && !binding.alt {
            self.assign_shortcut(action, None, window, cx);
        } else if !binding.allowed() {
            self.shortcut_error = Some(tr("shortcuts-reserved").to_owned());
            cx.notify();
        } else {
            self.assign_shortcut(action, Some(binding), window, cx);
        }
        true
    }
}
