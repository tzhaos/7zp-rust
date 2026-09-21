pub mod file_icons;
mod instance;
pub mod mail;
mod maintenance;
mod registry;

pub use instance::instance;
pub use maintenance::close_application;
pub use registry::{DEFAULT_APPS_URI, configure, register, unregister};
mod open;
pub use open::{open_directory, open_file};
use std::sync::mpsc::Receiver;

pub enum Command {
    Launch(Vec<String>),
    Error(String),
}
pub struct System {
    pub receiver: Receiver<Command>,
}

pub use sevenzip_shell_api::Action as ShellAction;

pub fn read_shell_request(path: &std::path::Path) -> anyhow::Result<Vec<String>> {
    if path.parent() != Some(std::env::temp_dir().as_path())
        || !path.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .starts_with(sevenzip_shell_api::REQUEST_PREFIX)
        })
    {
        anyhow::bail!(sevenzip_core::i18n::tr("shell-request-invalid"));
    }
    let bytes = std::fs::read(path);
    let _ = std::fs::remove_file(path);
    let request: sevenzip_shell_api::Request = serde_json::from_slice(&bytes?)?;
    let mut args = vec![request.action.argument().into()];
    args.extend(
        request
            .paths
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned()),
    );
    Ok(args)
}

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
