use crate::*;

impl Workspace {
    fn destination_choice(&self, selected: usize, cx: &Context<Self>) -> impl IntoElement + use<> {
        let label = self
            .destinations
            .get(selected)
            .map(|(kind, _)| kind.label())
            .unwrap_or_else(|| tr("browse-directory"));
        let destinations = self.destinations.clone();
        let owner = cx.entity().downgrade();
        settings_choice("extract-destination", label, cx).choice_menu(move |_, _| {
            let mut menu = Menu::new();
            for (index, (kind, path)) in destinations.iter().enumerate() {
                let view = owner.clone();
                let kind = *kind;
                menu = menu.item(
                    MenuItem::new(kind.label())
                        .icon(icon(kind.icon(), 16.))
                        .description(path.display().to_string())
                        .checked(index == selected)
                        .on_select(move |_, cx| {
                            let _ = view.update(cx, |this, cx| {
                                if let Some(index) = this
                                    .destinations
                                    .iter()
                                    .position(|(candidate, _)| *candidate == kind)
                                    && let Some(Modal::Extract { destination, .. }) =
                                        this.dialogs.current_mut()
                                {
                                    *destination = index;
                                    cx.notify();
                                }
                            });
                        }),
                );
            }
            let view = owner.clone();
            menu.separator().item(
                MenuItem::new(tr("browse-directory"))
                    .icon(icon("FolderOpen", 16.))
                    .on_select(move |window, cx| {
                        let _ =
                            view.update(cx, |this, cx| this.choose_extract_directory(window, cx));
                    }),
            )
        })
    }

    pub(super) fn extract_view(
        &self,
        folder: &Entity<InputState>,
        selected: &bool,
        destination: &usize,
        open_after: &bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let name = self
            .browser
            .view()
            .catalog
            .as_ref()
            .and_then(|catalog| catalog.path.file_name())
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let output = self
            .destinations
            .get(*destination)
            .map(|(_, path)| {
                path.join(folder.read(cx).value().trim())
                    .display()
                    .to_string()
            })
            .unwrap_or_default();
        let selected_count = self.browser.view().selected.len();
        let scope = settings_segments("extract-scope", tr("extract-scope")).children(
            [false, true]
                .into_iter()
                .enumerate()
                .map(|(index, selection_only)| {
                    let label = if selection_only {
                        tf("selected-count", &[("count", selected_count.into())])
                    } else {
                        tr("all-files").to_owned()
                    };
                    let owner = cx.entity().downgrade();
                    settings_segment(
                        ("extract-scope-option", index),
                        &label,
                        *selected == selection_only,
                        selection_only && selected_count == 0,
                        cx,
                    )
                    .set_position(index + 1, 2)
                    .on_change(move |_, _, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            if let Some(Modal::Extract { selected, .. }) =
                                this.dialogs.current_mut()
                            {
                                *selected = selection_only;
                                cx.notify();
                            }
                        });
                    })
                }),
        );
        panel_layout(cx)
            .child(
                panel_body("extract-options-body")
                    .child(
                        gpui_kit::component::h_flex()
                            .min_w_0()
                            .gap(px(12.))
                            .child(artwork(ToolIcon::Browser, window, cx))
                            .child(
                                body_text(name)
                                    .flex_1()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::SEMIBOLD),
                            ),
                    )
                    .child(settings_group(
                        [
                            settings_row(tr("extract-scope"), scope, cx).into_any_element(),
                            settings_row(
                                tr("save-location"),
                                self.destination_choice(*destination, cx),
                                cx,
                            )
                            .into_any_element(),
                            settings_row(
                                tr("new-folder"),
                                settings_input(folder, tr("new-folder"))
                                    .w(px(cardo_ui::settings::metrics::INPUT_WIDTH)),
                                cx,
                            )
                            .into_any_element(),
                            settings_row(
                                tr("extract-open-after"),
                                SettingsSwitch::new(
                                    "extract-open-after",
                                    tr("extract-open-after"),
                                    *open_after,
                                    false,
                                )
                                .on_click(cx.listener(
                                    |this, checked: &bool, _, cx| {
                                        if let Some(Modal::Extract { open_after, .. }) =
                                            this.dialogs.current_mut()
                                        {
                                            *open_after = *checked;
                                            cx.notify();
                                        }
                                    },
                                )),
                                cx,
                            )
                            .into_any_element(),
                        ],
                        cx,
                    ))
                    .child(panel_field(
                        tr("extract-location"),
                        path_strip("copy-extract-path", output, cx),
                    )),
            )
            .child(self.footer(
                tr("extract-start"),
                self.destinations.is_empty(),
                cx.listener(|this, _, _, cx| this.submit_extract(cx)),
                cx,
            ))
            .into_any_element()
    }
}
