use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShortcutAction {
    Open,
    Create,
    Save,
    Extract,
    QuickExtract,
    Add,
    AddFolder,
    ArchiveInfo,
    Comment,
    Check,
    Rename,
    CopyTo,
    MoveTo,
    Delete,
    Properties,
    Refresh,
    SelectAll,
    DeselectAll,
    InvertSelection,
    Home,
    Browse,
    Back,
    Up,
    FocusAddress,
    FocusSearch,
    Application,
    System,
    Advanced,
    About,
    SortName,
    SortModified,
    SortType,
    SortSize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: String,
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
}

pub type Shortcuts = BTreeMap<ShortcutAction, Option<Shortcut>>;

impl ShortcutAction {
    pub const ALL: &[Self] = &[
        Self::Open,
        Self::Create,
        Self::Save,
        Self::Extract,
        Self::QuickExtract,
        Self::Add,
        Self::AddFolder,
        Self::ArchiveInfo,
        Self::Comment,
        Self::Check,
        Self::Rename,
        Self::CopyTo,
        Self::MoveTo,
        Self::Delete,
        Self::Properties,
        Self::Refresh,
        Self::SelectAll,
        Self::DeselectAll,
        Self::InvertSelection,
        Self::Home,
        Self::Browse,
        Self::Back,
        Self::Up,
        Self::FocusAddress,
        Self::FocusSearch,
        Self::Application,
        Self::System,
        Self::Advanced,
        Self::About,
        Self::SortName,
        Self::SortModified,
        Self::SortType,
        Self::SortSize,
    ];

    pub fn description(self) -> &'static str {
        match self {
            Self::Open => "shortcut-desc-open",
            Self::Create => "shortcut-desc-create",
            Self::Save => "shortcut-desc-save",
            Self::Extract => "shortcut-desc-extract",
            Self::QuickExtract => "shortcut-desc-quick-extract",
            Self::Add => "shortcut-desc-add",
            Self::AddFolder => "shortcut-desc-add-folder",
            Self::ArchiveInfo => "shortcut-desc-archive-info",
            Self::Comment => "shortcut-desc-comment",
            Self::Check => "shortcut-desc-check",
            Self::Rename => "shortcut-desc-rename",
            Self::CopyTo => "shortcut-desc-copy-to",
            Self::MoveTo => "shortcut-desc-move-to",
            Self::Delete => "shortcut-desc-delete",
            Self::Properties => "shortcut-desc-properties",
            Self::Refresh => "shortcut-desc-refresh",
            Self::SelectAll => "shortcut-desc-select-all",
            Self::DeselectAll => "shortcut-desc-deselect-all",
            Self::InvertSelection => "shortcut-desc-invert-selection",
            Self::Home => "shortcut-desc-home",
            Self::Browse => "shortcut-desc-browse",
            Self::Back => "shortcut-desc-back",
            Self::Up => "shortcut-desc-up",
            Self::FocusAddress => "shortcut-desc-focus-address",
            Self::FocusSearch => "shortcut-desc-focus-search",
            Self::Application => "shortcut-desc-application",
            Self::System => "shortcut-desc-system",
            Self::Advanced => "shortcut-desc-advanced",
            Self::About => "shortcut-desc-about",
            Self::SortName => "shortcut-desc-sort-name",
            Self::SortModified => "shortcut-desc-sort-modified",
            Self::SortType => "shortcut-desc-sort-type",
            Self::SortSize => "shortcut-desc-sort-size",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "archive-open",
            Self::Create => "archive-create-command",
            Self::Save => "archive-save-as",
            Self::Extract => "extract-options",
            Self::QuickExtract => "extract-quick",
            Self::Add => "archive-add",
            Self::AddFolder => "archive-add-folder",
            Self::ArchiveInfo => "archive-info",
            Self::Comment => "archive-comment",
            Self::Check => "archive-check",
            Self::Rename => "archive-rename",
            Self::CopyTo => "archive-copy",
            Self::MoveTo => "archive-move",
            Self::Delete => "archive-delete",
            Self::Properties => "file-properties",
            Self::Refresh => "menu-refresh",
            Self::SelectAll => "select-all",
            Self::DeselectAll => "selection-clear",
            Self::InvertSelection => "shortcuts-invert",
            Self::Home => "browser-home",
            Self::Browse => "browser-browse",
            Self::Back => "back",
            Self::Up => "shortcuts-up",
            Self::FocusAddress => "browser-address",
            Self::FocusSearch => "search-placeholder",
            Self::Application => "settings-general",
            Self::System => "settings-integration",
            Self::Advanced => "settings-advanced",
            Self::About => "menu-about",
            Self::SortName => "shortcuts-sort-name",
            Self::SortModified => "shortcuts-sort-modified",
            Self::SortType => "shortcuts-sort-type",
            Self::SortSize => "shortcuts-sort-size",
        }
    }

    pub fn default_shortcut(self) -> Option<Shortcut> {
        let (key, control, shift, alt) = match self {
            Self::Open => ("o", true, false, false),
            Self::Create => ("n", true, false, false),
            Self::Save => ("s", true, true, false),
            Self::Extract => ("e", false, false, true),
            Self::QuickExtract => return None,
            Self::Add => ("a", false, false, true),
            Self::AddFolder => return None,
            Self::ArchiveInfo => ("i", false, false, true),
            Self::Comment => ("m", false, false, true),
            Self::Check => ("t", false, false, true),
            Self::Rename => ("f2", false, false, false),
            Self::CopyTo => ("c", true, true, false),
            Self::MoveTo => ("m", true, true, false),
            Self::Delete => ("delete", false, false, false),
            Self::Properties => ("enter", false, false, true),
            Self::Refresh => ("f5", false, false, false),
            Self::SelectAll => ("a", true, false, false),
            Self::DeselectAll => return None,
            Self::InvertSelection => ("i", true, false, false),
            Self::Home => return None,
            Self::Browse => return None,
            Self::Back => ("left", false, false, true),
            Self::Up => ("up", false, false, true),
            Self::FocusAddress => ("l", true, false, false),
            Self::FocusSearch => ("f", true, false, false),
            Self::Application => (",", true, false, false),
            Self::System => ("s", true, false, true),
            Self::Advanced => ("a", true, false, true),
            Self::About => ("i", true, false, true),
            Self::SortName => ("f3", true, false, false),
            Self::SortModified => ("f4", true, false, false),
            Self::SortType => ("f5", true, false, false),
            Self::SortSize => ("f6", true, false, false),
        };
        Some(Shortcut {
            key: key.into(),
            control,
            shift,
            alt,
        })
    }

    pub fn binding(self, values: &Shortcuts) -> Option<Shortcut> {
        values
            .get(&self)
            .cloned()
            .unwrap_or_else(|| self.default_shortcut())
    }
}

impl Shortcut {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.control {
            parts.push("Ctrl".to_owned());
        }
        if self.alt {
            parts.push("Alt".to_owned());
        }
        if self.shift {
            parts.push("Shift".to_owned());
        }
        parts.push(match self.key.as_str() {
            "enter" => "Enter".into(),
            "delete" => "Del".into(),
            "left" => "Left".into(),
            "up" => "Up".into(),
            other => other.to_uppercase(),
        });
        parts.join("+")
    }

    pub fn allowed(&self) -> bool {
        let key = self.key.as_str();
        let function = key
            .strip_prefix('f')
            .and_then(|n| n.parse::<u8>().ok())
            .is_some_and(|n| (1..=12).contains(&n));
        let letter = key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric();
        let supported = function
            || letter
            || matches!(
                key,
                "," | "."
                    | "/"
                    | ";"
                    | "'"
                    | "["
                    | "]"
                    | "-"
                    | "="
                    | "delete"
                    | "enter"
                    | "left"
                    | "up"
                    | "right"
                    | "down"
                    | "home"
                    | "end"
                    | "pageup"
                    | "pagedown"
            );
        if !supported {
            return false;
        }
        if self.control && self.alt && key == "delete" {
            return false;
        }
        if !self.control && !self.alt && !function && key != "delete" {
            return false;
        }
        // Editing and native window/navigation keys retain their established behavior.
        if self.control
            && !self.alt
            && matches!(key, "c" | "v" | "x" | "z" | "y" | "s" | "w")
            && !self.shift
        {
            return false;
        }
        if self.alt && matches!(key, "f4" | "space" | "tab" | "down") {
            return false;
        }
        if self.shift && !self.control && !self.alt && key == "f10" {
            return false;
        }
        if self.control && !self.alt && !self.shift && key == "pagedown" {
            return false;
        }
        true
    }
}

pub fn validate(values: &Shortcuts) -> anyhow::Result<()> {
    let mut used: Vec<(ShortcutAction, Shortcut)> = Vec::new();
    for &action in ShortcutAction::ALL {
        let Some(binding) = action.binding(values) else {
            continue;
        };
        anyhow::ensure!(
            binding.allowed(),
            "{}",
            crate::i18n::tf(
                "shortcuts-config-invalid",
                &[
                    ("command", crate::i18n::tr(action.label()).into()),
                    ("keys", binding.display().into())
                ]
            )
        );
        if let Some((other, _)) = used.iter().find(|(_, candidate)| candidate == &binding) {
            anyhow::bail!(crate::i18n::tf(
                "shortcuts-conflict",
                &[
                    ("command", crate::i18n::tr(other.label()).into()),
                    ("keys", binding.display().into())
                ]
            ));
        }
        used.push((action, binding));
    }
    Ok(())
}
