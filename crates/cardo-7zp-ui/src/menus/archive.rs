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
