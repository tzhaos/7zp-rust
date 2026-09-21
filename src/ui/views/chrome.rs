use crate::i18n::{tf, tr};
use crate::ui::theme::metrics::*;
use crate::ui::*;
use gpui_kit::base::ElementExt;
use gpui_kit::{
    component::{Disableable, Selectable, button::ButtonVariants, h_flex, menu::DropdownMenu},
    prelude::FluentBuilder,
};
use std::{cell::Cell, rc::Rc};

impl Workspace {
    pub(in crate::ui) fn titlebar(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
        h_flex()
            .h(px(36.))
            .flex_shrink_0()
            .bg(rgb(p.panel))
            .items_center()
            .child(
                h_flex()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .pl(px(16.))
                    .gap(px(12.))
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("7z+"),
                    )
                    .child(self.menubar(cx))
                    .child(div().flex_1())
                    .when_some(self.catalog.as_ref(), |el, c| {
                        el.child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(rgb(p.selected))
                                .text_size(px(10.))
                                .child(c.format.clone()),
                        )
                    })
                    .when(
                        self.catalog
                            .as_ref()
                            .is_some_and(|c| c.entries.iter().any(|e| e.encrypted)),
                        |el| el.child(icon("LockClosed", 14.)),
                    ),
            )
            .child(
                h_flex().gap_0().children(
                    [
                        ("minimize", "Subtract", tr("window-minimize")),
                        (
                            "maximize",
                            if window.is_maximized() {
                                "SquareMultiple"
                            } else {
                                "Square"
                            },
                            tr("window-maximize"),
                        ),
                        ("close", "Dismiss", tr("window-close")),
                    ]
                    .into_iter()
                    .map(|(id, name, label)| {
                        let disabled = id == "close" && (self.busy || self.settings_busy(cx));
                        bubble_tooltip(gpui_kit::component::button::Button::new(id), label)
                            .group("window-control")
                            .custom(if id == "close" {
                                subtle_variant(cx)
                                    .hover(rgb(0xe81123).into())
                                    .active(rgb(0xc50f1f).into())
                            } else {
                                subtle_variant(cx)
                            })
                            .accessibility_label(label)
                            .child(
                                div()
                                    .when(id == "close" && !disabled, |el| {
                                        el.group_hover("window-control", |el| {
                                            el.text_color(rgb(0xffffff))
                                        })
                                    })
                                    .child(icon(name, 16.)),
                            )
                            .border_0()
                            .w(px(42.))
                            .h(px(36.))
                            .rounded(px(0.))
                            .disabled(disabled)
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_click(move |_, window, _| match id {
                                "minimize" => window.minimize_window(),
                                "maximize" => window.zoom_window(),
                                _ => window.remove_window(),
                            })
                    }),
                ),
            )
    }

    pub(in crate::ui) fn toolbar(
        &self,
        active_center: Rc<Cell<Pixels>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
        let blocked = self.busy
            || self.modal.is_some()
            || self.settings_busy(cx)
            || self.preferences_task.is_some();
        let inactive = blocked || self.catalog.is_none() || self.page != Page::Files;
        // Reserve the left actions, toolbar padding/gaps, toggle and divider.
        let fixed_width = 4. * TOOL_MIN_WIDTH + 2. * TOOLBAR_PADDING + 8. * TOOL_GAP + 28.;
        let item_width = ((f32::from(window.viewport_size().width) - fixed_width) / 5. - TOOL_GAP)
            .clamp(TOOL_MIN_WIDTH, TOOL_MAX_WIDTH);
        h_flex()
            .relative()
            .h(px(TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(TOOLBAR_PADDING))
            .gap(px(TOOL_GAP))
            .items_center()
            .bg(rgb(p.panel))
            .child(
                tool("open", ToolIcon::Open, tr("open"), blocked, cx).on_click(cx.listener(
                    |this, _, window, cx| {
                        this.show_page(Page::Files, window, cx);
                        this.open(cx);
                    },
                )),
            )
            .child(
                tool(
                    "extract-options",
                    ToolIcon::Extract,
                    tr("extract-options"),
                    inactive,
                    cx,
                )
                .on_click(cx.listener(|this, _, window, cx| this.extract_dialog(window, cx))),
            )
            .child(
                tool(
                    "extract",
                    ToolIcon::QuickExtract,
                    if self.selected.is_empty() {
                        tr("extract-quick")
                    } else {
                        tr("extract-selected")
                    },
                    inactive,
                    cx,
                )
                .on_click(
                    cx.listener(|this, _, _, cx| this.quick_extract(!this.selected.is_empty(), cx)),
                ),
            )
            .child(
                tool("check", ToolIcon::Check, tr("check"), inactive, cx).on_click(cx.listener(
                    |this, _, _, cx| {
                        if let Some(catalog) = this.catalog.clone() {
                            this.execute(Request::Check(catalog), this.password.clone(), cx);
                        }
                    },
                )),
            )
            .child(div().flex_1())
            .child(
                icon_button(
                    "toolbar-toggle",
                    if self.tools_expanded {
                        "ArrowRight"
                    } else {
                        "ArrowLeft"
                    },
                    tr(if self.tools_expanded {
                        "toolbar-collapse"
                    } else {
                        "toolbar-expand"
                    }),
                    cx,
                )
                .custom(subtle_variant(cx))
                .border_0()
                .w(px(28.))
                .h(px(30.))
                .flex_shrink_0()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.tools_expanded = !this.tools_expanded;
                    cx.notify();
                })),
            )
            .child(
                div()
                    .w(px(1.))
                    .h(px(32.))
                    .flex_shrink_0()
                    .mx(px(2.))
                    .bg(rgb(p.border)),
            )
            .child(
                h_flex().h(px(TOOL_HEIGHT)).flex_shrink_0().children(
                    [
                        ("files", ToolIcon::Files, Page::Files),
                        (
                            "general",
                            ToolIcon::General,
                            Page::Settings(preferences::Tab::General),
                        ),
                        (
                            "appearance",
                            ToolIcon::Appearance,
                            Page::Settings(preferences::Tab::Appearance),
                        ),
                        (
                            "associations",
                            ToolIcon::Associations,
                            Page::Settings(preferences::Tab::Associations),
                        ),
                        (
                            "advanced",
                            ToolIcon::Advanced,
                            Page::Settings(preferences::Tab::Advanced),
                        ),
                    ]
                    .into_iter()
                    .map(|(id, icon, page)| {
                        let visible = self.tools_expanded || self.page == page;
                        let openness = gpui_kit::base::spring(
                            (id, "toolbar-width"),
                            if visible { 1.0_f32 } else { 0.0_f32 },
                            gpui_kit::base::Spring::new(std::time::Duration::from_millis(300)),
                            window,
                            cx,
                        );
                        let label = match page {
                            Page::Files => tr("files-view"),
                            Page::Settings(preferences::Tab::Associations) => {
                                tr("settings-associations-short")
                            }
                            Page::Settings(tab) => tab.label(),
                        };
                        let button = tool(id, icon, label, blocked, cx)
                            .disabled(blocked || !visible)
                            .relative()
                            .flex_none()
                            .w(px(item_width))
                            .selected(self.page == page)
                            .when(self.page == page, |el| {
                                let center = active_center.clone();
                                el.on_prepaint(move |bounds, _, _| center.set(bounds.center().x))
                            })
                            .on_click(cx.listener(move |this, _, window, cx| match page {
                                Page::Files => this.show_page(page, window, cx),
                                Page::Settings(tab) => this.preferences_category(tab, window, cx),
                            }));
                        div()
                            .id(("toolbar-slot", page.order() as u32))
                            .h(px(TOOL_HEIGHT))
                            .w(px((item_width + TOOL_GAP) * openness))
                            .flex_shrink_0()
                            .overflow_hidden()
                            .when(openness > 0., |el| el.child(button))
                    }),
                ),
            )
    }

    pub(in crate::ui) fn pathbar(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
        let owner = cx.entity().downgrade();
        let folders = self.recent_folders.clone();
        let archives = self.recent_archives.clone();
        let current_path = self.address_text();
        navigation_row(cx)
            .child(
                icon_button("back", "ArrowLeft", tr("back"), cx)
                    .w(px(30.))
                    .h(px(30.))
                    .disabled(self.busy || self.history.is_empty())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.back(window, cx);
                    })),
            )
            .child(
                icon_button("up", "ArrowUp", tr("parent-folder"), cx)
                    .w(px(30.))
                    .h(px(30.))
                    .disabled(self.busy || self.parent_location().is_none())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.up(window, cx);
                    })),
            )
            .child(
                div().flex_1().min_w_0().h(px(30.)).child(
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
                        .disabled(self.busy)
                        .suffix(
                            icon_button(
                                "recent-locations",
                                "ChevronDown",
                                tr("browser-location-menu"),
                                cx,
                            )
                            .w(px(24.))
                            .h(px(24.))
                            .disabled(self.busy)
                            .dropdown_menu_with_anchor(
                                Anchor::TopRight,
                                move |menu, _, _| {
                                    let mut menu = menu_style(menu)
                                        .min_w(px(460.))
                                        .max_w(px(460.))
                                        .max_h(px(420.))
                                        .scrollable(true);
                                    let browse = owner.clone();
                                    let copy_path = current_path.clone();
                                    menu = menu
                                        .item(PopupMenuItem::new(tr("browser-browse")).on_click(
                                            move |_, _, cx| {
                                                let _ =
                                                    browse.update(cx, |this, cx| this.browse(cx));
                                            },
                                        ))
                                        .item(
                                            PopupMenuItem::new(tr("browser-copy-path"))
                                                .disabled(copy_path.is_empty())
                                                .on_click(move |_, _, cx| {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(
                                                            copy_path.clone(),
                                                        ),
                                                    )
                                                }),
                                        );
                                    if !folders.is_empty() || !archives.is_empty() {
                                        menu = menu.separator();
                                    }
                                    for (title, locations) in [
                                        (
                                            tr("browser-recent-folders"),
                                            folders
                                                .iter()
                                                .cloned()
                                                .map(browser::Location::Directory)
                                                .collect::<Vec<_>>(),
                                        ),
                                        (
                                            tr("recent-title"),
                                            archives
                                                .iter()
                                                .cloned()
                                                .map(|p| {
                                                    browser::Location::Archive(p, String::new())
                                                })
                                                .collect(),
                                        ),
                                    ] {
                                        if locations.is_empty() {
                                            continue;
                                        }
                                        menu = menu.label(title);
                                        for location in locations {
                                            let path = match &location {
                                                browser::Location::Directory(path)
                                                | browser::Location::Archive(path, _) => path,
                                                browser::Location::Home => continue,
                                            };
                                            let owner = owner.clone();
                                            menu = menu.item(
                                                path_menu_item(path.display().to_string(), 440.)
                                                    .on_click(move |_, window, cx| {
                                                        let _ = owner.update(cx, |this, cx| {
                                                            this.visit(location.clone(), window, cx)
                                                        });
                                                    }),
                                            );
                                        }
                                    }
                                    menu
                                },
                            ),
                        ),
                ),
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
                                    icon_button("clear-search", "Dismiss", tr("search-clear"), cx)
                                        .w(px(24.))
                                        .h(px(24.))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.search.update(cx, |input, cx| {
                                                input.set_value("", window, cx)
                                            })
                                        })),
                                )
                            }),
                    ),
            )
    }

    pub(in crate::ui) fn statusbar(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
        let count = if self.catalog.is_none() && self.directory.is_none() {
            self.matching_recent_archives(cx).count()
        } else {
            self.rows.len()
        };
        let summary = if self.busy {
            self.status.clone()
        } else if !self.selected.is_empty() {
            let selected_size =
                self.catalog
                    .as_ref()
                    .map(|c| {
                        c.entries
                            .iter()
                            .filter(|e| {
                                !e.directory
                                    && self.selected.iter().any(|p| {
                                        e.path == *p || e.path.starts_with(&format!("{p}/"))
                                    })
                            })
                            .map(|e| e.size.unwrap_or(0))
                            .sum()
                    })
                    .unwrap_or_else(|| {
                        self.rows
                            .iter()
                            .filter(|e| self.selected.contains(&e.path))
                            .filter_map(|e| e.size)
                            .sum()
                    });
            tf(
                "selection-summary",
                &[
                    ("count", self.selected.len().into()),
                    ("size", size_text(selected_size).into()),
                ],
            )
        } else {
            tf(
                if self.search.read(cx).value().is_empty() {
                    "status-items"
                } else {
                    "status-results"
                },
                &[("count", count.into())],
            )
        };
        h_flex()
            .relative()
            .h(px(36.))
            .flex_shrink_0()
            .px(px(16.))
            .gap(px(8.))
            .items_center()
            .bg(rgb(p.surface))
            .border_t_1()
            .border_color(rgb(p.border))
            .text_size(px(11.))
            .text_color(rgb(p.muted))
            .child(div().flex_1().min_w_0().truncate().child(summary))
            .when(!self.busy && !self.selected.is_empty(), |el| {
                el.child(
                    icon_button("clear-selection", "Dismiss", tr("selection-clear"), cx)
                        .custom(subtle_variant(cx))
                        .border_0()
                        .w(px(24.))
                        .h(px(24.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.selected.clear();
                            cx.notify();
                        })),
                )
            })
            .when(self.busy, |el| {
                el.child(
                    command("cancel", tr("cancel"))
                        .custom(subtle_variant(cx))
                        .border_0()
                        .h(px(24.))
                        .flex_shrink_0()
                        .disabled(self.cancel.load(Ordering::Relaxed))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.cancel.store(true, Ordering::Relaxed);
                            this.status = tr("cancelling").into();
                            cx.notify();
                        })),
                )
            })
            .when(self.busy, |el| {
                el.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top_0()
                        .h(px(2.))
                        .overflow_hidden()
                        .bg(rgb(p.selected))
                        .child(
                            div()
                                .absolute()
                                .h_full()
                                .w(relative(0.3))
                                .bg(rgb(p.accent))
                                .with_animation(
                                    "operation-progress",
                                    Animation::new(std::time::Duration::from_millis(1500)).repeat(),
                                    |el, delta| el.left(relative(delta * 1.3 - 0.3)),
                                ),
                        ),
                )
            })
    }
}
