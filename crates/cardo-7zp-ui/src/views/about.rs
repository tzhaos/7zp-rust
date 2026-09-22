use crate::*;
use gpui_kit::component::{Disableable, h_flex};

impl Workspace {
    pub(super) fn about_view(
        &self,
        status: Option<&cardo_7zp_requests::update::Status>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use cardo_7zp_requests::update::Status;
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
        let mut rows = vec![
            div()
                .py(px(crate::theme::metrics::settings::ROW_PADDING))
                .child(panel_notice(name, description, color))
                .into_any_element(),
        ];
        let mut actions = h_flex()
            .w_full()
            .flex_wrap()
            .justify_end()
            .gap(px(8.))
            .py(px(crate::theme::metrics::settings::ROW_PADDING));
        if let Some(Status::Available {
            download, release, ..
        }) = status
        {
            let download = download.clone();
            let release = release.clone();
            rows.push(
                div()
                    .py(px(crate::theme::metrics::settings::ROW_PADDING))
                    .text_color(rgb(p.muted))
                    .whitespace_normal()
                    .child(tr("update-package-note"))
                    .into_any_element(),
            );
            actions = actions
                .child(command("update-notes", tr("update-notes")).on_click(
                    cx.listener(move |this, _, _, cx| this.open_update_link(&release, cx)),
                ))
                .child(
                    primary("update-download", tr("update-download"))
                        .icon(icon("ArrowDownload", 16.))
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.open_update_link(&download, cx)),
                        ),
                );
        } else {
            actions = actions.child(
                command("update-retry", tr("update-check"))
                    .disabled(self.settings_busy(cx) || matches!(status, Some(Status::Checking)))
                    .on_click(cx.listener(|this, _, _, cx| this.check_update(cx))),
            );
        }
        rows.push(actions.into_any_element());
        let mut body = settings_content("about-body")
            .child(settings_group(
                [
                    ("name", "7zplus"),
                    ("about-version", cardo_7zp_requests::update::VERSION),
                    ("about-engine", "7-Zip 26.03"),
                ]
                .into_iter()
                .map(|(label, value)| {
                    settings_row(tr(label), div().child(value), cx).into_any_element()
                }),
                cx,
            ))
            .child(settings_section(
                tr("settings-update-status"),
                settings_group(rows, cx),
                cx,
            ));
        if let Some(form) = &self.settings_form {
            body = body.child(settings_section(
                tr("settings-update-preferences"),
                gpui_kit::component::v_flex().min_w_0().child(form.clone()),
                cx,
            ));
        }
        settings_page(cx).child(body).into_any_element()
    }
}
