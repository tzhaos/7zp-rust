use crate::*;
use gpui_kit::{
    component::{Disableable, h_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(super) fn about_view(
        &self,
        status: Option<&zip_requests::update::Status>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use zip_requests::update::Status;
        let p = crate::theme::palette(cx);
        let (name, color, description) = match status {
            None => ("Info", p.muted, tr("update-not-checked").to_owned()),
            Some(Status::Checking) => ("ArrowDownload", p.accent, tr("update-checking").to_owned()),
            Some(Status::Unconfigured) => ("Info", p.muted, tr("update-unconfigured").to_owned()),
            Some(Status::Current) => (
                "CheckmarkCircle",
                p.success,
                tr("update-current").to_owned(),
            ),
            Some(Status::Downloading) => {
                use zip_requests::update::Phase;
                let description = self
                    .update_transfer
                    .as_ref()
                    .map(|progress| {
                        if progress.cancel.load(Ordering::Relaxed) {
                            return tr("update-cancelling").into();
                        }
                        match progress.phase() {
                            Phase::Download => tf(
                                "update-download-progress",
                                &[
                                    ("received", size_text(progress.received()).into()),
                                    ("total", size_text(progress.total()).into()),
                                ],
                            ),
                            Phase::Verify => tr("update-verifying").into(),
                            Phase::Prepare => tr("update-preparing").into(),
                            Phase::Install => tr("update-installing").into(),
                        }
                    })
                    .unwrap_or_else(|| tr("update-preparing").into());
                ("ArrowDownload", p.accent, description)
            }
            Some(Status::Available { version, .. }) => (
                "ArrowDownload",
                p.accent,
                tf("update-available", &[("version", version.as_str().into())]),
            ),
            Some(Status::Failed(error)) => (
                "Info",
                p.danger,
                tf("update-failed", &[("error", error.as_str().into())]),
            ),
        };
        let mut actions = h_flex()
            .min_w_0()
            .flex_wrap()
            .justify_end()
            .gap(px(8.))
            .child(bubble_tooltip(
                div()
                    .id("update-source-hint")
                    .aria_label(tr("settings-update-source"))
                    .flex_shrink_0()
                    .text_color(rgb(color))
                    .child(icon(name, 16.)),
                tr("settings-update-source"),
            ));
        if let Some(Status::Available { release, .. }) = status {
            let release = release.clone();
            actions = actions
                .child(
                    settings_action("update-notes", tr("update-notes"), cx).on_click(
                        cx.listener(move |this, _, _, cx| this.open_update_link(&release, cx)),
                    ),
                )
                .child(
                    settings_primary("update-download", tr("update-install"), cx)
                        .disabled(self.tasks.is_busy() || self.settings_busy(cx))
                        .icon(icon("ArrowDownload", 16.))
                        .on_click(cx.listener(|this, _, _, cx| this.install_update(cx))),
                );
        } else if matches!(status, Some(Status::Downloading)) {
            actions = actions.child(
                settings_action("update-cancel", tr("cancel"), cx)
                    .disabled(self.update_transfer.as_ref().is_none_or(|progress| {
                        progress.phase() == zip_requests::update::Phase::Install
                            || progress.cancel.load(Ordering::Relaxed)
                    }))
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_update(cx))),
            );
        } else {
            actions = actions.child(
                settings_action("update-retry", tr("update-check"), cx)
                    .disabled(
                        self.settings_controls_disabled(cx)
                            || matches!(status, Some(Status::Checking)),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.check_update(cx))),
            );
        }
        let rows = [
            settings_detail(tr("settings-update-status"), &description, actions, cx)
                .into_any_element(),
        ];
        let mut body = settings_frame()
            .child(settings_group(
                [
                    ("name", "7zplus"),
                    ("about-version", zip_requests::update::VERSION),
                    ("about-engine", "7-Zip 26.03"),
                ]
                .into_iter()
                .map(|(label, value)| {
                    settings_row(tr(label), div().child(value), cx).into_any_element()
                }),
                cx,
            ))
            .child(settings_section(
                tr("settings-update-page"),
                settings_group(rows, cx),
                cx,
            ))
            .when(matches!(status, Some(Status::Available { .. })), |body| {
                body.child(body_text(tr("update-package-note")).text_color(rgb(p.muted)))
            });
        if let Some(form) = &self.settings_form {
            body = body.child(settings_section(
                tr("settings-update-preferences"),
                gpui_kit::component::v_flex().min_w_0().child(form.clone()),
                cx,
            ));
        }
        settings_page(cx)
            .child(settings_content("about-body", body))
            .into_any_element()
    }
}
