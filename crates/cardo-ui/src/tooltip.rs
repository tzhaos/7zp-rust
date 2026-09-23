use gpui_kit::base::ElementExt;
use gpui_kit::*;
use std::{cell::Cell, rc::Rc, time::Duration};

struct BubbleTooltip {
    text: SharedString,
    trigger: Rc<Cell<Bounds<Pixels>>>,
}

impl Render for BubbleTooltip {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let trigger = self.trigger.get();
        let viewport = window.viewport_size();
        let bubble = gpui_kit::base::Tooltip::new("bubble-tooltip")
            .max_w(px(360.).min((viewport.width - px(16.)).max(px(1.))))
            .px(px(10.))
            .py(px(4.))
            .rounded(px(14.))
            .bg(rgb(0x202020))
            .text_color(rgb(0xffffff))
            .text_size(px(12.))
            .line_height(px(18.))
            .shadow_sm()
            .child(
                div()
                    .id("tooltip-text")
                    .min_w_0()
                    .max_h((viewport.height - px(24.)).max(px(1.)))
                    .overflow_y_scroll()
                    .whitespace_normal()
                    .child(self.text.clone()),
            );
        // Positioner measures the bubble before centering and clamping it in the viewport.
        gpui_kit::base::Positioner::side(trigger)
            .placement(gpui_kit::base::Placement::Bottom)
            .offset(px(6.))
            .margin(px(8.))
            .child(bubble)
    }
}

pub fn bubble_tooltip<E: InteractiveElement + ParentElement + Styled>(
    element: E,
    text: impl Into<SharedString>,
) -> E {
    let text = text.into();
    let trigger = Rc::new(Cell::new(Bounds::default()));
    let measured = trigger.clone();
    let mut element = element
        .relative()
        .on_prepaint(move |bounds, _, _| measured.set(bounds));
    element.interactivity().hoverable_tooltip(move |_, cx| {
        cx.new(|_| BubbleTooltip {
            text: text.clone(),
            trigger: trigger.clone(),
        })
        .into()
    });
    element
        .interactivity()
        .tooltip_show_delay(Duration::from_millis(700));
    element
}
