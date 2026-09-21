use crate::{Workspace, components::*, format::size_text};
use gpui_kit::{
    component::{
        Disableable,
        button::ButtonVariants,
        checkbox::Checkbox,
        h_flex,
        input::InputState,
        menu::{DropdownMenu, PopupMenuItem},
        switch::Switch,
        v_flex,
    },
    prelude::FluentBuilder,
    *,
};
use cardo_7zp_application::filesystem::{self, SourceInfo};
use cardo_7zp_archive::{CreateOptions, Format, Level, Method, Threads, Volume};
use cardo_7zp_core::i18n::{tf, tr};
use std::{collections::HashMap, path::PathBuf};

pub struct CreateForm {
    owner: WeakEntity<Workspace>,
    files: Vec<PathBuf>,
    metadata: HashMap<PathBuf, Result<SourceInfo, String>>,
    name: Entity<InputState>,
    automatic_name: String,
    password: Entity<InputState>,
    format: Format,
    level: Level,
    method: Method,
    threads: Threads,
    volume: Volume,
    solid: bool,
    encrypted: bool,
    encrypt_names: bool,
    visible: bool,
    advanced: bool,
    pub email: bool,
    _inputs: Vec<Subscription>,
}

impl CreateForm {
    pub fn new(
        owner: WeakEntity<Workspace>,
        files: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx));
        let initial = files
            .first()
            .and_then(|path| path.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| tr("archive-default-name").into());
        name.update(cx, |input, cx| input.set_value(initial.clone(), window, cx));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr("password-input"))
                .masked(true)
        });
        let inputs = vec![
            cx.observe(&name, |_, _, cx| cx.notify()),
            cx.observe(&password, |_, _, cx| cx.notify()),
        ];
        let mut this = Self {
            owner,
            files: Vec::new(),
            metadata: HashMap::new(),
            name,
            automatic_name: initial,
            password,
            format: Format::SevenZip,
            level: Level::Normal,
            method: Method::Lzma2,
            threads: Threads::Auto,
            volume: Volume::None,
            solid: true,
            encrypted: false,
            encrypt_names: true,
            visible: false,
            advanced: false,
            email: false,
            _inputs: inputs,
        };
        this.add(files, window, cx);
        this
    }

    pub fn suggest_name(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.automatic_name = name.clone();
        self.name
            .update(cx, |input, cx| input.set_value(name, window, cx));
    }

    pub fn add(&mut self, files: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        if self.files.is_empty()
            && self.name.read(cx).value().as_str() == self.automatic_name
            && let Some(stem) = files.first().and_then(|path| path.file_stem())
        {
            self.automatic_name = stem.to_string_lossy().into_owned();
            self.name.update(cx, |input, cx| {
                input.set_value(self.automatic_name.clone(), window, cx)
            });
        }
        let mut added = Vec::new();
        for path in files {
            if !self.files.contains(&path) {
                added.push(path.clone());
                self.files.push(path);
            }
        }
        if !added.is_empty() {
            let read = cx.background_executor().spawn(async move {
                added
                    .into_iter()
                    .map(|path| {
                        let metadata = filesystem::inspect_source(&path);
                        (path, metadata)
                    })
                    .collect::<Vec<_>>()
            });
            cx.spawn(async move |view, cx| {
                let metadata = read.await;
                let _ = view.update(cx, |this, cx| {
                    for (path, metadata) in metadata {
                        if this.files.contains(&path) {
                            this.metadata.insert(path, metadata);
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    fn choose_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = window.window_handle();
        let picker = cx.background_executor().spawn(async {
            rfd::FileDialog::new()
                .set_title(tr("add-files"))
                .pick_files()
        });
        cx.spawn(async move |view, cx| {
            if let Some(files) = picker.await {
                let _ = handle.update(cx, |_, window, cx| {
                    let _ = view.update(cx, |this, cx| this.add(files, window, cx));
                });
            }
        })
        .detach();
    }

    fn choice<T, F>(
        &self,
        id: &'static str,
        current: T,
        choices: Vec<(T, String)>,
        disabled: bool,
        on_pick: F,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<T, F>
    where
        T: Copy + PartialEq + 'static,
        F: Fn(&mut Self, T) + Clone + 'static,
    {
        let label = choices
            .iter()
            .find(|(value, _)| *value == current)
            .map(|(_, label)| label.clone())
            .unwrap_or_default();
        let view = cx.entity().downgrade();
        command(id, &label)
            .w_full()
            .dropdown_caret(true)
            .disabled(disabled)
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu_style(menu);
                for (value, label) in &choices {
                    let view = view.clone();
                    let on_pick = on_pick.clone();
                    let value = *value;
                    menu = menu.item(
                        PopupMenuItem::new(label.clone())
                            .checked(value == current)
                            .on_click(move |_, _, cx| {
                                let _ = view.update(cx, |this, cx| {
                                    on_pick(this, value);
                                    cx.notify();
                                });
                            }),
                    );
                }
                menu
            })
    }
}

fn method_label(method: Method) -> String {
    if method == Method::Default {
        tr("default-value").into()
    } else {
        method.argument().into()
    }
}

fn threads_label(threads: Threads) -> String {
    match threads.label_key() {
        Some(key) => tr(key).into(),
        None => threads.argument().unwrap_or_default().into(),
    }
}

fn volume_label(volume: Volume) -> String {
    match volume.label_key() {
        Some(key) => tr(key).into(),
        None => volume.caption().into(),
    }
}

fn field(label: &str, content: impl IntoElement) -> Div {
    v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(8.))
        .child(label.to_owned())
        .child(content)
}

impl Render for CreateForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = crate::theme::palette(cx);
        let total: u64 = self
            .files
            .iter()
            .filter_map(|path| {
                self.metadata
                    .get(path)
                    .and_then(|value| value.as_ref().ok())
            })
            .filter(|meta| meta.file)
            .map(|meta| meta.size)
            .sum();
        let encrypted = self.encrypted && self.format.supports_password();
        let total_label = if self
            .files
            .iter()
            .any(|path| !self.metadata.contains_key(path))
        {
            tr("source-reading").to_owned()
        } else {
            tf(
                "file-total",
                &[
                    ("count", self.files.len().into()),
                    ("size", size_text(total).into()),
                ],
            )
        };
        let disabled = self.files.is_empty()
            || self
                .files
                .iter()
                .any(|path| !matches!(self.metadata.get(path), Some(Ok(_))))
            || self.name.read(cx).value().trim().is_empty()
            || (encrypted && self.password.read(cx).value().is_empty())
            || (self.format.single_file()
                && (self.files.len() != 1
                    || self
                        .metadata
                        .get(&self.files[0])
                        .is_some_and(|value| value.as_ref().is_ok_and(|meta| meta.directory))));
        let extension = self.format.extension();
        v_flex()
            .max_h(window.viewport_size().height - px(140.))
            .min_h_0()
            .child(
                v_flex()
                    .id("create-body")
                    .overflow_y_scroll()
                    .min_h_0()
                    .px(px(24.))
                    .py(px(22.))
                    .gap(px(20.))
                    .child(
                        h_flex().justify_between().child(tr("source-files")).child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(p.muted))
                                .child(total_label.clone()),
                        ),
                    )
                    .child(
                        v_flex()
                            .border_1()
                            .border_color(rgb(p.border))
                            .rounded(px(4.))
                            .overflow_hidden()
                            .when(self.files.is_empty(), |el| {
                                el.child(
                                    command("add-empty", tr("add-files"))
                                        .custom(subtle_variant(cx))
                                        .icon(icon("Add", 24.))
                                        .border_0()
                                        .rounded(px(0.))
                                        .h(px(92.))
                                        .w_full()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.choose_files(window, cx)
                                        })),
                                )
                            })
                            .children(self.files.iter().enumerate().map(|(index, path)| {
                                let name = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned();
                                h_flex()
                                    .h(px(40.))
                                    .px(px(10.))
                                    .gap(px(8.))
                                    .items_center()
                                    .child(file_icon(
                                        path,
                                        self.metadata.get(path).is_some_and(|value| {
                                            value.as_ref().is_ok_and(|meta| meta.directory)
                                        }),
                                        true,
                                    ))
                                    .child(div().flex_1().truncate().child(name.clone()))
                                    .child(div().text_size(px(11.)).text_color(rgb(p.muted)).child(
                                        match self.metadata.get(path) {
                                            Some(Ok(meta)) if meta.file => size_text(meta.size),
                                            None => tr("source-reading").to_owned(),
                                            _ => String::new(),
                                        },
                                    ))
                                    .child(
                                        icon_button(
                                            ("remove-file", index),
                                            "Dismiss",
                                            &tf("remove-file", &[("name", name.as_str().into())]),
                                            cx,
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                let path = this.files.remove(index);
                                                this.metadata.remove(&path);
                                                cx.notify();
                                            }),
                                        ),
                                    )
                            }))
                            .when(!self.files.is_empty(), |el| {
                                el.child(
                                    command("add-more", tr("add-files"))
                                        .icon(icon("Add", 16.))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.choose_files(window, cx)
                                        })),
                                )
                            }),
                    )
                    .children(self.files.iter().filter_map(|path| {
                        self.metadata
                            .get(path)
                            .and_then(|value| value.as_ref().err())
                            .map(|error| {
                                div()
                                    .text_size(px(12.))
                                    .text_color(rgb(p.danger))
                                    .child(error.clone())
                            })
                    }))
                    .child(field(
                        tr("archive-name"),
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().min_w_0().child(text_input(&self.name)))
                            .child(format!(".{extension}")),
                    ))
                    .child(
                        h_flex()
                            .gap(px(16.))
                            .child(field(
                                tr("format"),
                                self.choice(
                                    "format",
                                    self.format,
                                    Format::ALL
                                        .into_iter()
                                        .map(|format| (format, format.title()))
                                        .collect(),
                                    false,
                                    |form, format| {
                                        form.format = format;
                                        form.method = format.default_method();
                                    },
                                    cx,
                                ),
                            ))
                            .child(field(
                                tr("compression-level"),
                                self.choice(
                                    "level",
                                    self.level,
                                    Level::ALL
                                        .into_iter()
                                        .map(|level| (level, tr(level.label_key()).into()))
                                        .collect(),
                                    self.format == Format::Tar,
                                    |form, level| form.level = level,
                                    cx,
                                ),
                            )),
                    )
                    .when(self.format.single_file(), |el| {
                        el.child(
                            div()
                                .text_color(rgb(p.muted))
                                .text_size(px(12.))
                                .child(tr("single-file-note")),
                        )
                    })
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .items_center()
                            .child(icon("LockClosed", 19.))
                            .child(div().flex_1().child(tr("password-protection")))
                            .child(
                                Switch::new("encrypted")
                                    .checked(encrypted)
                                    .disabled(!self.format.supports_password())
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.encrypted = *checked;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(encrypted, |el| {
                        el.child(
                            v_flex()
                                .gap(px(12.))
                                .child(field(
                                    tr("password"),
                                    h_flex()
                                        .gap(px(6.))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .child(text_input(&self.password)),
                                        )
                                        .child(
                                            icon_button(
                                                "password-visibility",
                                                if self.visible { "EyeOff" } else { "Eye" },
                                                if self.visible {
                                                    tr("password-hide")
                                                } else {
                                                    tr("password-show")
                                                },
                                                cx,
                                            )
                                            .on_click(
                                                cx.listener(|this, _, window, cx| {
                                                    this.visible = !this.visible;
                                                    this.password.update(cx, |input, cx| {
                                                        input.set_masked(!this.visible, window, cx)
                                                    });
                                                    cx.notify();
                                                }),
                                            ),
                                        ),
                                ))
                                .when(self.format.supports_header_encryption(), |el| {
                                    el.child(
                                        Checkbox::new("encrypt-names")
                                            .label(tr("encrypt-names"))
                                            .text_size(px(13.))
                                            .checked(self.encrypt_names)
                                            .on_click(cx.listener(
                                                |this, checked: &bool, _, cx| {
                                                    this.encrypt_names = *checked;
                                                    cx.notify();
                                                },
                                            )),
                                    )
                                }),
                        )
                    })
                    .child(
                        command("advanced", tr("advanced-options"))
                            .icon(icon(
                                if self.advanced {
                                    "ChevronDown"
                                } else {
                                    "ChevronRight"
                                },
                                14.,
                            ))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.advanced = !this.advanced;
                                cx.notify();
                            })),
                    )
                    .when(self.advanced, |el| {
                        el.child(
                            v_flex()
                                .gap(px(20.))
                                .child(
                                    h_flex()
                                        .gap(px(16.))
                                        .child(field(
                                            tr("compression-method"),
                                            self.choice(
                                                "method",
                                                self.method,
                                                self.format
                                                    .methods()
                                                    .iter()
                                                    .copied()
                                                    .map(|method| (method, method_label(method)))
                                                    .collect(),
                                                !self.format.supports_method(),
                                                |form, method| form.method = method,
                                                cx,
                                            ),
                                        ))
                                        .child(field(
                                            tr("threads"),
                                            self.choice(
                                                "threads",
                                                self.threads,
                                                Threads::ALL
                                                    .into_iter()
                                                    .map(|threads| {
                                                        (threads, threads_label(threads))
                                                    })
                                                    .collect(),
                                                false,
                                                |form, threads| form.threads = threads,
                                                cx,
                                            ),
                                        )),
                                )
                                .child(field(
                                    tr("volume-size"),
                                    self.choice(
                                        "split",
                                        self.volume,
                                        Volume::ALL
                                            .into_iter()
                                            .map(|volume| (volume, volume_label(volume)))
                                            .collect(),
                                        false,
                                        |form, volume| form.volume = volume,
                                        cx,
                                    ),
                                ))
                                .when(self.volume != Volume::None, |el| {
                                    el.child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(rgb(p.muted))
                                            .child(tr("volume-note")),
                                    )
                                })
                                .child(
                                    Checkbox::new("solid")
                                        .label(tr("solid"))
                                        .text_size(px(13.))
                                        .checked(self.solid && self.format.supports_solid())
                                        .disabled(!self.format.supports_solid())
                                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                            this.solid = *checked;
                                            cx.notify();
                                        })),
                                ),
                        )
                    }),
            )
            .child(
                h_flex()
                    .flex_shrink_0()
                    .px(px(24.))
                    .py(px(16.))
                    .gap(px(8.))
                    .items_center()
                    .bg(rgb(p.panel))
                    .rounded_b(px(7.))
                    .border_t_1()
                    .border_color(rgb(p.border))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(11.))
                            .text_color(rgb(p.muted))
                            .child(total_label),
                    )
                    .child(command("cancel-create", tr("cancel")).on_click(cx.listener(
                        |this, _, _, cx| {
                            let _ = this.owner.update(cx, |owner, cx| owner.close_modal(cx));
                        },
                    )))
                    .child(
                        primary("submit-create", tr("create-start"))
                            .icon(icon("Archive", 15.))
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, _, cx| {
                                let options = CreateOptions {
                                    format: this.format,
                                    level: this.level,
                                    method: this.method,
                                    threads: this.threads,
                                    volume: this.volume,
                                    solid: this.solid,
                                    password: if this.encrypted && this.format.supports_password() {
                                        this.password.read(cx).value().to_string()
                                    } else {
                                        String::new()
                                    },
                                    encrypt_names: this.encrypt_names,
                                };
                                let name = this.name.read(cx).value().trim().to_owned();
                                let files = this.files.clone();
                                let _ = this.owner.update(cx, |owner, cx| {
                                    owner.create(files, name, options, this.email, cx)
                                });
                            })),
                    ),
            )
    }
}
