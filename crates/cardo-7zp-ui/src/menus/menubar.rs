use crate::commands::Command;
use crate::*;
use gpui_kit::component::{button::ButtonVariants, h_flex, menu::DropdownMenu};

#[derive(Clone, Copy)]
enum MenuGroup {
    File,
    Archive,
    Edit,
    View,
    Tools,
    Help,
}

impl Workspace {
    pub(crate) fn menubar(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        let owner = cx.entity().downgrade();
        h_flex()
            .id("titlebar-menus")
            .h(px(36.))
            .flex_shrink_0()
            .px(px(8.))
            .gap(px(2.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .children(
                [
                    (MenuGroup::File, "menu-file"),
                    (MenuGroup::Archive, "menu-archive"),
                    (MenuGroup::Edit, "menu-edit"),
                    (MenuGroup::View, "menu-view"),
                    (MenuGroup::Tools, "menu-tools"),
                    (MenuGroup::Help, "menu-help"),
                ]
                .into_iter()
                .map(|(group, label)| {
                    let owner = owner.clone();
                    command(label, tr(label))
                        .h(px(28.))
                        .px(px(8.))
                        .custom(subtle_variant(cx))
                        .border_0()
                        .dropdown_menu(move |menu, window, cx| {
                            build_menu(&owner, group, menu, window, cx)
                        })
                }),
            )
    }
}

fn build_menu(
    owner: &WeakEntity<Workspace>,
    group: MenuGroup,
    menu: PopupMenu,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let mut menu = menu_style(menu);
    struct Panel {
        open: bool,
        open_inside: bool,
        open_outside: bool,
        rename: bool,
        copy: bool,
        revise: bool,
        rows: bool,
        selected: bool,
        all_selected: bool,
        sort: usize,
        up: bool,
    }
    let Ok((catalog, busy, recent, remember, panel)) = owner.read_with(cx, |this, cx| {
        let on_files = this.page == Page::Files;
        let open = on_files && this.current_item().is_some();
        let open_inside = on_files && this.can_open_inside();
        let open_outside = on_files && this.can_open_outside();
        let up = on_files && this.parent_location().is_some();
        let busy = this.tasks.is_busy() || this.settings_busy(cx);
        let recent = this.recent_archives.clone();
        let remember = this.preferences.remember_recent;
        let view = this.browser.view();
        let editable = on_files && view.catalog.as_ref().is_some_and(Catalog::editable);
        let selected = !view.selected.is_empty();
        let rows = view.rows.len();
        let panel = Panel {
            open,
            open_inside,
            open_outside,
            rename: editable && view.selected.len() == 1,
            copy: on_files && view.catalog.is_some() && selected,
            revise: editable && selected,
            rows: on_files && rows > 0,
            selected: on_files && selected,
            all_selected: on_files && rows > 0 && view.selected.len() == rows,
            sort: view.sort,
            up,
        };
        (
            view.catalog.clone().filter(|_| on_files),
            busy,
            recent,
            remember,
            panel,
        )
    }) else {
        return menu;
    };
    let active = catalog.is_some() && !busy;
    let commentable = active && catalog.as_ref().is_some_and(|c| c.accepts_comment());
    match group {
        MenuGroup::File => {
            menu = menu
                .item(command_item(
                    owner,
                    "browser-open",
                    "Enter",
                    Command::OpenItem,
                    panel.open && !busy,
                ))
                .item(command_item(
                    owner,
                    "file-open-inside",
                    "Ctrl+PgDn",
                    Command::OpenInside,
                    panel.open_inside && !busy,
                ))
                .item(command_item(
                    owner,
                    "file-open-outside",
                    "Shift+Enter",
                    Command::OpenOutside,
                    panel.open_outside && !busy,
                ))
                .separator()
                .item(command_item(
                    owner,
                    "archive-rename",
                    "F2",
                    Command::Rename,
                    panel.rename && !busy,
                ))
                .item(command_item(
                    owner,
                    "archive-copy",
                    "F5",
                    Command::CopyTo,
                    panel.copy && !busy,
                ))
                .item(command_item(
                    owner,
                    "archive-move",
                    "F6",
                    Command::MoveTo,
                    panel.revise && !busy,
                ))
                .item(command_item(
                    owner,
                    "archive-delete",
                    "Del",
                    Command::Delete,
                    panel.revise && !busy,
                ))
                .separator()
                .item(command_item(
                    owner,
                    "properties",
                    "Alt+Enter",
                    Command::Properties,
                    active,
                ))
                .item(command_item(
                    owner,
                    "archive-comment",
                    "Ctrl+Z",
                    Command::Comment,
                    commentable,
                ));
            menu = menus::archive::checksum_menu(owner, menu, window, cx);
            menu = menu.separator().item(command_item(
                owner,
                "menu-exit",
                "Alt+F4",
                Command::Exit,
                !busy,
            ));
        }
        MenuGroup::Archive => {
            menu = menu
                .item(command_item(
                    owner,
                    "archive-open",
                    "Ctrl+O",
                    Command::Open,
                    !busy,
                ))
                .item(command_item(
                    owner,
                    "archive-create-command",
                    "",
                    Command::Create,
                    !busy,
                ));
            menu = menus::archive::open_as_menu(owner, menu, window, cx);
            if remember && (busy || recent.is_empty()) {
                menu = menu.item(PopupMenuItem::new(tr("recent-title")).disabled(true));
            } else if remember {
                let history_owner = owner.clone();
                menu = menu.submenu(tr("recent-title"), window, cx, move |menu, _, _| {
                    let owner = &history_owner;
                    let mut menu = submenu_style(menu);
                    for path in &recent {
                        let path = path.clone();
                        let owner = owner.clone();
                        menu =
                            menu.item(path_menu_item(path.display().to_string()).on_click(
                                move |_, window, cx| {
                                    let _ = owner.update(cx, |this, cx| {
                                        this.show_page(Page::Files, window, cx);
                                        this.execute(Request::Open(path.clone()), String::new(), cx)
                                    });
                                },
                            ));
                    }
                    menu.separator().item(command_item(
                        owner,
                        "recent-clear",
                        "",
                        Command::ClearRecent,
                        true,
                    ))
                });
            }
            let extract_owner = owner.clone();
            menu = menu
                .separator()
                .submenu(tr("menu-extract"), window, cx, move |menu, _, _| {
                    submenu_style(menu)
                        .item(command_item(
                            &extract_owner,
                            "extract-options",
                            "Alt+E",
                            Command::Extract,
                            active,
                        ))
                        .item(command_item(
                            &extract_owner,
                            "extract-all-quick",
                            "",
                            Command::QuickExtract,
                            active,
                        ))
                })
                .item(command_item(
                    owner,
                    "archive-check",
                    "Alt+T",
                    Command::Check,
                    active,
                ))
                .separator()
                .item(command_item(
                    owner,
                    "archive-save-as",
                    "Ctrl+Shift+S",
                    Command::Save,
                    active,
                ))
                .item(command_item(
                    owner,
                    "archive-info",
                    "",
                    Command::ArchiveInfo,
                    active,
                ))
                .item(command_item(
                    owner,
                    "archive-close",
                    "",
                    Command::Close,
                    active,
                ));
        }
        MenuGroup::Edit => {
            menu = menu
                .item(command_item(
                    owner,
                    "select-all",
                    "Ctrl+A",
                    Command::SelectAll,
                    panel.rows && !panel.all_selected && !busy,
                ))
                .item(command_item(
                    owner,
                    "select-none",
                    "",
                    Command::DeselectAll,
                    panel.selected && !busy,
                ))
                .item(command_item(
                    owner,
                    "select-invert",
                    "",
                    Command::InvertSelection,
                    panel.rows && !busy,
                ))
                .separator()
                .item(command_item(
                    owner,
                    "select-by-type",
                    "",
                    Command::SelectByType,
                    panel.open && !busy,
                ))
                .item(command_item(
                    owner,
                    "deselect-by-type",
                    "",
                    Command::DeselectByType,
                    panel.open && !busy,
                ));
        }
        MenuGroup::View => {
            for (column, label, shortcut) in [
                (0, "name", "Ctrl+F3"),
                (2, "modified", "Ctrl+F5"),
                (1, "size", "Ctrl+F6"),
            ] {
                let sort_owner = owner.clone();
                menu = menu.item(
                    menu_command_item(tr(label), shortcut)
                        .checked(panel.sort == column)
                        .disabled(!panel.rows || busy)
                        .on_click(move |_, window, cx| {
                            let _ = sort_owner.update(cx, |this, cx| {
                                this.command(Command::Sort(column), window, cx);
                            });
                        }),
                );
            }
            menu = menu
                .separator()
                .item(command_item(
                    owner,
                    "parent-folder",
                    "Backspace",
                    Command::Up,
                    panel.up && !busy,
                ))
                .item(command_item(
                    owner,
                    "menu-refresh",
                    "Ctrl+R",
                    Command::Refresh,
                    !busy,
                ));
        }
        MenuGroup::Tools => {
            let options_owner = owner.clone();
            let options_enabled = !busy;
            menu = menu.submenu(
                tr("menu-options"),
                window,
                cx,
                move |menu, _, _| {
                    let mut menu = submenu_style(menu);
                    for tab in [
                        preferences::Tab::General,
                        preferences::Tab::Appearance,
                        preferences::Tab::Associations,
                        preferences::Tab::Advanced,
                    ] {
                        menu = menu.item(command_item_label(
                            &options_owner,
                            tab.label(),
                            "",
                            Command::Settings(tab),
                            options_enabled,
                        ));
                    }
                    menu
                },
            );
        }
        MenuGroup::Help => {
            menu = menu
                .item(command_item(owner, "menu-help", "F1", Command::Help, !busy))
                .separator()
                .item(command_item(
                    owner,
                    "update-check",
                    "",
                    Command::Updates,
                    !busy,
                ))
                .item(command_item(owner, "menu-about", "", Command::About, !busy));
        }
    }
    menu
}

fn command_item(
    owner: &WeakEntity<Workspace>,
    label: &str,
    shortcut: &'static str,
    command: Command,
    enabled: bool,
) -> PopupMenuItem {
    command_item_label(owner, tr(label), shortcut, command, enabled)
}

fn command_item_label(
    owner: &WeakEntity<Workspace>,
    label: &'static str,
    shortcut: &'static str,
    command: Command,
    enabled: bool,
) -> PopupMenuItem {
    let owner = owner.clone();
    menu_command_item(label, shortcut)
        .disabled(!enabled)
        .on_click(move |_, window, cx| {
            let _ = owner.update(cx, |this, cx| this.command(command, window, cx));
        })
}
