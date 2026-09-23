use super::{Cancellation, Catalog, Engine, engine::publish};
use anyhow::{Context, Result, bail};
use p7z_core::i18n::tr;
use std::fs::{File, OpenOptions};
use std::sync::atomic::Ordering;

impl Engine {
    pub fn comment(catalog: &Catalog) -> Result<String> {
        let archive = zip::ZipArchive::new(File::open(&catalog.path)?)?;
        Ok(std::str::from_utf8(archive.comment())
            .context(tr("comment-encoding"))?
            .to_owned())
    }

    pub fn write_comment(
        &self,
        catalog: &Catalog,
        text: &str,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<Catalog> {
        if !catalog.accepts_comment() {
            bail!(tr("archive-read-only"));
        }
        if text.len() > u16::MAX as usize {
            bail!(tr("comment-too-long"));
        }
        let stage =
            tempfile::tempdir_in(catalog.path.parent().context(tr("destination-required"))?)?;
        let temporary = stage
            .path()
            .join(catalog.path.file_name().context(tr("file-name-missing"))?);
        std::fs::copy(&catalog.path, &temporary)?;
        let file = OpenOptions::new().read(true).write(true).open(&temporary)?;
        let mut archive = zip::ZipWriter::new_append(file)?;
        archive.set_comment(text);
        archive.finish()?.sync_all()?;
        if self.test(&temporary, password, cancel)?.warning {
            bail!(tr("integrity-warning"));
        }
        let mut updated = self.list(&temporary, password, cancel)?;
        if cancel.load(Ordering::Relaxed) {
            bail!(tr("task-cancelled"));
        }
        publish(&temporary, &catalog.path, true).context(tr("publish-failed"))?;
        updated.path = catalog.path.clone();
        Ok(updated)
    }
}
