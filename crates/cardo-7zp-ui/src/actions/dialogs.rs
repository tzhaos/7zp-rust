use crate::*;

impl Workspace {
    pub(crate) fn close_modal(&mut self, cx: &mut Context<Self>) {
        if self.settings_busy(cx) {
            return;
        }
        match self.dialogs.current() {
            Some(Modal::Progress) => return,
            Some(Modal::Password { .. }) => {
                self.tasks.set_close_after(false);
                self.after_open = None;
                self.browser.cancel_navigation();
            }
            Some(Modal::ConfirmRun { .. }) => self.pending_run = None,
            Some(Modal::Conflict { .. }) => {
                self.tasks.set_close_after(false);
                self.tasks.set_close_archive(false);
                self.extract_follow.clear();
            }
            _ => {}
        }
        self.dialogs.take();
        self.dialogs.close_prompt(cx);
        cx.notify();
    }

    pub(crate) fn submit_rename(&mut self, cx: &mut Context<Self>) {
        let Some(Modal::Rename { source, input, .. }) = self.dialogs.current() else {
            return;
        };
        let name = input.read(cx).value().trim().to_owned();
        if name.is_empty() || name.contains(['/', '\\']) {
            if let Some(Modal::Rename { error, .. }) = self.dialogs.current_mut() {
                *error = Some(tr("archive-name-invalid").into());
            }
            cx.notify();
            return;
        }
        let source = source.clone();
        let destination = source
            .rsplit_once('/')
            .map(|(parent, _)| format!("{parent}/{name}"))
            .unwrap_or(name);
        if let Some(catalog) = self.browser.view().catalog.clone() {
            self.execute(
                Request::Edit(
                    catalog,
                    Edit::Rename {
                        source,
                        destination,
                    },
                ),
                self.browser.view().password.clone(),
                cx,
            );
        }
    }

    pub(crate) fn submit_password(&mut self, cx: &mut Context<Self>) {
        let Some(Modal::Password { input, .. }) = self.dialogs.current() else {
            return;
        };
        if input.read(cx).value().is_empty() {
            return;
        }
        let Some(Modal::Password { input, request }) = self.dialogs.take() else {
            return;
        };
        let password = input.read(cx).value().to_string();
        self.browser.set_password(password.clone());
        self.execute(request, password, cx);
    }

    pub(crate) fn submit_comment(&mut self, cx: &mut Context<Self>) {
        if let (Some(Modal::Comment(input)), Some(catalog)) =
            (self.dialogs.current(), &self.browser.view().catalog)
        {
            let request = Request::Comment(catalog.clone(), input.read(cx).value().to_string());
            self.execute(request, self.browser.view().password.clone(), cx);
        }
    }

    pub(crate) fn submit_extract(&mut self, cx: &mut Context<Self>) {
        if let Some(Modal::Extract {
            folder,
            selected,
            destination,
            open_after,
        }) = self.dialogs.take()
        {
            self.dialogs.close_prompt(cx);
            let folder = folder.read(cx).value().trim().to_owned();
            self.extract(selected, folder, destination, open_after, cx);
        }
    }

    pub(crate) fn confirm_run(&mut self, cx: &mut Context<Self>) {
        if let Some((directory, target)) = self.pending_run.take() {
            self.tasks.retain_file(directory);
            self.execute(Request::Launch(target), String::new(), cx);
        }
    }

    pub(crate) fn confirm_delete(&mut self, cx: &mut Context<Self>) {
        if let Some(Modal::ConfirmDelete {
            request, password, ..
        }) = self.dialogs.take()
        {
            self.execute(request, password, cx);
        }
    }
}
