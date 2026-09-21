use super::*;
use cardo_7zp_platform::ShellAction;

impl Workspace {
    pub fn attach_system(
        &mut self,
        receiver: std::sync::mpsc::Receiver<cardo_7zp_platform::Command>,
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
                                use cardo_7zp_platform::Command;
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
                            if this.tasks.extraction().is_some() {
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
        if self.tasks.is_busy()
            || self.settings_busy(cx)
            || self.preferences_task.is_some()
            || self.dialogs.awaiting_password()
            || self.shell_read_task.is_some()
            || self.context_menu.is_some()
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
                ShellAction::Extract => self.extract_dialog(window, cx),
                ShellAction::OneClickExtract | ShellAction::ExtractFolder => {
                    self.quick_extract(false, cx)
                }
                ShellAction::ExtractHere => {
                    let parent = catalog.path.parent().unwrap().to_path_buf();
                    self.begin_extract(
                        catalog,
                        Vec::new(),
                        parent,
                        String::new(),
                        self.preferences.open_after,
                        false,
                        cx,
                    );
                }
                ShellAction::Check => self.execute(
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
            let read = cx
                .background_executor()
                .spawn(async move { cardo_7zp_platform::read_shell_request(&path) });
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
        let action = match cardo_7zp_platform::parse_action(argument) {
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
                    if let Some(Modal::Create(form)) = self.dialogs.current() {
                        form.update(cx, |form, _| form.email = true);
                    }
                    self.dialogs.set_title(tr("shell-compress-email"));
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
            ShellAction::Checksum(method) => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: cardo_7zp_application::ChecksumKind::Hash(method),
                },
                String::new(),
                cx,
            ),
            ShellAction::GenerateChecksum => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: cardo_7zp_application::ChecksumKind::Generate,
                },
                String::new(),
                cx,
            ),
            ShellAction::VerifyChecksum => self.execute(
                Request::Checksum {
                    paths: paths.iter().map(PathBuf::from).collect(),
                    kind: cardo_7zp_application::ChecksumKind::Verify,
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
                    ShellAction::OpenAs(kind) => Request::OpenAs(PathBuf::from(&paths[0]), kind),
                    _ => Request::Open(PathBuf::from(&paths[0])),
                };
                self.after_open = Some(action);
                self.execute(request, String::new(), cx);
            }
        }
    }
}
