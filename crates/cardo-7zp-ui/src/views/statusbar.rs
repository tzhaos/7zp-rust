use crate::*;
use cardo_7zp_core::i18n::{tf, tr};
use gpui_kit::{
    component::{Disableable, button::ButtonVariants, h_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(crate) fn statusbar(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
        let browser = self.browser.view();
        let count = if browser.catalog.is_none() && browser.directory.is_none() {
            self.matching_recent_archives(cx).count()
        } else {
            browser.rows.len()
        };
        let summary = if self.tasks.is_busy() {
            self.tasks.status().to_owned()
        } else if !browser.selected.is_empty() {
            let selected_size = browser
                .catalog
                .as_ref()
                .map(|c| {
                    c.entries
                        .iter()
                        .filter(|e| {
                            !e.directory
                                && browser
                                    .selected
                                    .iter()
                                    .any(|p| e.path == *p || e.path.starts_with(&format!("{p}/")))
                        })
                        .map(|e| e.size.unwrap_or(0))
                        .sum()
                })
                .unwrap_or_else(|| {
                    browser
                        .rows
                        .iter()
                        .filter(|e| browser.selected.contains(&e.path))
                        .filter_map(|e| e.size)
                        .sum()
                });
            tf(
                "selection-summary",
                &[
                    ("count", browser.selected.len().into()),
                    ("size", size_text(selected_size).into()),
                ],
            )
        } else {
            tf(
                if self.search.read(cx).value().is_empty() {
                    "status-items"
                } else {
                    "status-results"
                },
                &[("count", count.into())],
            )
        };
        h_flex()
            .relative()
            .h(px(36.))
            .flex_shrink_0()
            .px(px(16.))
            .gap(px(8.))
            .items_center()
            .bg(rgb(p.surface))
            .border_t_1()
            .border_color(rgb(p.border))
            .text_size(px(11.))
            .text_color(rgb(p.muted))
            .child(div().flex_1().min_w_0().truncate().child(summary))
            .when(
                !self.tasks.is_busy() && !browser.selected.is_empty(),
                |el| {
                    el.child(
                        icon_button(
                            "clear-selection",
                            "Dismiss",
                            tr("selection-clear"),
                            self.allow_hint(true),
                            cx,
                        )
                        .custom(subtle_variant(cx))
                        .border_0()
                        .w(px(24.))
                        .h(px(24.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.browser.clear_selection(false);
                            cx.notify();
                        })),
                    )
                },
            )
            .when(self.tasks.is_busy(), |el| {
                el.child(
                    command("cancel", tr("cancel"))
                        .custom(subtle_variant(cx))
                        .border_0()
                        .h(px(24.))
                        .flex_shrink_0()
                        .disabled(self.tasks.cancelled())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tasks.cancel(tr("cancelling"));
                            cx.notify();
                        })),
                )
            })
            .when_some(browser.catalog.as_ref(), |el, catalog| {
                let encrypted = catalog.entries.iter().any(|entry| entry.encrypted);
                el.child(
                    h_flex()
                        .flex_shrink_0()
                        .h(px(22.))
                        .px(px(7.))
                        .gap(px(5.))
                        .rounded(px(4.))
                        .bg(rgb(p.selected))
                        .text_color(rgb(p.text))
                        .text_size(px(10.))
                        .when(encrypted, |el| el.child(icon("LockClosed", 12.)))
                        .child(catalog.format.clone()),
                )
            })
            .when(self.tasks.is_busy(), |el| {
                el.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top_0()
                        .h(px(2.))
                        .overflow_hidden()
                        .bg(rgb(p.selected))
                        .child(
                            div()
                                .absolute()
                                .h_full()
                                .w(relative(0.3))
                                .bg(rgb(p.accent))
                                .with_animation(
                                    "operation-progress",
                                    Animation::new(std::time::Duration::from_millis(1500)).repeat(),
                                    |el, delta| el.left(relative(delta * 1.3 - 0.3)),
                                ),
                        ),
                )
            })
    }
}
