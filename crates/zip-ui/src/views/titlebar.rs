use crate::theme::metrics::titlebar as metrics;
use crate::*;
use zip_core::i18n::tr;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, h_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn titlebar(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
        let title = self.window_title();
        h_flex()
            .relative()
            .h(px(metrics::HEIGHT))
            .flex_shrink_0()
            .bg(rgb(p.panel))
            .items_center()
            .child(
                h_flex()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .pl(px(16.))
                    .gap(px(12.))
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .id("window-heading")
                            .min_w_0()
                            .truncate()
                            .child(title)
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(self.menubar(cx)),
            )
            .child(
                h_flex().flex_shrink_0().gap_0().children(
                    [
                        (
                            WindowControlArea::Min,
                            "minimize",
                            "Subtract",
                            tr("window-minimize"),
                        ),
                        (
                            WindowControlArea::Max,
                            "maximize",
                            if window.is_maximized() {
                                "SquareMultiple"
                            } else {
                                "Square"
                            },
                            tr("window-maximize"),
                        ),
                        (
                            WindowControlArea::Close,
                            "close",
                            "Dismiss",
                            tr("window-close"),
                        ),
                    ]
                    .into_iter()
                    .map(|(area, id, name, label)| {
                        let disabled = area == WindowControlArea::Close
                            && (self.tasks.is_busy() || self.settings_busy(cx));
                        gpui_kit::component::button::Button::new(id)
                            .group("window-control")
                            .custom(if area == WindowControlArea::Close {
                                subtle_variant(cx)
                                    .hover(rgb(0xe81123).into())
                                    .active(rgb(0xc50f1f).into())
                            } else {
                                subtle_variant(cx)
                            })
                            .accessibility_label(label)
                            .child(
                                div()
                                    .when(area == WindowControlArea::Close && !disabled, |el| {
                                        el.group_hover("window-control", |el| {
                                            el.text_color(rgb(0xffffff))
                                        })
                                    })
                                    .child(icon(name, 16.)),
                            )
                            .border_0()
                            .w(px(metrics::CONTROL_WIDTH))
                            .h(px(metrics::HEIGHT))
                            .rounded(px(0.))
                            .disabled(disabled)
                            // The pinned Windows backend's zoom() only maximizes;
                            // native Max hit testing handles both maximize and restore.
                            .when(area == WindowControlArea::Max, |button| {
                                button.window_control_area(area)
                            })
                            .when(area == WindowControlArea::Min, |button| {
                                button
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .on_click(|_, window, _| window.minimize_window())
                            })
                            .when(area == WindowControlArea::Close, |button| {
                                button
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .on_click(|_, window, _| window.remove_window())
                            })
                    }),
                ),
            )
            .child(window_grip(cx))
    }

    fn window_title(&self) -> String {
        let view = self.browser.view();
        let path = view
            .catalog
            .as_ref()
            .map(|catalog| &catalog.path)
            .or_else(|| view.directory.as_ref().map(|directory| &directory.path));
        let Some(path) = path else {
            return "7zplus".to_owned();
        };
        let name = path
            .file_name()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        format!("{name} - 7zplus")
    }
}
