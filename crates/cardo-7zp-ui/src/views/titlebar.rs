use crate::theme::metrics::titlebar as metrics;
use crate::*;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, h_flex, v_flex},
    prelude::FluentBuilder,
};
use cardo_7zp_core::i18n::tr;

impl Workspace {
    pub(crate) fn titlebar(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
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
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("7z+"),
                    )
                    .child(self.menubar(cx))
                    .child(div().flex_1())
                    .when_some(self.browser.view().catalog.as_ref(), |el, c| {
                        el.child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(rgb(p.selected))
                                .text_size(px(10.))
                                .child(c.format.clone()),
                        )
                    })
                    .when(
                        self.browser
                            .view()
                            .catalog
                            .as_ref()
                            .is_some_and(|c| c.entries.iter().any(|e| e.encrypted)),
                        |el| el.child(icon("LockClosed", 14.)),
                    ),
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
                        bubble_tooltip(gpui_kit::component::button::Button::new(id), label)
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
            .child(
                v_flex()
                    .absolute()
                    .top_0()
                    .left((window.viewport_size().width - px(metrics::GRIP_WIDTH)) / 2.)
                    .w(px(metrics::GRIP_WIDTH))
                    .h_full()
                    .items_center()
                    .justify_center()
                    .gap(px(metrics::GRIP_GAP))
                    .window_control_area(WindowControlArea::Drag)
                    .opacity(metrics::GRIP_OPACITY)
                    .children((0..2).map(|_| {
                        h_flex()
                            .gap(px(metrics::GRIP_GAP))
                            .children((0..4).map(|_| {
                                div()
                                    .size(px(metrics::GRIP_DOT_SIZE))
                                    .rounded(px(metrics::GRIP_DOT_SIZE))
                                    .bg(rgb(p.muted))
                            }))
                    })),
            )
    }
}
