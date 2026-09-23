use super::*;

impl PreferencesForm {
    pub(super) fn step_font_size(
        &mut self,
        delta: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_busy() {
            return;
        }
        let next = p7z_core::settings::Appearance::clamp_font_size(self.font_size + delta);
        if next == self.font_size {
            return;
        }
        self.font_size = next;
        self.save(window, cx);
    }

    pub(crate) fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_busy() {
            return;
        }
        self.value.temp_directory = self.temporary.read(cx).value().trim().to_owned();
        self.value.extract_all = self
            .patterns
            .read(cx)
            .value()
            .split([';', '\n', '\r'])
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
            .join(";");
        let font = match crate::theme::resolve_font(&self.font_family.read(cx).value(), cx) {
            Ok(font) => font,
            Err(error) => {
                self.error = (self.tab != Tab::Application).then(|| error.to_string());
                self.font_error = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        self.font_error = None;
        let font_family = font.setting();
        if self.font_family.read(cx).value() != font_family {
            let normalized = font_family.clone();
            self.font_family.update(cx, |input, cx| {
                input.set_value(normalized, window, cx);
            });
        }
        self.error = None;
        let appearance = p7z_core::settings::Appearance {
            font_family,
            font_size: self.font_size,
        };
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
            p7z_core::settings::prepare_temp_directory(&value.temp_directory)?;
            if registration_changed {
                p7z_platform::configure(&std::env::current_exe()?, &value)?;
            }
            p7z_core::settings::save_all(&value, &appearance, theme, language)?;
            Ok::<_, anyhow::Error>((value, appearance))
        });
        self.saving = true;
        self.task = Some(cx.spawn(async move |view, cx| {
            let result = job.await;
            let _ = view.update(cx, |this, cx| {
                this.task = None;
                this.saving = false;
                match result {
                    Ok((value, appearance)) => {
                        tracing::info!("Preferences saved");
                        App::defer(cx, move |cx| {
                            let _ = handle.update(cx, |_, window, cx| {
                                let _ = owner.update(cx, |owner, cx| {
                                    owner.apply_preferences(value, cx);
                                    let appearance_changed = owner.appearance != appearance;
                                    owner.appearance = appearance;
                                    if theme != crate::theme::current(cx) {
                                        owner.change_theme(theme, window, cx);
                                    } else if appearance_changed {
                                        if let Err(error) = crate::theme::apply(
                                            crate::theme::current(cx),
                                            &owner.appearance,
                                            Some(window),
                                            cx,
                                        ) {
                                            tracing::error!(error = %error, "Cannot apply appearance");
                                            owner.notify_message(error.to_string());
                                            cx.notify();
                                        }
                                    }
                                    if language != p7z_core::i18n::current() {
                                        owner.change_language(language, window, cx);
                                    }
                                });
                            });
                        });
                    }
                    Err(error) => {
                        let details = format!("{error:#}");
                        tracing::error!(error = %details, "Cannot save preferences");
                        this.error = Some(details);
                    }
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
                        this.save(window, cx);
                    }
                    cx.notify();
                });
            });
        }));
        cx.notify();
    }
}
