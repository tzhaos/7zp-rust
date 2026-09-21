mod create;
mod error;
pub(super) use create::CreateForm;
pub(super) use error::ErrorDialog;

use crate::i18n::{tf, tr};
use crate::ui::*;
use gpui_kit::base::FocusTrapElement;
use gpui_kit::{
    component::{
        Disableable, Selectable, checkbox::Checkbox, h_flex, input::Textarea, menu::DropdownMenu,
        v_flex,
    },
    prelude::FluentBuilder,
};

impl Workspace {
    pub(in crate::ui) fn footer(
        &self,
        label: &str,
        disabled: bool,
        submit: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        cx: &Context<Self>,
    ) -> Div {
        let p = crate::ui::theme::palette(cx);
        h_flex()
            .flex_shrink_0()
            .px(px(24.))
            .py(px(16.))
            .gap_2()
            .justify_end()
            .bg(rgb(p.panel))
            .rounded_b(px(7.))
            .border_t_1()
            .border_color(rgb(p.border))
            .child(
                command("cancel-modal", tr("cancel"))
                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
            )
            .child(
                primary("submit-modal", label)
                    .disabled(disabled)
                    .on_click(submit),
            )
    }

    pub(in crate::ui) fn modal(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let p = crate::ui::theme::palette(cx);
        let modal = self.modal.as_ref()?;
        let body = match modal {
            Modal::Error(error) => self.error_view(error, window, cx).into_any_element(),
            Modal::Progress => self.extraction_progress_view(cx).into_any_element(),
            Modal::ExtractionResult(text) => v_flex()
                .p(px(24.))
                .gap(px(20.))
                .child(
                    div()
                        .id("extraction-result")
                        .max_h(window.viewport_size().height - px(220.))
                        .overflow_scroll()
                        .text_size(px(13.))
                        .child(div().whitespace_normal().child(text.clone())),
                )
                .child(
                    h_flex().justify_end().child(
                        command("close-extraction-result", tr("dialog-close"))
                            .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                    ),
                )
                .into_any_element(),
            Modal::Comment(input) => {
                v_flex()
                    .p(px(24.))
                    .gap(px(20.))
                    .child(Textarea::new(input).h(px(210.)).text_size(px(13.)))
                    .child(
                        h_flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(
                                command("comment-cancel", tr("cancel"))
                                    .on_click(cx.listener(|this, _, _, cx| this.close_modal(cx))),
                            )
                            .child(primary("comment-save", tr("settings-save")).on_click(
                                cx.listener(|this, _, _, cx| {
                                    if let (Some(Modal::Comment(input)), Some(catalog)) =
                                        (&this.modal, &this.catalog)
                                    {
                                        let request = Request::Comment(
                                            catalog.clone(),
                                            input.read(cx).value().to_string(),
                                        );
                                        this.execute(request, this.password.clone(), cx);
                                    }
                                }),
                            )),
                    )
                    .into_any_element()
            }
            Modal::Update(status) => {
                use crate::application::update::Status;
                let description = match status {
                    Status::Checking => tr("update-checking").to_owned(),
                    Status::Unconfigured => tr("update-unconfigured").to_owned(),
                    Status::Current => tr("update-current").to_owned(),
                    Status::Available { version, .. } => {
                        tf("update-available", &[("version", version.as_str().into())])
                    }
                    Status::Failed(error) => {
                        tf("update-failed", &[("error", error.as_str().into())])
                    }
                };
                v_flex()
                    .px(px(24.))
                    .py(px(22.))
                    .gap(px(16.))
                    .child(tf(
                        "update-installed",
                        &[("version", crate::application::update::VERSION.into())],
                    ))
                    .child(description)
                    .when(matches!(status, Status::Available { .. }), |el| {
                        let Status::Available {
                            download, release, ..
                        } = status
                        else {
                            return el;
                        };
                        let download = download.clone();
                        let release = release.clone();
                        el.child(tr("update-package-note")).child(
                            h_flex()
                                .gap(px(8.))
                                .child(
                                    primary("update-download", tr("update-download"))
                                        .on_click(move |_, _, cx| cx.open_url(&download)),
                                )
                                .child(
                                    command("update-notes", tr("update-notes"))
                                        .on_click(move |_, _, cx| cx.open_url(&release)),
                                ),
                        )
                    })
                    .when(!matches!(status, Status::Checking), |el| {
                        el.child(
                            command("update-retry", tr("update-check"))
                                .on_click(cx.listener(|this, _, _, cx| this.check_update(cx))),
                        )
                    })
                    .into_any_element()
            }
            Modal::Create(form) => form.clone().into_any_element(),
            Modal::Rename { source, input } => v_flex()
                .child(
                    v_flex()
                        .px(px(24.))
                        .py(px(22.))
                        .gap_3()
                        .child(source.clone())
                        .child(text_input(input)),
                )
                .child(self.footer(
                    tr("continue"),
                    input.read(cx).value().trim().is_empty(),
                    cx.listener(|this, _, _, cx| {
                        if let Some(Modal::Rename { source, input }) = this.modal.take() {
                            let name = input.read(cx).value().trim().to_owned();
                            if name.contains(['/', '\\']) {
                                this.message = Some(tr("archive-name-invalid").into());
                                cx.notify();
                                return;
                            }
                            let destination = source
                                .rsplit_once('/')
                                .map(|(parent, _)| format!("{parent}/{name}"))
                                .unwrap_or(name);
                            if let Some(catalog) = this.catalog.clone() {
                                this.execute(
                                    Request::Edit(
                                        catalog,
                                        Edit::Rename {
                                            source,
                                            destination,
                                        },
                                    ),
                                    this.password.clone(),
                                    cx,
                                );
                            }
                        }
                    }),
                    cx,
                ))
                .into_any_element(),
            Modal::Report(text) => v_flex()
                .px(px(24.))
                .py(px(22.))
                .gap(px(16.))
                .child(
                    div()
                        .id("checksum-report")
                        .max_h(window.viewport_size().height - px(240.))
                        .overflow_scroll()
                        .text_size(px(12.))
                        .child(div().whitespace_nowrap().child(text.clone())),
                )
                .child(
                    command("copy-checksum-report", tr("checksum-copy")).on_click({
                        let text = text.clone();
                        move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()))
                        }
                    }),
                )
                .into_any_element(),
            Modal::Info(fields) => v_flex()
                .id("properties-body")
                .max_h(window.viewport_size().height - px(180.))
                .overflow_y_scroll()
                .px(px(24.))
                .py(px(22.))
                .gap(px(16.))
                .children(fields.iter().map(|(key, value)| {
                    h_flex()
                        .gap(px(16.))
                        .child(
                            div()
                                .w(px(88.))
                                .flex_shrink_0()
                                .text_color(rgb(p.muted))
                                .child(key.clone()),
                        )
                        .child(div().flex_1().min_w_0().child(value.clone()))
                }))
                .into_any_element(),
            Modal::Password { input, .. } => v_flex()
                .child(
                    v_flex()
                        .px(px(24.))
                        .py(px(22.))
                        .gap_3()
                        .child(tr("password-prompt"))
                        .child(text_input(input)),
                )
                .child(self.footer(
                    tr("continue"),
                    input.read(cx).value().is_empty(),
                    cx.listener(|this, _, _, cx| {
                        if let Some(Modal::Password { input, request }) = this.modal.take() {
                            let password = input.read(cx).value().to_string();
                            this.password = password.clone();
                            this.execute(request, password, cx);
                        }
                    }),
                    cx,
                ))
                .into_any_element(),
            Modal::Extract {
                folder,
                selected,
                destination,
                overwrite,
                open_after,
            } => {
                let name = self
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
                                                    if let Some(Modal::Extract {
                                                        selected, ..
                                                    }) = &mut this.modal
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
                                                    &[("count", self.selected.len().into())],
                                                ),
                                            )
                                            .disabled(self.selected.is_empty())
                                            .selected(*selected)
                                            .on_click(
                                                cx.listener(|this, _, _, cx| {
                                                    if let Some(Modal::Extract {
                                                        selected, ..
                                                    }) = &mut this.modal
                                                    {
                                                        *selected = true;
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                        ),
                                ),
                            )
                            .child(v_flex().gap_2().child(tr("save-location")).child(
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
                                                    }) = &mut this.modal
                                                    {
                                                        *destination = index;
                                                        cx.notify();
                                                    }
                                                }))
                                        },
                                    ),
                                ),
                            ))
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
                                                                let _ =
                                                                    owner.update(cx, |this, cx| {
                                                                        if let Some(
                                                                            Modal::Extract {
                                                                                overwrite,
                                                                                ..
                                                                            },
                                                                        ) = &mut this.modal
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
                                            &mut this.modal
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
                            }) = this.modal.take()
                            {
                                let folder = folder.read(cx).value().trim().to_owned();
                                this.extract(
                                    selected,
                                    folder,
                                    destination,
                                    overwrite,
                                    open_after,
                                    cx,
                                );
                            }
                        }),
                        cx,
                    ))
                    .into_any_element()
            }
        };
        Some(
            div()
                .absolute()
                .inset_0()
                .occlude()
                .bg(rgba(p.scrim))
                .flex()
                .items_center()
                .justify_center()
                .p(px(24.))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.close_modal(cx);
                        cx.stop_propagation();
                    }),
                )
                .child(
                    v_flex()
                        .id("modal")
                        .track_focus(&self.modal_focus)
                        .w(px(match modal {
                            Modal::Report(_) => 680.,
                            Modal::Progress => 540.,
                            Modal::Error(_) => 520.,
                            _ => 464.,
                        }))
                        .max_w_full()
                        .max_h(window.viewport_size().height - px(48.))
                        .bg(rgb(p.surface))
                        .border_1()
                        .border_color(rgb(p.border))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .shadow_lg()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(
                            h_flex()
                                .flex_shrink_0()
                                .px(px(20.))
                                .py(px(14.))
                                .gap_3()
                                .items_center()
                                .border_b_1()
                                .border_color(rgb(p.border))
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(16.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(self.modal_title.clone()),
                                )
                                .when(!matches!(modal, Modal::Progress), |el| {
                                    el.child(
                                        icon_button(
                                            "close-modal",
                                            "Dismiss",
                                            tr("dialog-close"),
                                            cx,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.close_modal(cx)),
                                        ),
                                    )
                                }),
                        )
                        .child(body)
                        .focus_trap("modal-trap", &self.modal_focus),
                )
                .into_any_element(),
        )
    }
}
