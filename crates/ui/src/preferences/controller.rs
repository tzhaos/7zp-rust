use super::*;

impl PreferencesForm {
    pub(crate) fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_busy() {
            return;
        }
        self.value.temp_directory = self.temporary.read(cx).value().trim().to_owned();
        self.value.extract_all = self.patterns.read(cx).value().trim().to_owned();
        let value = self.value.clone();
        let owner = self.owner.clone();
        let handle = window.window_handle();
        let theme = self.theme;
        let language = self.language;
        let registration_changed = owner
            .read_with(cx, |workspace, _| {
                workspace.preferences.associations != value.associations
                    || workspace.preferences.shell_menu != value.shell_menu
            })
            .unwrap_or(false);
        let job = cx.background_executor().spawn(async move {
            sevenzip_core::settings::prepare_temp_directory(&value.temp_directory)?;
            if registration_changed {
                sevenzip_platform::configure(&std::env::current_exe()?, &value)?;
            }
            sevenzip_core::settings::save_preferences(&value)?;
            Ok::<_, anyhow::Error>(value)
        });
        self.task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                this.task = None;
                match result {
                    Ok(value) => {
                        App::defer(cx, move |cx| {
                            let _ = handle.update(cx, |_, window, cx| {
                                let _ = owner.update(cx, |owner, cx| {
                                    owner.apply_preferences(value, cx);
                                    if theme != crate::theme::current(cx) {
                                        owner.change_theme(theme, window, cx);
                                    }
                                    if language != sevenzip_core::i18n::current() {
                                        owner.change_language(language, window, cx);
                                    }
                                });
                            });
                        });
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(super) fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = window.window_handle();
        let job = cx
            .background_executor()
            .spawn(async { rfd::FileDialog::new().pick_folder() });
        self.task = Some(cx.spawn(async move |view, cx| {
            let path = job.await;
            let _ = handle.update(cx, |_, window, cx| {
                let _ = view.update(cx, |this, cx| {
                    this.task = None;
                    if let Some(path) = path {
                        this.temporary.update(cx, |input, cx| {
                            input.set_value(path.display().to_string(), window, cx)
                        });
                    }
                    cx.notify();
                });
            });
        }));
        cx.notify();
    }
}
