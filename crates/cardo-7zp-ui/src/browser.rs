use super::*;
pub(super) use cardo_7zp_application::filesystem::Directory;

pub(super) use cardo_7zp_application::browser::Location;

impl Workspace {
    pub(super) fn location(&self) -> Location {
        self.browser.location()
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
        self.browser.remember_navigation(target);
        self.address_dirty = true;
    }

    pub(super) fn visit(
        &mut self,
        location: Location,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.tasks.is_busy() || self.settings_busy(cx) {
            return;
        }
        self.show_page(Page::Files, window, cx);
        self.focus.focus(window, cx);
        match location {
            Location::Home => {
                self.reload_history(cx);
                self.remember_navigation(&Location::Home);
                self.browser.show_home();
                self.completion = None;
                self.go(String::new(), window, cx);
            }
            Location::Directory(path) => {
                self.start(tr("browser-loading"), cx, move |_| {
                    Directory::read(path).map(Outcome::Directory)
                });
            }
            Location::Archive(path, folder) => {
                if self
                    .browser
                    .view()
                    .catalog
                    .as_ref()
                    .is_some_and(|c| c.path == path)
                {
                    self.remember_navigation(&Location::Archive(path, folder.clone()));
                    self.go(folder, window, cx);
                } else {
                    self.browser.request_folder(folder);
                    self.execute(Request::Open(path), String::new(), cx);
                }
            }
        }
    }

    pub(super) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tasks.is_busy()
            && let Some(location) = self.browser.begin_back()
        {
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
        if value.is_empty() || self.tasks.is_busy() {
            return;
        }
        self.focus.focus(window, cx);
        let path = PathBuf::from(value);
        self.execute(Request::ResolveAddress(path), String::new(), cx);
    }

    pub(super) fn activate_directory(&mut self, directory: Directory, cx: &mut Context<Self>) {
        self.remember_navigation(&Location::Directory(directory.path.clone()));
        self.update_history(
            recent::Change::Remember(directory.path.clone()),
            recent::Kind::Folders,
            cx,
        );
        self.browser.show_directory(directory);
        self.clear_search = true;
        self.refresh(cx);
    }

    pub(super) fn open_entry(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_busy() {
            return;
        }
        let Some(entry) = self.browser.view().rows.iter().find(|e| e.path == path) else {
            return;
        };
        if self.browser.view().directory.is_some() {
            if entry.directory {
                self.visit(Location::Directory(PathBuf::from(path)), window, cx);
            } else if cardo_7zp_shell_api::may_extract(std::path::Path::new(&path)) {
                self.execute(Request::Open(PathBuf::from(path)), String::new(), cx);
            } else {
                self.start(tr("opening"), cx, move |_| {
                    cardo_7zp_platform::open_file(std::path::Path::new(&path))?;
                    Ok(Outcome::Cancelled)
                });
            }
        } else if entry.directory {
            self.navigate(path, window, cx);
        } else if let Some(catalog) = self.browser.view().catalog.clone() {
            self.execute(
                Request::OpenEntry {
                    catalog,
                    path,
                    preferences: self.preferences.clone(),
                },
                self.browser.view().password.clone(),
                cx,
            );
        }
    }
}

impl Workspace {
    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.browser.refresh(&self.search.read(cx).value());
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    pub(crate) fn navigate(&mut self, folder: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_busy() {
            return;
        }
        if let Some(catalog) = &self.browser.view().catalog {
            self.remember_navigation(&browser::Location::Archive(
                catalog.path.clone(),
                folder.clone(),
            ));
        }
        self.go(folder, window, cx);
    }

    pub(crate) fn go(&mut self, folder: String, window: &mut Window, cx: &mut Context<Self>) {
        self.address_dirty = true;
        self.browser.enter_folder(folder);
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.refresh(cx);
        self.focus.focus(window, cx);
    }

    pub(crate) fn select(&mut self, path: String, modifiers: Modifiers, cx: &mut Context<Self>) {
        self.browser
            .select(path, modifiers.shift, modifiers.control);
        cx.notify();
    }
}
