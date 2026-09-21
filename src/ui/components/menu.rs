use gpui_kit::component::{Side, menu::PopupMenu};
use gpui_kit::*;

pub fn menu_command_item(
    label: &'static str,
    shortcut: &'static str,
) -> gpui_kit::component::menu::PopupMenuItem {
    gpui_kit::component::menu::PopupMenuItem::element(move |_, cx| {
        let p = crate::ui::theme::palette(cx);
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

pub fn path_menu_item(label: String, width: f32) -> gpui_kit::component::menu::PopupMenuItem {
    gpui_kit::component::menu::PopupMenuItem::element(move |_, _| {
        gpui_kit::component::h_flex()
            .w(px(width))
            .min_w_0()
            .h(px(26.))
            .child(div().flex_1().min_w_0().truncate().child(label.clone()))
    })
}

pub fn menu_style(menu: PopupMenu) -> PopupMenu {
    menu.check_side(Side::Right).min_w(px(200.))
}
