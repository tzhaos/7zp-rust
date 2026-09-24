use crate::preferences::Tab;
use crate::*;
use p7z_core::settings::Preferences;

impl Workspace {
    pub(crate) fn open_default_apps(&self, cx: &mut App) {
        cx.open_url(p7z_platform::DEFAULT_APPS_URI);
    }

    pub(crate) fn open_update_link(&self, url: &str, cx: &mut App) {
        cx.open_url(url);
    }

    pub(crate) fn change_language(
        &mut self,
        language: p7z_core::i18n::Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The complete settings snapshot was persisted by PreferencesForm.
        p7z_core::i18n::select(language);
        self.search.update(cx, |input, cx| {
            input.set_placeholder(tr("search-placeholder"), window, cx)
        });
        self.address.update(cx, |input, cx| {
            input.set_placeholder(tr("browser-address"), window, cx)
        });
        self.toast.update(cx, |toast, cx| toast.clear(cx));
        self.completion = None;
        cx.refresh_windows();
        cx.notify();
    }

    pub(crate) fn check_update(&mut self, cx: &mut Context<Self>) {
        use p7z_requests::update::Status;
        if self.settings_busy(cx) || matches!(self.update_status, Some(Status::Checking)) {
            return;
        }
        self.update_status = Some(Status::Checking);
        let check = cx
            .background_executor()
            .spawn(async { p7z_requests::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let status = check
                .await
                .unwrap_or_else(|error| p7z_requests::update::Status::Failed(format!("{error:#}")));
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
            self.notify_message(error.to_string(), cx);
            cx.notify();
            return;
        }
        cx.notify();
        cx.notify();
    }
}

impl Workspace {
    pub(crate) fn settings_busy(&self, cx: &App) -> bool {
        self.update_transfer.is_some()
            || self
                .settings_form
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
        p7z_core::settings::LOW_PRIORITY.store(value.low_priority, Ordering::Relaxed);
        if !value.remember_recent {
            self.update_recent(recent::Change::Clear, cx);
        }
        self.preferences = value;
        cx.notify();
    }

    pub(crate) fn check_updates_quietly(&mut self, cx: &mut Context<Self>) {
        self.update_status = Some(p7z_requests::update::Status::Checking);
        let job = cx
            .background_executor()
            .spawn(async { p7z_requests::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            if let Err(error) = &result {
                tracing::warn!(error = %format!("{error:#}"), "Automatic release check failed");
            }
            let status = result
                .unwrap_or_else(|error| p7z_requests::update::Status::Failed(format!("{error:#}")));
            let _ = view.update(cx, |this, cx| {
                if let p7z_requests::update::Status::Available { version, .. } = &status {
                    this.notify_message(tf(
                        "update-available",
                        &[("version", version.as_str().into())],
                    ), cx);
                }
                this.update_status = Some(status);
                this.update_task = None;
                cx.notify();
            });
        }));
    }
}
