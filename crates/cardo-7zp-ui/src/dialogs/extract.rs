use crate::*;
use gpui_kit::component::{Disableable, Selectable, checkbox::Checkbox, h_flex, v_flex};

impl Workspace {
    fn destination_button(
        &self,
        kind: Option<DestinationKind>,
        selected: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let index = kind.and_then(|kind| {
            self.destinations
                .iter()
                .position(|(candidate, _)| *candidate == kind)
        });
        let custom = self
            .destinations
            .iter()
            .position(|(candidate, _)| *candidate == DestinationKind::Custom);
        let (id, label, icon_name, chosen, enabled) = match kind {
            None => (
                "browse-extract",
                tr("browse-directory"),
                "FolderOpen",
                custom == Some(selected),
                true,
            ),
            Some(kind) => (
                match kind {
                    DestinationKind::Archive => "destination-archive",
                    DestinationKind::Downloads => "destination-downloads",
                    DestinationKind::Desktop => "destination-desktop",
                    DestinationKind::Custom => "destination-custom",
                },
                kind.label(),
                kind.icon(),
                index == Some(selected),
                index.is_some(),
            ),
        };
        command(id, label)
            .icon(icon(icon_name, 18.))
            .flex_1()
            .min_w(px(160.))
            .selected(chosen)
            .disabled(!enabled)
            .on_click(cx.listener(move |this, _, window, cx| {
                if kind.is_none() {
                    this.choose_extract_directory(window, cx);
                    return;
                }
                let Some(index) = index else { return };
                if let Some(Modal::Extract { destination, .. }) = this.dialogs.current_mut() {
                    *destination = index;
                    cx.notify();
                }
            }))
    }

    pub(super) fn extract_view(
        &self,
        folder: &Entity<InputState>,
        selected: &bool,
        destination: &usize,
        open_after: &bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        {
            let name = self
                .browser
                .view()
                .catalog
                .as_ref()
                .and_then(|c| c.path.file_name())
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
            panel_layout(cx)
                .child(
                    panel_body("extract-options-body")
                        .child(panel_notice(
                            "FolderZip",
                            name,
                            crate::theme::palette(cx).accent,
                        ))
                        .child(panel_field(
                            tr("extract-scope"),
                            h_flex()
                                .flex_wrap()
                                .gap_2()
                                .child(
                                    command("scope-all", tr("all-files"))
                                        .selected(!*selected)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if let Some(Modal::Extract { selected, .. }) =
                                                this.dialogs.current_mut()
                                            {
                                                *selected = false;
                                                cx.notify();
                                            }
                                        })),
                                )
                                .child(
                                    command(
                                        "scope-selected",
                                        &tf(
                                            "selected-count",
                                            &[("count", self.browser.view().selected.len().into())],
                                        ),
                                    )
                                    .disabled(self.browser.view().selected.is_empty())
                                    .selected(*selected)
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            if let Some(Modal::Extract { selected, .. }) =
                                                this.dialogs.current_mut()
                                            {
                                                *selected = true;
                                                cx.notify();
                                            }
                                        },
                                    )),
                                ),
                        ))
                        .child(panel_field(
                            tr("save-location"),
                            v_flex()
                                .gap(px(8.))
                                .child(
                                    h_flex()
                                        .flex_wrap()
                                        .gap(px(8.))
                                        .child(self.destination_button(None, *destination, cx))
                                        .child(self.destination_button(
                                            Some(DestinationKind::Archive),
                                            *destination,
                                            cx,
                                        )),
                                )
                                .child(
                                    h_flex()
                                        .flex_wrap()
                                        .gap(px(8.))
                                        .child(self.destination_button(
                                            Some(DestinationKind::Downloads),
                                            *destination,
                                            cx,
                                        ))
                                        .child(self.destination_button(
                                            Some(DestinationKind::Desktop),
                                            *destination,
                                            cx,
                                        )),
                                ),
                        ))
                        .child(panel_field(tr("new-folder"), text_input(folder)))
                        .child(panel_field(
                            tr("extract-location"),
                            path_strip("copy-extract-path", output, cx),
                        ))
                        .child(
                            Checkbox::new("extract-open-after")
                                .label(tr("extract-open-after"))
                                .text_size(px(13.))
                                .checked(*open_after)
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    if let Some(Modal::Extract { open_after, .. }) =
                                        this.dialogs.current_mut()
                                    {
                                        *open_after = *checked;
                                        cx.notify();
                                    }
                                })),
                        ),
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
}
