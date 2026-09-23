pub mod file_icons;
mod instance;
pub mod mail;
mod maintenance;
mod registry;
pub mod updater;

pub use instance::instance;
pub use maintenance::close_application;
pub use registry::{DEFAULT_APPS_URI, configure, register, unregister};
mod open;
use anyhow::Context;
pub use open::{open_directory, open_file};
use std::sync::mpsc::Receiver;

pub enum Command {
    Launch(Vec<String>),
    Error(String),
}
pub struct System {
    pub receiver: Receiver<Command>,
}

pub use p7z_commands::Action as ExplorerAction;

pub fn read_shell_request(path: &std::path::Path) -> anyhow::Result<Vec<String>> {
    if path.parent() != Some(std::env::temp_dir().as_path())
        || !path.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .starts_with(p7z_commands::REQUEST_PREFIX)
        })
    {
        anyhow::bail!(p7z_core::i18n::tr("shell-request-invalid"));
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("Cannot read Explorer request {}", path.display()))?;
    let request = serde_json::from_slice::<p7z_commands::Request>(&bytes)
        .with_context(|| format!("Invalid Explorer request JSON in {}", path.display()));
    std::fs::remove_file(path)
        .with_context(|| format!("Cannot remove Explorer request {}", path.display()))?;
    let request = request?;
    let mut args = vec![request.action.argument().into()];
    args.extend(
        request
            .paths
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned()),
    );
    Ok(args)
}

pub fn parse_action(value: &str) -> anyhow::Result<ExplorerAction> {
    p7z_commands::ACTIONS
        .iter()
        .copied()
        .find(|action| action.argument() == value)
        .ok_or_else(|| {
            anyhow::anyhow!(p7z_core::i18n::tf(
                "shell-action-unknown",
                &[("action", value.into())]
            ))
        })
}
