use crate::settings::Preferences;
use crate::ui::preferences::{PreferencesForm, Tab};
use crate::ui::*;

impl Workspace {
    pub(in crate::ui) fn change_language(
        &mut self,
        language: crate::i18n::Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.language_task.is_some() {
            return;
        }
        let handle = window.window_handle();
        let save = cx
            .background_executor()
            .spawn(async move { crate::i18n::save(language) });
        self.language_task = Some(cx.spawn(async move |view, cx| {
            let result = save.await;
            let _ = handle.update(cx, |_, window, cx| {
                view.update(cx, |this, cx| {
                    match result {
                        Ok(()) => {
                            crate::i18n::select(language);
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
                            this.message = Some(tf(
                                "language-save-error",
                                &[("error", error.to_string().into())],
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

    pub(in crate::ui) fn check_update(&mut self, cx: &mut Context<Self>) {
        self.close_modal(cx);
        self.modal_title = tr("update-title").into();
        self.modal = Some(Modal::Update(crate::application::update::Status::Checking));
        let check = cx
            .background_executor()
            .spawn(async { crate::application::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let status = check.await.unwrap_or_else(|error| {
                crate::application::update::Status::Failed(format!("{error:#}"))
            });
            let _ = view.update(cx, |this, cx| {
                if matches!(this.modal, Some(Modal::Update(_))) {
                    this.modal = Some(Modal::Update(status));
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }

    pub(in crate::ui) fn change_theme(
        &mut self,
        id: crate::ui::theme::ThemeId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        crate::ui::theme::apply(id, Some(window), cx);
        let previous = self.theme_save.take();
        self.theme_save = Some(cx.spawn(async move |view, cx| {
            // Preserve the user's selection order when switching themes quickly.
            if let Some(previous) = previous {
                previous.await;
            }
            let result = cx
                .background_executor()
                .spawn(async move { crate::ui::theme::save(id) })
                .await;
            if let Err(error) = result {
                let _ = view.update(cx, |this, cx| {
                    this.message = Some(tf(
                        "theme-save-error",
                        &[("error", error.to_string().into())],
                    ));
                    cx.notify();
                });
            }
        }));
        cx.notify();
    }
}

impl Workspace {
    pub(in crate::ui) fn settings_busy(&self, cx: &App) -> bool {
        self.settings_form
            .as_ref()
            .is_some_and(|form| form.read(cx).is_busy())
    }

    pub(in crate::ui) fn show_page(
        &mut self,
        page: Page,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.settings_busy(cx) || self.page == page {
            return;
        }
        self.page_direction = if page.order() > self.page.order() {
            1.0
        } else {
            -1.0
        };
        self.page_revision += 1;
        self.page = page;
        self.context_menu = None;
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub(in crate::ui) fn preferences_category(
        &mut self,
        tab: Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy
            || self.settings_busy(cx)
            || self.preferences_task.is_some()
            || self.modal.is_some()
        {
            return;
        }
        if let Some(form) = &self.settings_form {
            form.update(cx, |form, cx| {
                form.set_tab(tab, cx);
            });
        } else {
            let owner = cx.entity().downgrade();
            let value = self.preferences.clone();
            let form = cx.new(|cx| PreferencesForm::new(owner, value, tab, window, cx));
            self.settings_subscription = Some(cx.observe(&form, |_, _, cx| cx.notify()));
            self.settings_form = Some(form);
        }
        self.show_page(Page::Settings(tab), window, cx);
    }

    pub(in crate::ui) fn apply_preferences(&mut self, value: Preferences, cx: &mut Context<Self>) {
        crate::settings::LOW_PRIORITY.store(value.low_priority, Ordering::Relaxed);
        if !value.remember_recent {
            self.update_recent(recent::Change::Clear, cx);
            self.update_history(recent::Change::Clear, recent::Kind::Folders, cx);
        }
        self.preferences = value;
        cx.notify();
    }

    pub(in crate::ui) fn load_preferences(&mut self, cx: &mut Context<Self>) {
        let job = cx
            .background_executor()
            .spawn(async { crate::settings::load_preferences() });
        self.preferences_task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                match result {
                    Ok(value) => {
                        let check = value.check_updates;
                        this.apply_preferences(value, cx);
                        if check {
                            this.check_updates_quietly(cx);
                        }
                    }
                    Err(error) => this.message = Some(error.to_string()),
                }
                this.preferences_task = None;
                cx.notify();
            });
        }));
    }

    fn check_updates_quietly(&mut self, cx: &mut Context<Self>) {
        let job = cx
            .background_executor()
            .spawn(async { crate::application::update::check() });
        self.update_task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            if let Ok(crate::application::update::Status::Available { version, .. }) = result {
                let _ = view.update(cx, |this, cx| {
                    this.message = Some(tf("update-available", &[("version", version.into())]));
                    cx.notify();
                });
            }
        }));
    }
}
