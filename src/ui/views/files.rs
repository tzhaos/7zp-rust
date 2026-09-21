use crate::ui::*;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, checkbox::Checkbox, h_flex, v_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(in crate::ui) fn home_view(&self, cx: &mut Context<Self>) -> Div {
        let p = crate::ui::theme::palette(cx);
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap(px(16.))
            .child(img("brand/logo.png").size(px(88.)).flex_shrink_0())
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("7zplus"),
            )
            .child(
                primary("home-open", tr("archive-open"))
                    .icon(icon("FolderOpen", 17.))
                    .on_click(cx.listener(|this, _, _, cx| this.open(cx))),
            )
            .when(!self.recent_archives.is_empty(), |el| {
                el.child(
                    div()
                        .w(px(560.))
                        .max_w(relative(0.85))
                        .h(px(180.))
                        .child(self.recent_archives_view(cx)),
                )
            })
            .text_color(rgb(p.text))
    }

    pub(in crate::ui) fn matching_recent_archives<'a>(
        &'a self,
        cx: &App,
    ) -> impl Iterator<Item = (usize, &'a PathBuf)> + use<'a> {
        let query = self.search.read(cx).value().to_lowercase();
        self.recent_archives
            .iter()
            .enumerate()
            .filter(move |(_, path)| path.to_string_lossy().to_lowercase().contains(&query))
    }

    pub(in crate::ui) fn recent_archives_view(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
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
                    list_row(("recent-row", index), false, self.busy, cx)
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                cx.stop_propagation();
                                this.open_context_menu(
                                    menu::Target::Recent(context_path.clone()),
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
                                .h(px(38.))
                                .px_0()
                                .rounded(px(0.))
                                .accessibility_label(name.to_string())
                                .disabled(self.busy)
                                .child(
                                    h_flex()
                                        .w_full()
                                        .min_w_0()
                                        .pl(px(45.))
                                        .items_center()
                                        .opacity(if self.busy { 0.4 } else { 1.0 })
                                        .child(list_entry(
                                            &name,
                                            file_icon(path, false, true),
                                            None,
                                            cx,
                                        ))
                                        .child(
                                            div()
                                                .w(px(240.))
                                                .flex_shrink_0()
                                                .pr(px(16.))
                                                .truncate()
                                                .text_size(px(12.))
                                                .text_color(rgb(p.muted))
                                                .child(
                                                    path.parent()
                                                        .unwrap_or(path)
                                                        .display()
                                                        .to_string(),
                                                ),
                                        ),
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
                            div().w(px(46.)).flex_shrink_0().child(
                                icon_button(
                                    ("recent-remove", index),
                                    "Dismiss",
                                    tr("recent-remove"),
                                    cx,
                                )
                                .custom(subtle_variant(cx))
                                .border_0()
                                .disabled(self.busy)
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

    pub(in crate::ui) fn row(
        &self,
        entry: Entry,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::ui::theme::palette(cx);
        let selected = self.selected.contains(&entry.path);
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
                            if !this.selected.insert(check.clone()) {
                                this.selected.remove(&check);
                            }
                            this.anchor = Some(check.clone());
                            cx.notify();
                        })),
                ),
            )
            .child(
                list_entry(
                    &entry.name,
                    file_icon(Path::new(&entry.path), directory, self.directory.is_some()),
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
                        menu::Target::Item(context_path.clone()),
                        event.position,
                        window,
                        cx,
                    );
                }),
            )
    }
}
