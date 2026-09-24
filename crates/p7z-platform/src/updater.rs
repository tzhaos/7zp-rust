use anyhow::{Result, ensure};
pub use cardo_update::Prepared;
use cardo_update::{InstallAdapter, UpdateConfig, UpdateJournal, UpdateService};
use p7z_core::i18n::tr;
use serde_json::Value;
use std::{
    os::windows::process::CommandExt,
    path::Path,
    process::{Child, Command},
    sync::Arc,
};
const FILES: &[&str] = &[
    "p7z-explorer.dll",
    "vcruntime140.dll",
    "Fluent-LICENSE.txt",
    "LICENSE.txt",
    "Cardo-LICENSE.txt",
    "Cardo-dependencies-LICENSE.txt",
    "SQLite-binding-LICENSE.txt",
    "runtime/7zip/7z.dll",
    "runtime/7zip/7z.exe",
    "runtime/7zip/7z.sfx",
    "runtime/7zip/7zCon.sfx",
    "runtime/7zip/License.txt",
    "runtime/7zip/readme.txt",
    "runtime/7zip/History.txt",
    "runtime/7zip/7-zip.chm",
    "p7z.exe",
];

struct Journal;
impl UpdateJournal for Journal {
    fn pending(&self) -> Result<Option<Value>> {
        Ok(p7z_core::settings::store()?
            .read_json::<Option<Value>>("update-pending")?
            .flatten())
    }
    fn save_pending(&self, value: &Value) -> Result<()> {
        p7z_core::settings::store()?.write_json("update-pending", value)
    }
    fn clear_pending(&self) -> Result<()> {
        p7z_core::settings::store()?.remove("update-pending")
    }
    fn outcome(&self) -> Result<Option<Value>> {
        Ok(p7z_core::settings::store()?
            .read_json::<Option<Value>>("update-result")?
            .flatten())
    }
    fn finish(&self, value: &Value) -> Result<()> {
        p7z_core::settings::store()?.finish_update(value)
    }
    fn clear_outcome(&self) -> Result<()> {
        p7z_core::settings::store()?.remove("update-result")
    }
}
struct Installation;
impl InstallAdapter for Installation {
    fn registration(&self, target: &Path) -> Result<Option<Value>> {
        crate::registry::update_registration(target)?
            .map(serde_json::to_value)
            .transpose()
            .map_err(Into::into)
    }
    fn backup_files(&self, target: &Path, value: &Value) -> Result<Vec<String>> {
        let registration: crate::registry::UpdateRegistration =
            serde_json::from_value(value.clone())?;
        Ok(vec![
            "Uninstall.exe".into(),
            registration
                .dll
                .strip_prefix(target)?
                .to_string_lossy()
                .into_owned(),
        ])
    }
    fn restore(&self, target: &Path, value: &Value) -> Result<()> {
        crate::registry::restore_update_registration(
            target,
            &serde_json::from_value(value.clone())?,
        )
    }
    fn spawn(&self, package: &Path, target: &Path) -> Result<Child> {
        Ok(Command::new(package)
            .arg("/S")
            .arg("/UPDATE")
            .raw_arg(format!("/D={}", target.display()))
            .creation_flags(0x08000000)
            .spawn()?)
    }
    fn verify(&self, target: &Path, version: &str) -> Result<()> {
        ensure!(
            crate::registry::update_registration(target)?
                .and_then(|r| r.version)
                .as_deref()
                == Some(version),
            tr("update-installer-failed")
        );
        Ok(())
    }
}
pub fn service() -> Result<UpdateService> {
    UpdateService::new(
        UpdateConfig {
            application_id: "p7z".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            repository: option_env!("SEVENZIP_RELEASE_REPOSITORY").map(str::to_owned),
            installer: "p7z-amd64-installer.exe".into(),
            portable: "p7z-amd64-portable.zip".into(),
            checksums: "SHA256SUMS.txt".into(),
            archive_root: "p7z".into(),
            files: FILES.iter().map(|s| (*s).into()).collect(),
            executable: "p7z.exe".into(),
            helper_files: vec!["vcruntime140.dll".into()],
            helper_argument: "--apply-update".into(),
            jobs: p7z_core::settings::directory()?.join("updates"),
            max_package_bytes: 512 * 1024 * 1024,
            stage_prefix: ".p7z-update-".into(),
        },
        Arc::new(Journal),
        Arc::new(Installation),
    )
}
pub fn apply() -> Result<()> {
    service()?.apply()
}
pub fn recover() -> Result<Option<String>> {
    service()?.recover()
}
