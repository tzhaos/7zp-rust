use super::*;
use crate::platform::ShellAction;

impl Workspace {
    pub fn attach_system(
        &mut self,
        receiver: std::sync::mpsc::Receiver<crate::platform::Command>,
        args: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !args.is_empty() {
            self.launches.push_back(args);
        }
        let handle = window.window_handle();
        self.system_task = Some(cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(80))
                    .await;
                let events: Vec<_> = receiver.try_iter().collect();
                if handle
                    .update(cx, |_, window, cx| {
                        let _ = view.update(cx, |this, cx| {
                            for command in events {
                                use crate::platform::Command;
                                match command {
                                    Command::Launch(args) => {
                                        window.activate_window();
                                        if !args.is_empty() {
                                            this.launches.push_back(args);
                                        }
                                    }
                                    Command::Error(error) => this.message = Some(error),
                                }
                                cx.notify();
                            }
                            this.dispatch(window, cx);
                            if this.extraction.is_some() {
                                cx.notify();
                            }
                            if this.dragging && !cx.has_active_drag() {
                                this.dragging = false;
                                cx.notify();
                            }
                            if let Some(message) = &this.message {
                                if this
                                    .message_since
                                    .as_ref()
                                    .is_none_or(|(previous, _)| previous != message)
                                {
                                    this.message_since =
                                        Some((message.clone(), std::time::Instant::now()));
                                } else if this.message_since.as_ref().is_some_and(|(_, since)| {
                                    since.elapsed() >= std::time::Duration::from_secs(6)
                                }) {
                                    this.message = None;
                                    this.message_since = None;
                                    cx.notify();
                                }
                            } else {
                                this.message_since = None;
                            }
                        });
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }

    fn dispatch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy
            || self.settings_busy(cx)
            || self.preferences_task.is_some()
            || self.pending_password.is_some()
            || self.shell_read_task.is_some()
            || self.context_menu.is_some()
        {
            return;
        }
        if self.modal.is_none()
            && let Some(action) = self.after_open.take()
        {
            let Some(catalog) = self.catalog.clone() else {
                return;
            };
            match action {
                ShellAction::Extract => self.extract_dialog(window, cx),
                ShellAction::OneClickExtract | ShellAction::ExtractFolder => {
                    self.quick_extract(false, cx)
                }
                ShellAction::ExtractHere => {
                    let parent = catalog.path.parent().unwrap().to_path_buf();
                    self.execute(
                        Request::Extract {
                            catalog,
                            selected: Vec::new(),
                            parent,
                            folder: String::new(),
                            overwrite: Overwrite::RenameIncoming,
                            open_after: self.preferences.open_after,
                        },
                        self.password.clone(),
                        cx,
                    );
                }
                ShellAction::Check => {
                    self.execute(Request::Check(catalog), self.password.clone(), cx)
                }
                _ => {}
            }
            return;
        }
        if self.modal.is_some() {
            return;
        }
        let Some(args) = self.launches.pop_front() else {
            return;
        };
        self.show_page(Page::Files, window, cx);
        let (argument, paths) = if args[0].starts_with("--") {
            (args[0].as_str(), &args[1..])
        } else {
            ("--open", &args[..])
        };
        if argument == "--shell-request" {
            let Some(path) = paths.first().map(PathBuf::from) else {
                self.message = Some(tr("shell-path-required").into());
                cx.notify();
                return;
            };
            let read = cx.background_executor().spawn(async move {
                if path.parent() != Some(std::env::temp_dir().as_path())
                    || !path.file_name().is_some_and(|name| {
                        name.to_string_lossy()
                            .starts_with(sevenzip_shell_api::REQUEST_PREFIX)
                    })
                {
                    bail!(tr("shell-request-invalid"));
                }
                let file = tempfile::TempPath::try_from_path(path)?;
                let request: sevenzip_shell_api::Request =
                    serde_json::from_slice(&std::fs::read(&file)?)?;
                let mut args = vec![request.action.argument().into()];
                args.extend(
                    request
                        .paths
                        .into_iter()
                        .map(|path| path.to_string_lossy().into_owned()),
                );
                anyhow::Ok(args)
            });
            self.shell_read_task = Some(cx.spawn(async move |view, cx| {
                let result = read.await;
                let _ = view.update(cx, |this, cx| {
                    match result {
                        Ok(args) => this.launches.push_front(args),
                        Err(error) => this.message = Some(format!("{error:#}")),
                    }
                    this.shell_read_task = None;
                    cx.notify();
                });
            }));
            return;
        }
        let action = match crate::platform::parse_action(argument) {
            Ok(action) => action,
            Err(error) => {
                self.message = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        if paths.is_empty() {
            self.message = Some(tr("shell-path-required").into());
            cx.notify();
            return;
        }
        match action {
            ShellAction::Compress | ShellAction::CompressEmail => {
                self.create_dialog(paths.iter().map(PathBuf::from).collect(), window, cx);
                if action == ShellAction::CompressEmail {
                    if let Some(Modal::Create(form)) = &self.modal {
                        form.update(cx, |form, _| form.email = true);
                    }
                    self.modal_title = tr("shell-compress-email").into();
                }
            }
            ShellAction::QuickCompress(format, email) => {
                self.execute(
                    Request::QuickCompress {
                        paths: paths.iter().map(PathBuf::from).collect(),
                        format,
                        email,
                    },
                    String::new(),
                    cx,
                );
            }
            ShellAction::Checksum(_)
            | ShellAction::GenerateChecksum
            | ShellAction::VerifyChecksum => {
                let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
                self.start(tr("shell-checksums"), cx, move |cancel| {
                    let engine = Engine::bundled()?;
                    let output = match action {
                        ShellAction::Checksum(method) => {
                            engine.checksum(&paths, method, &cancel)?
                        }
                        ShellAction::VerifyChecksum => engine.verify_checksums(&paths, &cancel)?,
                        ShellAction::GenerateChecksum => {
                            return Ok(Outcome::Saved(
                                engine.generate_checksum(&paths, &cancel)?,
                                tr("checksum-saved").into(),
                            ));
                        }
                        _ => unreachable!(),
                    };
                    Ok(Outcome::Report(
                        tr(if output.warning {
                            "checksum-warning"
                        } else {
                            "shell-checksums"
                        })
                        .into(),
                        output.text,
                    ))
                });
            }
            _ => {
                // Each archive is opened before applying its action; preserve input order.
                for path in paths[1..].iter().rev() {
                    self.launches
                        .push_front(vec![action.argument().into(), path.clone()]);
                }
                let request = match action {
                    ShellAction::OpenAs(kind) => Request::OpenAs(PathBuf::from(&paths[0]), kind),
                    _ => Request::Open(PathBuf::from(&paths[0])),
                };
                self.after_open = Some(action);
                self.execute(request, String::new(), cx);
            }
        }
    }
}
