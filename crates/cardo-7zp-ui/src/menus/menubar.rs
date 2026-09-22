use crate::commands::Command;
use crate::*;
use gpui_kit::component::{button::ButtonVariants, h_flex};

#[derive(Clone, Copy)]
pub(crate) enum MenuGroup {
    File,
    Selection,
    Open,
    Extract,
    Check,
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
                [(MenuGroup::File, "menu-file")]
                    .into_iter()
                    .map(|(group, label)| {
                        let owner = owner.clone();
                        command(label, tr(label))
                            .h(px(28.))
                            .px(px(8.))
                            .custom(subtle_variant(cx))
                            .border_0()
                            .popup_menu(move |_, cx| build_menu(&owner, group, cx))
                    }),
            )
    }
}

pub(crate) fn build_menu(owner: &WeakEntity<Workspace>, group: MenuGroup, cx: &mut App) -> Menu {
    use Command::*;
    let mut menu = Menu::new();
    let items: &[(&str, &str, Command)] = match group {
        MenuGroup::File => &[
            ("browser-open", "Enter", OpenItem),
            ("file-open-inside", "Ctrl+PgDn", OpenInside),
            ("file-open-outside", "Shift+Enter", OpenOutside),
            ("", "", Open),
            ("archive-add", "Alt+A", Add),
            ("archive-add-folder", "", AddFolder),
            ("archive-rename", "F2", Rename),
            ("archive-copy", "F5", CopyTo),
            ("archive-move", "F6", MoveTo),
            ("archive-delete", "Del", Delete),
            ("", "", Open),
            ("properties", "Alt+Enter", Properties),
            ("archive-comment", "Ctrl+Z", Comment),
            ("archive-save-as", "Ctrl+Shift+S", Save),
            ("archive-info", "", ArchiveInfo),
            ("archive-close", "", Close),
            ("", "", Open),
            (
                "settings-title",
                "Ctrl+,",
                Settings(preferences::Tab::Application),
            ),
            ("menu-exit", "Alt+F4", Exit),
        ],
        MenuGroup::Selection => &[
            ("select-all", "Ctrl+A", SelectAll),
            ("select-none", "", DeselectAll),
            ("select-invert", "", InvertSelection),
            ("", "", Open),
            ("select-by-type", "", SelectByType),
            ("deselect-by-type", "", DeselectByType),
        ],
        MenuGroup::Open => &[("archive-open", "Ctrl+O", Open)],
        MenuGroup::Extract => &[
            ("extract-options", "Alt+E", Extract),
            ("extract-all-quick", "", QuickExtract),
            ("extract-selected", "", QuickExtractSelection),
        ],
        MenuGroup::Check => &[("archive-check", "Alt+T", Check)],
    };
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
                    SelectByType | DeselectByType => this.current_item().is_some(),
                    QuickExtractSelection => !view.selected.is_empty(),
                    _ => true,
                };
                (
                    relevant && this.command_available(action, cx),
                    matches!(action, Sort(column) if view.sort == column),
                )
            })
            .unwrap_or((false, false));
        let owner = owner.clone();
        menu = menu.item(
            menu_command_item(tr(label), shortcut)
                .disabled(!enabled)
                .checked(checked)
                .on_select(move |window, cx| {
                    let _ = owner.update(cx, |this, cx| this.command(action, window, cx));
                }),
        );
    }
    match group {
        MenuGroup::Open => {
            menu = menus::archive::open_as_menu(owner, menu, cx);
        }
        MenuGroup::Check => menu = menus::archive::checksum_menu(owner, menu, cx),
        _ => {}
    }
    menu
}
