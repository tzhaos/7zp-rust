use cardo_ui::ConditionalBuilder;
use crate::theme::metrics::file_table as columns;
use crate::theme::metrics::home as metrics;
use crate::*;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, checkbox::Checkbox, h_flex, v_flex},
};

impl Workspace {
    pub(crate) fn file_header(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let view = self.browser.view();
        let all = !view.rows.is_empty() && view.selected.len() == view.rows.len();
        let busy = self.tasks.is_busy();
        let p = crate::theme::palette(cx);
        h_flex()
            .w_full()
            .h(px(columns::HEADER_HEIGHT))
            .flex_shrink_0()
            .border_b_1()
            .border_color(rgb(p.border))
            .child(
                h_flex()
                    .w(px(columns::SELECTION_WIDTH))
                    .flex_shrink_0()
                    .pl(px(10.))
                    .child(
                        Checkbox::new("select-visible-files")
                            .accessibility_label(tr(if all { "select-none" } else { "select-all" }))
                            .checked(all)
                            .disabled(busy || view.rows.is_empty())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.command(
                                    if all {
                                        commands::Command::DeselectAll
                                    } else {
                                        commands::Command::SelectAll
                                    },
                                    window,
                                    cx,
                                )
                            })),
                    ),
            )
            .children(
                [
                    (0, "name", None),
                    (1, "size", Some(columns::SIZE_WIDTH)),
                    (3, "type", Some(columns::TYPE_WIDTH)),
                    (2, "modified", Some(columns::MODIFIED_WIDTH)),
                ]
                .into_iter()
                .map(|(column, label, width)| {
                    let sorted = view.sort == column;
                    let direction = if view.descending {
                        "ChevronDown"
                    } else {
                        "ChevronUp"
                    };
                    let button = command(("file-column", column), tr(label))
                        .custom(subtle_variant(cx))
                        .border_0()
                        .rounded(px(0.))
                        .h(px(columns::HEADER_HEIGHT))
                        .w_full()
                        .px_0()
                        .disabled(busy)
                        .child(icon(if sorted { direction } else { "ArrowSort" }, 12.))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.command(commands::Command::Sort(column), window, cx)
                        }));
                    div()
                        .min_w_0()
                        .when_some(width, |el, width| el.w(px(width)).flex_shrink_0())
                        .when(width.is_none(), |el| el.flex_1())
                        .child(button)
                }),
            )
    }

    pub(crate) fn matching_recent_archives<'a>(
        &'a self,
        cx: &App,
    ) -> impl Iterator<Item = (usize, &'a PathBuf)> + use<'a> {
        let query = self.search.read(cx).value().to_lowercase();
        self.history
            .iter()
            .filter(|entry| entry.kind == recent::Kind::Archives)
            .map(|entry| &entry.path)
            .enumerate()
            .filter(move |(_, path)| path.to_string_lossy().to_lowercase().contains(&query))
    }

    pub(crate) fn recent_archives_view(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        v_flex().size_full().child(
            v_flex()
                .id("recent-list")
                .w_full()
                .flex_shrink_0()
                .children(
                    self.matching_recent_archives(cx)
                        .take(metrics::HISTORY_LIMIT)
                        .map(|(index, path)| {
                            let open_path = path.clone();
                            let remove_path = path.clone();
                            let context_path = path.clone();
                            let name = path
                                .file_name()
                                .unwrap_or(path.as_os_str())
                                .to_string_lossy();
                            list_row(("recent-row", index), false, self.tasks.is_busy(), cx)
                                .h(px(metrics::HISTORY_ROW_HEIGHT))
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                        cx.stop_propagation();
                                        this.open_context_menu(
                                            menus::context::Target::Recent(context_path.clone()),
                                            event.position,
                                            window,
                                            cx,
                                        );
                                    }),
                                )
                                .child(
                                    gpui_kit::component::button::Button::new((
                                        "recent-open",
                                        index,
                                    ))
                                    .custom(
                                        subtle_variant(cx)
                                            .hover(rgba(0x00000000).into())
                                            .active(rgba(0x00000000).into()),
                                    )
                                    .flex_1()
                                    .min_w_0()
                                    .h(px(metrics::HISTORY_ROW_HEIGHT))
                                    .px_0()
                                    .rounded(px(0.))
                                    .accessibility_label(name.to_string())
                                    .disabled(self.tasks.is_busy())
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .min_w_0()
                                            .pl(px(metrics::HISTORY_ROW_PADDING))
                                            .items_center()
                                            .opacity(if self.tasks.is_busy() { 0.4 } else { 1.0 })
                                            .child(list_entry(
                                                &name,
                                                file_icon(path, false, true),
                                                Some(
                                                    path.parent()
                                                        .unwrap_or(path)
                                                        .display()
                                                        .to_string(),
                                                ),
                                                cx,
                                            )),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            cx.stop_propagation();
                                            this.open_archive_path(open_path.clone(), window, cx);
                                        },
                                    )),
                                )
                                .child(
                                    div()
                                        .w(px(metrics::HISTORY_REMOVE_WIDTH))
                                        .flex_shrink_0()
                                        .child(
                                            icon_button(
                                                ("recent-remove", index),
                                                "Dismiss",
                                                tr("recent-remove"),
                                                self.allow_hint(!self.tasks.is_busy()),
                                                cx,
                                            )
                                            .custom(subtle_variant(cx))
                                            .border_0()
                                            .disabled(self.tasks.is_busy())
                                            .on_click(
                                                cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.update_recent(
                                                        recent::Change::Remove(remove_path.clone()),
                                                        cx,
                                                    );
                                                }),
                                            ),
                                        ),
                                )
                        }),
                ),
        )
    }

    pub(crate) fn row(&self, entry: Entry, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
        let selected = self.browser.view().selected.contains(&entry.path);
        let path = entry.path.clone();
        let check = path.clone();
        let context_path = path.clone();
        let directory = entry.directory;
        list_row(SharedString::from(path.clone()), selected, false, cx)
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                cx.stop_propagation();
                this.focus.focus(window, cx);
                if event.click_count() == 2 {
                    this.open_entry(path.clone(), window, cx);
                } else {
                    this.select(path.clone(), event.modifiers(), cx);
                }
            }))
            .child(
                div()
                    .w(px(columns::SELECTION_WIDTH))
                    .flex_shrink_0()
                    .pl(px(10.))
                    .child(
                        Checkbox::new(SharedString::from(format!("check:{check}")))
                            .checked(selected)
                            .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                cx.stop_propagation();
                                this.browser.toggle_selection(check.clone(), true);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                list_entry(
                    &entry.name,
                    file_icon(
                        Path::new(&entry.path),
                        directory,
                        self.browser.view().directory.is_some(),
                    ),
                    None,
                    cx,
                )
                .when(entry.encrypted, |el| el.child(icon("LockClosed", 12.))),
            )
            .child(
                compact_text(
                    "entry-size",
                    if directory {
                        "—".into()
                    } else {
                        size_text(entry.size.unwrap_or(0))
                    },
                )
                .w(px(columns::SIZE_WIDTH))
                .flex_shrink_0()
                .text_size(crate::theme::ui_font_size(cx))
                .text_color(rgb(p.muted)),
            )
            .child(
                compact_text("entry-type", file_kind(&entry.name, directory))
                    .w(px(columns::TYPE_WIDTH))
                    .flex_shrink_0()
                    .text_size(crate::theme::ui_font_size(cx))
                    .text_color(rgb(p.muted)),
            )
            .child(
                compact_text(
                    "entry-modified",
                    if entry.modified.is_empty() {
                        "—".into()
                    } else {
                        entry.modified.replace('-', "/")
                    },
                )
                .w(px(columns::MODIFIED_WIDTH))
                .flex_shrink_0()
                .text_size(crate::theme::ui_font_size(cx))
                .text_color(rgb(p.muted)),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.open_context_menu(
                        menus::context::Target::Item(context_path.clone()),
                        event.position,
                        window,
                        cx,
                    );
                }),
            )
    }
}
