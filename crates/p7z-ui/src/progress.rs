use cardo_ui::ConditionalBuilder;
use super::*;
use gpui_kit::component::{Disableable, h_flex, progress::Progress};

use std::time::Instant;

pub(super) struct ExtractionProgress {
    source: PathBuf,
    destination: Option<PathBuf>,
    progress: p7z_engine::Progress,
    started: Instant,
}

impl Workspace {
    pub(super) fn show_extraction_progress(
        &mut self,
        source: PathBuf,
        destination: Option<PathBuf>,
        progress: p7z_engine::Progress,
        cx: &mut Context<Self>,
    ) {
        self.tasks.show_extraction(ExtractionProgress {
            source,
            destination,
            progress,
            started: Instant::now(),
        });
        self.show_dialog(tr("extract-progress-title"), Modal::Progress, cx);
    }

    pub(super) fn extraction_progress_view(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(task) = self.tasks.extraction() else {
            return div();
        };
        let p = crate::theme::palette(cx);
        let percent = task.progress.percent();
        let stopping = self.tasks.cancelled();
        let elapsed = task.started.elapsed().as_secs();
        let status = if stopping {
            tr("cancelling")
        } else if percent == Some(100) {
            tr("extract-finalizing")
        } else if percent.is_none() {
            tr("extract-preparing")
        } else {
            tr("extracting")
        };
        panel_layout(cx)
            .child(
                panel_body("progress-body")
                    .child(
                        panel_card(cx)
                            .child(
                                h_flex()
                                    .gap(px(16.))
                                    .items_center()
                                    .child(artwork(ToolIcon::Extract, window, cx))
                                    .child(
                                        div().flex_1().min_w_0().whitespace_normal().child(status),
                                    )
                                    .when_some(percent, |el, value| {
                                        el.child(
                                            div()
                                                .flex_shrink_0()
                                                .text_size(px(18.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(format!("{value}%")),
                                        )
                                    }),
                            )
                            .child(
                                Progress::new("extraction-progress")
                                    .value(f32::from(percent.unwrap_or(0)))
                                    .loading(percent.is_none() && !stopping)
                                    .color(rgb(p.accent))
                                    .accessibility_label(tr("extract-progress-title")),
                            ),
                    )
                    .child(panel_field(
                        tr("extract-progress-source"),
                        path_strip("progress-source", task.source.display().to_string(), cx),
                    ))
                    .when_some(task.destination.clone(), |el, destination| {
                        el.child(panel_field(
                            tr("extract-progress-destination"),
                            path_strip(
                                "progress-destination",
                                destination.display().to_string(),
                                cx,
                            ),
                        ))
                    })
                    .child(
                        div().text_size(px(12.)).text_color(rgb(p.muted)).child(tf(
                            "extract-progress-elapsed",
                            &[(
                                "time",
                                format!(
                                    "{:02}:{:02}:{:02}",
                                    elapsed / 3600,
                                    elapsed / 60 % 60,
                                    elapsed % 60
                                )
                                .into(),
                            )],
                        )),
                    ),
            )
            .child(
                self.action_row(cx).child(
                    panel_button("stop-extraction", tr("extract-stop"))
                        .disabled(stopping)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tasks.cancel(tr("cancelling"));
                            cx.notify();
                        })),
                ),
            )
    }
}
