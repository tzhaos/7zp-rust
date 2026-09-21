mod addressbar;
mod files;
mod home;
mod settings;
mod statusbar;
mod titlebar;
mod toolbar;

use crate::*;
use gpui_kit::{
    component::{Root, h_flex, v_flex},
    prelude::FluentBuilder,
};
use cardo_7zp_core::i18n::{tf, tr};
use std::{cell::Cell, rc::Rc};

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_view(window, cx);
        let p = crate::theme::palette(cx);
        let modal = self.modal(window, cx);
        let settings_page = self.settings_page(cx);
        let searching = !self.search.read(cx).value().is_empty();
        let browser = self.browser.view();
        let home = browser.catalog.is_none() && browser.directory.is_none();
        let has_recent_matches = self.matching_recent_archives(cx).next().is_some();
        let page_direction = self.page_direction;
        let active_center = Rc::new(Cell::new(px(0.)));
        v_flex()
            .id("workspace")
            .track_focus(&self.focus)
            .relative()
            .size_full()
            .bg(rgb(p.panel))
            .text_color(rgb(p.text))
            .font_family("Microsoft YaHei UI")
            .text_size(px(13.))
            .on_key_down(cx.listener(Self::keyboard))
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
            .child(self.toolbar(active_center.clone(), window, cx))
            .child(
                v_flex()
                    .id("content-panel")
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .mx(px(12.))
                    .mb(px(12.))
                    .rounded(px(crate::theme::metrics::PANEL_RADIUS))
                    .p(px(8.))
                    .bg(rgb(p.surface))
                    .border_1()
                    .border_color(rgb(p.border))
                    .shadow_sm()
                    .child(panel_pointer(active_center, cx))
                    .child(
                        v_flex()
                            .size_full()
                            .min_h_0()
                            .overflow_hidden()
                            .rounded(px(6.))
                            .when_some(settings_page, |el, page| el.child(page))
                            .when(self.page == Page::Files, |el| {
                                el.when(home, |el| el.child(self.home_view(cx)))
                                    .when(!home, |el| {
                                        el.child(self.pathbar(window, cx))
                                    .child(
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
                                            })
                                            .when(home && has_recent_matches, |el| {
                                                el.child(self.recent_archives_view(cx))
                                            })
                                            .when(
                                                browser.rows.is_empty()
                                                    && (!home || !has_recent_matches),
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
                                            .with_animation(
                                                ("files-page-transition", self.page_revision),
                                                Animation::new(std::time::Duration::from_millis(
                                                    180,
                                                ))
                                                .with_easing(ease_out_quint()),
                                                move |el, delta| {
                                                    el.left(px((1.0 - delta)
                                                        * page_direction
                                                        * 24.))
                                                        .opacity(delta)
                                                },
                                            ),
                                    )
                                    })
                            })
                            .when_some(
                                self.completion.clone().filter(|_| self.page == Page::Files),
                                |el, (label, path)| {
                                    el.child(
                                        h_flex()
                                            .flex_shrink_0()
                                            .px(px(16.))
                                            .py(px(8.))
                                            .gap(px(10.))
                                            .items_center()
                                            .bg(rgb(p.surface))
                                            .border_t_1()
                                            .border_color(rgb(p.border))
                                            .text_color(rgb(p.text))
                                            .child(
                                                filled_icon("CheckmarkCircle", 20.)
                                                    .text_color(rgb(p.success)),
                                            )
                                            .child(
                                                h_flex()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .gap(px(12.))
                                                    .child(
                                                        div()
                                                            .flex_shrink_0()
                                                            .text_size(px(13.))
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .child(label),
                                                    )
                                                    .child(div().flex_1().min_w_0().child(
                                                        path_strip(
                                                            "copy-completion-path",
                                                            path.display().to_string(),
                                                            cx,
                                                        ),
                                                    )),
                                            )
                                            .child(
                                                command("reveal", tr("folder-open"))
                                                    .icon(icon("FolderOpen", 15.))
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            match cardo_7zp_platform::open_directory(
                                                                &path,
                                                            ) {
                                                                Ok(true) => {}
                                                                Ok(false) => cx.reveal_path(&path),
                                                                Err(error) => {
                                                                    this.message =
                                                                        Some(error.to_string());
                                                                    cx.notify();
                                                                }
                                                            }
                                                        },
                                                    )),
                                            )
                                            .child(
                                                icon_button(
                                                    "dismiss-completion",
                                                    "Dismiss",
                                                    tr("completion-close"),
                                                    cx,
                                                )
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.completion = None;
                                                    cx.notify();
                                                })),
                                            ),
                                    )
                                },
                            )
                            .when(
                                self.page == Page::Files && self.tasks.extraction().is_none() && !home,
                                |el| el.child(self.statusbar(cx)),
                            ),
                    ),
            )
            .when_some(self.message.clone(), |el, message| {
                el.child(
                    h_flex()
                        .absolute()
                        .bottom(px(48.))
                        .left(px(24.))
                        .w(px(520.))
                        .max_w(window.viewport_size().width - px(48.))
                        .px_4()
                        .py_2()
                        .gap_3()
                        .bg(rgb(p.surface))
                        .border_1()
                        .border_color(rgb(p.border))
                        .rounded(px(4.))
                        .shadow_md()
                        .child(
                            div()
                                .id("notification-body")
                                .flex_1()
                                .min_w_0()
                                .max_h(px(96.))
                                .overflow_scroll()
                                .text_size(px(12.))
                                .child(div().whitespace_normal().child(message)),
                        )
                        .child(
                            icon_button("dismiss-message", "Dismiss", tr("message-close"), cx)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.message = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .children(modal)
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
            .when_some(self.context_menu.clone(), |el, popup| {
                el.child(
                    deferred(
                        anchored()
                            .position(self.context_menu_position)
                            .snap_to_window_with_margin(px(8.))
                            .child(popup),
                    )
                    .with_priority(gpui_kit::base::POPUP_PRIORITY),
                )
            })
            .children(Root::render_dialog_layer(window, cx))
    }
}
