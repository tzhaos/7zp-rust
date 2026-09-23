use crate::*;

impl Workspace {
    pub(crate) fn compress_from_context(
        &mut self,
        paths: Vec<PathBuf>,
        directory: PathBuf,
        name: String,
        format: Option<p7z_commands::ArchiveFormat>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_available(commands::Command::Create, cx)
            || self
                .browser
                .view()
                .directory
                .as_ref()
                .is_none_or(|current| current.path != directory)
        {
            return;
        }
        match format {
            None => {
                self.create_dialog(paths, window, cx);
                self.dialogs.pending_name = Some(name);
            }
            Some(format) => self.execute(
                Request::QuickCompress {
                    paths,
                    format,
                    email: false,
                },
                String::new(),
                cx,
            ),
        }
    }

    pub(crate) fn reveal_completion(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match p7z_platform::open_directory(&path) {
            Ok(true) => {}
            Ok(false) => cx.reveal_path(&path),
            Err(error) => {
                if let Some(Modal::Completion { notice, .. }) = self.dialogs.current_mut() {
                    *notice = Some(error.to_string());
                } else {
                    self.notify_message(error.to_string());
                }
                cx.notify();
            }
        }
    }

    pub(crate) fn open_archive_path(
        &mut self,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_available(commands::Command::Open, cx) {
            return;
        }
        self.settings_page = None;
        self.execute(Request::Open(path), String::new(), cx);
    }

    pub(crate) fn open(&mut self, cx: &mut Context<Self>) {
        self.start(tr("opening"), cx, |_| {
            let Some(path) = rfd::FileDialog::new()
                .set_title(tr("archive-open"))
                .pick_file()
            else {
                return Ok(Outcome::Cancelled);
            };
            Ok(Outcome::Dispatch(Request::Open(path)))
        });
    }

    pub(crate) fn create_dialog(
        &mut self,
        files: Vec<PathBuf>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dialogs.pending_name = None;
        self.dialogs.pending_email = false;
        if self.tasks.is_busy() {
            return;
        }
        self.close_modal(cx);
        self.dialogs.pending_create = Some(files);
        self.dialogs.set_title(tr("create-title"));
        self.schedule_prompt(cx);
        cx.notify();
    }

    pub(crate) fn create(
        &mut self,
        files: Vec<PathBuf>,
        name: String,
        options: CreateOptions,
        email: bool,
        cx: &mut Context<Self>,
    ) {
        self.start(tr("creating"), cx, move |_| {
            if name.contains(['/', '\\', ':']) {
                bail!(tr("archive-name-invalid"));
            }
            let filename = options.archive_file_name(&name);
            let Some(destination) = rfd::FileDialog::new()
                .set_title(tr("archive-save"))
                .set_file_name(filename)
                .save_file()
            else {
                return Ok(Outcome::Cancelled);
            };
            Ok(Outcome::Dispatch(Request::Create {
                files,
                destination,
                options,
                email,
            }))
        });
    }

    pub(crate) fn extract_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(c) = &self.browser.view().catalog else {
            return;
        };
        let name = p7z_commands::extract_folder(&c.path);
        let folder = cx.new(|cx| InputState::new(window, cx));
        folder.update(cx, |input, cx| input.set_value(name, window, cx));
        self.dialogs.watch_input(&folder, cx);
        self.show_dialog(
            tr("extract-options"),
            Modal::Extract {
                folder,
                selected: !self.browser.view().selected.is_empty(),
                destination: self.destination,
                open_after: self.preferences.open_after,
            },
            cx,
        );
    }

    pub(crate) fn choose_extract_directory(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
                        if let Some(Modal::Extract { destination, .. }) = this.dialogs.current_mut()
                        {
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

    pub(crate) fn quick_extract(&mut self, selected: bool, cx: &mut Context<Self>) {
        if self.tasks.is_busy() {
            return;
        }
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let Some(parent) = catalog.path.parent().map(Path::to_path_buf) else {
            return;
        };
        let folder = p7z_commands::extract_folder(&catalog.path);
        let chosen = if selected {
            self.browser.view().selected.iter().cloned().collect()
        } else {
            Vec::new()
        };
        self.begin_extract(
            catalog,
            chosen,
            parent,
            folder,
            self.preferences.open_after,
            self.preferences.close_after_quick,
            self.preferences.close_archive_after_quick,
            cx,
        );
    }

    pub(crate) fn begin_extract(
        &mut self,
        catalog: Catalog,
        selected: Vec<String>,
        parent: PathBuf,
        folder: String,
        open_after: bool,
        close_after: bool,
        close_archive: bool,
        cx: &mut Context<Self>,
    ) {
        self.tasks.set_close_after(close_after);
        self.tasks.set_close_archive(close_archive);
        self.tasks.reset_extraction_result();
        self.execute(
            Request::PrepareExtraction(p7z_requests::extraction::Extraction {
                catalog,
                selected,
                parent,
                folder,
                open_after,
            }),
            self.browser.view().password.clone(),
            cx,
        );
    }

    pub(crate) fn extract(
        &mut self,
        selected: bool,
        folder: String,
        destination: usize,
        open_after: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(catalog) = self.browser.view().catalog.clone() else {
            return;
        };
        let Some((_, parent)) = self.destinations.get(destination) else {
            self.notify_message(tr("destinations-unavailable").into());
            cx.notify();
            return;
        };
        let parent = parent.clone();
        self.destination = destination;
        let saved_parent = parent.clone();
        let save = cx
            .background_executor()
            .spawn(async move { p7z_core::settings::save_destination(&saved_parent) });
        cx.spawn(async move |view, cx| {
            if let Err(error) = save.await {
                let _ = view.update(cx, |this, cx| {
                    this.notify_message(tf(
                        "destination-save-error",
                        &[("error", error.to_string().into())],
                    ));
                    cx.notify();
                });
            }
        })
        .detach();
        let selected = if selected {
            self.browser.view().selected.iter().cloned().collect()
        } else {
            Vec::new()
        };
        self.begin_extract(
            catalog, selected, parent, folder, open_after, false, false, cx,
        );
    }

    pub(crate) fn properties(&mut self, selected: bool, cx: &mut Context<Self>) {
        let browser = self.browser.view();
        let Some(c) = &browser.catalog else { return };
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
                    browser
                        .selected
                        .iter()
                        .any(|p| e.path == *p || e.path.starts_with(&format!("{p}/")))
                })
                .collect();
            let single = (browser.selected.len() == 1)
                .then(|| {
                    browser
                        .rows
                        .iter()
                        .find(|e| browser.selected.contains(&e.path))
                })
                .flatten();
            fields = vec![
                (
                    tr("name").into(),
                    single.map(|e| e.name.clone()).unwrap_or_else(|| {
                        tf("item-count", &[("count", browser.selected.len().into())])
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
                    browser
                        .selected
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n"),
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
        let title = if selected {
            tr("properties")
        } else {
            tr("archive-info")
        };
        self.show_dialog(title, Modal::Info(fields), cx);
    }

    pub(crate) fn save_copy(&mut self, cx: &mut Context<Self>) {
        let Some(c) = self.browser.view().catalog.clone() else {
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
            p7z_requests::filesystem::copy_file(&c.path, &destination)?;
            Ok(Outcome::Message(tf(
                "archive-saved",
                &[("path", destination.display().to_string().into())],
            )))
        });
    }

    pub(crate) fn drop_files(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tasks.is_busy() || self.settings_busy(cx) {
            return;
        }
        if let Some(Modal::Create(form)) = self.dialogs.current() {
            form.update(cx, |form, cx| form.add(paths, window, cx));
            return;
        }
        if self.dialogs.is_open() {
            return;
        }
        let archive = paths.len() == 1 && p7z_commands::opens_directly(&paths[0]);
        if archive {
            self.execute(Request::Open(paths[0].clone()), String::new(), cx);
        } else {
            self.create_dialog(paths, window, cx);
        }
    }
}
