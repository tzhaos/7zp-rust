use crate::*;
use cardo_7zp_commands::Action as ExplorerAction;

pub(crate) enum Target {
    Item(String),
    Background,
    Keyboard,
    Recent(PathBuf),
}

#[derive(Clone)]
struct Selection {
    archive: Option<PathBuf>,
    paths: BTreeSet<String>,
}

enum Action {
    Open,
    Create,
    Enter(String),
    Extract,
    ExtractOptions,
    Check,
    CopyPaths,
    SelectAll,
    Invert,
    Properties,
    Sort(usize),
    Descending(bool),
}

impl Selection {
    fn item(
        &self,
        view: &WeakEntity<Workspace>,
        label: impl Into<SharedString>,
        action: Action,
        disabled: bool,
    ) -> PopupMenuItem {
        let view = view.clone();
        let selection = self.clone();
        PopupMenuItem::new(label)
            .disabled(disabled)
            .on_click(move |_, window, cx| {
                let _ = view.update(cx, |this, cx| {
                    // Menu commands refer to the selection at opening, not a later Shell request.
                    if this.tasks.is_busy()
                        || this.dialogs.is_open()
                        || this
                            .browser
                            .view()
                            .catalog
                            .as_ref()
                            .map(|catalog| &catalog.path)
                            != selection.archive.as_ref()
                    {
                        return;
                    }
                    this.browser.restore_selection(selection.paths.clone());
                    match &action {
                        Action::Open => this.open(cx),
                        Action::Create => this.create_dialog(Vec::new(), window, cx),
                        Action::Enter(path) => this.open_entry(path.clone(), window, cx),
                        Action::Extract => this.quick_extract(!selection.paths.is_empty(), cx),
                        Action::ExtractOptions => this.extract_dialog(window, cx),
                        Action::Check => {
                            if let Some(catalog) = this.browser.view().catalog.clone() {
                                this.execute(
                                    Request::Check(catalog),
                                    this.browser.view().password.clone(),
                                    cx,
                                );
                            }
                        }
                        Action::CopyPaths => {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                selection
                                    .paths
                                    .iter()
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            ));
                            this.message = Some(tr("paths-copied").into());
                        }
                        Action::SelectAll => this.browser.select_all(),
                        Action::Invert => this.browser.invert_selection(),
                        Action::Properties => this.properties(!selection.paths.is_empty(), cx),
                        Action::Sort(sort) => {
                            this.browser.set_sort(*sort);
                            this.refresh(cx);
                        }
                        Action::Descending(descending) => {
                            this.browser.set_descending(*descending);
                            this.refresh(cx);
                        }
                    }
                    cx.notify();
                });
            })
    }
}

pub(crate) fn build(
    view: &WeakEntity<Workspace>,
    menu: PopupMenu,
    target: Target,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let mut menu = menu_style(menu).min_w(px(232.));
    if let Target::Recent(path) = target {
        return match view.read_with(cx, |this, _| this.tasks.is_busy()) {
            Ok(busy) => recent(view, menu, path, busy),
            Err(_) => menu,
        };
    }
    let Ok((selection, busy, folder, sort, descending, row_count, all_archives)) =
        view.update(cx, |this, cx| {
            let entry = match &target {
                Target::Item(path) => Some(path.clone()),
                Target::Keyboard if this.browser.view().selected.is_empty() => {
                    this.browser.view().cursor.clone()
                }
                _ => None,
            };
            if let Some(path) = entry
                && !this.browser.view().selected.contains(&path)
            {
                this.browser.select_only(path);
                cx.notify();
            }
            let browser = this.browser.view();
            let folder = browser
                .rows
                .iter()
                .find(|entry| {
                    browser.selected.len() == 1
                        && entry.directory
                        && browser.selected.contains(&entry.path)
                })
                .map(|entry| entry.path.clone());
            (
                Selection {
                    archive: browser.catalog.as_ref().map(|catalog| catalog.path.clone()),
                    paths: browser.selected.clone(),
                },
                this.tasks.is_busy(),
                folder,
                browser.sort,
                browser.descending,
                browser.rows.len(),
                !browser.selected.is_empty()
                    && browser.selected.iter().all(|path| {
                        browser.rows.iter().any(|entry| {
                            &entry.path == path
                                && !entry.directory
                                && cardo_7zp_commands::may_extract(Path::new(path))
                        })
                    }),
            )
        })
    else {
        return menu;
    };

    if let Some(directory) = view
        .read_with(cx, |this, _| {
            this.browser
                .view()
                .directory
                .as_ref()
                .map(|directory| directory.path.clone())
        })
        .ok()
        .flatten()
    {
        let open = view.clone();
        let browse = view.clone();
        let paths = selection.paths.iter().cloned().collect::<Vec<_>>();
        let path = (paths.len() == 1).then(|| paths[0].clone());
        if all_archives && !busy {
            menu = archive_file_menu(view, menu, &directory, &paths, window, cx);
        } else {
            menu = menu.item(
                PopupMenuItem::new(tr("browser-open"))
                    .disabled(busy || path.is_none())
                    .on_click(move |_, window, cx| {
                        if let Some(path) = &path {
                            let _ = open
                                .update(cx, |this, cx| this.open_entry(path.clone(), window, cx));
                        }
                    }),
            );
        }
        if !busy && !paths.is_empty() {
            let sources = paths.iter().map(PathBuf::from).collect::<Vec<_>>();
            let name = cardo_7zp_commands::archive_name(&sources, folder.is_some(), false);
            menu = menu.separator();
            for format in [
                None,
                Some(cardo_7zp_commands::ArchiveFormat::Zip),
                Some(cardo_7zp_commands::ArchiveFormat::SevenZip),
            ] {
                let label = match format {
                    None => tr("shell-add-archive").to_owned(),
                    Some(format) => tf(
                        "shell-compress-named",
                        &[("name", format!("{name}.{}", format.value()).into())],
                    ),
                };
                let owner = view.clone();
                let sources = sources.clone();
                let directory = directory.clone();
                let name = name.clone();
                menu = menu.item(PopupMenuItem::new(label).on_click(move |_, window, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        if this.tasks.is_busy()
                            || this.dialogs.is_open()
                            || this
                                .browser
                                .view()
                                .directory
                                .as_ref()
                                .map(|directory| &directory.path)
                                != Some(&directory)
                        {
                            return;
                        }
                        match format {
                            None => {
                                this.create_dialog(sources.clone(), window, cx);
                                this.dialogs.pending_name = Some(name.clone());
                            }
                            Some(format) => this.execute(
                                Request::QuickCompress {
                                    paths: sources.clone(),
                                    format,
                                    email: false,
                                },
                                String::new(),
                                cx,
                            ),
                        }
                    });
                }));
            }
            menu = menu.separator();
        }
        return menu
            .item(
                PopupMenuItem::new(tr("browser-copy-path"))
                    .disabled(paths.is_empty())
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(paths.join("\n")))
                    }),
            )
            .separator()
            .item(
                PopupMenuItem::new(tr("browser-browse"))
                    .disabled(busy)
                    .on_click(move |_, _, cx| {
                        let _ = browse.update(cx, |this, cx| this.browse(cx));
                    }),
            );
    }
    if selection.archive.is_none() {
        return menu
            .item(selection.item(view, tr("archive-open-command"), Action::Open, busy))
            .item(selection.item(view, tr("archive-create-command"), Action::Create, busy));
    }

    let selected = !selection.paths.is_empty();
    if let Some(folder) = folder {
        menu = menu.item(selection.item(view, tr("folder-open"), Action::Enter(folder), busy));
    } else if selection.paths.len() == 1 {
        menu = menu.item(selection.item(
            view,
            tr("browser-open"),
            Action::Enter(selection.paths.first().unwrap().clone()),
            busy,
        ));
    }
    let extract_label = if selected {
        tf(
            "extract-selection-count",
            &[("count", selection.paths.len().into())],
        )
    } else {
        tr("extract-all-quick").into()
    };
    menu = menu
        .item(selection.item(view, extract_label, Action::Extract, busy))
        .item(selection.item(view, tr("extract-options"), Action::ExtractOptions, busy));
    if selected {
        menu =
            menu.separator()
                .item(selection.item(view, tr("copy-paths"), Action::CopyPaths, busy));
    } else {
        menu = menu.item(selection.item(view, tr("archive-check"), Action::Check, busy));
        let sorting = view.clone();
        let scope = selection.clone();
        menu = menu
            .separator()
            .submenu(tr("sort-by"), window, cx, move |mut menu, _, _| {
                menu = submenu_style(menu);
                for (value, label) in [(0, tr("name")), (1, tr("size")), (2, tr("modified"))] {
                    menu = menu.item(
                        scope
                            .item(&sorting, label, Action::Sort(value), busy)
                            .checked(sort == value),
                    );
                }
                menu = menu.separator();
                for (value, label) in [(false, tr("sort-ascending")), (true, tr("sort-descending"))]
                {
                    menu = menu.item(
                        scope
                            .item(&sorting, label, Action::Descending(value), busy)
                            .checked(descending == value),
                    );
                }
                menu.min_w(px(200.))
            });
    }
    menu = menu.separator().item(selection.item(
        view,
        tr("select-all"),
        Action::SelectAll,
        busy || row_count == 0 || selection.paths.len() == row_count,
    ));
    if selected {
        menu = menu.item(selection.item(view, tr("select-invert"), Action::Invert, busy));
    }
    menu = menus::archive::entry_menu(view, menu.separator(), true, cx);
    menu.separator().item(selection.item(
        view,
        if selected {
            tr("properties")
        } else {
            tr("archive-info")
        },
        Action::Properties,
        busy,
    ))
}

fn archive_file_item(
    owner: &WeakEntity<Workspace>,
    directory: &Path,
    paths: &[String],
    action: ExplorerAction,
    label: impl Into<SharedString>,
) -> PopupMenuItem {
    let owner = owner.clone();
    let directory = directory.to_path_buf();
    let mut args = vec![action.argument().to_owned()];
    args.extend_from_slice(paths);
    PopupMenuItem::new(label).on_click(move |_, _, cx| {
        let _ = owner.update(cx, |this, cx| {
            if this.tasks.is_busy()
                || this.dialogs.is_open()
                || this
                    .browser
                    .view()
                    .directory
                    .as_ref()
                    .map(|directory| &directory.path)
                    != Some(&directory)
            {
                return;
            }
            this.launches.push_back(args.clone());
            cx.notify();
        });
    })
}

fn archive_file_menu(
    owner: &WeakEntity<Workspace>,
    mut menu: PopupMenu,
    directory: &Path,
    paths: &[String],
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    if paths.len() == 1 {
        menu = menu.item(archive_file_item(
            owner,
            directory,
            paths,
            ExplorerAction::Open,
            tr("archive-open"),
        ));
        let owner = owner.clone();
        let directory = directory.to_path_buf();
        let paths = paths.to_vec();
        menu = menu
            .submenu(tr("archive-open-type"), window, cx, move |menu, _, _| {
                let mut menu = submenu_style(menu);
                for action in cardo_7zp_commands::ACTIONS.iter().copied() {
                    if let ExplorerAction::OpenAs(kind) = action {
                        menu = menu.item(archive_file_item(
                            &owner,
                            &directory,
                            &paths,
                            action,
                            kind.value(),
                        ));
                    }
                }
                menu
            })
            .separator();
    }
    let folder_label = if paths.len() == 1 {
        tf(
            "shell-extract-named",
            &[(
                "name",
                cardo_7zp_commands::extract_folder(Path::new(&paths[0])).into(),
            )],
        )
    } else {
        tr("extract-each-folder").to_owned()
    };
    for (action, label) in [
        (ExplorerAction::Extract, tr("shell-extract-files").to_owned()),
        (ExplorerAction::ExtractHere, tr("extract-here").to_owned()),
        (ExplorerAction::ExtractFolder, folder_label),
        (ExplorerAction::Check, tr("shell-test-archive").to_owned()),
    ] {
        menu = menu.item(archive_file_item(owner, directory, paths, action, label));
    }
    menu
}

impl Workspace {
    pub(crate) fn open_context_menu(
        &mut self,
        target: Target,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        let previous_focus = window.focused(cx);
        self.context_menu = None;
        // Defer because building the menu reads and updates the workspace selection.
        window.defer(cx, move |window, cx| {
            let owner = view.clone();
            let popup = PopupMenu::build(window, cx, move |popup, window, cx| {
                build(&owner, popup, target, window, cx)
            });
            let dismissed = view.clone();
            let subscription =
                window.subscribe(&popup, cx, move |_, _: &DismissEvent, window, cx| {
                    let restore_focus = dismissed.update(cx, |this, cx| {
                        this.context_menu = None;
                        cx.notify();
                        !this.dialogs.is_open()
                    });
                    if matches!(restore_focus, Ok(true))
                        && let Some(focus) = &previous_focus
                    {
                        focus.focus(window, cx);
                    }
                });
            popup.focus_handle(cx).focus(window, cx);
            let _ = view.update(cx, |this, cx| {
                this.menu_dismiss = Some(subscription);
                this.context_menu_position = position;
                this.context_menu = Some(popup);
                cx.notify();
            });
        });
    }
}

pub(crate) fn recent(
    view: &WeakEntity<Workspace>,
    menu: PopupMenu,
    path: PathBuf,
    busy: bool,
) -> PopupMenu {
    let open = view.clone();
    let remove = view.clone();
    let open_path = path.clone();
    let copy_path = path.clone();
    menu_style(menu)
        .min_w(px(232.))
        .item(
            PopupMenuItem::new(tr("archive-open"))
                .disabled(busy)
                .on_click(move |_, window, cx| {
                    let _ = open.update(cx, |this, cx| {
                        this.show_page(Page::Files, window, cx);
                        this.execute(Request::Open(open_path.clone()), String::new(), cx)
                    });
                }),
        )
        .item(
            PopupMenuItem::new(tr("recent-copy-path")).on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_path.display().to_string()))
            }),
        )
        .separator()
        .item(
            PopupMenuItem::new(tr("recent-remove"))
                .disabled(busy)
                .on_click(move |_, _, cx| {
                    let _ = remove.update(cx, |this, cx| {
                        this.update_recent(recent::Change::Remove(path.clone()), cx)
                    });
                }),
        )
}
