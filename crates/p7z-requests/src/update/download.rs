use super::Status;
use anyhow::{Context, Result, ensure};
use p7z_core::i18n::tr;
use p7z_platform::updater;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Download,
    Verify,
    Prepare,
    Install,
}

#[derive(Clone, Default)]
pub struct Progress {
    pub cancel: Arc<AtomicBool>,
    received: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
    phase: Arc<AtomicU8>,
}

impl Progress {
    pub fn received(&self) -> u64 {
        self.received.load(Ordering::Relaxed)
    }
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }
    pub fn phase(&self) -> Phase {
        match self.phase.load(Ordering::Relaxed) {
            0 => Phase::Download,
            1 => Phase::Verify,
            2 => Phase::Prepare,
            _ => Phase::Install,
        }
    }
    pub fn installing(&self) {
        self.phase.store(3, Ordering::Relaxed);
    }
}

pub fn prepare(release: &Status, progress: &Progress) -> Result<updater::Prepared> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(download(release, progress))
}

async fn cancellable<T>(
    future: impl std::future::Future<Output = reqwest::Result<T>>,
    progress: &Progress,
) -> Result<T> {
    tokio::select! {
        result = future => Ok(result?),
        _ = async {
            while !progress.cancel.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } => anyhow::bail!(tr("task-cancelled")),
    }
}

async fn download(release: &Status, progress: &Progress) -> Result<updater::Prepared> {
    let Status::Available {
        version,
        download,
        portable,
        checksums,
        installer_size,
        portable_size,
        ..
    } = release
    else {
        anyhow::bail!(tr("update-package-invalid"));
    };
    let target = updater::Target::current()?;
    let (url, name, size) = if target.installed() {
        (download, "p7z-amd64-installer.exe", *installer_size)
    } else {
        (portable, "p7z-amd64-portable.zip", *portable_size)
    };
    ensure!(
        size > 0 && size <= 512 * 1024 * 1024,
        tr("update-package-invalid")
    );
    progress.total.store(size, Ordering::Relaxed);
    let client = reqwest::Client::builder()
        .user_agent(concat!("p7z/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(600))
        .build()?;
    let mut manifest_response = cancellable(
        client
            .get(checksums)
            .timeout(Duration::from_secs(25))
            .send(),
        progress,
    )
    .await?
    .error_for_status()?;
    let mut manifest = Vec::new();
    while let Some(chunk) = cancellable(manifest_response.chunk(), progress).await? {
        ensure!(
            manifest.len() + chunk.len() <= 8192,
            tr("update-package-invalid")
        );
        manifest.extend_from_slice(&chunk);
    }
    let manifest = String::from_utf8(manifest).context(tr("update-package-invalid"))?;
    let mut hashes = std::collections::BTreeMap::new();
    for line in manifest.lines() {
        let (hash, file) = line
            .split_once("  ")
            .context(tr("update-package-invalid"))?;
        ensure!(
            hash.len() == 64
                && hash.bytes().all(|b| b.is_ascii_hexdigit())
                && hashes.insert(file, hash.to_ascii_lowercase()).is_none(),
            tr("update-package-invalid")
        );
    }
    let expected = hashes.get(name).context(tr("update-package-invalid"))?;
    ensure!(
        !progress.cancel.load(Ordering::Relaxed),
        tr("task-cancelled")
    );
    let job = updater::job_directory()?;
    let mut file = File::create(job.path().join(name))?;
    let mut response = cancellable(client.get(url).send(), progress)
        .await?
        .error_for_status()?;
    let mut hash = Sha256::new();
    let mut received = 0u64;
    while let Some(chunk) = cancellable(response.chunk(), progress).await? {
        received += chunk.len() as u64;
        ensure!(received <= size, tr("update-package-invalid"));
        file.write_all(&chunk)?;
        hash.update(&chunk);
        progress.received.store(received, Ordering::Relaxed);
    }
    file.sync_all()?;
    drop(file);
    progress.phase.store(1, Ordering::Relaxed);
    ensure!(
        received == size && format!("{:x}", hash.finalize()) == *expected,
        tr("update-hash-mismatch")
    );
    ensure!(
        !progress.cancel.load(Ordering::Relaxed),
        tr("task-cancelled")
    );
    progress.phase.store(2, Ordering::Relaxed);
    updater::prepare(
        target,
        job,
        name,
        expected.clone(),
        version.clone(),
        &progress.cancel,
    )
}
