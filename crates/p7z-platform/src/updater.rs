use anyhow::{Context, Result, bail, ensure};
use p7z_core::i18n::{tf, tr};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        QueryFullProcessImageNameW, WaitForSingleObject,
    },
};

const FILES: &[&str] = &[
    "p7z-explorer.dll",
    "vcruntime140.dll",
    "Glyphs-LICENSE.txt",
    "LICENSE.txt",
    "Cardo-LICENSE.txt",
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

pub struct Target {
    directory: PathBuf,
    registration: Option<crate::registry::UpdateRegistration>,
}

impl Target {
    pub fn current() -> Result<Self> {
        let executable = std::env::current_exe()?;
        let directory = executable
            .parent()
            .context(tr("executable-directory-missing"))?
            .to_owned();
        let registration = crate::registry::update_registration(&directory)?;
        Ok(Self {
            directory,
            registration,
        })
    }

    pub fn installed(&self) -> bool {
        self.registration.is_some()
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Backup {
    name: String,
    existed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Plan {
    target: PathBuf,
    job: PathBuf,
    stage: PathBuf,
    package: String,
    digest: String,
    version: String,
    parent_pid: u32,
    helper_pid: u32,
    installer_pid: u32,
    registration: Option<crate::registry::UpdateRegistration>,
    backup: Vec<Backup>,
    applying: bool,
}

#[derive(Serialize, Deserialize)]
struct Outcome {
    job: PathBuf,
    helper_pid: u32,
    error: Option<String>,
}

pub struct Prepared {
    job: tempfile::TempDir,
    stage: tempfile::TempDir,
    plan: Plan,
}

pub fn job_directory() -> Result<tempfile::TempDir> {
    let root = p7z_core::settings::directory()?.join("updates");
    fs::create_dir_all(&root).with_context(|| format!("Cannot create {}", root.display()))?;
    Ok(tempfile::Builder::new().prefix("job-").tempdir_in(root)?)
}

pub fn sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("Cannot read {}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn prepare(
    target: Target,
    job: tempfile::TempDir,
    package: &str,
    digest: String,
    version: String,
    cancel: &AtomicBool,
) -> Result<Prepared> {
    ensure!(
        sha256(&job.path().join(package))? == digest,
        tr("update-hash-mismatch")
    );
    let stage = tempfile::Builder::new()
        .prefix(".p7z-update-")
        .tempdir_in(&target.directory)
        .with_context(|| format!("Cannot stage update in {}", target.directory.display()))?;
    if !target.installed() {
        unpack(&job.path().join(package), &stage.path().join("new"), cancel)?;
    }
    let plan = Plan {
        target: target.directory,
        job: job.path().to_owned(),
        stage: stage.path().to_owned(),
        package: package.into(),
        digest,
        version,
        parent_pid: std::process::id(),
        helper_pid: 0,
        installer_pid: 0,
        registration: target.registration,
        backup: Vec::new(),
        applying: false,
    };
    Ok(Prepared { job, stage, plan })
}

fn unpack(package: &Path, destination: &Path, cancel: &AtomicBool) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(package)?)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for index in 0..zip.len() {
        ensure!(!cancel.load(Ordering::Relaxed), tr("task-cancelled"));
        let mut entry = zip.by_index(index)?;
        let enclosed = entry
            .enclosed_name()
            .context(tr("update-package-invalid"))?;
        ensure!(
            !entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000),
            tr("update-package-invalid")
        );
        let relative = enclosed
            .strip_prefix("p7z")
            .context(tr("update-package-invalid"))?;
        if entry.is_dir() {
            continue;
        }
        let name = relative.to_string_lossy().replace('\\', "/");
        ensure!(
            FILES.contains(&name.as_str()) && seen.insert(name),
            tr("update-package-invalid")
        );
        total = total
            .checked_add(entry.size())
            .context(tr("update-package-invalid"))?;
        ensure!(total <= 512 * 1024 * 1024, tr("update-package-invalid"));
        let output = destination.join(relative);
        fs::create_dir_all(output.parent().unwrap())?;
        let mut file = File::create(&output)?;
        let mut buffer = [0u8; 65536];
        loop {
            ensure!(!cancel.load(Ordering::Relaxed), tr("task-cancelled"));
            let count = entry.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            file.write_all(&buffer[..count])?;
        }
        file.sync_all()?;
    }
    ensure!(seen.len() == FILES.len(), tr("update-package-invalid"));
    Ok(())
}

impl Prepared {
    pub fn launch(self, cancel: &AtomicBool) -> Result<()> {
        ensure!(!cancel.load(Ordering::Relaxed), tr("task-cancelled"));
        let executable = std::env::current_exe()?;
        fs::copy(&executable, self.job.path().join("updater.exe"))?;
        fs::copy(
            self.plan.target.join("vcruntime140.dll"),
            self.job.path().join("vcruntime140.dll"),
        )?;
        let store = p7z_core::settings::store()?;
        store.write_json("update-pending", &Some(&self.plan))?;
        let spawn = Command::new(self.job.path().join("updater.exe"))
            .arg("--apply-update")
            .creation_flags(0x08000000)
            .spawn();
        let mut child = match spawn {
            Ok(child) => child,
            Err(error) => {
                store.remove("update-pending")?;
                return Err(error.into());
            }
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if self.job.path().join("ready").is_file() {
                break;
            }
            if child.try_wait()?.is_some() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                store.remove("update-pending")?;
                bail!(tr("update-helper-failed"));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.job.keep();
        let _ = self.stage.keep();
        Ok(())
    }
}

fn process(id: u32, expected: &Path) -> Result<Option<OwnedHandle>> {
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            id,
        )
    };
    if handle.is_null() {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(87) {
            return Ok(None);
        }
        return Err(error.into());
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    if unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } == WAIT_OBJECT_0 {
        return Ok(None);
    }
    let mut name = vec![0u16; 32768];
    let mut len = name.len() as u32;
    ensure!(
        unsafe {
            QueryFullProcessImageNameW(handle.as_raw_handle(), 0, name.as_mut_ptr(), &mut len)
        } != 0,
        tr("update-helper-failed")
    );
    let actual = PathBuf::from(String::from_utf16(&name[..len as usize])?);
    let expected = match fs::canonicalize(expected) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if fs::canonicalize(actual)? != expected {
        return Ok(None);
    }
    Ok(Some(handle))
}

fn validate_plan(plan: &Plan) -> Result<()> {
    let root = p7z_core::settings::directory()?.join("updates");
    ensure!(
        plan.job.parent() == Some(root.as_path())
            && plan
                .job
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("job-")),
        tr("update-package-invalid")
    );
    ensure!(
        plan.target.is_absolute()
            && plan.stage.parent() == Some(plan.target.as_path())
            && plan
                .stage
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(".p7z-update-")),
        tr("update-package-invalid")
    );
    ensure!(
        matches!(
            plan.package.as_str(),
            "p7z-amd64-installer.exe" | "p7z-amd64-portable.zip"
        ),
        tr("update-package-invalid")
    );
    Ok(())
}

fn safe_target(root: &Path, name: &str) -> Result<PathBuf> {
    let relative = Path::new(name);
    ensure!(
        relative
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_))),
        tr("update-package-invalid")
    );
    let path = root.join(relative);
    if path.exists() {
        ensure!(
            !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            tr("update-package-invalid")
        );
        ensure!(
            fs::canonicalize(&path)?.starts_with(fs::canonicalize(root)?),
            tr("update-package-invalid")
        );
    } else {
        let mut ancestor = path.parent().context(tr("update-package-invalid"))?;
        while !ancestor.exists() {
            ancestor = ancestor.parent().context(tr("update-package-invalid"))?;
        }
        ensure!(
            fs::canonicalize(ancestor)?.starts_with(fs::canonicalize(root)?),
            tr("update-package-invalid")
        );
    }
    Ok(path)
}

fn atomic_copy(source: &Path, destination: &Path) -> Result<()> {
    let parent = destination.parent().context(tr("update-package-invalid"))?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    std::io::copy(&mut File::open(source)?, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(destination)
        .map_err(|error| error.error)
        .with_context(|| format!("Cannot replace {}", destination.display()))?;
    Ok(())
}

fn backup(plan: &mut Plan) -> Result<()> {
    let mut names: Vec<String> = FILES.iter().map(|name| (*name).into()).collect();
    if let Some(registration) = &plan.registration {
        names.push("Uninstall.exe".into());
        names.push(
            registration
                .dll
                .strip_prefix(&plan.target)?
                .to_string_lossy()
                .into_owned(),
        );
    }
    for name in names {
        let path = safe_target(&plan.target, &name)?;
        let existed = match fs::metadata(&path) {
            Ok(metadata) => {
                ensure!(metadata.is_file(), tr("update-package-invalid"));
                true
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => {
                return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
            }
        };
        if existed {
            atomic_copy(&path, &plan.stage.join("backup").join(&name))?;
        }
        plan.backup.push(Backup { name, existed });
    }
    plan.applying = true;
    p7z_core::settings::store()?.write_json("update-pending", &Some(&plan))?;
    Ok(())
}

fn rollback(plan: &Plan) -> Result<()> {
    if !plan.applying {
        return Ok(());
    }
    for entry in &plan.backup {
        let target = safe_target(&plan.target, &entry.name)?;
        if entry.existed {
            let source = plan.stage.join("backup").join(&entry.name);
            if target.is_file() && sha256(&target)? == sha256(&source)? {
                continue;
            }
            atomic_copy(&source, &target)?;
        } else if target.exists() {
            fs::remove_file(target)?;
        }
    }
    if let Some(registration) = &plan.registration {
        crate::registry::restore_update_registration(&plan.target, registration)?;
    }
    Ok(())
}

fn install(plan: &mut Plan) -> Result<()> {
    ensure!(
        sha256(&plan.job.join(&plan.package))? == plan.digest,
        tr("update-hash-mismatch")
    );
    if plan.registration.is_some() {
        ensure!(
            crate::registry::update_registration(&plan.target)?.is_some(),
            tr("update-ownership-error")
        );
        let mut installer = Command::new(plan.job.join(&plan.package))
            .arg("/S")
            .arg("/UPDATE")
            .raw_arg(format!("/D={}", plan.target.display()))
            .creation_flags(0x08000000)
            .spawn()?;
        plan.installer_pid = installer.id();
        if let Err(error) =
            p7z_core::settings::store()?.write_json("update-pending", &Some(&plan))
        {
            let _ = installer.kill();
            let _ = installer.wait();
            return Err(error);
        }
        let status = installer.wait()?;
        ensure!(status.success(), tr("update-installer-failed"));
        ensure!(
            crate::registry::update_registration(&plan.target)?
                .and_then(|registration| registration.version)
                .as_deref()
                == Some(plan.version.as_str()),
            tr("update-installer-failed")
        );
    } else {
        for &name in FILES {
            let target = safe_target(&plan.target, name)?;
            atomic_copy(&plan.stage.join("new").join(name), &target)?;
        }
    }
    Ok(())
}

pub fn apply() -> Result<()> {
    let store = p7z_core::settings::store()?;
    let mut plan = store
        .read_json::<Option<Plan>>("update-pending")?
        .flatten()
        .context(tr("update-package-invalid"))?;
    validate_plan(&plan)?;
    let helper = fs::canonicalize(std::env::current_exe()?)?;
    let job = fs::canonicalize(&plan.job)?;
    ensure!(
        helper.parent() == Some(job.as_path()),
        tr("update-package-invalid")
    );
    let parent = process(plan.parent_pid, &plan.target.join("p7z.exe"))?
        .context(tr("update-helper-failed"))?;
    plan.helper_pid = std::process::id();
    store.write_json("update-pending", &Some(&plan))?;
    File::create(plan.job.join("ready"))?.sync_all()?;
    match unsafe { WaitForSingleObject(parent.as_raw_handle(), 60000) } {
        WAIT_OBJECT_0 => {}
        WAIT_TIMEOUT => {
            store.remove("update-pending")?;
            bail!(tr("maintenance-app-busy"));
        }
        _ => return Err(std::io::Error::last_os_error().into()),
    }
    let result = backup(&mut plan).and_then(|_| install(&mut plan));
    let error = if let Err(error) = result {
        tracing::error!(error = %format!("{error:#}"), "Update installation failed");
        if plan.installer_pid != 0
            && process(plan.installer_pid, &plan.job.join(&plan.package))?.is_some()
        {
            bail!(tr("update-running"));
        }
        rollback(&plan).with_context(|| {
            tf(
                "update-rollback-failed",
                &[("path", plan.stage.display().to_string().into())],
            )
        })?;
        Some(tf(
            "update-rolled-back",
            &[("error", format!("{error:#}").into())],
        ))
    } else {
        None
    };
    store.finish_update(
        &Some(Outcome {
            job: plan.job.clone(),
            helper_pid: plan.helper_pid,
            error,
        }),
    )?;
    if let Err(error) = fs::remove_dir_all(&plan.stage) {
        tracing::warn!(error = %error, "Cannot clean update staging directory");
    }
    Command::new(plan.target.join("p7z.exe"))
        .current_dir(&plan.target)
        .spawn()
        .context(tr("update-restart-failed"))?;
    Ok(())
}

/// Called before opening a normal application window, never during rendering.
pub fn recover() -> Result<Option<String>> {
    let store = p7z_core::settings::store()?;
    if let Some(plan) = store
        .read_json::<Option<Plan>>("update-pending")?
        .flatten()
    {
        validate_plan(&plan)?;
        if plan.installer_pid != 0
            && process(plan.installer_pid, &plan.job.join(&plan.package))?.is_some()
        {
            bail!(tr("update-running"));
        }
        if plan.helper_pid != 0
            && process(plan.helper_pid, &plan.job.join("updater.exe"))?.is_some()
        {
            bail!(tr("update-running"));
        }
        if plan.helper_pid == 0
            && process(plan.parent_pid, &plan.target.join("p7z.exe"))?.is_some()
        {
            bail!(tr("update-running"));
        }
        rollback(&plan).with_context(|| {
            tf(
                "update-rollback-failed",
                &[("path", plan.stage.display().to_string().into())],
            )
        })?;
        store.remove("update-pending")?;
        for directory in [&plan.stage, &plan.job] {
            if let Err(error) = fs::remove_dir_all(directory) {
                tracing::warn!(path = %directory.display(), error = %error, "Cannot clean interrupted update directory");
            }
        }
        return Ok(Some(tr("update-recovered").into()));
    }
    if let Some(outcome) = store
        .read_json::<Option<Outcome>>("update-result")?
        .flatten()
    {
        let root = p7z_core::settings::directory()?.join("updates");
        if outcome.job.parent() == Some(root.as_path())
            && outcome
                .job
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("job-"))
        {
            if let Some(handle) = process(outcome.helper_pid, &outcome.job.join("updater.exe"))? {
                unsafe { WaitForSingleObject(handle.as_raw_handle(), 3000) };
            }
            if let Err(error) = fs::remove_dir_all(&outcome.job) {
                tracing::warn!(error = %error, "Cannot clean completed update download");
            }
        }
        store.remove("update-result")?;
        return Ok(outcome.error);
    }
    Ok(None)
}
