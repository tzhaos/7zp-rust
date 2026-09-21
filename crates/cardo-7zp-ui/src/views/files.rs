use crate::theme::metrics::home as metrics;
use crate::*;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, checkbox::Checkbox, h_flex, v_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn matching_recent_archives<'a>(
        &'a self,
        cx: &App,
    ) -> impl Iterator<Item = (usize, &'a PathBuf)> + use<'a> {
        let query = self.search.read(cx).value().to_lowercase();
        self.recent_archives
            .iter()
            .enumerate()
            .filter(move |(_, path)| path.to_string_lossy().to_lowercase().contains(&query))
    }

    pub(crate) fn recent_archives_view(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        v_flex().size_full().child(
            v_flex()
                .id("recent-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .children(self.matching_recent_archives(cx).map(|(index, path)| {
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
                            gpui_kit::component::button::Button::new(("recent-open", index))
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
                                                path.parent().unwrap_or(path).display().to_string(),
                                            ),
                                            cx,
                                        )),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.show_page(Page::Files, window, cx);
                                    this.execute(
                                        Request::Open(open_path.clone()),
                                        String::new(),
                                        cx,
                                    );
                                })),
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
                                        cx,
                                    )
                                    .custom(subtle_variant(cx))
                                    .border_0()
                                    .disabled(self.tasks.is_busy())
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            this.update_recent(
                                                recent::Change::Remove(remove_path.clone()),
                                                cx,
                                            );
                                        },
                                    )),
                                ),
                        )
                })),
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
                div().w(px(45.)).flex_shrink_0().pl(px(18.)).child(
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
                    (!self.search.read(cx).value().is_empty()).then(|| entry.path.clone()),
                    cx,
                )
                .when(entry.encrypted, |el| el.child(icon("LockClosed", 12.))),
            )
            .child(
                div()
                    .w(px(108.))
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(rgb(p.muted))
                    .child(if directory {
                        "—".into()
                    } else {
                        size_text(entry.size.unwrap_or(0))
                    }),
            )
            .child(
                div()
                    .w(px(120.))
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(rgb(p.muted))
                    .child(file_kind(&entry.name, directory)),
            )
            .child(
                div()
                    .w(px(166.))
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(rgb(p.muted))
                    .child(if entry.modified.is_empty() {
                        "—".into()
                    } else {
                        entry
                            .modified
                            .chars()
                            .take(16)
                            .collect::<String>()
                            .replace('-', "/")
                    }),
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
