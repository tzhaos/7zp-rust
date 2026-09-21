use super::{Cancellation, Catalog, CreateOptions, Entry, Overwrite, Progress, progress};
use crate::i18n::{tf, tr};
use anyhow::{Context, Result, bail};
use sevenzip_shell_api::OpenType;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{Read, Write},
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

#[derive(Debug)]
struct CommandFailure {
    code: Option<i32>,
    details: String,
    password_required: bool,
}

impl std::fmt::Display for CommandFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&tf(
            "engine-failed",
            &[
                ("code", format!("{:?}", self.code).into()),
                ("details", self.details.as_str().into()),
            ],
        ))
    }
}
impl std::error::Error for CommandFailure {}

pub fn needs_password(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<CommandFailure>()
        .is_some_and(|failure| failure.password_required)
}

#[derive(Debug)]
pub struct SourceError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub struct Output {
    pub text: String,
    pub warning: bool,
}

pub struct Engine {
    executable: PathBuf,
}

impl Engine {
    pub fn bundled() -> Result<Self> {
        let exe = std::env::current_exe()?;
        Ok(Self {
            executable: exe
                .parent()
                .context(tr("executable-directory-missing"))?
                .join("runtime/7zip/7z.exe"),
        })
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.executable);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let priority = if crate::settings::LOW_PRIORITY.load(Ordering::Relaxed) {
                0x00004000
            } else {
                0
            };
            cmd.creation_flags(0x08000000 | priority);
        }
        cmd
    }

    pub(super) fn run(&self, args: Vec<OsString>, cancel: &Cancellation) -> Result<Output> {
        self.run_in(args, None, cancel, None)
    }

    pub(super) fn run_in(
        &self,
        args: Vec<OsString>,
        directory: Option<&Path>,
        cancel: &Cancellation,
        progress: Option<Progress>,
    ) -> Result<Output> {
        if cancel.load(Ordering::Relaxed) {
            bail!(tr("task-cancelled"));
        }
        let mut command = self.command();
        if let Some(directory) = directory {
            command.current_dir(directory);
        }
        let mut child = command
            .args(args)
            .spawn()
            .context(tr("engine-unavailable"))?;
        let stdout = child.stdout.take().context(tr("stdout-missing"))?;
        let stderr = child.stderr.take().context(tr("stderr-missing"))?;
        // Drain both pipes while polling the process, so a large listing cannot block it.
        let read = |mut pipe: Box<dyn Read + Send>| {
            thread::spawn(move || {
                let mut bytes = Vec::new();
                pipe.read_to_end(&mut bytes).map(|_| bytes)
            })
        };
        let out = read(Box::new(stdout));
        let err = if let Some(progress) = progress {
            thread::spawn(move || progress::read(stderr, &progress))
        } else {
            read(Box::new(stderr))
        };
        let completion = (|| -> std::io::Result<_> {
            loop {
                if cancel.load(Ordering::Relaxed) {
                    let _ = child.kill();
                    break Ok((child.wait()?, true));
                }
                if let Some(status) = child.try_wait()? {
                    break Ok((status, false));
                }
                thread::sleep(Duration::from_millis(60));
            }
        })();
        if completion.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let stdout = out.join();
        let stderr = err.join();
        let (status, cancelled) = completion?;
        let stdout = stdout.map_err(|_| anyhow::anyhow!(tr("stdout-reader-stopped")))??;
        let stderr = stderr.map_err(|_| anyhow::anyhow!(tr("stderr-reader-stopped")))??;
        if cancelled {
            bail!(tr("extract-cancelled"));
        }
        let text = String::from_utf8(stdout).context(tr("engine-encoding-invalid"))?;
        match status.code() {
            Some(0) => Ok(Output {
                text,
                warning: false,
            }),
            Some(1) => Ok(Output {
                text,
                warning: true,
            }),
            code => {
                let details = format!("{text}\n{}", String::from_utf8_lossy(&stderr));
                let password_required = details.contains("Wrong password")
                    || details.contains("Password is not defined")
                    || details.contains("Data Error in encrypted file");
                Err(CommandFailure {
                    code,
                    details,
                    password_required,
                }
                .into())
            }
        }
    }

    pub fn list(&self, path: &Path, password: &str, cancel: &Cancellation) -> Result<Catalog> {
        self.list_as(path, None, password, cancel)
    }

    pub fn list_as(
        &self,
        path: &Path,
        open_type: Option<OpenType>,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<Catalog> {
        let path = std::path::absolute(path)?;
        let metadata = std::fs::metadata(&path).map_err(|source| SourceError {
            path: path.clone(),
            source,
        })?;
        let mut args = vec![
            "l".into(),
            "-slt".into(),
            "-sccUTF-8".into(),
            format!("-p{password}").into(),
        ];
        if let Some(kind) = open_type {
            args.push(format!("-t{}", kind.value()).into());
        }
        args.extend(["--".into(), path.as_os_str().into()]);
        let result = self.run(args, cancel)?;
        let text = result.text.replace("\r\n", "\n");
        let (header, records) = text
            .split_once("\n----------\n")
            .context(tr("listing-invalid"))?;
        let stream = header
            .lines()
            .any(|line| matches!(line, "Type = xz" | "Type = bzip2"));
        let stream_name = path.file_stem().unwrap_or_default().to_string_lossy();
        let mut entries = Vec::new();
        for block in records.split("\n\n") {
            let fields: BTreeMap<_, _> = block
                .lines()
                .filter_map(|line| line.split_once(" = "))
                .collect();
            let raw_path = match fields.get("Path") {
                Some(path) => *path,
                None if stream && fields.contains_key("Size") => &stream_name,
                None => continue,
            };
            let path = raw_path.replace('\\', "/").trim_end_matches('/').to_owned();
            let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
            let attributes = fields.get("Attributes").copied().unwrap_or("");
            entries.push(Entry {
                directory: fields.get("Folder") == Some(&"+") || attributes.starts_with('D'),
                link: fields.contains_key("Symbolic Link")
                    || fields.contains_key("Hard Link")
                    || attributes
                        .split_whitespace()
                        .any(|part| part.starts_with('l')),
                size: fields
                    .get("Size")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse())
                    .transpose()?,
                modified: fields.get("Modified").copied().unwrap_or("").into(),
                encrypted: fields.get("Encrypted") == Some(&"+"),
                path,
                name,
            });
        }
        Ok(Catalog {
            open_type,
            size: metadata.len(),
            format: header
                .lines()
                .find_map(|line| line.strip_prefix("Type = "))
                .context(tr("format-missing"))?
                .to_uppercase(),
            path,
            entries,
            warning: result.warning,
        })
    }

    pub fn test(&self, path: &Path, password: &str, cancel: &Cancellation) -> Result<Output> {
        self.test_as(path, None, password, cancel)
    }

    pub fn test_as(
        &self,
        path: &Path,
        open_type: Option<OpenType>,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<Output> {
        let mut args = vec![
            "t".into(),
            "-sccUTF-8".into(),
            format!("-p{password}").into(),
        ];
        if let Some(kind) = open_type {
            args.push(format!("-t{}", kind.value()).into());
        }
        args.extend(["--".into(), path.as_os_str().into()]);
        self.run(args, cancel)
    }

    pub fn extract(
        &self,
        catalog: &Catalog,
        destination: &Path,
        selected: &[String],
        password: &str,
        overwrite: Overwrite,
        cancel: &Cancellation,
        progress: Option<Progress>,
    ) -> Result<Output> {
        let entries: Vec<_> = catalog
            .entries
            .iter()
            .filter(|entry| {
                selected.is_empty()
                    || selected.iter().any(|path| {
                        entry.path == *path || entry.path.starts_with(&format!("{path}/"))
                    })
            })
            .collect();
        let protected_source =
            if matches!(overwrite, Overwrite::Replace | Overwrite::RenameExisting) {
                Some(std::fs::canonicalize(&catalog.path)?)
            } else {
                None
            };
        for entry in &entries {
            if cancel.load(Ordering::Relaxed) {
                bail!(tr("extract-cancelled"));
            }
            if entry.link
                || entry.path.starts_with('/')
                || entry.path.contains([':', '\r', '\n'])
                || entry
                    .path
                    .split('/')
                    .any(|part| part == ".." || part.ends_with([' ', '.']))
            {
                bail!(tf("unsafe-entry", &[("path", entry.path.as_str().into())]));
            }
            // Existing junctions can redirect an otherwise relative archive path.
            let target = destination.join(&entry.path);
            if let Some(source) = &protected_source {
                match std::fs::canonicalize(&target) {
                    Ok(path) if path == *source => bail!(tr("extract-source-conflict")),
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            for ancestor in target.ancestors() {
                match std::fs::symlink_metadata(ancestor) {
                    Ok(meta) if meta.file_attributes() & 0x400 != 0 => {
                        bail!(tf(
                            "unsafe-entry",
                            &[("path", ancestor.to_string_lossy().as_ref().into())]
                        ));
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
        }
        std::fs::create_dir_all(destination).context(tr("extract-directory-failed"))?;
        let mut args = vec![
            "x".into(),
            overwrite.argument().into(),
            "-y".into(),
            "-sccUTF-8".into(),
            "-spd".into(),
            format!("-p{password}").into(),
        ];
        let mut output = OsString::from("-o");
        if let Some(kind) = catalog.open_type {
            args.push(format!("-t{}", kind.value()).into());
        }
        output.push(destination);
        args.push(output);
        // A listfile avoids the Windows command-line limit on large selections.
        let selection = if selected.is_empty() {
            None
        } else {
            let mut file = tempfile::NamedTempFile::new()?;
            for entry in entries {
                writeln!(file, "{}", entry.path)?;
            }
            file.flush()?;
            let mut include = OsString::from("-i@");
            include.push(file.path());
            args.extend(["-scsUTF-8".into(), include]);
            Some(file)
        };
        args.extend(["--".into(), catalog.path.as_os_str().into()]);
        if progress.is_some() {
            args.splice(1..1, ["-bsp2".into(), "-bso1".into(), "-bse1".into()]);
        }
        let result = self.run_in(args, None, cancel, progress);
        drop(selection);
        result
    }

    pub fn create(
        &self,
        sources: &[PathBuf],
        destination: &Path,
        options: &CreateOptions,
        cancel: &Cancellation,
    ) -> Result<PathBuf> {
        let format = options.format.as_str();
        let password = options.password.as_str();
        if sources.is_empty() {
            bail!(tr("source-required"));
        }
        let replacing = destination.exists();
        if replacing && options.split != "none" {
            bail!(tr("split-update-unsupported"));
        }
        if ["gzip", "bzip2", "xz"].contains(&format) && (sources.len() != 1 || sources[0].is_dir())
        {
            bail!(tr("single-file-only"));
        }
        let parent = destination.parent().context(tr("destination-required"))?;
        let stage = tempfile::tempdir_in(parent)?;
        let filename = destination
            .file_name()
            .context(tr("file-name-missing"))?
            .to_string_lossy();
        let filename = if options.split != "none" {
            filename.strip_suffix("-volumes.zip").unwrap_or(&filename)
        } else {
            &filename
        };
        let temporary = stage.path().join(filename);
        if replacing {
            std::fs::copy(destination, &temporary)?;
        }
        let mut args = vec![
            "a".into(),
            format!("-t{format}").into(),
            format!("-mx={}", options.level).into(),
            "-sccUTF-8".into(),
            "-spd".into(),
        ];
        if !password.is_empty() {
            if !["7z", "zip"].contains(&format) {
                bail!(tr("password-unsupported"));
            }
            args.push(format!("-p{password}").into());
            if format == "7z" && options.encrypt_names {
                args.push("-mhe=on".into());
            }
            if format == "zip" {
                args.push("-mem=AES256".into());
            }
        }
        if format == "7z" {
            args.push(format!("-m0={}", options.method).into());
            args.push(format!("-ms={}", if options.solid { "on" } else { "off" }).into());
        } else if format == "zip" {
            args.push(format!("-mm={}", options.method).into());
        }
        if options.threads != "auto" {
            args.push(format!("-mmt={}", options.threads).into());
        }
        if options.split != "none" {
            args.push(format!("-v{}", options.split).into());
        }
        let inputs = input_list(sources)?;
        args.extend([
            "-scsUTF-8".into(),
            "--".into(),
            temporary.as_os_str().into(),
            list_argument("@", inputs.path()),
        ]);
        let result = self.run(args, cancel)?;
        if result.warning {
            bail!(tr("create-warning"));
        }
        let first = if options.split == "none" {
            temporary.clone()
        } else {
            PathBuf::from(format!("{}.001", temporary.display()))
        };
        if self.test(&first, password, cancel)?.warning {
            bail!(tr("integrity-warning"));
        }
        if cancel.load(Ordering::Relaxed) {
            bail!(tr("task-cancelled"));
        }
        if options.split == "none" {
            publish(&temporary, destination, replacing).context(tr("publish-failed"))?;
        } else {
            let bundle = stage.path().join("volumes.zip");
            let mut args = vec![
                "a".into(),
                "-tzip".into(),
                "-mx=0".into(),
                "-sccUTF-8".into(),
                "--".into(),
                bundle.as_os_str().into(),
            ];
            let mut volumes = std::fs::read_dir(stage.path())?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            volumes.sort();
            args.extend(volumes.iter().map(|path| path.as_os_str().into()));
            let bundled = self.run(args, cancel)?;
            if bundled.warning {
                bail!(tr("volume-warning"));
            }
            if cancel.load(Ordering::Relaxed) {
                bail!(tr("task-cancelled"));
            }
            publish(&bundle, destination, false).context(tr("volume-publish-failed"))?;
        }
        Ok(destination.to_owned())
    }
}

pub(super) fn input_list(sources: &[PathBuf]) -> Result<tempfile::NamedTempFile> {
    let mut file = tempfile::NamedTempFile::new()?;
    for path in sources {
        writeln!(file, "{}", path.display())?;
    }
    file.flush()?;
    Ok(file)
}

pub(super) fn list_argument(prefix: &str, path: &Path) -> OsString {
    let mut argument = OsString::from(prefix);
    argument.push(path);
    argument
}

pub(super) fn publish(source: &Path, destination: &Path, replace: bool) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let source: Vec<_> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<_> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // Publish only the validated staged archive; never update the original in place.
    if unsafe {
        windows_sys::Win32::Storage::FileSystem::MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            if replace {
                windows_sys::Win32::Storage::FileSystem::MOVEFILE_REPLACE_EXISTING
            } else {
                0
            },
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
