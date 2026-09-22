use cardo_ui::menu::MenuItem;

pub fn menu_command_item(label: &'static str, shortcut: &'static str) -> MenuItem {
    MenuItem::new(label).shortcut(shortcut)
}
