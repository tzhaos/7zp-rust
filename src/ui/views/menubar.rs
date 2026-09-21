use crate::ui::commands::Command;
use crate::ui::*;
use gpui_kit::component::{button::ButtonVariants, h_flex, menu::DropdownMenu};

#[derive(Clone, Copy)]
enum MenuGroup {
    File,
    Edit,
    Tools,
    Help,
}

impl Workspace {
    pub(in crate::ui) fn menubar(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
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
                    (MenuGroup::Edit, "menu-edit"),
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
                        .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, window, cx| {
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
    let mut menu = menu_style(menu).min_w(px(280.));
    let Ok((catalog, busy, recent, remember)) = owner.read_with(cx, |this, cx| {
        (
            this.catalog.clone().filter(|_| this.page == Page::Files),
            this.busy || this.settings_busy(cx),
            this.recent_archives.clone(),
            this.preferences.remember_recent,
        )
    }) else {
        return menu;
    };
    let active = catalog.is_some() && !busy;
    let commentable = active
        && catalog
            .as_ref()
            .is_some_and(|c| c.editable() && c.format == "ZIP");
    match group {
        MenuGroup::File => {
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
            menu = manage::open_as_menu(owner, menu, window, cx);
            menu = menu.item(command_item(owner, "files-view", "", Command::Files, !busy));
            if remember && (busy || recent.is_empty()) {
                menu = menu.item(PopupMenuItem::new(tr("recent-title")).disabled(true));
            } else if remember {
                let history_owner = owner.clone();
                menu = menu.submenu(tr("recent-title"), window, cx, move |menu, _, _| {
                    let owner = &history_owner;
                    let mut menu = menu_style(menu).scrollable(true).max_h(px(320.));
                    for path in &recent {
                        let path = path.clone();
                        let owner = owner.clone();
                        menu =
                            menu.item(path_menu_item(path.display().to_string(), 360.).on_click(
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
            menu = menu
                .separator()
                .item(command_item(
                    owner,
                    "extract-options",
                    "Alt+E",
                    Command::Extract,
                    active,
                ))
                .item(command_item(
                    owner,
                    "extract-all-quick",
                    "",
                    Command::QuickExtract,
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
                ))
                .separator()
                .item(command_item(
                    owner,
                    "menu-exit",
                    "Alt+F4",
                    Command::Exit,
                    !busy,
                ));
        }
        MenuGroup::Edit => {
            menu = manage::entry_menu(owner, menu, false, cx)
                .separator()
                .item(command_item(
                    owner,
                    "archive-comment",
                    "Alt+M",
                    Command::Comment,
                    commentable,
                ))
                .item(command_item(
                    owner,
                    "properties",
                    "Alt+I",
                    Command::Properties,
                    active,
                ));
        }
        MenuGroup::Tools => {
            menu = menu.item(command_item(
                owner,
                "archive-check",
                "Alt+T",
                Command::Check,
                active,
            ));
            menu = manage::checksum_menu(owner, menu, window, cx).separator();
            for tab in [
                preferences::Tab::General,
                preferences::Tab::Advanced,
                preferences::Tab::Appearance,
                preferences::Tab::Associations,
            ] {
                menu = menu.item(command_item_label(
                    owner,
                    tab.label(),
                    "",
                    Command::Settings(tab),
                    !busy,
                ));
            }
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
