use crate::commands::Command;
use crate::*;
use gpui_kit::base::ElementExt;
use gpui_kit::component::{button::ButtonVariants, h_flex};

#[derive(Clone, Copy)]
pub(crate) enum MenuGroup {
    File,
    Edit,
    View,
    Archive,
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
                    (MenuGroup::Edit, "menu-edit"),
                    (MenuGroup::View, "menu-view"),
                    (MenuGroup::Archive, "menu-archive"),
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
            ("archive-create-command", "", Create),
            ("", "", Open),
            ("browser-open", "Enter", OpenItem),
            ("file-open-inside", "Ctrl+PgDn", OpenInside),
            ("file-open-outside", "Shift+Enter", OpenOutside),
            ("file-properties", "Alt+Enter", Properties),
            ("", "", Open),
            ("archive-close", "", Close),
            ("", "", Open),
            (
                "settings-title",
                "Ctrl+,",
                Settings(preferences::Tab::Application),
            ),
            ("menu-exit", "Alt+F4", Exit),
        ],
        MenuGroup::Edit => &[
            ("archive-rename", "F2", Rename),
            ("archive-copy", "F5", CopyTo),
            ("archive-move", "F6", MoveTo),
            ("archive-delete", "Del", Delete),
        ],
        MenuGroup::View => &[("menu-refresh", "F5", Refresh)],
        MenuGroup::Archive => &[
            ("archive-add", "Alt+A", Add),
            ("archive-add-folder", "", AddFolder),
            ("", "", Add),
            ("archive-comment", "Ctrl+Z", Comment),
            ("archive-save-as", "Ctrl+Shift+S", Save),
            ("", "", Save),
            ("archive-info", "", ArchiveInfo),
        ],
        MenuGroup::Tools => &[
            ("extract-options", "Alt+E", Extract),
            ("extract-all-quick", "", QuickExtract),
            ("extract-selected", "", QuickExtractSelection),
            ("", "", Extract),
            ("archive-check", "Alt+T", Check),
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
        if label.is_empty() {
            menu = menu.separator();
            continue;
        }
        let (enabled, checked) = owner
            .read_with(cx, |this, cx| {
                let view = this.browser.view();
                let relevant = match action {
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
        match (group, action) {
            (MenuGroup::Archive, ArchiveInfo) => {
                menu = menus::archive::open_as_menu(owner, menu, cx);
            }
            (MenuGroup::Tools, Check) => {
                menu = menus::archive::checksum_menu(owner, menu, cx);
            }
            _ => {}
        }
    }
    menu
}
