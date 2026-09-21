pub mod file_icons;
mod instance;
pub mod mail;
mod maintenance;
mod registry;

pub use instance::instance;
pub use maintenance::close_application;
pub use registry::{DEFAULT_APPS_URI, configure, register, unregister};
mod open;
pub use open::open_file;
use std::sync::mpsc::Receiver;

pub enum Command {
    Launch(Vec<String>),
    Error(String),
}
pub struct System {
    pub receiver: Receiver<Command>,
}

pub use sevenzip_shell_api::Action as ShellAction;

pub fn parse_action(value: &str) -> anyhow::Result<ShellAction> {
    sevenzip_shell_api::ACTIONS
        .iter()
        .copied()
        .find(|action| action.argument() == value)
        .ok_or_else(|| {
            anyhow::anyhow!(sevenzip_core::i18n::tf(
                "shell-action-unknown",
                &[("action", value.into())]
            ))
        })
}
