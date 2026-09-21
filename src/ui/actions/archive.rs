use crate::ui::*;

impl Workspace {
    pub(in crate::ui) fn open(&mut self, cx: &mut Context<Self>) {
        self.start(tr("opening"), cx, |cancel| {
            let Some(path) = rfd::FileDialog::new()
                .set_title(tr("archive-open"))
                .pick_file()
            else {
                return Ok(Outcome::Cancelled);
            };
            match Engine::bundled()?.list(&path, "", &cancel) {
                Ok(c) => Ok(Outcome::Opened(c, String::new())),
                Err(error) if crate::archive::needs_password(&error) => {
                    Ok(Outcome::Password(Request::Open(path)))
                }
                Err(error) => Err(error),
            }
        });
    }

    pub(in crate::ui) fn create_dialog(
        &mut self,
        files: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        let owner = cx.entity().downgrade();
        self.modal = Some(Modal::Create(
            cx.new(|cx| CreateForm::new(owner, files, window, cx)),
        ));
        self.modal_title = tr("create-title").into();
        cx.notify();
    }

    pub(in crate::ui) fn create(
        &mut self,
        files: Vec<PathBuf>,
        name: String,
        options: CreateOptions,
        email: bool,
        cx: &mut Context<Self>,
    ) {
        self.start(tr("creating"), cx, move |cancel| {
            if name.contains(['/', '\\', ':']) {
                bail!(tr("archive-name-invalid"));
            }
            let extension = match options.format.as_str() {
                "gzip" => "gz",
                "bzip2" => "bz2",
                format => format,
            };
            let filename = if options.split == "none" {
                format!("{name}.{extension}")
            } else {
                format!("{name}.{extension}-volumes.zip")
            };
            let Some(destination) = rfd::FileDialog::new()
                .set_title(tr("archive-save"))
                .set_file_name(filename)
                .save_file()
            else {
                return Ok(Outcome::Cancelled);
            };
            let engine = Engine::bundled()?;
            let path = engine.create(&files, &destination, &options, &cancel)?;
            if email {
                crate::platform::mail::compose(&path).map_err(|error| {
                    error.context(tf(
                        "mail-archive-saved",
                        &[("path", path.display().to_string().into())],
                    ))
                })?;
            }
            let password = if options.split == "none" {
                options.password
            } else {
                String::new()
            };
            Ok(Outcome::Created(
                engine.list(&path, &password, &cancel)?,
                password,
            ))
        });
    }

    pub(in crate::ui) fn extract_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(c) = &self.catalog else { return };
        let name = sevenzip_shell_api::extract_folder(&c.path);
        let folder = cx.new(|cx| InputState::new(window, cx));
        folder.update(cx, |input, cx| input.set_value(name, window, cx));
        self.modal_input = Some(cx.observe(&folder, |_, _, cx| cx.notify()));
        self.modal = Some(Modal::Extract {
            folder,
            selected: !self.selected.is_empty(),
            destination: self.destination,
            overwrite: Overwrite::RenameIncoming,
            open_after: self.preferences.open_after,
        });
        self.modal_title = tr("extract-options").into();
        cx.notify();
    }

    pub(in crate::ui) fn choose_extract_directory(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let handle = window.window_handle();
        let picker = cx.background_executor().spawn(async {
            rfd::FileDialog::new()
                .set_title(tr("save-location"))
                .pick_folder()
        });
        cx.spawn(async move |view, cx| {
            if let Some(path) = picker.await {
                let _ = handle.update(cx, |_, _, cx| {
                    view.update(cx, |this, cx| {
                        if let Some(Modal::Extract { destination, .. }) = &mut this.modal {
                            if let Some(index) = this
                                .destinations
                                .iter()
                                .position(|(kind, _)| *kind == DestinationKind::Custom)
                            {
                                this.destinations[index].1 = path;
                                *destination = index;
                            } else {
                                *destination = this.destinations.len();
                                this.destinations.push((DestinationKind::Custom, path));
                            }
                            cx.notify();
                        }
                    })
                });
            }
        })
        .detach();
    }

    pub(in crate::ui) fn quick_extract(&mut self, selected: bool, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(catalog) = self.catalog.clone() else {
            return;
        };
        let Some(parent) = catalog.path.parent().map(Path::to_path_buf) else {
            return;
        };
        let folder = sevenzip_shell_api::extract_folder(&catalog.path);
        self.close_after_task = self.preferences.close_after_quick;
        self.execute(
            Request::Extract {
                catalog,
                selected: if selected {
                    self.selected.iter().cloned().collect()
                } else {
                    Vec::new()
                },
                parent,
                folder,
                overwrite: Overwrite::RenameIncoming,
                open_after: self.preferences.open_after,
            },
            self.password.clone(),
            cx,
        );
    }

    pub(in crate::ui) fn extract(
        &mut self,
        selected: bool,
        folder: String,
        destination: usize,
        overwrite: Overwrite,
        open_after: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(catalog) = self.catalog.clone() else {
            return;
        };
        let Some((_, parent)) = self.destinations.get(destination) else {
            self.message = Some(tr("destinations-unavailable").into());
            cx.notify();
            return;
        };
        let parent = parent.clone();
        self.destination = destination;
        let saved_parent = parent.clone();
        let save = cx.background_executor().spawn(async move {
            let root = crate::settings::directory()?;
            std::fs::create_dir_all(&root)?;
            std::fs::write(
                root.join("destination"),
                saved_parent.to_string_lossy().as_bytes(),
            )?;
            anyhow::Ok(())
        });
        cx.spawn(async move |view, cx| {
            if let Err(error) = save.await {
                let _ = view.update(cx, |this, cx| {
                    this.message = Some(tf(
                        "destination-save-error",
                        &[("error", error.to_string().into())],
                    ));
                    cx.notify();
                });
            }
        })
        .detach();
        self.execute(
            Request::Extract {
                catalog,
                selected: if selected {
                    self.selected.iter().cloned().collect()
                } else {
                    Vec::new()
                },
                parent,
                folder,
                overwrite,
                open_after,
            },
            self.password.clone(),
            cx,
        );
    }

    pub(in crate::ui) fn properties(&mut self, selected: bool, cx: &mut Context<Self>) {
        let Some(c) = &self.catalog else { return };
        let name = c
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let mut fields;
        if selected {
            let contents: Vec<_> = c
                .entries
                .iter()
                .filter(|e| {
                    self.selected
                        .iter()
                        .any(|p| e.path == *p || e.path.starts_with(&format!("{p}/")))
                })
                .collect();
            let single = (self.selected.len() == 1)
                .then(|| self.rows.iter().find(|e| self.selected.contains(&e.path)))
                .flatten();
            fields = vec![
                (
                    tr("name").into(),
                    single.map(|e| e.name.clone()).unwrap_or_else(|| {
                        tf("item-count", &[("count", self.selected.len().into())])
                    }),
                ),
                (
                    tr("type").into(),
                    single
                        .map(|e| file_kind(&e.name, e.directory).into())
                        .unwrap_or_else(|| tr("multiple-items").into()),
                ),
                (
                    tr("location").into(),
                    self.selected.iter().cloned().collect::<Vec<_>>().join("\n"),
                ),
                (tr("archive").into(), name),
                (
                    tr("original-size").into(),
                    size_text(
                        contents
                            .iter()
                            .filter(|e| !e.directory)
                            .map(|e| e.size.unwrap_or(0))
                            .sum(),
                    ),
                ),
                (
                    tr("contains").into(),
                    tf(
                        "contents-count",
                        &[
                            (
                                "files",
                                contents.iter().filter(|e| !e.directory).count().into(),
                            ),
                            (
                                "folders",
                                contents.iter().filter(|e| e.directory).count().into(),
                            ),
                        ],
                    ),
                ),
            ];
            if let Some(e) = single
                && !e.modified.is_empty()
            {
                fields.push((tr("modified").into(), e.modified.clone()));
            }
            fields.push((
                tr("encrypted").into(),
                if contents.iter().any(|e| e.encrypted) {
                    tr("yes")
                } else {
                    tr("no")
                }
                .into(),
            ));
        } else {
            fields = vec![
                (tr("name").into(), name),
                (tr("format").into(), c.format.clone()),
                (tr("packed-size").into(), size_text(c.size)),
                (
                    tr("original-size").into(),
                    size_text(
                        c.entries
                            .iter()
                            .filter(|e| !e.directory)
                            .map(|e| e.size.unwrap_or(0))
                            .sum(),
                    ),
                ),
                (
                    tr("file-count").into(),
                    c.entries
                        .iter()
                        .filter(|e| !e.directory)
                        .count()
                        .to_string(),
                ),
                (
                    tr("encrypted").into(),
                    if c.entries.iter().any(|e| e.encrypted) {
                        tr("yes")
                    } else {
                        tr("no")
                    }
                    .into(),
                ),
            ];
        }
        self.modal = Some(Modal::Info(fields));
        self.modal_title = if selected {
            tr("properties")
        } else {
            tr("archive-info")
        }
        .into();
        cx.notify();
    }

    pub(in crate::ui) fn save_copy(&mut self, cx: &mut Context<Self>) {
        let Some(c) = self.catalog.clone() else {
            return;
        };
        self.start(tr("saving"), cx, move |_| {
            let Some(destination) = rfd::FileDialog::new()
                .set_file_name(c.path.file_name().unwrap_or_default().to_string_lossy())
                .save_file()
            else {
                return Ok(Outcome::Cancelled);
            };
            if destination == c.path {
                return Ok(Outcome::Cancelled);
            }
            std::fs::copy(&c.path, &destination)?;
            Ok(Outcome::Message(tf(
                "archive-saved",
                &[("path", destination.display().to_string().into())],
            )))
        });
    }

    pub(in crate::ui) fn drop_files(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.settings_busy(cx) {
            return;
        }
        if let Some(Modal::Create(form)) = &self.modal {
            form.update(cx, |form, cx| form.add(paths, window, cx));
            return;
        }
        if self.modal.is_some() {
            return;
        }
        self.show_page(Page::Files, window, cx);
        let archive = paths.len() == 1
            && paths[0].extension().is_some_and(|ext| {
                [
                    "7z", "zip", "rar", "tar", "gz", "bz2", "xz", "wim", "iso", "cab", "001",
                ]
                .contains(&ext.to_string_lossy().to_lowercase().as_str())
            });
        if archive {
            self.execute(Request::Open(paths[0].clone()), String::new(), cx);
        } else {
            self.create_dialog(paths, window, cx);
        }
    }
}
