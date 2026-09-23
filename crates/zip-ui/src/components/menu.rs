use cardo_ui::menu::MenuItem;

pub fn menu_command_item(
    label: &'static str,
    shortcut: impl Into<gpui_kit::SharedString>,
) -> MenuItem {
    MenuItem::new(label).shortcut(shortcut)
}
