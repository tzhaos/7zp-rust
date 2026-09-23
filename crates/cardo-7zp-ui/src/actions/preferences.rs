use crate::preferences::Tab;
use crate::*;
use cardo_7zp_core::settings::Preferences;

impl Workspace {
    pub(crate) fn open_default_apps(&self, cx: &mut App) {
        cx.open_url(cardo_7zp_platform::DEFAULT_APPS_URI);
    }

    pub(crate) fn open_update_link(&self, url: &str, cx: &mut App) {
        cx.open_url(url);
    }

    pub(crate) fn change_language(
        &mut self,
        language: cardo_7zp_core::i18n::Language,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tasks.is_busy() || self.language_task.is_some() {
            return;
        }
        let handle = self.main_window;
        let save = cx
            .background_executor()
            .spawn(async move { cardo_7zp_core::i18n::save(language) });
        self.language_task = Some(cx.spawn(async move |view, cx| {
            let result = save.await;
            let _ = handle.update(cx, |_, window, cx| {
                view.update(cx, |this, cx| {
                    match result {
                        Ok(()) => {
                            cardo_7zp_core::i18n::select(language);
                            this.search.update(cx, |input, cx| {
                                input.set_placeholder(tr("search-placeholder"), window, cx)
                            });
                            this.address.update(cx, |input, cx| {
                                input.set_placeholder(tr("browser-address"), window, cx)
                            });
                            this.message = None;
                            this.completion = None;
                            cx.refresh_windows();
                        }
                        Err(error) => {
                            tracing::error!(error = %format!("{error:#}"), "Cannot save language");
                            this.notify_message(tf(
                                "language-save-error",
                                &[("error", format!("{error:#}").into())],
                            ))
                        }
                    }
                    this.language_task = None;
                    cx.notify();
                })
            });
        }));
        cx.notify();
    }

    pub(crate) fn check_update(&mut self, cx: &mut Context<Self>) {
        use cardo_7zp_requests::update::Status;
        if self.settings_busy(cx) || matches!(self.update_status, Some(Status::Checking)) {
            return;
        }
        self.update_status = Some(Status::Checking);
        let check = cx
            .background_executor()
            .spawn(async { cardo_7zp_requests::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let status = check.await.unwrap_or_else(|error| {
                cardo_7zp_requests::update::Status::Failed(format!("{error:#}"))
            });
            let _ = view.update(cx, |this, cx| {
                this.update_status = Some(status);
                this.update_task = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(crate) fn change_theme(
        &mut self,
        id: crate::theme::ThemeId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = crate::theme::apply(id, &self.appearance, Some(window), cx) {
            tracing::error!(error = %error, "Cannot apply theme");
            self.notify_message(error.to_string());
            cx.notify();
            return;
        }
        let previous = self.theme_save.take();
        self.theme_save = Some(cx.spawn(async move |view, cx| {
            // Preserve the user's selection order when switching themes quickly.
            if let Some(previous) = previous {
                previous.await;
            }
            let result = cx
                .background_executor()
                .spawn(async move { crate::theme::save(id) })
                .await;
            if let Err(error) = result {
                tracing::error!(error = %format!("{error:#}"), "Cannot save theme");
                let _ = view.update(cx, |this, cx| {
                    this.notify_message(tf(
                        "theme-save-error",
                        &[("error", format!("{error:#}").into())],
                    ));
                    cx.notify();
                });
            }
        }));
        cx.notify();
    }
}

impl Workspace {
    pub(crate) fn settings_busy(&self, cx: &App) -> bool {
        self.settings_form
            .as_ref()
            .is_some_and(|form| form.read(cx).is_busy())
    }

    pub(crate) fn settings_saving(&self, cx: &App) -> bool {
        self.settings_form
            .as_ref()
            .is_some_and(|form| form.read(cx).is_saving())
    }

    pub(crate) fn settings_controls_disabled(&self, cx: &App) -> bool {
        self.settings_form
            .as_ref()
            .is_some_and(|form| form.read(cx).controls_disabled())
    }

    pub(crate) fn preferences_category(
        &mut self,
        tab: Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_settings(crate::views::SettingsPage::Preferences(tab), window, cx);
    }

    pub(crate) fn apply_preferences(&mut self, value: Preferences, cx: &mut Context<Self>) {
        cardo_7zp_core::settings::LOW_PRIORITY.store(value.low_priority, Ordering::Relaxed);
        if !value.remember_recent {
            self.update_recent(recent::Change::Clear, cx);
        }
        self.preferences = value;
        cx.notify();
    }

    pub(crate) fn check_updates_quietly(&mut self, cx: &mut Context<Self>) {
        let job = cx
            .background_executor()
            .spawn(async { cardo_7zp_requests::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            if let Err(error) = &result {
                tracing::warn!(error = %format!("{error:#}"), "Automatic release check failed");
            }
            if let Ok(cardo_7zp_requests::update::Status::Available { version, .. }) = result {
                let _ = view.update(cx, |this, cx| {
                    this.notify_message(tf("update-available", &[("version", version.into())]));
                    cx.notify();
                });
            }
        }));
    }
}
