use gpui_kit::base::ElementExt;
use gpui_kit::*;
use std::{cell::Cell, rc::Rc};

struct BubbleTooltip {
    text: SharedString,
    trigger: Rc<Cell<Bounds<Pixels>>>,
}

impl Render for BubbleTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let trigger = self.trigger.get();
        let bubble = gpui_kit::base::Tooltip::new("bubble-tooltip")
            .min_w(px(32.))
            .max_w(px(360.))
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .rounded(px(crate::theme::metrics::TOOLTIP_RADIUS))
                    .bg(rgb(0x202020))
                    .text_color(rgb(0xffffff))
                    .text_size(px(12.))
                    .line_height(px(18.))
                    .whitespace_normal()
                    .shadow_sm()
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
    element.interactivity().tooltip(move |_, cx| {
        cx.new(|_| BubbleTooltip {
            text: text.clone(),
            trigger: trigger.clone(),
        })
        .into()
    });
    element
}
