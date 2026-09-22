use crate::*;

#[derive(Clone, Copy)]
enum Command {
    AddFiles,
    AddFolder,
    Copy,
    Move,
    Delete,
    Rename,
}

pub(crate) fn entry_menu(
    owner: &WeakEntity<Workspace>,
    mut menu: PopupMenu,
    context_menu: bool,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    menu = menu_style(menu);
    let Ok((catalog, selected, busy)) = owner.read_with(cx, |this, _| {
        (
            this.browser.view().catalog.clone(),
            this.browser.view().selected.clone(),
            this.tasks.is_busy() || this.page != Page::Files,
        )
    }) else {
        return menu;
    };
    let editable = catalog.as_ref().is_some_and(Catalog::editable);
    for (command, label, shortcut) in [
        (Command::AddFiles, "archive-add", "Alt+A"),
        (Command::AddFolder, "archive-add-folder", ""),
        (Command::Copy, "archive-copy", ""),
        (Command::Move, "archive-move", ""),
        (Command::Rename, "archive-rename", ""),
        (Command::Delete, "archive-delete", ""),
    ] {
        let enabled = !busy
            && catalog.is_some()
            && match command {
                Command::AddFiles | Command::AddFolder => editable,
                Command::Copy => !selected.is_empty(),
                Command::Rename => editable && selected.len() == 1,
                Command::Move | Command::Delete => editable && !selected.is_empty(),
            };
        if context_menu && !enabled {
            continue;
        }
        if matches!(command, Command::Copy) {
            menu = menu.separator();
        }
        let owner = owner.clone();
        let catalog = catalog.clone();
        let selected = selected.clone();
        menu = menu.item(
            menu_command_item(tr(label), shortcut)
                .disabled(!enabled)
                .on_click(move |_, window, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        if this.tasks.is_busy()
                            || this.browser.view().catalog.as_ref().map(|c| &c.path)
                                != catalog.as_ref().map(|c| &c.path)
                        {
                            return;
                        }
                        this.browser.restore_selection(selected.clone());
                        match command {
                            Command::AddFiles => this.add_to_archive(false, cx),
                            Command::AddFolder => this.add_to_archive(true, cx),
                            Command::Copy => this.copy_entries(false, cx),
                            Command::Move => this.copy_entries(true, cx),
                            Command::Delete => this.delete_entries(cx),
                            Command::Rename => this.rename_entry(window, cx),
                        }
                    });
                }),
        );
    }
    menu
}

pub(crate) fn checksum_menu(
    owner: &WeakEntity<Workspace>,
    menu: PopupMenu,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let Ok((catalog, busy)) = owner.read_with(cx, |this, _| {
        (
            this.browser.view().catalog.clone(),
            this.tasks.is_busy() || this.page != Page::Files,
        )
    }) else {
        return menu;
    };
    let hashes = owner.clone();
    let Some(hash_path) = catalog.map(|catalog| catalog.path).filter(|_| !busy) else {
        return menu.item(PopupMenuItem::new(tr("archive-file-checksum")).disabled(true));
    };
    menu.submenu(
        tr("archive-file-checksum"),
        window,
        cx,
        move |mut menu, _, _| {
            menu = submenu_style(menu);
            for action in cardo_7zp_commands::ACTIONS.iter().copied() {
                let cardo_7zp_commands::Action::Checksum(method) = action else {
                    continue;
                };
                let owner = hashes.clone();
                let path = hash_path.clone();
                menu = menu.item(
                    PopupMenuItem::new(method.title()).on_click(move |_, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.launches.push_back(vec![
                                action.argument().into(),
                                path.display().to_string(),
                            ]);
                            cx.notify();
                        });
                    }),
                );
            }
            menu
        },
    )
}

pub(crate) fn open_as_menu(
    owner: &WeakEntity<Workspace>,
    menu: PopupMenu,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let Ok((open_path, busy)) = owner.read_with(cx, |this, _| {
        (
            this.browser.view().catalog.as_ref().map(|c| c.path.clone()),
            this.tasks.is_busy() || this.page != Page::Files,
        )
    }) else {
        return menu;
    };
    let opening = owner.clone();
    let Some(open_path) = open_path.filter(|_| !busy) else {
        return menu.item(PopupMenuItem::new(tr("archive-open-as")).disabled(true));
    };
    menu.submenu(tr("archive-open-as"), window, cx, move |mut menu, _, _| {
        menu = submenu_style(menu);
        for action in cardo_7zp_commands::ACTIONS.iter().copied() {
            let cardo_7zp_commands::Action::OpenAs(kind) = action else {
                continue;
            };
            let owner = opening.clone();
            let path = open_path.clone();
            menu = menu.item(PopupMenuItem::new(kind.value()).on_click(move |_, _, cx| {
                let _ = owner.update(cx, |this, cx| {
                    this.execute(Request::OpenAs(path.clone(), kind), String::new(), cx)
                });
            }));
        }
        menu
    })
}
