use cardo_ui::ConditionalBuilder;
use crate::{ScrollableElement, Workspace, components::*, format::size_text};
use p7z_core::i18n::{tf, tr};
use p7z_engine::{CreateOptions, Format, Level, Method, Threads, Volume};
use p7z_requests::filesystem::{self, SourceInfo};
use cardo_ui::menu::{Menu, MenuItem, MenuTrigger};
use gpui_kit::{
    component::{
        Disableable, h_flex,
        input::{InputEvent, InputState},
        v_flex,
    },
    *,
};
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
            cx.subscribe(&name, |this, _, event, cx| {
                if matches!(
                    event,
                    InputEvent::PressEnter {
                        secondary: false,
                        shift: false
                    }
                ) {
                    this.submit(cx);
                }
            }),
            cx.subscribe(&password, |this, _, event, cx| {
                if matches!(
                    event,
                    InputEvent::PressEnter {
                        secondary: false,
                        shift: false
                    }
                ) {
                    this.submit(cx);
                }
            }),
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

    fn submit(&self, cx: &mut Context<Self>) {
        if !self.can_submit(cx) {
            return;
        }
        let options = CreateOptions {
            format: self.format,
            level: self.level,
            method: self.method,
            threads: self.threads,
            volume: self.volume,
            solid: self.solid,
            password: if self.encrypted && self.format.supports_password() {
                self.password.read(cx).value().to_string()
            } else {
                String::new()
            },
            encrypt_names: self.encrypt_names,
        };
        let name = self.name.read(cx).value().trim().to_owned();
        let files = self.files.clone();
        let _ = self.owner.update(cx, |owner, cx| {
            owner.create(files, name, options, self.email, cx)
        });
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
        settings_choice(id, &label, cx)
            .disabled(disabled)
            .choice_menu(move |_, _| {
                let mut menu = Menu::new();
                for (value, label) in &choices {
                    let view = view.clone();
                    let on_pick = on_pick.clone();
                    let value = *value;
                    menu = menu.item(
                        MenuItem::new(label.clone())
                            .checked(value == current)
                            .on_select(move |_, cx| {
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

impl CreateForm {
    fn source_list(&self, total_label: String, cx: &mut Context<Self>) -> Div {
        let p = crate::theme::palette(cx);
        cardo_ui::settings::frame(cx)
            .p(px(8.))
            .child(
                h_flex()
                    .min_h(px(36.))
                    .py(px(4.))
                    .flex_wrap()
                    .px(px(10.))
                    .gap(px(8.))
                    .items_center()
                    .flex_shrink_0()
                    .bg(rgb(p.surface))
                    .border_b_1()
                    .border_color(rgb(p.border))
                    .child(
                        compact_text("source-heading", tr("source-files"))
                            .flex_1()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(
                        compact_text("source-total", total_label)
                            .max_w(relative(0.4))
                            .text_size(px(11.))
                            .text_color(rgb(p.muted)),
                    )
                    .child(
                        panel_button("add-files", tr("add-files"))
                            .icon(icon("Add", 14.))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.choose_files(window, cx)),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .id("source-list")
                    .min_h(px(112.))
                    .max_h(px(220.))
                    .overflow_y_scrollbar()
                    .when(self.files.is_empty(), |el| {
                        el.h(px(112.))
                            .items_center()
                            .justify_center()
                            .gap(px(6.))
                            .text_color(rgb(p.muted))
                            .child(icon("FolderOpen", 22.))
                            .child(body_text(tr("source-empty")).text_size(px(12.)))
                    })
                    .children(self.files.iter().enumerate().map(|(index, path)| {
                        let name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        let directory = path
                            .parent()
                            .map(|parent| parent.display().to_string())
                            .filter(|parent| !parent.is_empty());
                        let detail = match self.metadata.get(path) {
                            Some(Err(error)) => Some(error.clone()),
                            _ => directory,
                        };
                        let failed = self.metadata.get(path).is_some_and(|value| value.is_err());
                        let size = match self.metadata.get(path) {
                            Some(Ok(meta)) if meta.file => size_text(meta.size),
                            None => tr("source-reading").to_owned(),
                            _ => String::new(),
                        };
                        let folder = self
                            .metadata
                            .get(path)
                            .is_some_and(|value| value.as_ref().is_ok_and(|meta| meta.directory));
                        div()
                            .id(("source-file", index))
                            .h(px(48.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .border_b_1()
                            .border_color(rgb(p.border))
                            .hover(move |el| el.bg(rgb(p.hover)))
                            .child(file_icon(path, folder, true))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        compact_text("source-name", name.clone())
                                            .w_full()
                                            .text_ellipsis_middle()
                                            .text_size(px(13.)),
                                    )
                                    .when_some(detail, |el, detail| {
                                        el.child(
                                            compact_text("source-detail", detail)
                                                .w_full()
                                                .text_size(px(11.))
                                                .text_color(rgb(if failed {
                                                    p.danger
                                                } else {
                                                    p.muted
                                                })),
                                        )
                                    }),
                            )
                            .child(
                                compact_text("source-size", size)
                                    .w(px(72.))
                                    .flex_shrink_0()
                                    .text_right()
                                    .text_size(px(11.))
                                    .text_color(rgb(p.muted)),
                            )
                            .child(
                                icon_button(
                                    ("remove-file", index),
                                    "Dismiss",
                                    &tf("remove-file", &[("name", name.as_str().into())]),
                                    true,
                                    cx,
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        let path = this.files.remove(index);
                                        this.metadata.remove(&path);
                                        cx.notify();
                                    },
                                )),
                            )
                    })),
            )
    }
}

impl CreateForm {
    fn can_submit(&self, cx: &App) -> bool {
        let name = self.name.read(cx).value();
        !self.files.is_empty()
            && self
                .files
                .iter()
                .all(|path| matches!(self.metadata.get(path), Some(Ok(_))))
            && !name.trim().is_empty()
            && !name.contains(['/', '\\', ':'])
            && !(self.encrypted
                && self.format.supports_password()
                && self.password.read(cx).value().is_empty())
            && !(self.format.single_file()
                && (self.files.len() != 1
                    || self
                        .metadata
                        .get(&self.files[0])
                        .is_some_and(|value| value.as_ref().is_ok_and(|meta| meta.directory))))
    }
}

impl Render for CreateForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
        let disabled = !self.can_submit(cx);
        let invalid_name = self.name.read(cx).value().contains(['/', '\\', ':']);
        let encrypted = self.encrypted && self.format.supports_password();
        let input_width = px(cardo_ui::settings::metrics::INPUT_WIDTH);
        let name = v_flex()
            .w(input_width)
            .max_w_full()
            .min_w_0()
            .gap(px(6.))
            .child(
                h_flex()
                    .min_w_0()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(settings_input(&self.name, tr("archive-name"))),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_color(rgb(p.muted))
                            .child(format!(".{}", self.format.extension())),
                    ),
            )
            .when(invalid_name, |el| {
                el.child(body_text(tr("archive-name-invalid")).text_color(rgb(p.danger)))
            });
        let output = settings_group(
            [
                settings_row(tr("archive-name"), name, cx).into_any_element(),
                cardo_ui::settings::row(
                    tr("format"),
                    self.format
                        .single_file()
                        .then(|| tr("single-file-note").into()),
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
                    cx,
                )
                .into_any_element(),
                settings_row(
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
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );
        let mut security = vec![
            settings_row(
                tr("password-protection"),
                SettingsSwitch::new(
                    "encrypted",
                    tr("password-protection"),
                    encrypted,
                    !self.format.supports_password(),
                )
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.encrypted = *checked;
                    cx.notify();
                })),
                cx,
            )
            .into_any_element(),
        ];
        if encrypted {
            security.push(
                settings_row(
                    tr("password"),
                    h_flex()
                        .w(input_width)
                        .max_w_full()
                        .min_w_0()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(settings_input(&self.password, tr("password"))),
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
                                true,
                                cx,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.visible = !this.visible;
                                    this.password.update(cx, |input, cx| {
                                        input.set_masked(!this.visible, window, cx)
                                    });
                                    cx.notify();
                                },
                            )),
                        ),
                    cx,
                )
                .into_any_element(),
            );
            if self.format.supports_header_encryption() {
                security.push(
                    settings_row(
                        tr("encrypt-names"),
                        SettingsSwitch::new(
                            "encrypt-names",
                            tr("encrypt-names"),
                            self.encrypt_names,
                            false,
                        )
                        .on_click(cx.listener(
                            |this, checked: &bool, _, cx| {
                                this.encrypt_names = *checked;
                                cx.notify();
                            },
                        )),
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
        panel_layout(cx)
            .child(
                panel_body("create-body")
                    .child(self.source_list(total_label, cx))
                    .child(output)
                    .child(settings_group(security, cx))
                    .child(
                        panel_button("advanced", tr("advanced-options"))
                            .self_start()
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
                        el.child(settings_group(
                            [
                                settings_row(
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
                                    cx,
                                )
                                .into_any_element(),
                                settings_row(
                                    tr("threads"),
                                    self.choice(
                                        "threads",
                                        self.threads,
                                        Threads::ALL
                                            .into_iter()
                                            .map(|threads| (threads, threads_label(threads)))
                                            .collect(),
                                        false,
                                        |form, threads| form.threads = threads,
                                        cx,
                                    ),
                                    cx,
                                )
                                .into_any_element(),
                                cardo_ui::settings::row(
                                    tr("volume-size"),
                                    (self.volume != Volume::None).then(|| tr("volume-note").into()),
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
                                    cx,
                                )
                                .into_any_element(),
                                settings_row(
                                    tr("solid"),
                                    SettingsSwitch::new(
                                        "solid",
                                        tr("solid"),
                                        self.solid && self.format.supports_solid(),
                                        !self.format.supports_solid(),
                                    )
                                    .on_click(cx.listener(
                                        |this, checked: &bool, _, cx| {
                                            this.solid = *checked;
                                            cx.notify();
                                        },
                                    )),
                                    cx,
                                )
                                .into_any_element(),
                            ],
                            cx,
                        ))
                    }),
            )
            .child(
                panel_actions(cx)
                    .child(
                        panel_button("cancel-create", tr("cancel")).on_click(cx.listener(
                            |this, _, _, cx| {
                                let _ = this.owner.update(cx, |owner, cx| owner.close_modal(cx));
                            },
                        )),
                    )
                    .child(
                        panel_primary("submit-create", tr("create-start"))
                            .icon(icon("Archive", 15.))
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                    ),
            )
    }
}
