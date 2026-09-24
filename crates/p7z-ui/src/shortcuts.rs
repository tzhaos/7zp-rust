use crate::*;
use p7z_core::settings::shortcuts::ShortcutAction;

impl commands::Command {
    pub(crate) fn shortcut_label(
        self,
        values: &p7z_core::settings::shortcuts::Shortcuts,
    ) -> String {
        ShortcutAction::ALL
            .iter()
            .copied()
            .find(|&action| command_for(action) == Some(self))
            .and_then(|action| action.binding(values))
            .map(|key| key.display())
            .unwrap_or_default()
    }
}

pub(crate) use cardo_ui::shortcuts::chord;

impl Workspace {
    pub(crate) fn dispatch_shortcut(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.dialogs.is_open()
            || self.settings_busy(cx)
            || self.tasks.is_busy()
            || self.history_popup.read(cx).is_open()
            || cardo_ui::menu::MenuHost::is_open(window, cx)
        {
            return false;
        }
        let Some(chord) = chord(event) else {
            return false;
        };
        // Custom global bindings must not replace editing in a focused input.
        if !self.focus.is_focused(window) && chord.protects_input()
        {
            return false;
        }
        let Some(action) = ShortcutAction::ALL
            .iter()
            .copied()
            .find(|action| action.binding(&self.preferences.shortcuts).as_ref() == Some(&chord))
        else {
            return false;
        };
        let global = matches!(
            action,
            ShortcutAction::Open
                | ShortcutAction::Create
                | ShortcutAction::Application
                | ShortcutAction::System
                | ShortcutAction::Advanced
                | ShortcutAction::About
        );
        let address = matches!(
            action,
            ShortcutAction::FocusAddress | ShortcutAction::FocusSearch
        );
        if !global && (self.settings_page.is_some() || (!address && !self.focus.is_focused(window)))
        {
            return false;
        }
        if event.is_held {
            return true;
        }
        match action {
            ShortcutAction::Back => self.back(window, cx),
            ShortcutAction::FocusAddress => self.address.update(cx, |input, cx| {
                input.focus(window, cx);
                input.select_all(window, cx);
            }),
            ShortcutAction::FocusSearch => {
                self.search.update(cx, |input, cx| input.focus(window, cx))
            }
            _ => {
                if let Some(command) = command_for(action) {
                    self.command(command, window, cx);
                }
            }
        }
        true
    }
}

fn command_for(action: ShortcutAction) -> Option<commands::Command> {
    Some(match action {
        ShortcutAction::Open => commands::Command::Open,
        ShortcutAction::Create => commands::Command::Create,
        ShortcutAction::Save => commands::Command::Save,
        ShortcutAction::Extract => commands::Command::Extract,
        ShortcutAction::QuickExtract => commands::Command::QuickExtractSelection,
        ShortcutAction::Add => commands::Command::Add,
        ShortcutAction::AddFolder => commands::Command::AddFolder,
        ShortcutAction::ArchiveInfo => commands::Command::ArchiveInfo,
        ShortcutAction::Comment => commands::Command::Comment,
        ShortcutAction::Check => commands::Command::Check,
        ShortcutAction::Rename => commands::Command::Rename,
        ShortcutAction::CopyTo => commands::Command::CopyTo,
        ShortcutAction::MoveTo => commands::Command::MoveTo,
        ShortcutAction::Delete => commands::Command::Delete,
        ShortcutAction::Properties => commands::Command::Properties,
        ShortcutAction::Refresh => commands::Command::Refresh,
        ShortcutAction::SelectAll => commands::Command::SelectAll,
        ShortcutAction::DeselectAll => commands::Command::DeselectAll,
        ShortcutAction::InvertSelection => commands::Command::InvertSelection,
        ShortcutAction::Home => commands::Command::Home,
        ShortcutAction::Browse => commands::Command::Browse,
        ShortcutAction::Back => return None,
        ShortcutAction::Up => commands::Command::Up,
        ShortcutAction::FocusAddress => return None,
        ShortcutAction::FocusSearch => return None,
        ShortcutAction::Application => commands::Command::Settings(preferences::Tab::Application),
        ShortcutAction::System => commands::Command::Settings(preferences::Tab::Integration),
        ShortcutAction::Advanced => commands::Command::Settings(preferences::Tab::Advanced),
        ShortcutAction::About => commands::Command::About,
        ShortcutAction::SortName => commands::Command::Sort(0),
        ShortcutAction::SortModified => commands::Command::Sort(3),
        ShortcutAction::SortType => commands::Command::Sort(2),
        ShortcutAction::SortSize => commands::Command::Sort(1),
    })
}
