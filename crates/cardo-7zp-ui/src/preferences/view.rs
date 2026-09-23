use super::*;

impl Render for PreferencesForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.tab {
            Tab::Application => self.application_page(cx),
            Tab::Integration => self.integration_page(cx),
            Tab::Advanced => self.advanced_page(cx),
            Tab::Updates => settings_group(
                [self
                    .toggle_row(
                        "settings-updates",
                        Toggle::Updates,
                        self.value.check_updates,
                        cx,
                    )
                    .into_any_element()],
                cx,
            ),
        };
        let error = self
            .error
            .clone()
            .map(|error| body_text(error).text_color(rgb(crate::theme::palette(cx).danger)));
        if self.tab == Tab::Updates {
            return v_flex()
                .flex_shrink_0()
                .min_w_0()
                .child(content)
                .children(error);
        }
        settings_page(cx)
            .child(
                settings_content("preferences-content")
                    .id(("settings-content", self.tab.order() as usize))
                    .child(content)
                    .children(error),
            )
            .children(self.association_dropdown(window, cx))
    }
}
