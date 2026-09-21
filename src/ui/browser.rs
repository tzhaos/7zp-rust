use super::*;
pub(super) use crate::application::filesystem::Directory;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Location {
    Home,
    Directory(PathBuf),
    Archive(PathBuf, String),
}

impl Workspace {
    pub(super) fn location(&self) -> Location {
        if let Some(catalog) = &self.catalog {
            Location::Archive(catalog.path.clone(), self.folder.clone())
        } else if let Some(directory) = &self.directory {
            Location::Directory(directory.path.clone())
        } else {
            Location::Home
        }
    }

    pub(super) fn address_text(&self) -> String {
        match self.location() {
            Location::Home => String::new(),
            Location::Directory(path) => path.display().to_string(),
            Location::Archive(path, folder) => if folder.is_empty() {
                path
            } else {
                path.join(folder)
            }
            .display()
            .to_string(),
        }
    }

    pub(super) fn remember_navigation(&mut self, target: &Location) {
        if self.returning {
            self.history.pop();
            self.returning = false;
        } else {
            let current = self.location();
            if current != *target {
                self.history.push(current);
            }
        }
        self.address_dirty = true;
    }

    pub(super) fn visit(
        &mut self,
        location: Location,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.settings_busy(cx) {
            return;
        }
        self.show_page(Page::Files, window, cx);
        self.focus.focus(window, cx);
        match location {
            Location::Home => {
                self.reload_history(cx);
                self.remember_navigation(&Location::Home);
                self.catalog = None;
                self.directory = None;
                self.password.clear();
                self.completion = None;
                self.go(String::new(), window, cx);
            }
            Location::Directory(path) => {
                self.start(tr("browser-loading"), cx, move |_| {
                    Directory::read(path).map(Outcome::Directory)
                });
            }
            Location::Archive(path, folder) => {
                if self.catalog.as_ref().is_some_and(|c| c.path == path) {
                    self.remember_navigation(&Location::Archive(path, folder.clone()));
                    self.go(folder, window, cx);
                } else {
                    self.pending_folder = Some(folder);
                    self.execute(Request::Open(path), String::new(), cx);
                }
            }
        }
    }

    pub(super) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.busy
            && let Some(location) = self.history.last().cloned()
        {
            self.returning = true;
            self.visit(location, window, cx);
        }
    }

    pub(super) fn parent_location(&self) -> Option<Location> {
        match self.location() {
            Location::Home => None,
            Location::Directory(path) => path.parent().map(|p| Location::Directory(p.to_owned())),
            Location::Archive(path, folder) if folder.is_empty() => {
                path.parent().map(|p| Location::Directory(p.to_owned()))
            }
            Location::Archive(path, folder) => Some(Location::Archive(
                path,
                folder
                    .rsplit_once('/')
                    .map(|(p, _)| p.to_owned())
                    .unwrap_or_default(),
            )),
        }
    }

    pub(super) fn up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(location) = self.parent_location() {
            self.visit(location, window, cx);
        }
    }

    pub(super) fn browse(&mut self, cx: &mut Context<Self>) {
        let current = match self.location() {
            Location::Directory(path) => Some(path),
            Location::Archive(path, _) => path.parent().map(|p| p.to_owned()),
            Location::Home => None,
        };
        self.start(tr("browser-loading"), cx, move |_| {
            let mut picker = rfd::FileDialog::new().set_title(tr("browser-browse"));
            if let Some(path) = current {
                picker = picker.set_directory(path);
            }
            match picker.pick_folder() {
                Some(path) => Directory::read(path).map(Outcome::Directory),
                None => Ok(Outcome::Cancelled),
            }
        });
    }

    pub(super) fn open_address(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .address
            .read(cx)
            .value()
            .trim()
            .trim_matches('"')
            .to_owned();
        if value.is_empty() || self.busy {
            return;
        }
        self.focus.focus(window, cx);
        let path = PathBuf::from(value);
        self.start(tr("browser-loading"), cx, move |cancel| {
            let path = std::path::absolute(path)?;
            // A virtual archive path resolves at the first existing filesystem ancestor.
            let mut archive = path.clone();
            loop {
                match std::fs::metadata(&archive) {
                    Ok(metadata) if metadata.is_dir() && archive == path => {
                        return Directory::read(path).map(Outcome::Directory);
                    }
                    Ok(metadata) if metadata.is_file() => break,
                    Ok(_) => anyhow::bail!(tr("browser-path-missing")),
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                        ) =>
                    {
                        if !archive.pop() {
                            return Err(error.into());
                        }
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            let folder = path
                .strip_prefix(&archive)?
                .to_string_lossy()
                .replace('\\', "/");
            match Engine::bundled()?.list(&archive, "", &cancel) {
                Ok(catalog) => Ok(Outcome::Address(catalog, folder)),
                Err(error) if crate::archive::needs_password(&error) => {
                    Ok(Outcome::AddressPassword(Request::Open(archive), folder))
                }
                Err(error) => Err(error),
            }
        });
    }

    pub(super) fn activate_directory(&mut self, directory: Directory, cx: &mut Context<Self>) {
        self.remember_navigation(&Location::Directory(directory.path.clone()));
        self.update_history(
            recent::Change::Remember(directory.path.clone()),
            recent::Kind::Folders,
            cx,
        );
        self.directory = Some(directory);
        self.catalog = None;
        self.password.clear();
        self.folder.clear();
        self.selected.clear();
        self.anchor = None;
        self.cursor = None;
        self.clear_search = true;
        self.refresh(cx);
    }

    pub(super) fn open_entry(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(entry) = self.rows.iter().find(|e| e.path == path) else {
            return;
        };
        if self.directory.is_some() {
            if entry.directory {
                self.visit(Location::Directory(PathBuf::from(path)), window, cx);
            } else if sevenzip_shell_api::may_extract(std::path::Path::new(&path)) {
                self.execute(Request::Open(PathBuf::from(path)), String::new(), cx);
            } else {
                self.start(tr("opening"), cx, move |_| {
                    crate::platform::open_file(std::path::Path::new(&path))?;
                    Ok(Outcome::Cancelled)
                });
            }
        } else if entry.directory {
            self.navigate(path, window, cx);
        } else if let Some(catalog) = self.catalog.clone() {
            self.execute(
                Request::OpenEntry {
                    catalog,
                    path,
                    preferences: self.preferences.clone(),
                },
                self.password.clone(),
                cx,
            );
        }
    }
}

impl Workspace {
    pub(in crate::ui) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.rows = self
            .catalog
            .as_ref()
            .map(|c| c.children(&self.folder, &self.search.read(cx).value()))
            .unwrap_or_else(|| {
                let query = self.search.read(cx).value().to_lowercase();
                self.directory
                    .as_ref()
                    .map(|d| {
                        d.entries
                            .iter()
                            .filter(|entry| entry.name.to_lowercase().contains(&query))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default()
            });
        self.rows.sort_by(|a, b| {
            let order = match self.sort {
                1 => a.size.cmp(&b.size),
                2 => a.modified.cmp(&b.modified),
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
            .then_with(|| a.path.cmp(&b.path));
            b.directory.cmp(&a.directory).then(if self.descending {
                order.reverse()
            } else {
                order
            })
        });
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    pub(in crate::ui) fn navigate(
        &mut self,
        folder: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        if let Some(catalog) = &self.catalog {
            self.remember_navigation(&browser::Location::Archive(
                catalog.path.clone(),
                folder.clone(),
            ));
        }
        self.go(folder, window, cx);
    }

    pub(in crate::ui) fn go(
        &mut self,
        folder: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.address_dirty = true;
        self.folder = folder;
        self.selected.clear();
        self.anchor = None;
        self.cursor = None;
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.refresh(cx);
        self.focus.focus(window, cx);
    }

    pub(in crate::ui) fn select(
        &mut self,
        path: String,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        if modifiers.shift
            && let Some(anchor) = &self.anchor
            && let (Some(start), Some(end)) = (
                self.rows.iter().position(|e| &e.path == anchor),
                self.rows.iter().position(|e| e.path == path),
            )
        {
            self.selected = self.rows[start.min(end)..=start.max(end)]
                .iter()
                .map(|e| e.path.clone())
                .collect();
        } else {
            if !modifiers.control {
                self.selected.clear();
            }
            if !self.selected.insert(path.clone()) {
                self.selected.remove(&path);
            }
            self.anchor = Some(path.clone());
        }
        self.cursor = Some(path);
        cx.notify();
    }
}
