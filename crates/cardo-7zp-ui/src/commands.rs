use super::*;

#[derive(Clone, Copy)]
pub(super) enum Command {
    Open,
    Browse,
    Home,
    OpenItem,
    OpenInside,
    OpenOutside,
    Create,
    Save,
    ArchiveInfo,
    ClearRecent,
    Exit,
    Extract,
    QuickExtractSelection,
    Add,
    AddFolder,
    Rename,
    CopyTo,
    MoveTo,
    Delete,
    Properties,
    Comment,
    Check,
    SelectAll,
    DeselectAll,
    InvertSelection,
    Sort(usize),
    Up,
    Refresh,
    Settings(preferences::Tab),
    About,
}

impl Workspace {
    pub(crate) fn command_available(&self, command: Command, cx: &App) -> bool {
        if self.settings_busy(cx) {
            return false;
        }
        self.command_enabled(command, cx)
    }

    pub(crate) fn command_enabled(&self, command: Command, cx: &App) -> bool {
        if self.tasks.is_busy() || self.settings_controls_disabled(cx) || self.dialogs.is_open() {
            return false;
        }
        let view = self.browser.view();
        let archive = view.catalog.is_some();
        let editable = archive && view.catalog.as_ref().is_some_and(Catalog::editable);
        let selected = !view.selected.is_empty();
        match command {
            Command::Rename => editable && view.selected.len() == 1,
            Command::Delete | Command::MoveTo => editable && selected,
            Command::CopyTo => archive && selected,
            Command::Add | Command::AddFolder => editable,
            Command::Comment => {
                archive && view.catalog.as_ref().is_some_and(Catalog::accepts_comment)
            }
            Command::Save
            | Command::ArchiveInfo
            | Command::Properties
            | Command::Extract
            | Command::QuickExtractSelection
            | Command::Check => archive,
            Command::OpenItem => self.current_item().is_some(),
            Command::OpenInside => self.can_open_inside(),
            Command::OpenOutside => self.can_open_outside(),
            Command::Up => self.parent_location().is_some(),
            Command::SelectAll
            | Command::DeselectAll
            | Command::InvertSelection
            | Command::Sort(_)
            | Command::Refresh => true,
            Command::Open
            | Command::Browse
            | Command::Home
            | Command::Create
            | Command::ClearRecent
            | Command::Exit
            | Command::Settings(_)
            | Command::About => true,
        }
    }

    pub(super) fn command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_available(command, cx) {
            return;
        }
        if !matches!(
            command,
            Command::Settings(_) | Command::About | Command::ClearRecent | Command::Exit
        ) {
            self.show_browser(window, cx);
        }
        match command {
            Command::Open => self.open(cx),
            Command::Browse => self.browse(cx),
            Command::Home => self.visit(browser::Location::Home, window, cx),
            Command::OpenItem => self.open_current(window, cx),
            Command::OpenInside => self.open_inside(window, cx),
            Command::OpenOutside => self.open_outside(window, cx),
            Command::Create => self.create_dialog(Vec::new(), window, cx),
            Command::Save => self.save_copy(cx),
            Command::ArchiveInfo => self.properties(false, cx),
            Command::ClearRecent => self.update_recent(recent::Change::Clear, cx),
            Command::Exit => window.remove_window(),
            Command::Extract => self.extract_dialog(window, cx),
            Command::QuickExtractSelection => {
                self.quick_extract(!self.browser.view().selected.is_empty(), cx)
            }
            Command::Rename => self.rename_entry(window, cx),
            Command::CopyTo => self.copy_entries(false, cx),
            Command::MoveTo => self.copy_entries(true, cx),
            Command::Delete => self.delete_entries(cx),
            Command::Add => self.add_to_archive(false, cx),
            Command::AddFolder => self.add_to_archive(true, cx),
            Command::Properties => self.properties(!self.browser.view().selected.is_empty(), cx),
            Command::Comment => {
                if let Some(catalog) = self.browser.view().catalog.clone() {
                    self.execute(Request::ReadComment(catalog), String::new(), cx);
                }
            }
            Command::SelectAll => {
                self.browser.select_all();
                cx.notify();
            }
            Command::DeselectAll => {
                self.browser.deselect_all();
                cx.notify();
            }
            Command::InvertSelection => {
                self.browser.invert_selection();
                cx.notify();
            }
            Command::Sort(column) => {
                if self.browser.view().sort == column {
                    let descending = !self.browser.view().descending;
                    self.browser.set_descending(descending);
                } else {
                    self.browser.set_sort(column);
                    self.browser.set_descending(false);
                }
                self.refresh(cx);
            }
            Command::Up => self.up(window, cx),
            Command::Refresh => self.reload_location(cx),
            Command::Check => {
                if let Some(catalog) = self.browser.view().catalog.clone() {
                    self.execute(
                        Request::Check(catalog),
                        self.browser.view().password.clone(),
                        cx,
                    );
                }
            }
            Command::Settings(tab) => self.preferences_category(tab, window, cx),
            Command::About => self.open_settings(views::SettingsPage::About, window, cx),
        }
    }
}

impl Workspace {
    pub(crate) fn keyboard(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if self.history_popup.read(cx).is_open() || cardo_ui::menu::MenuHost::is_open(window, cx) {
            return;
        }
        if modifiers.alt
            && key == "down"
            && self.address.read(cx).focus_handle(cx).is_focused(window)
        {
            self.toggle_history(window, cx);
            cx.stop_propagation();
            return;
        }
        if key == "escape" {
            if self.tasks.is_busy() {
                return;
            }
            if self.address.read(cx).focus_handle(cx).is_focused(window) {
                self.address_dirty = true;
                self.focus.focus(window, cx);
                cx.notify();
            } else if self.dialogs.is_open() {
                self.close_modal(cx);
            } else if self.settings_page.is_some() {
                self.show_browser(window, cx);
            } else if !self.search.read(cx).value().is_empty() {
                self.search
                    .update(cx, |input, cx| input.set_value("", window, cx));
            } else {
                self.browser.clear_selection(false);
                cx.notify();
            }
            return;
        }
        if self.dialogs.is_open() {
            return;
        }
        if self.settings_page.is_some() {
            if modifiers.control
                && key == "s"
                && let Some(form) = self.settings_form.clone()
            {
                window.defer(cx, move |window, cx| {
                    form.update(cx, |form, cx| form.save(window, cx))
                });
                cx.stop_propagation();
            }
            return;
        }
        let global_command = match (modifiers.control, modifiers.alt, modifiers.shift, key) {
            (true, false, false, "o") => Some(Command::Open),
            (true, false, false, ",") => Some(Command::Settings(preferences::Tab::Application)),
            _ => None,
        };
        if let Some(command) = global_command {
            self.command(command, window, cx);
            cx.stop_propagation();
            return;
        }
        let command = match (modifiers.control, modifiers.alt, modifiers.shift, key) {
            (true, false, true, "s") => Some(commands::Command::Save),
            (false, true, _, "e") => Some(commands::Command::Extract),
            (false, true, _, "a") => Some(commands::Command::Add),
            (false, true, _, "i") => Some(commands::Command::ArchiveInfo),
            (false, true, _, "m") => Some(commands::Command::Comment),
            (false, true, _, "t") => Some(commands::Command::Check),
            _ => None,
        };
        if let Some(command) = command {
            self.command(command, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && key == "l" && !self.tasks.is_busy() {
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
        if self.tasks.is_busy() {
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
        if key == "contextmenu" || (modifiers.shift && key == "f10") {
            self.open_context_menu(
                menus::context::Target::Keyboard,
                point(px(32.), px(196.)),
                window,
                cx,
            );
            cx.stop_propagation();
            return;
        }
        if modifiers.control && !modifiers.alt && !modifiers.shift && key == "a" {
            self.browser.select_all();
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if modifiers.control && !modifiers.alt && !modifiers.shift && key == "r" {
            self.command(commands::Command::Refresh, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && !modifiers.alt && !modifiers.shift && key == "z" {
            self.command(commands::Command::Comment, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && !modifiers.alt && !modifiers.shift {
            let column = match key {
                "f3" => Some(0),
                "f4" => Some(3),
                "f5" => Some(2),
                "f6" => Some(1),
                _ => None,
            };
            if let Some(column) = column {
                self.command(commands::Command::Sort(column), window, cx);
                cx.stop_propagation();
                return;
            }
        }
        if modifiers.alt && !modifiers.control && key == "enter" {
            self.command(commands::Command::Properties, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.shift && !modifiers.control && !modifiers.alt && key == "enter" {
            self.command(commands::Command::OpenOutside, window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.control && !modifiers.shift && !modifiers.alt && key == "pagedown" {
            self.command(commands::Command::OpenInside, window, cx);
            cx.stop_propagation();
            return;
        }
        if !modifiers.control && !modifiers.alt && !modifiers.shift {
            let command = match key {
                "f2" => Some(commands::Command::Rename),
                "f5" => Some(commands::Command::CopyTo),
                "f6" => Some(commands::Command::MoveTo),
                "delete" => Some(commands::Command::Delete),
                _ => None,
            };
            if let Some(command) = command {
                self.command(command, window, cx);
                cx.stop_propagation();
                return;
            }
        }
        if ["up", "down", "home", "end"].contains(&key) && !self.browser.view().rows.is_empty() {
            let browser = self.browser.view();
            let current = browser
                .cursor
                .as_ref()
                .and_then(|p| browser.rows.iter().position(|e| &e.path == p))
                .unwrap_or(0);
            let next = match key {
                "home" => 0,
                "end" => browser.rows.len() - 1,
                "up" => current.saturating_sub(1),
                _ => (current + 1).min(browser.rows.len() - 1),
            };
            self.select(browser.rows[next].path.clone(), modifiers, cx);
            self.scroll.scroll_to_item(next, ScrollStrategy::Center);
            cx.stop_propagation();
        }
        if key == "enter" {
            if self.command_available(commands::Command::OpenItem, cx) {
                self.command(commands::Command::OpenItem, window, cx);
                cx.stop_propagation();
            }
        }
        if key == "space"
            && let Some(path) = self.browser.view().cursor.clone()
        {
            self.browser.toggle_selection(path, false);
            cx.notify();
        }
    }
}
