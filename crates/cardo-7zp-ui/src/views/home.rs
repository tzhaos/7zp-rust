use crate::commands::Command;
use crate::theme::metrics::home as metrics;
use crate::*;
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, h_flex, v_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn home_view(&self, cx: &mut Context<Self>) -> Div {
        let palette = crate::theme::palette(cx);
        let busy = self.tasks.is_busy() || self.settings_busy(cx);
        let has_history = self.matching_recent_archives(cx).next().is_some();
        v_flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .p(px(metrics::PADDING))
            .items_center()
            .child(
                v_flex()
                    .size_full()
                    .min_h_0()
                    .max_w(px(metrics::MAX_WIDTH))
                    .gap(px(28.))
                    .child(
                        h_flex()
                            .w_full()
                            .flex_shrink_0()
                            .justify_center()
                            .gap(px(metrics::ACTION_GAP))
                            .child(
                                primary("home-open", tr("archive-open"))
                                    .icon(icon("FolderOpen", metrics::ACTION_ICON_SIZE))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.command(Command::Open, window, cx)
                                    })),
                            )
                            .child(
                                command("home-create", tr("archive-create-command"))
                                    .custom(subtle_variant(cx).color(rgb(palette.surface).into()))
                                    .border_color(rgb(palette.border))
                                    .icon(icon("Add", metrics::ACTION_ICON_SIZE))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.command(Command::Create, window, cx)
                                    })),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .w_full()
                            .max_w(px(metrics::HISTORY_MAX_WIDTH))
                            .mx_auto()
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
                                el.child(
                                    body_text(tr("recent-empty"))
                                        .text_color(rgb(crate::theme::palette(cx).muted)),
                                )
                            }),
                    ),
            )
    }
}
