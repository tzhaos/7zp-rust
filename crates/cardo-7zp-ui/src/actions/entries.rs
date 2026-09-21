use crate::*;

impl Workspace {
    pub(crate) fn add_to_archive(&mut self, directory: bool, cx: &mut Context<Self>) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let password = self.browser.view().password.clone();
        self.start(tr("archive-editing"), cx, move |cancel| {
            let picker = rfd::FileDialog::new().set_title(tr("archive-add"));
            let paths = if directory {
                picker.pick_folder().map(|path| vec![path])
            } else {
                picker.pick_files()
            };
            let Some(paths) = paths else {
                return Ok(Outcome::Cancelled);
            };
            Request::Edit(catalog, Edit::Add(paths)).run(password, &cancel, None)
        });
    }

    pub(crate) fn delete_entries(&mut self, cx: &mut Context<Self>) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let paths: Vec<_> = self.browser.view().selected.iter().cloned().collect();
        let password = self.browser.view().password.clone();
        self.start(tr("archive-editing"), cx, move |cancel| {
            let answer = rfd::MessageDialog::new()
                .set_title(tr("archive-delete"))
                .set_description(tf(
                    "archive-delete-confirm",
                    &[("count", paths.len().into())],
                ))
                .set_level(rfd::MessageLevel::Warning)
                .set_buttons(rfd::MessageButtons::YesNo)
                .show();
            if answer != rfd::MessageDialogResult::Yes {
                return Ok(Outcome::Cancelled);
            }
            Request::Edit(catalog, Edit::Delete(paths)).run(password, &cancel, None)
        });
    }

    pub(crate) fn rename_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.browser.view().selected.first().cloned() else {
            return;
        };
        let input = cx.new(|cx| InputState::new(window, cx));
        input.update(cx, |input, cx| {
            input.set_value(
                source.rsplit('/').next().unwrap_or(&source).to_owned(),
                window,
                cx,
            )
        });
        self.dialogs
            .observe_input(cx.observe(&input, |_, _, cx| cx.notify()));
        self.dialogs
            .show(tr("archive-rename"), Modal::Rename { source, input });
        cx.notify();
    }

    pub(crate) fn copy_entries(&mut self, move_entries: bool, cx: &mut Context<Self>) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let paths: Vec<_> = self.browser.view().selected.iter().cloned().collect();
        let password = self.browser.view().password.clone();
        let source = catalog.path.clone();
        let progress = cardo_7zp_archive::Progress::new();
        let reporting = progress.clone();
        self.start(tr("extracting"), cx, move |cancel| {
            let Some(destination) = rfd::FileDialog::new()
                .set_title(tr("save-location"))
                .pick_folder()
            else {
                return Ok(Outcome::Cancelled);
            };
            Request::Transfer {
                catalog,
                selected: paths,
                destination,
                move_entries,
            }
            .run(password, &cancel, Some(reporting))
        });
        self.show_extraction_progress(source, None, progress, cx);
    }
}
