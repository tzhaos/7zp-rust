use crate::theme::metrics::titlebar as metrics;
use crate::*;
use cardo_7zp_core::i18n::tr;
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
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(self.menubar(cx)),
            )
            .child(
                h_flex().gap_0().children(
                    [
                        ("minimize", "Subtract", tr("window-minimize")),
                        (
                            "maximize",
                            if window.is_maximized() {
                                "SquareMultiple"
                            } else {
                                "Square"
                            },
                            tr("window-maximize"),
                        ),
                        ("close", "Dismiss", tr("window-close")),
                    ]
                    .into_iter()
                    .map(|(id, name, label)| {
                        let disabled =
                            id == "close" && (self.tasks.is_busy() || self.settings_busy(cx));
                        let button = gpui_kit::component::button::Button::new(id);
                        let button = if self.allow_hint(!disabled) {
                            bubble_tooltip(button, label.to_owned())
                        } else {
                            button
                        };
                        button
                            .group("window-control")
                            .custom(if id == "close" {
                                subtle_variant(cx)
                                    .hover(rgb(0xe81123).into())
                                    .active(rgb(0xc50f1f).into())
                            } else {
                                subtle_variant(cx)
                            })
                            .accessibility_label(label)
                            .child(
                                div()
                                    .when(id == "close" && !disabled, |el| {
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
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_click(move |_, window, _| match id {
                                "minimize" => window.minimize_window(),
                                "maximize" => window.zoom_window(),
                                _ => window.remove_window(),
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
