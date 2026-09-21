use crate::*;

impl Workspace {
    pub(crate) fn start(
        &mut self,
        label: &str,
        cx: &mut Context<Self>,
        work: impl FnOnce(Cancellation) -> Result<Outcome> + Send + 'static,
    ) {
        if self.tasks.is_busy() {
            return;
        }
        self.close_modal(cx);
        let cancel = self.tasks.begin(label);
        self.message = None;
        self.completion = None;
        let job = cx.background_executor().spawn(async move { work(cancel) });
        self.tasks.attach(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                let mut dispatch = None;
                let extraction = this.tasks.finish();
                if matches!(this.dialogs.current(), Some(Modal::Progress)) {
                    this.dialogs.take();
                }
                match result {
                    Ok(Outcome::Comment(text)) => this.dialogs.request_comment(text),
                    Ok(Outcome::External(directory)) => this.tasks.retain_file(directory),
                    Ok(Outcome::Opened(c, p)) => this.activate(c, p, cx),
                    Ok(Outcome::Directory(directory)) => this.activate_directory(directory, cx),
                    Ok(Outcome::Address(catalog, folder)) => {
                        this.browser.request_folder(folder);
                        this.activate(catalog, String::new(), cx);
                    }
                    Ok(Outcome::AddressPassword(request, folder)) => {
                        this.browser.request_folder(folder);
                        this.dialogs.request_password(request);
                    }
                    Ok(Outcome::Created(c, p)) => {
                        let path = c.path.clone();
                        this.activate(c, p, cx);
                        this.completion = Some((tr("create-complete").into(), path));
                    }
                    Ok(Outcome::Extracted(path, warning, open_error)) => {
                        if this.tasks.close_after()
                            && !warning
                            && open_error.is_none()
                            && this.launches.is_empty()
                        {
                            cx.quit();
                        }
                        this.tasks.set_close_after(false);
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
                        this.dialogs.show(title, Modal::Report(text));
                    }
                    Ok(Outcome::Saved(path, message)) => this.completion = Some((message, path)),
                    Ok(Outcome::Moved(catalog, password, path, open_error)) => {
                        this.activate(catalog, password, cx);
                        this.completion = Some((tr("archive-move-complete").into(), path));
                        this.message = open_error;
                    }
                    Ok(Outcome::Password(request)) => this.dialogs.request_password(request),
                    Ok(Outcome::Dispatch(request)) => dispatch = Some(request),
                    Ok(Outcome::ConfirmRun {
                        directory,
                        target,
                        entry,
                    }) => {
                        this.pending_run = Some((directory, target));
                        this.dialogs
                            .show(tr("browser-open"), Modal::ConfirmRun { entry });
                    }
                    Ok(Outcome::Launched) => {}
                    Ok(Outcome::Cancelled) => {
                        this.tasks.set_close_after(false);
                        this.browser.cancel_navigation();
                    }
                    Err(error) => {
                        this.tasks.set_close_after(false);
                        this.browser.cancel_navigation();
                        this.after_open = None;
                        if this.tasks.cancelled() {
                            let title = tr(if extraction.is_some() {
                                "extract-stopped"
                            } else {
                                "task-cancelled"
                            });
                            this.dialogs.show(
                                title,
                                Modal::ExtractionResult(
                                    tr(if extraction.is_some() {
                                        "extract-stopped-detail"
                                    } else {
                                        "task-cancelled"
                                    })
                                    .into(),
                                ),
                            );
                        } else {
                            this.show_task_error(error, extraction.is_some(), cx);
                        }
                    }
                }
                this.tasks.release();
                if let Some(request) = dispatch {
                    this.execute(request, String::new(), cx);
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(crate) fn activate(&mut self, catalog: Catalog, password: String, cx: &mut Context<Self>) {
        let folder = self.browser.take_requested_folder();
        if !folder.is_empty() {
            let prefix = format!("{folder}/");
            if !catalog
                .entries
                .iter()
                .any(|e| e.directory && e.path == folder || e.path.starts_with(&prefix))
            {
                self.browser.cancel_navigation();
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
        self.browser.show_archive(catalog, password, folder);
        self.clear_search = true;
        self.refresh(cx);
    }

    pub(crate) fn execute(&mut self, request: Request, password: String, cx: &mut Context<Self>) {
        if self.tasks.is_busy() {
            return;
        }
        let extraction = request.extraction_paths();
        let progress = extraction
            .as_ref()
            .map(|_| sevenzip_archive::Progress::new());
        let reporting = progress.clone();
        let close_after = self.tasks.close_after() && matches!(request, Request::Extract { .. });
        self.start(request.label(), cx, move |cancel| {
            request.run(password, &cancel, reporting)
        });
        self.tasks.set_close_after(close_after);
        if let Some(((source, destination), progress)) = extraction.zip(progress) {
            self.show_extraction_progress(source, destination, progress, cx);
        }
    }
}
