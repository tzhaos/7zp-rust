use crate::*;
use gpui_kit::{component::v_flex, prelude::FluentBuilder};

impl Workspace {
    pub(super) fn update_view(
        &self,
        status: &cardo_7zp_requests::update::Status,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        {
            use cardo_7zp_requests::update::Status;
            let description = match status {
                Status::Checking => tr("update-checking").to_owned(),
                Status::Unconfigured => tr("update-unconfigured").to_owned(),
                Status::Current => tr("update-current").to_owned(),
                Status::Available { version, .. } => {
                    tf("update-available", &[("version", version.as_str().into())])
                }
                Status::Failed(error) => tf("update-failed", &[("error", error.as_str().into())]),
            };
            let actions = match status {
                Status::Available { download, release, .. } => Some((download.clone(), release.clone())),
                _ => None,
            };
            let retry = !matches!(status, Status::Checking);
            v_flex()
                .flex_1()
                .min_h_0()
                .child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .px(px(24.))
                        .py(px(20.))
                        .gap(px(16.))
                        .child(tf(
                            "update-installed",
                            &[("version", cardo_7zp_requests::update::VERSION.into())],
                        ))
                        .child(description)
                        .when(actions.is_some(), |el| el.child(tr("update-package-note"))),
                )
                .when(actions.is_some() || retry, |el| {
                    el.child(self.action_row(cx).when_some(actions, |row, (download, release)| {
                        row.child(
                            primary("update-download", tr("update-download"))
                                .on_click(move |_, _, cx| cx.open_url(&download)),
                        )
                        .child(
                            command("update-notes", tr("update-notes"))
                                .on_click(move |_, _, cx| cx.open_url(&release)),
                        )
                    }).when(retry, |row| {
                        row.child(
                            command("update-retry", tr("update-check"))
                                .on_click(cx.listener(|this, _, _, cx| this.check_update(cx))),
                        )
                    }))
                })
                .into_any_element()
        }
    }
}
