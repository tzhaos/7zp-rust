use crate::*;
use gpui_kit::{
    base::{Align, Placement, Positioner},
    component::{Disableable, h_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn dismiss_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.history_popup
            .update(cx, |state, cx| state.dismiss(window, cx));
    }

    pub(crate) fn toggle_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.command_available(commands::Command::Open, cx) {
            return;
        }
        if self.history_popup.read(cx).is_open() {
            self.dismiss_history(window, cx);
        } else {
            self.history_cursor = 0;
            self.history_popup
                .update(cx, |state, cx| state.show(window, cx));
        }
        cx.notify();
    }

    fn open_history(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.history.get(index).cloned() else {
            return;
        };
        self.visit_history(entry, window, cx);
    }

    fn visit_history(
        &mut self,
        entry: cardo_7zp_core::settings::recent::Entry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dismiss_history(window, cx);
        let location = match entry.kind {
            recent::Kind::Archives => browser::Location::Archive(entry.path, String::new()),
            recent::Kind::Folders => browser::Location::Directory(entry.path),
        };
        self.visit(location, window, cx);
    }

    pub(crate) fn history_dropdown(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        if !self.history_popup.read(cx).is_open() || self.tasks.is_busy() || self.dialogs.is_open()
        {
            return None;
        }
        let bounds = self.address_bounds.get();
        let available =
            (window.viewport_size().height - bounds.bottom()).max(bounds.top()) - px(24.);
        let row_height = crate::theme::ui_font_size(cx).as_f32() + 28.;
        let count = ((available.as_f32() - 88.) / row_height)
            .floor()
            .clamp(1., 8.) as usize;
        let cursor = self
            .history_cursor
            .min(self.history.len().saturating_sub(1));
        let start = cursor / count * count;
        let end = (start + count).min(self.history.len());
        let p = crate::theme::palette(cx);
        let focus = self.history_popup.read(cx).focus_handle(cx);
        let surface = popup_surface(cx)
            .id("path-history")
            .role(Role::Dialog)
            .w(bounds.size.width)
            .track_focus(&focus)
            .tab_group()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if !this.address_bounds.get().contains(&event.position) {
                    this.dismiss_history(window, cx);
                }
            }))
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                let last = this.history.len().saturating_sub(1);
                match event.keystroke.key.as_str() {
                    "escape" => this.dismiss_history(window, cx),
                    "enter" => this.open_history(this.history_cursor.min(last), window, cx),
                    "up" => this.history_cursor = this.history_cursor.saturating_sub(1),
                    "down" => this.history_cursor = (this.history_cursor + 1).min(last),
                    "home" => this.history_cursor = 0,
                    "end" => this.history_cursor = last,
                    "pageup" => this.history_cursor = this.history_cursor.saturating_sub(count),
                    "pagedown" => this.history_cursor = (this.history_cursor + count).min(last),
                    _ => return,
                }
                cx.stop_propagation();
                cx.notify();
            }))
            .child(
                h_flex()
                    .h(px(32.))
                    .px(px(8.))
                    .gap(px(8.))
                    .child(compact_text("history-heading", tr("history-title")).flex_1())
                    .child(
                        icon_button("history-clear", "Delete", tr("recent-clear"), true, cx)
                            .disabled(self.history.is_empty())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.dismiss_history(window, cx);
                                this.command(commands::Command::ClearRecent, window, cx);
                            })),
                    ),
            )
            .when(self.history.is_empty(), |el| {
                el.child(
                    body_text(tr("history-empty"))
                        .p(px(12.))
                        .text_color(rgb(p.muted)),
                )
            })
            .children(self.history.iter().enumerate().skip(start).take(count).map(
                |(index, entry)| {
                    let target = entry.clone();
                    let path = entry.path.display().to_string();
                    let directory = entry.kind == recent::Kind::Folders;
                    bubble_tooltip(
                        list_row(("history-location", index), cursor == index, false, cx)
                            .h(px(row_height))
                            .px(px(8.))
                            .gap(px(8.))
                            .child(icon(if directory { "Folder" } else { "Archive" }, 18.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_ellipsis_middle()
                                    .child(path.clone()),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.visit_history(target.clone(), window, cx)
                            })),
                        path,
                    )
                },
            ))
            .when(self.history.len() > count, |el| {
                el.child(
                    h_flex()
                        .h(px(36.))
                        .px(px(8.))
                        .gap(px(8.))
                        .child(
                            compact_text(
                                "history-range",
                                tf(
                                    "history-range",
                                    &[
                                        ("first", (start + 1).to_string().into()),
                                        ("last", end.to_string().into()),
                                        ("total", self.history.len().to_string().into()),
                                    ],
                                ),
                            )
                            .flex_1()
                            .text_color(rgb(p.muted)),
                        )
                        .child(
                            icon_button(
                                "history-previous",
                                "ArrowLeft",
                                tr("history-previous"),
                                true,
                                cx,
                            )
                            .disabled(start == 0)
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.history_cursor = start.saturating_sub(count);
                                    cx.notify();
                                },
                            )),
                        )
                        .child(
                            icon_button("history-next", "ArrowRight", tr("history-next"), true, cx)
                                .disabled(end == self.history.len())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.history_cursor =
                                        end.min(this.history.len().saturating_sub(1));
                                    cx.notify();
                                })),
                        ),
                )
            });
        Some(
            deferred(
                Positioner::side(bounds)
                    .placement(Placement::Bottom)
                    .align(Align::Start)
                    .offset(px(6.))
                    .margin(px(8.))
                    .child(surface),
            )
            .with_priority(100)
            .into_any_element(),
        )
    }
}
