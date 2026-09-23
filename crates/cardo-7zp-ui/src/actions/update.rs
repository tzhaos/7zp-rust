use crate::*;
use cardo_7zp_requests::update::{self, Phase, Progress, Status};

impl Workspace {
    pub(crate) fn install_update(&mut self, cx: &mut Context<Self>) {
        if self.tasks.is_busy() || self.settings_busy(cx) || self.dialogs.is_open() {
            return;
        }
        let Some(release @ Status::Available { .. }) = self.update_status.clone() else {
            return;
        };
        let progress = Progress::default();
        self.update_transfer = Some(progress.clone());
        self.update_status = Some(Status::Downloading);
        if let Some(form) = &self.settings_form {
            form.update(cx, |form, cx| form.set_update_busy(true, cx));
        }
        let download_release = release.clone();
        let worker_progress = progress.clone();
        let work = cx.background_executor().spawn(async move {
            let prepared = update::prepare(&download_release, &worker_progress)?;
            worker_progress.installing();
            prepared.launch(&worker_progress.cancel)
        });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let result = work.await;
            let _ = view.update(cx, |this, cx| {
                this.update_task = None;
                this.update_transfer = None;
                if let Some(form) = &this.settings_form {
                    form.update(cx, |form, cx| form.set_update_busy(false, cx));
                }
                match result {
                    Ok(()) => cx.quit(),
                    Err(error) => {
                        this.update_status = Some(release);
                        if progress.cancel.load(Ordering::Relaxed) {
                            this.notify_message(tr("update-cancelled").into());
                        } else {
                            tracing::error!(error = %format!("{error:#}"), "Cannot prepare update");
                            this.show_dialog(
                                tr("update-title"),
                                Modal::Error(crate::dialogs::ErrorDialog {
                                    summary: tr("update-install-error").into(),
                                    path: None,
                                    details: format!("{error:#}"),
                                    expanded: false,
                                }),
                                cx,
                            );
                        }
                    }
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(crate) fn cancel_update(&mut self, cx: &mut Context<Self>) {
        if let Some(progress) = &self.update_transfer {
            if progress.phase() != Phase::Install {
                progress.cancel.store(true, Ordering::Relaxed);
                cx.notify();
            }
        }
    }
}
