use crate::*;
use gpui_kit::component::{checkbox::Checkbox, h_flex};

impl Workspace {
    pub(super) fn conflict_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(Modal::Conflict {
            plan,
            index,
            repeat,
            ..
        }) = self.dialogs.current()
        else {
            return div().into_any_element();
        };
        let Some(conflict) = plan.conflicts.get(*index) else {
            return div().into_any_element();
        };
        let p = crate::theme::palette(cx);
        let file = |label: &'static str, bytes: Option<u64>, modified: &str| {
            panel_card(cx)
                .flex_1()
                .min_w(px(220.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(p.muted))
                        .child(label),
                )
                .child(panel_property(
                    tr("size"),
                    bytes
                        .map(size_text)
                        .unwrap_or_else(|| tr("metadata-unavailable").to_owned()),
                    cx,
                ))
                .child(panel_property(
                    tr("modified"),
                    if modified.is_empty() {
                        tr("metadata-unavailable")
                    } else {
                        modified
                    }
                    .to_owned(),
                    cx,
                ))
        };
        panel_layout(cx)
            .child(
                panel_body("conflict-body")
                    .child(panel_notice(
                        "Info",
                        tr("extract-conflict-exists"),
                        p.accent,
                    ))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(
                                conflict
                                    .destination
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned(),
                            ),
                    )
                    .child(path_strip(
                        "conflict-path",
                        conflict.destination.display().to_string(),
                        cx,
                    ))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .w_full()
                            .items_start()
                            .gap(px(12.))
                            .child(file(
                                tr("extract-conflict-incoming"),
                                conflict.incoming_size,
                                &conflict.incoming_modified,
                            ))
                            .child(file(
                                tr("extract-conflict-existing"),
                                Some(conflict.existing_size),
                                &conflict.existing_modified,
                            )),
                    )
                    .child(
                        Checkbox::new("conflict-repeat")
                            .label(tr("conflict-repeat"))
                            .text_size(px(12.))
                            .checked(*repeat)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                if let Some(Modal::Conflict { repeat, .. }) =
                                    this.dialogs.current_mut()
                                {
                                    *repeat = *checked;
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(
                panel_actions(cx)
                    .child(
                        panel_button("conflict-cancel", tr("cancel"))
                            .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                    )
                    .child(panel_button("conflict-skip", tr("conflict-skip")).on_click(
                        cx.listener(|this, _, _, cx| this.apply_conflict(Overwrite::Skip, cx)),
                    ))
                    .child(
                        panel_button("conflict-rename", tr("conflict-rename")).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.apply_conflict(Overwrite::RenameIncoming, cx)
                            }),
                        ),
                    )
                    .child(
                        panel_danger("conflict-replace", tr("conflict-replace")).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.apply_conflict(Overwrite::Replace, cx)
                            }),
                        ),
                    ),
            )
            .into_any_element()
    }
}
