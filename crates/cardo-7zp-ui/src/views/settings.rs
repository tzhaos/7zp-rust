use crate::*;
use gpui_kit::component::{Disableable, v_flex};

impl Workspace {
    pub(crate) fn settings_page(&self, cx: &Context<Self>) -> Option<Div> {
        let Some((form, tab)) = self.settings_form.as_ref().zip(match self.page {
            Page::Settings(tab) => Some(tab),
            Page::Files => None,
        }) else {
            return None;
        };
        let direction = self.page_direction;
        Some(
            v_flex()
                .size_full()
                .min_h_0()
                .child(self.subpage_header(tab.label(), cx))
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h_0()
                        .child(form.clone())
                        .with_animation(
                            ("settings-page-transition", self.page_revision),
                            Animation::new(std::time::Duration::from_millis(180))
                                .with_easing(ease_out_quint()),
                            move |el, delta| {
                                el.left(px((1.0 - delta) * direction * 24.)).opacity(delta)
                            },
                        ),
                ),
        )
    }

    pub(crate) fn subpage_header(&self, title: &'static str, cx: &Context<Self>) -> Div {
        navigation_row(cx)
            .child(
                icon_button("page-back", "ArrowLeft", tr("back"), cx)
                    .disabled(self.tasks.is_busy() || self.settings_busy(cx))
                    .on_click(
                        cx.listener(|this, _, window, cx| this.show_page(Page::Files, window, cx)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pl(px(6.))
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
    }
}
