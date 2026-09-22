use crate::dialogs::ErrorDialog;
use crate::*;

impl Workspace {
    pub(crate) fn show_task_error(
        &mut self,
        error: anyhow::Error,
        extracting: bool,
        cx: &mut Context<Self>,
    ) {
        let mut title = tr(if extracting {
            "extract-failed"
        } else {
            "operation-failed"
        });
        let mut summary = tr("operation-failed-summary");
        let mut path = None;
        if let Some(source) = error.downcast_ref::<cardo_7zp_engine::SourceError>() {
            path = Some(source.path.clone());
            match source.source.kind() {
                std::io::ErrorKind::NotFound => {
                    title = tr("archive-missing-title");
                    let in_history = self.history.iter().any(|entry| entry.path == source.path);
                    summary = tr(if in_history {
                        "archive-missing-history"
                    } else {
                        "archive-missing-summary"
                    });
                    self.history.retain(|entry| entry.path != source.path);
                    self.update_recent(recent::Change::Remove(source.path.clone()), cx);
                }
                std::io::ErrorKind::PermissionDenied => {
                    title = tr("archive-access-title");
                    summary = tr("archive-access-summary");
                }
                _ => {
                    title = tr("archive-access-title");
                    summary = tr("archive-read-summary");
                }
            }
        } else {
            self.reload_history(cx);
        }
        self.message = None;
        self.show_dialog(
            title,
            Modal::Error(ErrorDialog {
                summary: summary.into(),
                path,
                details: format!("{error:#}"),
                expanded: false,
            }),
            cx,
        );
    }
}
