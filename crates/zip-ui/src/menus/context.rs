use crate::commands::Command;
use crate::*;
use zip_commands::Action as ExplorerAction;

pub(crate) enum Target {
    Item(String),
    Background,
    Keyboard,
    Recent(PathBuf),
}

#[derive(Clone)]
struct Selection {
    location: browser::Location,
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
    Browse,
    Compress {
        sources: Vec<PathBuf>,
        directory: PathBuf,
        name: String,
        format: Option<zip_commands::ArchiveFormat>,
    },
}

impl Selection {
    fn item(
        &self,
        view: &WeakEntity<Workspace>,
        label: impl Into<SharedString>,
        action: Action,
        disabled: bool,
    ) -> Option<MenuItem> {
        if disabled {
            return None;
        }
        let view = view.clone();
        let selection = self.clone();
        Some(MenuItem::new(label).on_select(move |window, cx| {
            let _ = view.update(cx, |this, cx| {
                // Menu commands refer to the selection at opening, not a later Shell request.
                if this.tasks.is_busy()
                    || this.dialogs.is_open()
                    || this.settings_busy(cx)
                    || this.browser.location() != selection.location
                {
                    return;
                }
                this.browser.restore_selection(selection.paths.clone());
                match &action {
                    Action::Browse => this.browse(cx),
                    Action::Compress {
                        sources,
                        directory,
                        name,
                        format,
                    } => {
                        this.compress_from_context(
                            sources.clone(),
                            directory.clone(),
                            name.clone(),
                            *format,
                            window,
                            cx,
                        );
                    }
                    Action::Open => this.command(Command::Open, window, cx),
                    Action::Create => this.command(Command::Create, window, cx),
                    Action::Enter(path) => this.open_entry(path.clone(), window, cx),
                    Action::Extract => this.command(Command::QuickExtractSelection, window, cx),
                    Action::ExtractOptions => this.command(Command::Extract, window, cx),
                    Action::Check => this.command(Command::Check, window, cx),
                    Action::CopyPaths => {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            selection
                                .paths
                                .iter()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ));
                        this.notify_message(tr("paths-copied").into());
                    }
                    Action::SelectAll => this.command(Command::SelectAll, window, cx),
                    Action::Invert => this.command(Command::InvertSelection, window, cx),
                    Action::Properties => this.command(Command::Properties, window, cx),
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
        }))
    }
}

pub(crate) fn build(
    view: &WeakEntity<Workspace>,
    menu: Menu,
    target: Target,
    cx: &mut App,
) -> Menu {
    let mut menu = menu;
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
                    location: this.browser.location(),
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
                                && zip_commands::may_extract(Path::new(path))
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
        let paths = selection.paths.iter().cloned().collect::<Vec<_>>();
        let path = (paths.len() == 1).then(|| paths[0].clone());
        if all_archives && !busy {
            menu = archive_file_menu(view, menu, &directory, &paths);
        } else if let Some(path) = path {
            menu = menu.item(selection.item(view, tr("browser-open"), Action::Enter(path), busy));
        }
        if !busy && !paths.is_empty() {
            let sources = paths.iter().map(PathBuf::from).collect::<Vec<_>>();
            let name = zip_commands::archive_name(&sources, folder.is_some(), false);
            menu = menu.separator();
            for format in [
                None,
                Some(zip_commands::ArchiveFormat::Zip),
                Some(zip_commands::ArchiveFormat::SevenZip),
            ] {
                let label = match format {
                    None => tr("shell-add-archive").to_owned(),
                    Some(format) => tf(
                        "shell-compress-named",
                        &[("name", format!("{name}.{}", format.value()).into())],
                    ),
                };
                let sources = sources.clone();
                let directory = directory.clone();
                let name = name.clone();
                menu = menu.item(selection.item(
                    view,
                    label,
                    Action::Compress {
                        sources,
                        directory,
                        name,
                        format,
                    },
                    busy,
                ));
            }
            menu = menu.separator();
        }
        return menu
            .item(selection.item(
                view,
                tr("browser-copy-path"),
                Action::CopyPaths,
                paths.is_empty(),
            ))
            .separator()
            .item(selection.item(view, tr("browser-browse"), Action::Browse, busy));
    }
    if !matches!(selection.location, browser::Location::Archive(..)) {
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
        menu = menu.separator().submenu(tr("sort-by"), {
            let mut menu = Menu::new();
            for (value, label) in [
                (0, tr("name")),
                (1, tr("size")),
                (3, tr("type")),
                (2, tr("modified")),
            ] {
                menu = menu.item(
                    scope
                        .item(&sorting, label, Action::Sort(value), busy)
                        .map(|item| item.checked(sort == value)),
                );
            }
            menu = menu.separator();
            for (value, label) in [(false, tr("sort-ascending")), (true, tr("sort-descending"))] {
                menu = menu.item(
                    scope
                        .item(&sorting, label, Action::Descending(value), busy)
                        .map(|item| item.checked(descending == value)),
                );
            }
            menu
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
) -> MenuItem {
    let owner = owner.clone();
    let directory = directory.to_path_buf();
    let mut args = vec![action.argument().to_owned()];
    args.extend_from_slice(paths);
    MenuItem::new(label).on_select(move |_, cx| {
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
    mut menu: Menu,
    directory: &Path,
    paths: &[String],
) -> Menu {
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
            .submenu(tr("archive-open-type"), {
                let mut menu = Menu::new();
                for action in zip_commands::ACTIONS.iter().copied() {
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
                zip_commands::extract_folder(Path::new(&paths[0])).into(),
            )],
        )
    } else {
        tr("extract-each-folder").to_owned()
    };
    for (action, label) in [
        (
            ExplorerAction::Extract,
            tr("shell-extract-files").to_owned(),
        ),
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
        if self.tasks.is_busy() || self.dialogs.is_open() || self.settings_busy(cx) {
            return;
        }
        let view = cx.entity().downgrade();
        // Release the workspace borrow before resolving the clicked selection.
        window.defer(cx, move |window, cx| {
            build(&view, Menu::new(), target, cx).show(position, window, cx);
        });
    }
}

pub(crate) fn recent(view: &WeakEntity<Workspace>, menu: Menu, path: PathBuf, busy: bool) -> Menu {
    let open = view.clone();
    let remove = view.clone();
    let open_path = path.clone();
    let copy_path = path.clone();
    menu.item(
        MenuItem::new(tr("archive-open"))
            .disabled(busy)
            .on_select(move |window, cx| {
                let _ = open.update(cx, |this, cx| {
                    this.open_archive_path(open_path.clone(), window, cx)
                });
            }),
    )
    .item(
        MenuItem::new(tr("recent-copy-path")).on_select(move |_, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(copy_path.display().to_string()))
        }),
    )
    .separator()
    .item(
        MenuItem::new(tr("recent-remove"))
            .disabled(busy)
            .on_select(move |_, cx| {
                let _ = remove.update(cx, |this, cx| {
                    this.update_recent(recent::Change::Remove(path.clone()), cx)
                });
            }),
    )
}
