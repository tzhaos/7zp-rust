use cardo_ui::ConditionalBuilder;
use crate::commands::Command;
use crate::theme::metrics::*;
use crate::views::SettingsPage;
use crate::*;
use gpui_kit::{base::ElementExt, component::h_flex};

impl Workspace {
    pub(crate) fn toolbar(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let show_labels = !self.preferences.hide_tool_labels;
        let mut row = h_flex()
            .h(px(TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(TOOLBAR_PADDING))
            .gap(px(TOOL_GAP))
            .items_center()
            .bg(rgb(crate::theme::palette(cx).panel));
        for (id, icon, label, action) in [
            ("open", ToolIcon::Open, "open", Command::Open),
            (
                "extract-options",
                ToolIcon::Extract,
                "extract-options",
                Command::Extract,
            ),
            (
                "extract",
                ToolIcon::QuickExtract,
                if self.browser.view().selected.is_empty() {
                    "extract-quick"
                } else {
                    "extract-selected"
                },
                Command::QuickExtractSelection,
            ),
            ("check", ToolIcon::Check, "check", Command::Check),
        ] {
            let button = tool(
                id,
                icon,
                tr(label),
                !self.command_enabled(action, cx),
                show_labels,
                window,
                cx,
            );
            row = row.child(button.on_click(
                cx.listener(move |this, _, window, cx| this.command(action, window, cx)),
            ));
        }
        row = row.child(div().flex_1().min_w(px(8.)));
        for (id, icon, page) in [
            ("page-browser", ToolIcon::Browser, None),
            (
                "page-application",
                ToolIcon::General,
                Some(SettingsPage::Preferences(preferences::Tab::Application)),
            ),
            (
                "page-integration",
                ToolIcon::Associations,
                Some(SettingsPage::Preferences(preferences::Tab::Integration)),
            ),
            (
                "page-advanced",
                ToolIcon::Advanced,
                Some(SettingsPage::Preferences(preferences::Tab::Advanced)),
            ),
            ("page-about", ToolIcon::About, Some(SettingsPage::About)),
        ] {
            let label = page.map(SettingsPage::title).unwrap_or(tr("files-view"));
            let indicator = self.active_tab_indicator.clone();
            row = row.child(
                tool(
                    id,
                    icon,
                    label,
                    self.tasks.is_busy()
                        || self.dialogs.is_open()
                        || self.settings_controls_disabled(cx),
                    true,
                    window,
                    cx,
                )
                .when(self.settings_page == page, |button| {
                    button.on_prepaint(move |bounds, _, cx| {
                        let mut state = indicator.get();
                        state.move_to(bounds.center().x, cx.reduce_motion());
                        indicator.set(state);
                    })
                })
                .on_click(cx.listener(move |this, _, window, cx| match page {
                    Some(page) => this.open_settings(page, window, cx),
                    None => this.show_browser(window, cx),
                })),
            );
        }
        row
    }
}
