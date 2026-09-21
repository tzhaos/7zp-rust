use crate::*;
use gpui_kit::component::{
    Disableable, Selectable, checkbox::Checkbox, h_flex, menu::DropdownMenu, v_flex,
};

impl Workspace {
    pub(super) fn extract_view(
        &self,
        folder: &Entity<InputState>,
        selected: &bool,
        destination: &usize,
        overwrite: &Overwrite,
        open_after: &bool,
        window: &mut Window,
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
            v_flex()
                .child(
                    v_flex()
                        .id("extract-options-body")
                        .max_h(window.viewport_size().height - px(230.))
                        .overflow_y_scroll()
                        .px(px(24.))
                        .py(px(22.))
                        .gap(px(20.))
                        .child(h_flex().gap_2().child(icon("FolderZip", 17.)).child(name))
                        .child(
                            v_flex().gap_2().child(tr("extract-scope")).child(
                                h_flex()
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
                                                &[(
                                                    "count",
                                                    self.browser.view().selected.len().into(),
                                                )],
                                            ),
                                        )
                                        .disabled(self.browser.view().selected.is_empty())
                                        .selected(*selected)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                if let Some(Modal::Extract { selected, .. }) =
                                                    this.dialogs.current_mut()
                                                {
                                                    *selected = true;
                                                    cx.notify();
                                                }
                                            }),
                                        ),
                                    ),
                            ),
                        )
                        .child(
                            v_flex().gap_2().child(tr("save-location")).child(
                                h_flex().flex_wrap().gap(px(12.)).children(
                                    self.destinations.iter().enumerate().map(
                                        |(index, (kind, _))| {
                                            command(("destination", index), kind.label())
                                                .icon(icon(kind.icon(), 22.))
                                                .w(px(192.))
                                                .h(px(48.))
                                                .selected(index == *destination)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    if let Some(Modal::Extract {
                                                        destination,
                                                        ..
                                                    }) = this.dialogs.current_mut()
                                                    {
                                                        *destination = index;
                                                        cx.notify();
                                                    }
                                                }))
                                        },
                                    ),
                                ),
                            ),
                        )
                        .child(
                            command("browse-extract", tr("browse-directory"))
                                .icon(icon("FolderOpen", 18.))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.choose_extract_directory(window, cx)
                                })),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .child(tr("new-folder"))
                                .child(text_input(folder)),
                        )
                        .child(
                            h_flex().w_full().child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(tr("extract-location"))
                                    .child(path_strip("copy-extract-path", output, cx)),
                            ),
                        )
                        .child(
                            v_flex().gap_2().child(tr("overwrite-label")).child(
                                command("overwrite-mode", tr(overwrite.label_key()))
                                    .dropdown_caret(true)
                                    .dropdown_menu({
                                        let owner = cx.entity().downgrade();
                                        let current = *overwrite;
                                        move |mut menu, _, _| {
                                            menu = menu_style(menu);
                                            for mode in Overwrite::ALL {
                                                let owner = owner.clone();
                                                menu = menu.item(
                                                    PopupMenuItem::new(tr(mode.label_key()))
                                                        .checked(current == mode)
                                                        .on_click(move |_, _, cx| {
                                                            let _ = owner.update(cx, |this, cx| {
                                                                if let Some(Modal::Extract {
                                                                    overwrite,
                                                                    ..
                                                                }) = this.dialogs.current_mut()
                                                                {
                                                                    *overwrite = mode;
                                                                    cx.notify();
                                                                }
                                                            });
                                                        }),
                                                );
                                            }
                                            menu
                                        }
                                    }),
                            ),
                        )
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
                    cx.listener(|this, _, _, cx| {
                        if let Some(Modal::Extract {
                            folder,
                            selected,
                            destination,
                            overwrite,
                            open_after,
                        }) = this.dialogs.take()
                        {
                            let folder = folder.read(cx).value().trim().to_owned();
                            this.extract(selected, folder, destination, overwrite, open_after, cx);
                        }
                    }),
                    cx,
                ))
                .into_any_element()
        }
    }
}
