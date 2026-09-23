use crate::filesystem::Directory;
use cardo_7zp_engine::{Catalog, Entry};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, PartialEq, Eq)]
pub enum Location {
    Home,
    Directory(PathBuf),
    Archive(PathBuf, String),
}

#[derive(Default)]
pub struct BrowserSnapshot {
    pub catalog: Option<Catalog>,
    pub directory: Option<Directory>,
    pub password: String,
    pub folder: String,
    pub history: Vec<Location>,
    pub selected: BTreeSet<String>,
    pub anchor: Option<String>,
    pub cursor: Option<String>,
    pub rows: Vec<Entry>,
    pub sort: usize,
    pub descending: bool,
}

#[derive(Default)]
pub struct Browser {
    view: BrowserSnapshot,
    returning: bool,
    pending_folder: Option<String>,
}

impl Browser {
    pub fn view(&self) -> &BrowserSnapshot {
        &self.view
    }

    pub fn location(&self) -> Location {
        if let Some(catalog) = &self.view.catalog {
            Location::Archive(catalog.path.clone(), self.view.folder.clone())
        } else if let Some(directory) = &self.view.directory {
            Location::Directory(directory.path.clone())
        } else {
            Location::Home
        }
    }

    pub fn remember_navigation(&mut self, target: &Location) {
        if self.returning {
            self.view.history.pop();
            self.returning = false;
        } else {
            let current = self.location();
            if current != *target {
                self.view.history.push(current);
            }
        }
    }

    pub fn begin_back(&mut self) -> Option<Location> {
        let target = self.view.history.last()?.clone();
        self.returning = true;
        Some(target)
    }

    pub fn request_folder(&mut self, folder: String) {
        self.pending_folder = Some(folder);
    }

    pub fn take_requested_folder(&mut self) -> String {
        self.pending_folder.take().unwrap_or_default()
    }

    pub fn cancel_navigation(&mut self) {
        self.returning = false;
        self.pending_folder = None;
    }

    pub fn show_home(&mut self) {
        self.view.catalog = None;
        self.view.directory = None;
        self.view.password.clear();
        self.enter_folder(String::new());
    }

    pub fn show_directory(&mut self, directory: Directory) {
        self.view.directory = Some(directory);
        self.view.catalog = None;
        self.view.password.clear();
        self.enter_folder(String::new());
    }

    pub fn show_archive(&mut self, catalog: Catalog, password: String, folder: String) {
        self.view.catalog = Some(catalog);
        self.view.directory = None;
        self.view.password = password;
        self.enter_folder(folder);
    }

    pub fn enter_folder(&mut self, folder: String) {
        self.view.folder = folder;
        self.clear_selection(true);
        self.view.cursor = None;
    }

    pub fn set_password(&mut self, password: String) {
        self.view.password = password;
    }

    pub fn clear_selection(&mut self, reset_anchor: bool) {
        self.view.selected.clear();
        if reset_anchor {
            self.view.anchor = None;
        }
    }

    pub fn restore_selection(&mut self, paths: BTreeSet<String>) {
        self.view.selected = paths;
    }

    pub fn select_only(&mut self, path: String) {
        self.view.selected = BTreeSet::from([path.clone()]);
        self.view.anchor = Some(path.clone());
        self.view.cursor = Some(path);
    }

    pub fn toggle_selection(&mut self, path: String, set_anchor: bool) {
        if !self.view.selected.insert(path.clone()) {
            self.view.selected.remove(&path);
        }
        if set_anchor {
            self.view.anchor = Some(path);
        }
    }

    pub fn select(&mut self, path: String, extend: bool, toggle: bool) {
        if extend
            && let Some(anchor) = &self.view.anchor
            && let (Some(start), Some(end)) = (
                self.view.rows.iter().position(|e| &e.path == anchor),
                self.view.rows.iter().position(|e| e.path == path),
            )
        {
            self.view.selected = self.view.rows[start.min(end)..=start.max(end)]
                .iter()
                .map(|e| e.path.clone())
                .collect();
        } else {
            if !toggle {
                self.view.selected.clear();
            }
            self.toggle_selection(path.clone(), true);
        }
        self.view.cursor = Some(path);
    }

    pub fn select_all(&mut self) {
        self.view.selected = self
            .view
            .rows
            .iter()
            .map(|entry| entry.path.clone())
            .collect();
    }

    pub fn deselect_all(&mut self) {
        self.clear_selection(true);
    }

    pub fn invert_selection(&mut self) {
        self.view.selected = self
            .view
            .rows
            .iter()
            .filter(|entry| !self.view.selected.contains(&entry.path))
            .map(|entry| entry.path.clone())
            .collect();
    }

    pub fn set_sort(&mut self, column: usize) {
        self.view.sort = column;
    }

    pub fn set_descending(&mut self, descending: bool) {
        self.view.descending = descending;
    }

    pub fn refresh(&mut self, query: &str) {
        let view = &mut self.view;
        view.rows = view
            .catalog
            .as_ref()
            .map(|catalog| catalog.children(&view.folder, query))
            .unwrap_or_else(|| {
                let query = query.to_lowercase();
                view.directory
                    .as_ref()
                    .map(|directory| {
                        directory
                            .entries
                            .iter()
                            .filter(|entry| entry.name.to_lowercase().contains(&query))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default()
            });
        view.rows.sort_by(|a, b| {
            let order = match view.sort {
                1 => a.size.cmp(&b.size),
                2 => a.modified.cmp(&b.modified),
                3 => name_extension(&a.name).cmp(&name_extension(&b.name)),
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
            .then_with(|| a.path.cmp(&b.path));
            b.directory.cmp(&a.directory).then(if view.descending {
                order.reverse()
            } else {
                order
            })
        });
    }
}

fn name_extension(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            extension.to_ascii_lowercase()
        }
        _ => String::new(),
    }
}
