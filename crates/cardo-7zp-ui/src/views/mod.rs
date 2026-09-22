mod about;
mod addressbar;
mod files;
mod history;
mod home;
mod settings;
pub(crate) use settings::SettingsPage;
mod statusbar;
mod titlebar;
mod toolbar;

use crate::*;
use cardo_7zp_core::i18n::{tf, tr};
use gpui_kit::base::ElementExt;
use gpui_kit::{
    component::{Root, v_flex},
    prelude::FluentBuilder,
};

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_view(window, cx);
        let p = crate::theme::palette(cx);
        let appearance = crate::theme::appearance(cx);
        let modal = self.modal(cx);
        let searching = !self.search.read(cx).value().is_empty();
        let browser = self.browser.view();
        let home = browser.catalog.is_none() && browser.directory.is_none();
        let content_bounds = self.content_bounds.clone();
        v_flex()
            .id("workspace")
            .track_focus(&self.focus)
            .relative()
            .size_full()
            .bg(rgb(p.panel))
            .text_color(rgb(p.text))
            .font(crate::theme::interface_font(cx))
            .text_size(px(appearance.font_size))
            .on_key_down(cx.listener(Self::keyboard))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.hint_anchor = Some(event.position);
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                let Some(anchor) = this.hint_anchor else {
                    return;
                };
                let moved = (event.position.x - anchor.x).abs() > px(8.)
                    || (event.position.y - anchor.y).abs() > px(8.);
                if moved {
                    this.hint_anchor = None;
                    cx.notify();
                }
            }))
            .on_drag_move(
                cx.listener(|this, _: &DragMoveEvent<ExternalPaths>, _, cx| {
                    if !this.dialogs.is_open()
                        && !this.tasks.is_busy()
                        && !this.settings_busy(cx)
                        && !this.dragging
                    {
                        this.dragging = true;
                        cx.notify();
                    }
                }),
            )
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.dragging = false;
                this.drop_files(paths.paths().to_vec(), window, cx)
            }))
            .child(self.titlebar(window, cx))
            .child(self.toolbar(window, cx))
            .child(
                panel_surface(cx)
                    .border_0()
                    .shadow_none()
                    .child(crate::components::panel::connected_panel_outline(self.active_tab_center.clone()))
                    .id("content-panel")
                    .on_prepaint(move |bounds, _, _| content_bounds.set(bounds))
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .mx(px(12.))
                    .mt(px(crate::theme::metrics::CONTENT_TOP_GAP))
                    .mb(px(12.))
                    .when(self.settings_page.is_none(), |panel| panel.child(
                        v_flex()
                            .size_full()
                            .min_h_0()
                            .overflow_hidden()
                            .rounded(px(6.))
                            .child(self.pathbar(window, cx))
                            .when(!home, |el| el.child(self.file_header(cx)))
                            .map(|el| {
                                el.when(home, |el| el.child(self.home_view(cx)))
                                    .when(!home, |el| {
                                        el.child(
                                        div()
                                            .id("file-area")
                                            .relative()
                                            .flex_1()
                                            .min_h_0()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.focus.focus(window, cx);
                                                this.browser.clear_selection(false);
                                                cx.notify();
                                            }))
                                            .when(!browser.rows.is_empty(), |el| {
                                                el.child(
                                                    uniform_list("files", browser.rows.len(), {
                                                        let entity = cx.entity();
                                                        move |range, _, cx| {
                                                            entity.update(cx, |this, cx| {
                                                                range
                                                                    .map(|index| {
                                                                        this.row(
                                                                            this.browser.view().rows[index]
                                                                                .clone(),
                                                                            cx,
                                                                        )
                                                                        .into_any_element()
                                                                    })
                                                                    .collect::<Vec<_>>()
                                                            })
                                                        }
                                                    })
                                                    .track_scroll(&self.scroll)
                                                    .size_full(),
                                                )
                                                .child(Scrollbar::vertical(&self.scroll).id("files-scrollbar"))
                                            })
                                            .when(
                                                browser.rows.is_empty(),
                                                |el| {
                                                    el.child(
                                                        v_flex()
                                                            .size_full()
                                                            .px(px(24.))
                                                            .py(px(20.))
                                                            .items_center()
                                                            .justify_center()
                                                            .gap(px(16.))
                                                            .text_color(rgb(p.muted))
                                                            .child(artwork(
                                                                if browser.directory.is_some() {
                                                                    ToolIcon::Open
                                                                } else {
                                                                    ToolIcon::History
                                                                },
                                                                window,
                                                                cx,
                                                            ))
                                                            .child(div().text_size(px(14.)).child(
                                                                if searching {
                                                                    tf(
                                                                        "search-empty",
                                                                        &[(
                                                                            "query",
                                                                            self.search
                                                                                .read(cx)
                                                                                .value()
                                                                                .to_string()
                                                                                .into(),
                                                                        )],
                                                                    )
                                                                } else if browser.catalog.is_none()
                                                                    && browser.directory.is_none()
                                                                {
                                                                    if self.tasks.is_busy() {
                                                                        self.tasks.status().to_owned()
                                                                    } else {
                                                                        tr("archive-empty").into()
                                                                    }
                                                                } else {
                                                                    tr("folder-empty").into()
                                                                },
                                                            ))
                                                            .when(
                                                                browser.catalog.is_none()
                                                                    && browser.directory.is_none()
                                                                    && !self.tasks.is_busy()
                                                                    && !searching,
                                                                |el| {
                                                                    el.child(
                                                                        primary(
                                                                            "empty-open",
                                                                            tr("archive-open"))
                                                                        .icon(icon(
                                                                            "FolderOpen",
                                                                            18.,
                                                                        ))
                                                                        .on_click(cx.listener(
                                                                            |this, _, _, cx| {
                                                                                this.open(cx)
                                                                            },
                                                                        )),
                                                                    )
                                                                },
                                                            )
                                                            .when(searching, |el| {
                                                                el.child(
                                                                    command(
                                                                        "empty-clear",
                                                                        tr("search-clear"))
                                                                    .on_click(cx.listener(
                                                                        |this, _, window, cx| {
                                                                            this.search.update(
                                                                                cx,
                                                                                |input, cx| {
                                                                                    input.set_value(
                                                                                        "", window,
                                                                                        cx,
                                                                                    )
                                                                                },
                                                                            )
                                                                        },
                                                                    )),
                                                                )
                                                            }),
                                                    )
                                                },
                                            )
                                            .on_mouse_down(
                                                MouseButton::Right,
                                                cx.listener(
                                                    |this, event: &MouseDownEvent, window, cx| {
                                                        cx.stop_propagation();
                                                        this.open_context_menu(
                                                            menus::context::Target::Background,
                                                            event.position,
                                                            window,
                                                            cx,
                                                        );
                                                    },
                                                ),
                                            )
                                            .overflow_hidden(),
                                    )
                                    })
                            })
                            .when(
                                self.tasks.extraction().is_none() && !home,
                                |el| el.child(self.statusbar(cx)),
                            ),
                    )
                    )
                    .when_some(self.settings_page, |panel, page| panel.child(self.settings_view(page, cx)))
                    .when_some(self.notification_stack(cx), |el, stack| el.child(stack))
                    .children(modal),
            )
            .when(self.dragging, |el| {
                el.child(
                    v_flex()
                        .absolute()
                        .inset(px(14.))
                        .items_center()
                        .justify_center()
                        .gap(px(20.))
                        .border_2()
                        .border_color(rgb(p.accent))
                        .rounded(px(6.))
                        .bg(rgb(p.selected))
                        .text_color(rgb(p.accent))
                        .child(icon("ArrowUpload", 36.))
                        .child(tr("drop-prompt")),
                )
            })
            .children(self.history_dropdown(window, cx))
            .child(self.menu_host.clone())
            .children(Root::render_dialog_layer(window, cx))
    }
}

impl Workspace {
    fn notification_stack(&self, cx: &Context<Self>) -> Option<Div> {
        let message = self.message.clone()?;
        Some(
            v_flex()
                .absolute()
                .top(px(12.))
                .left(px(12.))
                .right(px(12.))
                .items_end()
                .child(notification(
                    "message-notification",
                    div().whitespace_normal().child(message),
                    icon_button("dismiss-message", "Dismiss", tr("message-close"), false, cx)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.message = None;
                            cx.notify();
                        })),
                    cx,
                )),
        )
    }
}
