use crate::*;
use gpui_kit::{
    component::{h_flex, v_flex},
    prelude::FluentBuilder,
};

impl Workspace {
    pub(super) fn update_view(
        &self,
        status: &cardo_7zp_application::update::Status,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        {
            use cardo_7zp_application::update::Status;
            let description = match status {
                Status::Checking => tr("update-checking").to_owned(),
                Status::Unconfigured => tr("update-unconfigured").to_owned(),
                Status::Current => tr("update-current").to_owned(),
                Status::Available { version, .. } => {
                    tf("update-available", &[("version", version.as_str().into())])
                }
                Status::Failed(error) => tf("update-failed", &[("error", error.as_str().into())]),
            };
            v_flex()
                .px(px(24.))
                .py(px(22.))
                .gap(px(16.))
                .child(tf(
                    "update-installed",
                    &[("version", cardo_7zp_application::update::VERSION.into())],
                ))
                .child(description)
                .when(matches!(status, Status::Available { .. }), |el| {
                    let Status::Available {
                        download, release, ..
                    } = status
                    else {
                        return el;
                    };
                    let download = download.clone();
                    let release = release.clone();
                    el.child(tr("update-package-note")).child(
                        h_flex()
                            .gap(px(8.))
                            .child(
                                primary("update-download", tr("update-download"))
                                    .on_click(move |_, _, cx| cx.open_url(&download)),
                            )
                            .child(
                                command("update-notes", tr("update-notes"))
                                    .on_click(move |_, _, cx| cx.open_url(&release)),
                            ),
                    )
                })
                .when(!matches!(status, Status::Checking), |el| {
                    el.child(
                        command("update-retry", tr("update-check"))
                            .on_click(cx.listener(|this, _, _, cx| this.check_update(cx))),
                    )
                })
                .into_any_element()
        }
    }
}
