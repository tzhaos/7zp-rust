use crate::commands::Command;
use crate::*;

pub(crate) fn entry_menu(
    owner: &WeakEntity<Workspace>,
    mut menu: Menu,
    context_menu: bool,
    cx: &mut App,
) -> Menu {
    let Ok((location, selected)) = owner.read_with(cx, |this, _| {
        (
            this.browser.location(),
            this.browser.view().selected.clone(),
        )
    }) else {
        return menu;
    };
    for (command, label, shortcut) in [
        (Command::Add, "archive-add", "Alt+A"),
        (Command::AddFolder, "archive-add-folder", ""),
        (Command::CopyTo, "archive-copy", "F5"),
        (Command::MoveTo, "archive-move", "F6"),
        (Command::Rename, "archive-rename", "F2"),
        (Command::Delete, "archive-delete", "Del"),
    ] {
        let enabled = owner
            .read_with(cx, |this, cx| this.command_available(command, cx))
            .unwrap_or(false);
        if context_menu && !enabled {
            continue;
        }
        if matches!(command, Command::CopyTo) {
            menu = menu.separator();
        }
        let owner = owner.clone();
        let location = location.clone();
        let selected = selected.clone();
        menu = menu.item(
            menu_command_item(tr(label), shortcut)
                .disabled(!enabled)
                .on_select(move |window, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        if this.tasks.is_busy()
                            || this.dialogs.is_open()
                            || this.settings_busy(cx)
                            || this.browser.location() != location
                        {
                            return;
                        }
                        this.browser.restore_selection(selected.clone());
                        this.command(command, window, cx);
                    });
                }),
        );
    }
    menu
}

pub(crate) fn checksum_menu(owner: &WeakEntity<Workspace>, menu: Menu, cx: &mut App) -> Menu {
    let Ok((catalog, busy)) = owner.read_with(cx, |this, _| {
        (this.browser.view().catalog.clone(), this.tasks.is_busy())
    }) else {
        return menu;
    };
    let hashes = owner.clone();
    let Some(hash_path) = catalog.map(|catalog| catalog.path).filter(|_| !busy) else {
        return menu.item(MenuItem::new(tr("archive-file-checksum")).disabled(true));
    };
    menu.submenu(tr("archive-file-checksum"), {
        let mut menu = Menu::new();
        for action in cardo_7zp_commands::ACTIONS.iter().copied() {
            let cardo_7zp_commands::Action::Checksum(method) = action else {
                continue;
            };
            let owner = hashes.clone();
            let path = hash_path.clone();
            menu = menu.item(MenuItem::new(method.title()).on_select(move |_, cx| {
                let _ = owner.update(cx, |this, cx| {
                    this.launches
                        .push_back(vec![action.argument().into(), path.display().to_string()]);
                    cx.notify();
                });
            }));
        }
        menu
    })
}

pub(crate) fn open_as_menu(owner: &WeakEntity<Workspace>, menu: Menu, cx: &mut App) -> Menu {
    let Ok((open_path, busy)) = owner.read_with(cx, |this, _| {
        (
            this.browser.view().catalog.as_ref().map(|c| c.path.clone()),
            this.tasks.is_busy(),
        )
    }) else {
        return menu;
    };
    let opening = owner.clone();
    let Some(open_path) = open_path.filter(|_| !busy) else {
        return menu.item(MenuItem::new(tr("archive-open-as")).disabled(true));
    };
    menu.submenu(tr("archive-open-as"), {
        let mut menu = Menu::new();
        for action in cardo_7zp_commands::ACTIONS.iter().copied() {
            let cardo_7zp_commands::Action::OpenAs(kind) = action else {
                continue;
            };
            let owner = opening.clone();
            let path = open_path.clone();
            menu = menu.item(MenuItem::new(kind.value()).on_select(move |_, cx| {
                let _ = owner.update(cx, |this, cx| this.open_as(path.clone(), kind, cx));
            }));
        }
        menu
    })
}
