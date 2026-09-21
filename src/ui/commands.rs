use super::*;

#[derive(Clone, Copy)]
pub(super) enum Command {
    Open,
    Create,
    Save,
    Close,
    ArchiveInfo,
    ClearRecent,
    Files,
    Exit,
    Extract,
    QuickExtract,
    Add,
    Properties,
    Comment,
    Check,
    Settings(preferences::Tab),
    Updates,
    Help,
    About,
}

impl Workspace {
    pub(super) fn command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.settings_busy(cx) {
            return;
        }
        if matches!(command, Command::Open | Command::Create) {
            self.show_page(Page::Files, window, cx);
        }
        match command {
            Command::Open => self.open(cx),
            Command::Create => self.create_dialog(Vec::new(), window, cx),
            Command::Save => self.save_copy(cx),
            Command::Close => {
                if let Some(parent) = self
                    .catalog
                    .as_ref()
                    .and_then(|c| c.path.parent())
                    .map(Path::to_owned)
                {
                    self.visit(browser::Location::Directory(parent), window, cx);
                }
            }
            Command::ArchiveInfo => self.properties(false, cx),
            Command::ClearRecent => self.update_recent(recent::Change::Clear, cx),
            Command::Files => self.show_page(Page::Files, window, cx),
            Command::Exit => window.remove_window(),
            Command::Extract => self.extract_dialog(window, cx),
            Command::QuickExtract => self.quick_extract(false, cx),
            Command::Add => {
                if self.catalog.as_ref().is_some_and(|c| c.editable()) {
                    self.add_to_archive(false, cx);
                }
            }
            Command::Properties => self.properties(!self.selected.is_empty(), cx),
            Command::Comment => {
                if let Some(catalog) = self
                    .catalog
                    .clone()
                    .filter(|c| c.format == "ZIP" && c.editable())
                {
                    self.start(tr("archive-comment"), cx, move |_| {
                        Engine::comment(&catalog).map(Outcome::Comment)
                    });
                }
            }
            Command::Check => {
                if let Some(catalog) = self.catalog.clone() {
                    self.execute(Request::Check(catalog), self.password.clone(), cx);
                }
            }
            Command::Settings(tab) => self.preferences_category(tab, window, cx),
            Command::Updates => self.check_update(cx),
            Command::Help => self.start(tr("menu-help"), cx, |_| {
                let path = std::env::current_exe()?
                    .parent()
                    .unwrap()
                    .join("runtime/7zip/7-zip.chm");
                crate::platform::open_file(&path)?;
                Ok(Outcome::Cancelled)
            }),
            Command::About => {
                self.modal_title = tr("menu-about").into();
                self.modal = Some(Modal::Info(vec![
                    (tr("name").into(), "7zplus".into()),
                    (
                        tr("about-version").into(),
                        crate::application::update::VERSION.into(),
                    ),
                    (tr("about-engine").into(), "7-Zip 26.03".into()),
                    (tr("about-ui").into(), "GPUI Kit / Microsoft Fluent".into()),
                ]));
                cx.notify();
            }
        }
    }
}

impl Workspace {
    pub(in crate::ui) fn keyboard(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.context_menu.is_some() {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if key == "escape" {
            if self.busy {
                return;
            }
            if self.address.read(cx).focus_handle(cx).is_focused(window) {
                self.address_dirty = true;
                self.focus.focus(window, cx);
                cx.notify();
            } else if self.modal.is_some() {
                self.close_modal(cx);
            } else if self.page != Page::Files {
                self.show_page(Page::Files, window, cx);
            } else if !self.search.read(cx).value().is_empty() {
                self.search
                    .update(cx, |input, cx| input.set_value("", window, cx));
            } else {
                self.selected.clear();
                cx.notify();
            }
            return;
        }
        if self.modal.is_some() {
            return;
        }
        if self.page != Page::Files {
            if modifiers.control && key == "s" {
                if let Some(form) = self.settings_form.clone() {
                    window.defer(cx, move |window, cx| {
                        form.update(cx, |form, cx| form.save(window, cx));
                    });
                }
                cx.stop_propagation();
            }
            return;
        }
        let command = match (modifiers.control, modifiers.alt, modifiers.shift, key) {
            (true, false, false, "o") => Some(commands::Command::Open),
            (true, false, true, "s") => Some(commands::Command::Save),
            (false, true, _, "e") => Some(commands::Command::Extract),
            (false, true, _, "a") => Some(commands::Command::Add),
            (false, true, _, "i") => Some(commands::Command::Properties),
            (false, true, _, "m") => Some(commands::Command::Comment),
            (false, true, _, "t") => Some(commands::Command::Check),
            (false, false, _, "f1") => Some(commands::Command::Help),
            _ => None,
        };
        if let Some(command) = command {
            self.command(command, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && key == "l" && !self.busy {
            self.address.update(cx, |input, cx| {
                input.focus(window, cx);
                input.select_all(window, cx);
            });
            cx.stop_propagation();
            return;
        }
        if modifiers.control && key == "f" {
            self.search.update(cx, |input, cx| input.focus(window, cx));
            cx.stop_propagation();
            return;
        }
        if !self.focus.is_focused(window) {
            return;
        }
        if self.busy {
            return;
        }
        if modifiers.alt && key == "left" {
            self.back(window, cx);
            cx.stop_propagation();
            return;
        }
        if key == "backspace" || modifiers.alt && key == "up" {
            self.up(window, cx);
            cx.stop_propagation();
            return;
        }
        if key == "f5"
            && let Some(directory) = &self.directory
        {
            self.visit(
                browser::Location::Directory(directory.path.clone()),
                window,
                cx,
            );
            cx.stop_propagation();
            return;
        }
        if key == "contextmenu" || (modifiers.shift && key == "f10") {
            self.open_context_menu(menu::Target::Keyboard, point(px(32.), px(196.)), window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && key == "a" {
            self.selected = self.rows.iter().map(|e| e.path.clone()).collect();
            cx.notify();
        }
        if ["up", "down", "home", "end"].contains(&key) && !self.rows.is_empty() {
            let current = self
                .cursor
                .as_ref()
                .and_then(|p| self.rows.iter().position(|e| &e.path == p))
                .unwrap_or(0);
            let next = match key {
                "home" => 0,
                "end" => self.rows.len() - 1,
                "up" => current.saturating_sub(1),
                _ => (current + 1).min(self.rows.len() - 1),
            };
            self.select(self.rows[next].path.clone(), modifiers, cx);
            self.scroll.scroll_to_item(next, ScrollStrategy::Center);
            cx.stop_propagation();
        }
        if key == "enter"
            && let Some(e) = self
                .rows
                .iter()
                .find(|e| self.selected.len() == 1 && self.selected.contains(&e.path))
        {
            self.open_entry(e.path.clone(), window, cx);
        }
        if key == "space"
            && let Some(path) = self.cursor.clone()
        {
            if !self.selected.insert(path.clone()) {
                self.selected.remove(&path);
            }
            cx.notify();
        }
    }
}
