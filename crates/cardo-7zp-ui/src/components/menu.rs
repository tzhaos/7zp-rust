use gpui_kit::component::{Side, menu::PopupMenu};
use gpui_kit::*;

/// Every menu, including a submenu, uses this width.
///
/// The menu host places a submenu by offsetting it from the parent item and
/// then clamping the measured box into the window. A child wider than the
/// parent is measured together with that offset, so the clamp pulls the
/// submenu away from the item that opened it.
pub const MENU_WIDTH: f32 = 280.;

pub fn menu_command_item(
    label: &'static str,
    shortcut: &'static str,
) -> gpui_kit::component::menu::PopupMenuItem {
    gpui_kit::component::menu::PopupMenuItem::element(move |_, cx| {
        let p = crate::theme::palette(cx);
        gpui_kit::component::h_flex()
            .flex_1()
            .min_w(px(230.))
            .h(px(26.))
            .gap(px(16.))
            .child(div().flex_1().child(label))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(p.muted))
                    .child(shortcut),
            )
    })
}

pub fn path_menu_item(label: String) -> gpui_kit::component::menu::PopupMenuItem {
    gpui_kit::component::menu::PopupMenuItem::element(move |_, _| {
        gpui_kit::component::h_flex()
            .w_full()
            .min_w_0()
            .h(px(26.))
            .child(div().flex_1().min_w_0().truncate().child(label.clone()))
    })
}

pub fn menu_style(menu: PopupMenu) -> PopupMenu {
    menu.check_side(Side::Right)
        .min_w(px(MENU_WIDTH))
        .max_w(px(MENU_WIDTH))
}

/// A nested menu. It keeps [`MENU_WIDTH`] and scrolls instead of growing past
/// the window, so its anchor stays on the parent item.
pub fn submenu_style(menu: PopupMenu) -> PopupMenu {
    menu_style(menu).max_h(px(320.)).scrollable(true)
}
