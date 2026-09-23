use crate::commands::Command;
use crate::*;
use gpui_kit::base::ElementExt;
use gpui_kit::component::{button::ButtonVariants, h_flex};

#[derive(Clone, Copy)]
pub(crate) enum MenuGroup {
    File,
    Operations,
    Tools,
}

impl Workspace {
    pub(crate) fn menubar(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        h_flex()
            .id("titlebar-menus")
            .h(px(36.))
            .max_w(px(crate::theme::metrics::titlebar::MENU_MAX_WIDTH))
            .flex_shrink_0()
            .px(px(8.))
            .gap(px(2.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .children(
                [
                    (MenuGroup::File, "menu-file"),
                    (MenuGroup::Operations, "menu-operations"),
                    (MenuGroup::Tools, "menu-tools"),
                ]
                .into_iter()
                .map(|(group, label)| {
                    let bounds =
                        std::rc::Rc::new(std::cell::Cell::new(Bounds::<Pixels>::default()));
                    let measured = bounds.clone();
                    command(label, tr(label))
                        .h(px(28.))
                        .px(px(8.))
                        .custom(header_variant(cx))
                        .border_0()
                        .on_prepaint(move |rect, _, _| measured.set(rect))
                        .on_click({
                            let owner = cx.entity().downgrade();
                            move |_, window, cx| {
                                let anchor = bounds.get();
                                build_menu(&owner, group, cx).show_native(
                                    point(anchor.left(), anchor.bottom()),
                                    window,
                                    cx,
                                );
                            }
                        })
                }),
            )
    }
}

pub(crate) fn menu_items(group: MenuGroup) -> &'static [(&'static str, &'static str, Command)] {
    use Command::*;
    match group {
        MenuGroup::File => &[
            ("archive-open", "Ctrl+O", Open),
            ("archive-save-as", "Ctrl+Shift+S", Save),
            ("", "", Open),
            ("recent-clear", "", ClearRecent),
            ("", "", Open),
            ("menu-exit", "Alt+F4", Exit),
        ],
        MenuGroup::Operations => &[
            ("extract-options", "Alt+E", Extract),
            ("archive-add", "Alt+A", Add),
            ("file-properties", "Alt+I", ArchiveInfo),
            ("archive-comment", "Alt+M", Comment),
            ("archive-check", "Alt+T", Check),
        ],
        MenuGroup::Tools => &[
            (
                "settings-general",
                "Ctrl+,",
                Settings(preferences::Tab::Application),
            ),
            (
                "settings-integration",
                "",
                Settings(preferences::Tab::Integration),
            ),
            (
                "settings-advanced",
                "",
                Settings(preferences::Tab::Advanced),
            ),
            ("menu-about", "", About),
        ],
    }
}

pub(crate) fn build_menu(owner: &WeakEntity<Workspace>, group: MenuGroup, cx: &mut App) -> Menu {
    use Command::*;
    let mut menu = Menu::new();
    let items = menu_items(group);
    let snapshot = owner
        .read_with(cx, |this, _| {
            (
                this.browser.location(),
                this.browser.view().selected.clone(),
            )
        })
        .ok();
    for &(label, shortcut, action) in items {
        if matches!(action, ClearRecent) {
            menu = history_items(owner, menu, cx);
        }
        if label.is_empty() {
            menu = menu.separator();
            continue;
        }
        let (enabled, checked) = owner
            .read_with(cx, |this, cx| {
                let view = this.browser.view();
                let relevant = match action {
                    ClearRecent => !this.history.is_empty(),
                    SelectAll => !view.rows.is_empty() && view.selected.len() < view.rows.len(),
                    DeselectAll => !view.selected.is_empty(),
                    InvertSelection | Sort(_) => !view.rows.is_empty(),
                    QuickExtractSelection => !view.selected.is_empty(),
                    _ => true,
                };
                (
                    relevant && this.command_available(action, cx),
                    matches!(action, Sort(column) if view.sort == column),
                )
            })
            .unwrap_or((false, false));
        let callback_owner = owner.clone();
        let snapshot = snapshot.clone();
        menu = menu.item(
            menu_command_item(tr(label), shortcut)
                .disabled(!enabled)
                .checked(checked)
                .on_select(move |window, cx| {
                    let _ = callback_owner.update(cx, |this, cx| {
                        if let Some((location, selected)) = &snapshot {
                            if this.browser.location() != *location
                                || this.tasks.is_busy()
                                || this.dialogs.is_open()
                                || this.settings_busy(cx)
                            {
                                return;
                            }
                            this.browser.restore_selection(selected.clone());
                        }
                        this.command(action, window, cx);
                    });
                }),
        );
    }
    menu
}

fn history_items(owner: &WeakEntity<Workspace>, mut menu: Menu, cx: &App) -> Menu {
    let Ok((entries, enabled)) = owner.read_with(cx, |this, cx| {
        (
            this.history
                .iter()
                .filter(|entry| entry.kind == recent::Kind::Archives)
                .take(crate::theme::metrics::home::HISTORY_LIMIT)
                .cloned()
                .collect::<Vec<_>>(),
            this.command_available(Command::Open, cx),
        )
    }) else {
        return menu;
    };
    for (index, entry) in entries.into_iter().enumerate() {
        let owner = owner.clone();
        menu = menu.item(
            MenuItem::new(tf(
                "menu-history-entry",
                &[
                    ("number", (index + 1).into()),
                    ("path", entry.path.display().to_string().into()),
                ],
            ))
            .disabled(!enabled)
            .on_select(move |window, cx| {
                let _ = owner.update(cx, |this, cx| {
                    if this.command_available(Command::Open, cx) {
                        this.show_browser(window, cx);
                        this.visit_history(entry.clone(), window, cx);
                    }
                });
            }),
        );
    }
    menu
}
