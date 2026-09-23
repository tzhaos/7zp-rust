use crate::preferences::Tab;
use crate::theme::metrics::{ICON_BUTTON_SIZE, settings as metrics};
use crate::*;
use gpui_kit::component::{Disableable, h_flex, v_flex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    Preferences(Tab),
    About,
}

impl SettingsPage {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Preferences(tab) => tab.label(),
            Self::About => tr("menu-about"),
        }
    }
}

impl Workspace {
    pub(crate) fn open_settings(
        &mut self,
        page: SettingsPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.settings_busy(cx) || self.tasks.is_busy() || self.dialogs.is_open() {
            return;
        }
        let tab = match page {
            SettingsPage::Preferences(tab) => tab,
            SettingsPage::About => Tab::Updates,
        };
        if let Some(form) = &self.settings_form {
            form.update(cx, |form, cx| form.set_tab(tab, window, cx));
        } else {
            let owner = cx.entity().downgrade();
            let form = cx.new(|cx| {
                preferences::PreferencesForm::new(
                    owner,
                    self.preferences.clone(),
                    self.appearance.clone(),
                    tab,
                    window,
                    cx,
                )
            });
            self.settings_subscription = Some(cx.observe(&form, |_, _, cx| cx.notify()));
            self.settings_form = Some(form);
        }
        self.dismiss_history(window, cx);
        self.focus.focus(window, cx);
        self.settings_page = Some(page);
        cx.notify();
    }

    pub(crate) fn show_browser(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings_busy(cx) || self.dialogs.is_open() {
            return;
        }
        self.settings_page = None;
        if let Some(form) = &self.settings_form {
            form.update(cx, |form, cx| form.dismiss_associations(window, cx));
        }
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn settings_view(&self, page: SettingsPage, cx: &mut Context<Self>) -> AnyElement {
        let body = match page {
            SettingsPage::Preferences(_) => self
                .settings_form
                .as_ref()
                .map(|form| form.clone().into_any_element())
                .unwrap_or_else(|| div().into_any_element()),
            SettingsPage::About => self.about_view(self.update_status.as_ref(), cx),
        };
        v_flex()
            .size_full()
            .pl(px(metrics::NAV_GUTTER))
            .min_h_0()
            .min_w_0()
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(metrics::CONTENT_PADDING))
                    .pt(px(20.))
                    .child(
                        h_flex()
                            .relative()
                            .min_h(px(ICON_BUTTON_SIZE))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(-ICON_BUTTON_SIZE - metrics::BACK_BUTTON_GAP))
                                    .top_0()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .child(
                                        icon_button(
                                            "settings-back",
                                            "ArrowLeft",
                                            tr("back"),
                                            true,
                                            cx,
                                        )
                                        .disabled(self.settings_busy(cx))
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.show_browser(window, cx)
                                            }),
                                        ),
                                    ),
                            )
                            .child(
                                compact_text("settings-heading", page.title())
                                    .flex_1()
                                    .text_size(px(metrics::TITLE_SIZE))
                                    .font_weight(FontWeight::MEDIUM),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(body),
            )
            .into_any_element()
    }
}
