use super::*;
use p7z_platform::ExplorerAction;

impl Workspace {
    pub fn attach_system(
        &mut self,
        receiver: std::sync::mpsc::Receiver<p7z_platform::Command>,
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
                                use p7z_platform::Command;
                                match command {
                                    Command::Launch(args) => {
                                        window.activate_window();
                                        if !args.is_empty() {
                                            this.launches.push_back(args);
                                        }
                                    }
                                    Command::Error(error) => {
                                        tracing::error!(error = %error, "Platform command failed");
                                        this.notify_message(error, cx);
                                    }
                                }
                                cx.notify();
                            }
                            this.dispatch(window, cx);
                            if this.tasks.extraction().is_some() {
                                cx.notify();
                            }
                            if this.update_transfer.is_some() {
                                cx.notify();
                            }
                            if this.dragging && !cx.has_active_drag() {
                                this.dragging = false;
                                cx.notify();
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
        if self.tasks.is_busy()
            || self.settings_busy(cx)
            || self.dialogs.awaiting_password()
            || self.shell_read_task.is_some()
        {
            return;
        }
        if !self.dialogs.is_open()
            && let Some(action) = self.after_open.take()
        {
            let Some(catalog) = self.browser.view().catalog.clone() else {
                return;
            };
            match action {
                ExplorerAction::Extract => self.extract_dialog(window, cx),
                ExplorerAction::OneClickExtract | ExplorerAction::ExtractFolder => {
                    self.quick_extract(false, cx)
                }
                ExplorerAction::ExtractHere => {
                    let parent = catalog.path.parent().unwrap().to_path_buf();
                    self.begin_extract(
                        catalog,
                        Vec::new(),
                        parent,
                        String::new(),
                        self.preferences.open_after,
                        false,
                        self.preferences.close_archive_after_quick,
                        cx,
                    );
                }
                ExplorerAction::Check => self.execute(
                    Request::Check(catalog),
                    self.browser.view().password.clone(),
                    cx,
                ),
                _ => {}
            }
            return;
        }
        if self.dialogs.is_open() {
            return;
        }
        let Some(args) = self.launches.pop_front() else {
            return;
        };
        let (argument, paths) = if args[0].starts_with("--") {
            (args[0].as_str(), &args[1..])
        } else {
            ("--open", &args[..])
        };
        if argument == "--shell-request" {
            let Some(path) = paths.first().map(PathBuf::from) else {
                self.notify_message(tr("shell-path-required").into(), cx);
                cx.notify();
                return;
            };
            let read = cx
                .background_executor()
                .spawn(async move { p7z_platform::read_shell_request(&path) });
            self.shell_read_task = Some(cx.spawn(async move |view, cx| {
                let result = read.await;
                let _ = view.update(cx, |this, cx| {
                    match result {
                        Ok(args) => this.launches.push_front(args),
                        Err(error) => this.notify_message(format!("{error:#}"), cx),
                    }
                    this.shell_read_task = None;
                    cx.notify();
                });
            }));
            return;
        }
        let action = match p7z_platform::parse_action(argument) {
            Ok(action) => action,
            Err(error) => {
                self.notify_message(error.to_string(), cx);
                cx.notify();
                return;
            }
        };
        if paths.is_empty() {
            self.notify_message(tr("shell-path-required").into(), cx);
            cx.notify();
            return;
        }
        match action {
            ExplorerAction::Compress | ExplorerAction::CompressEmail => {
                self.create_dialog(paths.iter().map(PathBuf::from).collect(), window, cx);
                if action == ExplorerAction::CompressEmail {
                    self.dialogs.pending_email = true;
                    self.dialogs.set_title(tr("shell-compress-email"));
                }
            }
            ExplorerAction::QuickCompress(format, email) => {
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
            ExplorerAction::Checksum(method) => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: p7z_requests::ChecksumKind::Hash(method),
                },
                String::new(),
                cx,
            ),
            ExplorerAction::GenerateChecksum => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: p7z_requests::ChecksumKind::Generate,
                },
                String::new(),
                cx,
            ),
            ExplorerAction::VerifyChecksum => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: p7z_requests::ChecksumKind::Verify,
                },
                String::new(),
                cx,
            ),
            _ => {
                // Each archive is opened before applying its action; preserve input order.
                for path in paths[1..].iter().rev() {
                    self.launches
                        .push_front(vec![action.argument().into(), path.clone()]);
                }
                let request = match action {
                    ExplorerAction::OpenAs(kind) => Request::OpenAs(PathBuf::from(&paths[0]), kind),
                    _ => Request::Open(PathBuf::from(&paths[0])),
                };
                self.after_open = Some(action);
                self.execute(request, String::new(), cx);
            }
        }
    }
}
