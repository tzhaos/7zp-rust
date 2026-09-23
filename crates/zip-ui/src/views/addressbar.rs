use crate::*;
use zip_core::i18n::tr;
use gpui_kit::{base::ElementExt, component::Disableable, prelude::FluentBuilder};

impl Workspace {
    pub(crate) fn pathbar(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
        let bounds = self.address_bounds.clone();
        let current_path = self.address_text();
        navigation_row(cx)
            .child(
                icon_button(
                    "home",
                    "Home",
                    tr("browser-home"),
                    self.allow_hint(!self.tasks.is_busy()),
                    cx,
                )
                .w(px(30.))
                .h(px(30.))
                .disabled(!self.command_available(commands::Command::Home, cx))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.command(commands::Command::Home, window, cx)
                })),
            )
            .child(
                icon_button(
                    "back",
                    "ArrowLeft",
                    tr("back"),
                    self.allow_hint(
                        !self.tasks.is_busy() && !self.browser.view().history.is_empty(),
                    ),
                    cx,
                )
                .w(px(30.))
                .h(px(30.))
                .disabled(self.tasks.is_busy() || self.browser.view().history.is_empty())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.back(window, cx);
                })),
            )
            .child(
                icon_button(
                    "up",
                    "ArrowUp",
                    tr("parent-folder"),
                    self.allow_hint(!self.tasks.is_busy() && self.parent_location().is_some()),
                    cx,
                )
                .w(px(30.))
                .h(px(30.))
                .disabled(self.tasks.is_busy() || self.parent_location().is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.command(commands::Command::Up, window, cx);
                })),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h(px(30.))
                    .on_prepaint(move |rect, window, _| {
                        if bounds.replace(rect) != rect {
                            window.request_animation_frame();
                        }
                    })
                    .child(
                        Styled::h(text_input(&self.address), px(30.))
                            .bg(rgb(p.title))
                            .border_color(rgb(p.border))
                            .focus_bordered(false)
                            .when(
                                self.address.read(cx).focus_handle(cx).is_focused(window),
                                |input| input.border_color(rgb(p.muted)),
                            )
                            .rounded(px(6.))
                            .text_size(px(12.))
                            .aria_label(tr("browser-address"))
                            .disabled(self.tasks.is_busy())
                            .suffix(
                                icon_button(
                                    "recent-locations",
                                    "ChevronDown",
                                    tr("browser-location-menu"),
                                    self.allow_hint(!self.tasks.is_busy()),
                                    cx,
                                )
                                .w(px(24.))
                                .h(px(24.))
                                .disabled(self.tasks.is_busy())
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.toggle_history(window, cx);
                                    },
                                )),
                            ),
                    ),
            )
            .child(
                icon_button(
                    "browse-folder",
                    "FolderOpen",
                    tr("browser-browse"),
                    self.allow_hint(!self.tasks.is_busy()),
                    cx,
                )
                .w(px(30.))
                .h(px(30.))
                .disabled(!self.command_available(commands::Command::Browse, cx))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.command(commands::Command::Browse, window, cx)
                })),
            )
            .child(
                icon_button(
                    "refresh-folder",
                    "ArrowClockwise",
                    tr("menu-refresh"),
                    self.allow_hint(!self.tasks.is_busy()),
                    cx,
                )
                .disabled(!self.command_available(commands::Command::Refresh, cx))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.command(commands::Command::Refresh, window, cx)
                })),
            )
            .child(
                icon_button(
                    "copy-path",
                    "Copy",
                    tr("browser-copy-path"),
                    self.allow_hint(!current_path.is_empty()),
                    cx,
                )
                .w(px(30.))
                .h(px(30.))
                .disabled(current_path.is_empty())
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(current_path.clone()))
                }),
            )
            .child(
                div()
                    .w(px(200.))
                    .max_w(relative(0.25))
                    .h(px(30.))
                    .flex_shrink_0()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.search.update(cx, |input, cx| input.focus(window, cx));
                        }),
                    )
                    .child(
                        Styled::h(text_input(&self.search), px(30.))
                            .text_size(px(12.))
                            .bg(rgb(p.title))
                            .border_color(rgb(p.border))
                            .focus_bordered(false)
                            .when(
                                self.search.read(cx).focus_handle(cx).is_focused(window),
                                |input| input.border_color(rgb(p.muted)),
                            )
                            .rounded(px(6.))
                            .aria_label(tr("search-placeholder"))
                            .prefix(icon("Search", 16.).text_color(rgb(p.muted)))
                            .when(!self.search.read(cx).value().is_empty(), |input| {
                                input.suffix(
                                    icon_button(
                                        "clear-search",
                                        "Dismiss",
                                        tr("search-clear"),
                                        self.allow_hint(true),
                                        cx,
                                    )
                                    .w(px(24.))
                                    .h(px(24.))
                                    .on_click(cx.listener(
                                        |this, _, window, cx| {
                                            this.search.update(cx, |input, cx| {
                                                input.set_value("", window, cx)
                                            })
                                        },
                                    )),
                                )
                            }),
                    ),
            )
    }
}
