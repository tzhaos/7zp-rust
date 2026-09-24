pub use cardo_update::{Phase, Progress, Status};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: Option<&str> = option_env!("SEVENZIP_RELEASE_REPOSITORY");
pub fn check() -> anyhow::Result<Status> {
    p7z_platform::updater::service()?.check()
}
pub fn prepare(release: &Status, progress: &Progress) -> anyhow::Result<cardo_update::Prepared> {
    p7z_platform::updater::service()?.download(release, progress)
}
