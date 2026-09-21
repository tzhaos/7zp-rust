use super::*;

use crate::settings::recent::apply;
pub(super) use crate::settings::recent::{Change, Kind};

impl Workspace {
    pub(super) fn reload_history(&mut self, cx: &mut Context<Self>) {
        self.update_recent(Change::Load, cx);
        self.update_history(Change::Load, Kind::Folders, cx);
    }

    pub(super) fn update_recent(&mut self, change: Change, cx: &mut Context<Self>) {
        self.update_history(change, Kind::Archives, cx);
    }

    pub(super) fn update_history(&mut self, change: Change, kind: Kind, cx: &mut Context<Self>) {
        if !self.preferences.remember_recent && matches!(change, Change::Remember(_)) {
            return;
        }
        let previous = self.recent_task.take();
        self.recent_task = Some(cx.spawn(async move |view, cx| {
            // Keep startup loading and subsequent writes in user action order.
            if let Some(previous) = previous {
                previous.await;
            }
            let result = cx
                .background_executor()
                .spawn(async move { apply(change, kind) })
                .await;
            let _ = view.update(cx, |this, cx| {
                match result {
                    Ok(paths) => match kind {
                        Kind::Archives => this.recent_archives = paths,
                        Kind::Folders => this.recent_folders = paths,
                    },
                    Err(error) => {
                        this.message = Some(tf(
                            "recent-storage-error",
                            &[("error", error.to_string().into())],
                        ))
                    }
                }
                cx.notify();
            });
        }));
    }
}
