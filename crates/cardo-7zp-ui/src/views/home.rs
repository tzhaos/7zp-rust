use crate::commands::Command;
use crate::theme::metrics::home as metrics;
use crate::*;
use gpui_kit::{
    component::{Disableable, h_flex, v_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn home_view(&self, cx: &mut Context<Self>) -> Div {
        let p = crate::theme::palette(cx);
        let busy = self.tasks.is_busy() || self.settings_busy(cx);
        let has_history = self.matching_recent_archives(cx).next().is_some();
        h_flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .p(px(metrics::PADDING))
            .items_center()
            .justify_center()
            .text_color(rgb(p.text))
            .child(
                h_flex()
                    .size_full()
                    .max_w(px(metrics::MAX_WIDTH))
                    .max_h(px(metrics::MAX_HEIGHT))
                    .min_h_0()
                    .gap(px(metrics::COLUMN_GAP))
                    .child(
                        v_flex()
                            .w(px(metrics::BRAND_WIDTH))
                            .h_full()
                            .flex_shrink_0()
                            .items_center()
                            .justify_center()
                            .gap(px(metrics::BRAND_GAP))
                            .child(
                                img("brand/logo.png")
                                    .size(px(metrics::LOGO_SIZE))
                                    .flex_shrink_0(),
                            )
                            .child(
                                div()
                                    .text_size(px(metrics::BRAND_TEXT_SIZE))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("7zplus"),
                            )
                            .child(
                                v_flex()
                                    .w(px(200.))
                                    .gap(px(metrics::ACTION_GAP))
                                    .mt(px(8.))
                                    .child(
                                        primary("home-open", tr("archive-open"))
                                            .w_full()
                                            .icon(icon("FolderOpen", metrics::ACTION_ICON_SIZE))
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.command(Command::Open, window, cx)
                                            })),
                                    )
                                    .child(
                                        command("home-create", tr("archive-create-command"))
                                            .w_full()
                                            .icon(icon("Add", metrics::ACTION_ICON_SIZE))
                                            .disabled(busy)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.command(Command::Create, window, cx)
                                            })),
                                    ),
                            ),
                    )
                    .child(div().w(px(1.)).h_full().flex_shrink_0().bg(rgb(p.border)))
                    .child(
                        v_flex().flex_1().min_w_0().h_full().child(
                            v_flex()
                                .flex_1()
                                .min_h_0()
                                .gap(px(metrics::HISTORY_GAP))
                                .child(
                                    body_text(tr("recent-title"))
                                        .text_size(px(14.))
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .when(has_history, |el| {
                                    el.child(
                                        div()
                                            .flex_1()
                                            .min_h_0()
                                            .child(self.recent_archives_view(cx)),
                                    )
                                })
                                .when(!has_history, |el| {
                                    el.child(body_text(tr("recent-empty")).text_color(rgb(p.muted)))
                                }),
                        ),
                    ),
            )
    }
}
