use crate::*;

impl Workspace {
    pub(crate) fn open_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.current_item().map(|entry| entry.path.clone()) else {
            return;
        };
        self.open_entry(path, window, cx);
    }

    pub(crate) fn open_inside(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((path, directory)) = self
            .current_item()
            .map(|entry| (entry.path.clone(), entry.directory))
        else {
            return;
        };
        if self.browser.view().directory.is_some() {
            if directory {
                self.visit(
                    browser::Location::Directory(PathBuf::from(path)),
                    window,
                    cx,
                );
            } else if p7z_commands::may_extract(Path::new(&path)) {
                self.execute(Request::Open(PathBuf::from(path)), String::new(), cx);
            }
        } else if directory {
            self.navigate(path, window, cx);
        }
    }

    pub(crate) fn open_outside(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some((path, directory)) = self
            .current_item()
            .map(|entry| (entry.path.clone(), entry.directory))
        else {
            return;
        };
        if self.browser.view().directory.is_some() {
            self.start(tr("opening"), cx, move |_| {
                if directory {
                    p7z_platform::open_directory(Path::new(&path))?;
                } else {
                    p7z_platform::open_file(Path::new(&path))?;
                }
                Ok(Outcome::Cancelled)
            });
            return;
        }
        if directory {
            return;
        }
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        self.execute(
            Request::OpenEntry {
                catalog,
                path,
                preferences: self.preferences.clone(),
            },
            self.browser.view().password.clone(),
            cx,
        );
    }

    pub(crate) fn open_entry(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_busy() {
            return;
        }
        let Some(entry) = self
            .browser
            .view()
            .rows
            .iter()
            .find(|entry| entry.path == path)
        else {
            return;
        };
        if self.browser.view().directory.is_some() {
            if entry.directory {
                self.visit(
                    browser::Location::Directory(PathBuf::from(path)),
                    window,
                    cx,
                );
            } else if p7z_commands::may_extract(Path::new(&path)) {
                self.execute(Request::Open(PathBuf::from(path)), String::new(), cx);
            } else {
                self.start(tr("opening"), cx, move |_| {
                    p7z_platform::open_file(Path::new(&path))?;
                    Ok(Outcome::Cancelled)
                });
            }
        } else if entry.directory {
            self.navigate(path, window, cx);
        } else if let Some(catalog) = self.browser.view().catalog.clone() {
            self.execute(
                Request::OpenEntry {
                    catalog,
                    path,
                    preferences: self.preferences.clone(),
                },
                self.browser.view().password.clone(),
                cx,
            );
        }
    }

    pub(crate) fn add_to_archive(&mut self, directory: bool, cx: &mut Context<Self>) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let password = self.browser.view().password.clone();
        self.start(tr("archive-editing"), cx, move |cancel| {
            let picker = cardo_platform::picker::FileDialog::new().set_title(tr("archive-add"));
            let paths = if directory {
                picker.pick_folder()?.map(|path| vec![path])
            } else {
                picker.pick_files()?
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
        self.show_dialog(
            tr("archive-delete"),
            Modal::ConfirmDelete {
                count: paths.len(),
                request: Request::Edit(catalog, Edit::Delete(paths)),
                password,
            },
            cx,
        );
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
        self.dialogs.watch_input(&input, cx);
        self.dialogs.show(
            tr("archive-rename"),
            Modal::Rename {
                source,
                input,
                error: None,
            },
        );
        cx.notify();
    }

    pub(crate) fn copy_entries(&mut self, move_entries: bool, cx: &mut Context<Self>) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let paths: Vec<_> = self.browser.view().selected.iter().cloned().collect();
        let password = self.browser.view().password.clone();
        let source = catalog.path.clone();
        let progress = p7z_engine::Progress::new();
        let reporting = progress.clone();
        self.start(tr("extracting"), cx, move |cancel| {
            let Some(destination) = cardo_platform::picker::FileDialog::new()
                .set_title(tr("save-location"))
                .pick_folder()?
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
