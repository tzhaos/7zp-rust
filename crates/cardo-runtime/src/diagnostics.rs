use anyhow::{Context, Result};
use std::path::Path;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{Builder, Rotation},
};

pub fn init(directory: &Path, application: &str) -> Result<WorkerGuard> {
    std::fs::create_dir_all(directory)
        .with_context(|| format!("Cannot create log directory {}", directory.display()))?;
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix(application)
        .filename_suffix("log")
        .max_log_files(14)
        .build(directory)
        .with_context(|| format!("Cannot initialize logs in {}", directory.display()))?;
    let (writer, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(false)
        .finish(appender);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_max_level(tracing::Level::INFO)
        .with_writer(writer)
        .try_init()
        .map_err(|error| anyhow::anyhow!("Cannot install log subscriber: {error}"))?;
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        tracing::error!(message = %panic, backtrace = %std::backtrace::Backtrace::force_capture(), "Application panic");
        previous(panic);
    }));
    tracing::info!(
        application,
        process_id = std::process::id(),
        "Application started"
    );
    Ok(guard)
}
