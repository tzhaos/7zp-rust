use crate::ui::*;

impl Workspace {
    pub(in crate::ui) fn start(
        &mut self,
        label: &str,
        cx: &mut Context<Self>,
        work: impl FnOnce(Cancellation) -> Result<Outcome> + Send + 'static,
    ) {
        if self.busy {
            return;
        }
        self.close_modal(cx);
        self.busy = true;
        self.status = label.into();
        self.message = None;
        self.completion = None;
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let job = cx.background_executor().spawn(async move { work(cancel) });
        self.task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                this.busy = false;
                let extraction = this.extraction.take();
                if matches!(this.modal, Some(Modal::Progress)) {
                    this.modal = None;
                }
                match result {
                    Ok(Outcome::Comment(text)) => this.pending_comment = Some(text),
                    Ok(Outcome::External(directory)) => this.temporary_files.push(directory),
                    Ok(Outcome::Opened(c, p)) => this.activate(c, p, cx),
                    Ok(Outcome::Directory(directory)) => this.activate_directory(directory, cx),
                    Ok(Outcome::Address(catalog, folder)) => {
                        this.pending_folder = Some(folder);
                        this.activate(catalog, String::new(), cx);
                    }
                    Ok(Outcome::AddressPassword(request, folder)) => {
                        this.pending_folder = Some(folder);
                        this.pending_password = Some(request);
                    }
                    Ok(Outcome::Created(c, p)) => {
                        let path = c.path.clone();
                        this.activate(c, p, cx);
                        this.completion = Some((tr("create-complete").into(), path));
                    }
                    Ok(Outcome::Extracted(path, warning, open_error)) => {
                        if this.close_after_task
                            && !warning
                            && open_error.is_none()
                            && this.launches.is_empty()
                        {
                            cx.quit();
                        }
                        this.close_after_task = false;
                        this.completion = Some((
                            if warning {
                                tr("extract-warning").into()
                            } else {
                                tr("extract-finished").into()
                            },
                            path,
                        ));
                        this.message = open_error;
                    }
                    Ok(Outcome::Message(message)) => this.message = Some(message),
                    Ok(Outcome::Report(title, text)) => {
                        this.modal_title = title;
                        this.modal = Some(Modal::Report(text));
                    }
                    Ok(Outcome::Saved(path, message)) => this.completion = Some((message, path)),
                    Ok(Outcome::Moved(catalog, password, path, open_error)) => {
                        this.activate(catalog, password, cx);
                        this.completion = Some((tr("archive-move-complete").into(), path));
                        this.message = open_error;
                    }
                    Ok(Outcome::Password(request)) => this.pending_password = Some(request),
                    Ok(Outcome::Cancelled) => {
                        this.close_after_task = false;
                        this.returning = false;
                        this.pending_folder = None;
                    }
                    Err(error) => {
                        this.close_after_task = false;
                        this.returning = false;
                        this.pending_folder = None;
                        this.after_open = None;
                        if this.cancel.load(Ordering::Relaxed) {
                            this.modal_title = tr(if extraction.is_some() {
                                "extract-stopped"
                            } else {
                                "task-cancelled"
                            })
                            .into();
                            this.modal = Some(Modal::ExtractionResult(
                                tr(if extraction.is_some() {
                                    "extract-stopped-detail"
                                } else {
                                    "task-cancelled"
                                })
                                .into(),
                            ));
                        } else {
                            this.show_task_error(error, extraction.is_some(), cx);
                        }
                    }
                }
                this.task = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(in crate::ui) fn activate(
        &mut self,
        catalog: Catalog,
        password: String,
        cx: &mut Context<Self>,
    ) {
        let folder = self.pending_folder.take().unwrap_or_default();
        if !folder.is_empty() {
            let prefix = format!("{folder}/");
            if !catalog
                .entries
                .iter()
                .any(|e| e.directory && e.path == folder || e.path.starts_with(&prefix))
            {
                self.returning = false;
                self.message = Some(tr("browser-path-missing").into());
                return;
            }
        }
        self.remember_navigation(&browser::Location::Archive(
            catalog.path.clone(),
            folder.clone(),
        ));
        if let Some(parent) = catalog.path.parent() {
            if let Some(index) = self
                .destinations
                .iter()
                .position(|(kind, _)| *kind == DestinationKind::Archive)
            {
                self.destinations[index].1 = parent.to_owned();
                self.destination = index;
            } else {
                self.destination = self.destinations.len();
                self.destinations
                    .push((DestinationKind::Archive, parent.to_owned()));
            }
        }
        self.update_recent(recent::Change::Remember(catalog.path.clone()), cx);
        self.message = catalog.warning.then(|| tr("open-warning").into());
        self.catalog = Some(catalog);
        self.directory = None;
        self.password = password;
        self.folder = folder;
        self.selected.clear();
        self.anchor = None;
        self.cursor = None;
        self.clear_search = true;
        self.refresh(cx);
    }

    pub(in crate::ui) fn execute(
        &mut self,
        request: Request,
        password: String,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        let extraction = request.extraction_paths();
        let progress = extraction.as_ref().map(|_| crate::archive::Progress::new());
        let reporting = progress.clone();
        let close_after = self.close_after_task && matches!(request, Request::Extract { .. });
        self.start(request.label(), cx, move |cancel| {
            request.run(password, &cancel, reporting)
        });
        self.close_after_task = close_after;
        if let Some(((source, destination), progress)) = extraction.zip(progress) {
            self.show_extraction_progress(source, destination, progress, cx);
        }
    }
}
