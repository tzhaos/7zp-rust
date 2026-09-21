use anyhow::{Context, Result, bail};
use sevenzip_archive::{Cancellation, Catalog, CreateOptions, Edit, Engine, Overwrite, Progress};
use sevenzip_core::{
    i18n::{tf, tr},
    settings::Preferences,
};
use sevenzip_platform as platform;
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

#[derive(Clone)]
pub enum Request {
    Comment(Catalog, String),
    OpenEntry {
        catalog: Catalog,
        path: String,
        preferences: Preferences,
    },
    Open(PathBuf),
    OpenAs(PathBuf, sevenzip_shell_api::OpenType),
    QuickCompress {
        paths: Vec<PathBuf>,
        format: sevenzip_shell_api::ArchiveFormat,
        email: bool,
    },
    Extract {
        catalog: Catalog,
        selected: Vec<String>,
        parent: PathBuf,
        folder: String,
        overwrite: Overwrite,
        open_after: bool,
    },
    Check(Catalog),
    Edit(Catalog, Edit),
    Transfer {
        catalog: Catalog,
        selected: Vec<String>,
        destination: PathBuf,
        move_entries: bool,
    },
    Create {
        files: Vec<PathBuf>,
        destination: PathBuf,
        options: CreateOptions,
        email: bool,
    },
    ReadComment(Catalog),
    ResolveAddress(PathBuf),
    Checksum {
        paths: Vec<PathBuf>,
        kind: ChecksumKind,
    },
}

#[derive(Clone, Copy)]
pub enum ChecksumKind {
    Hash(sevenzip_shell_api::HashMethod),
    Generate,
    Verify,
}

pub enum Outcome {
    Comment(String),
    External(tempfile::TempDir),
    Opened(Catalog, String),
    Directory(super::filesystem::Directory),
    Address(Catalog, String),
    AddressPassword(Request, String),
    Extracted(PathBuf, bool, Option<String>),
    Created(Catalog, String),
    Message(String),
    Report(String, String),
    Saved(PathBuf, String),
    Moved(Catalog, String, PathBuf, Option<String>),
    Password(Request),
    Dispatch(Request),
    Cancelled,
}

impl Request {
    pub fn label(&self) -> &'static str {
        tr(match self {
            Self::Comment(..) | Self::Edit(..) => "archive-editing",
            Self::OpenEntry { .. } | Self::Open(_) | Self::OpenAs(..) => "opening",
            Self::QuickCompress { .. } | Self::Create { .. } => "creating",
            Self::Extract { .. } | Self::Transfer { .. } => "extracting",
            Self::Check(_) => "checking",
            Self::ReadComment(_) => "archive-comment",
            Self::ResolveAddress(_) => "browser-loading",
            Self::Checksum { .. } => "shell-checksums",
        })
    }

    pub fn extraction_paths(&self) -> Option<(PathBuf, Option<PathBuf>)> {
        match self {
            Self::Extract {
                catalog,
                parent,
                folder,
                ..
            } => Some((catalog.path.clone(), Some(parent.join(folder)))),
            Self::Transfer {
                catalog,
                destination,
                ..
            } => Some((catalog.path.clone(), Some(destination.clone()))),
            Self::OpenEntry { catalog, .. } => Some((catalog.path.clone(), None)),
            _ => None,
        }
    }

    pub fn run(
        self,
        password: String,
        cancel: &Cancellation,
        progress: Option<Progress>,
    ) -> Result<Outcome> {
        let result = if let Self::ReadComment(catalog) = &self {
            Engine::comment(catalog).map(Outcome::Comment)
        } else {
            let engine = Engine::bundled()?;
            self.perform(&engine, &password, cancel, progress)
        };
        match result {
            Err(error)
                if self.prompts_for_password() && sevenzip_archive::needs_password(&error) =>
            {
                Ok(Outcome::Password(self))
            }
            result => result,
        }
    }

    fn prompts_for_password(&self) -> bool {
        !matches!(
            self,
            Self::Create { .. }
                | Self::ReadComment(_)
                | Self::ResolveAddress(_)
                | Self::Checksum { .. }
        )
    }

    fn perform(
        &self,
        engine: &Engine,
        password: &str,
        cancel: &Cancellation,
        progress: Option<Progress>,
    ) -> Result<Outcome> {
        match self {
            Self::Comment(catalog, text) => engine
                .write_comment(catalog, text, password, cancel)
                .map(|catalog| Outcome::Opened(catalog, password.into())),
            Self::Edit(catalog, edit) => engine
                .edit(catalog, edit, password, cancel)
                .map(|catalog| Outcome::Opened(catalog, password.into())),
            Self::Open(path) => engine
                .list(path, password, cancel)
                .map(|catalog| Outcome::Opened(catalog, password.into())),
            Self::OpenAs(path, kind) => engine
                .list_as(path, Some(*kind), password, cancel)
                .map(|catalog| Outcome::Opened(catalog, password.into())),
            Self::OpenEntry {
                catalog,
                path,
                preferences,
            } => open_entry(
                engine,
                catalog,
                path,
                preferences,
                password,
                cancel,
                progress,
            ),
            Self::QuickCompress {
                paths,
                format,
                email,
            } => {
                let path = engine.quick_compress(paths, *format, password, cancel)?;
                if *email {
                    compose_archive(&path)?;
                }
                Ok(Outcome::Created(
                    engine.list(&path, password, cancel)?,
                    password.into(),
                ))
            }
            Self::Check(catalog) => {
                let output = engine.test_as(&catalog.path, catalog.open_type, password, cancel)?;
                Ok(Outcome::Message(if output.warning {
                    tr("check-warning").into()
                } else {
                    tf(
                        "check-complete",
                        &[(
                            "count",
                            catalog
                                .entries
                                .iter()
                                .filter(|e| !e.directory)
                                .count()
                                .into(),
                        )],
                    )
                }))
            }
            Self::Extract {
                catalog,
                selected,
                parent,
                folder,
                overwrite,
                open_after,
            } => {
                if folder == "." || folder == ".." || folder.contains(['/', '\\', ':']) {
                    bail!(tr("folder-name-invalid"));
                }
                let destination = parent.join(folder);
                let output = engine
                    .extract(
                        catalog,
                        &destination,
                        selected,
                        password,
                        *overwrite,
                        cancel,
                        progress,
                    )
                    .with_context(|| {
                        tf(
                            "extract-target",
                            &[("path", destination.display().to_string().into())],
                        )
                    })?;
                let open_error = if *open_after {
                    open_destination(&destination)
                } else {
                    None
                };
                Ok(Outcome::Extracted(destination, output.warning, open_error))
            }
            Self::ReadComment(catalog) => Engine::comment(catalog).map(Outcome::Comment),
            Self::Create {
                files,
                destination,
                options,
                email,
            } => {
                let path = engine.create(files, destination, options, cancel)?;
                if *email {
                    compose_archive(&path)?;
                }
                let password = options.list_password().to_owned();
                Ok(Outcome::Created(
                    engine.list(&path, &password, cancel)?,
                    password,
                ))
            }
            Self::ResolveAddress(path) => resolve_address(engine, path.clone(), cancel),
            Self::Checksum { paths, kind } => run_checksum(engine, paths, *kind, cancel),
            Self::Transfer {
                catalog,
                selected,
                destination,
                move_entries,
            } => {
                let output = engine.extract(
                    catalog,
                    destination,
                    selected,
                    password,
                    Overwrite::RenameIncoming,
                    cancel,
                    progress,
                )?;
                let updated = if *move_entries {
                    if output.warning {
                        bail!(tr("archive-move-warning"));
                    }
                    Some(engine.edit(catalog, &Edit::Delete(selected.clone()), password, cancel)?)
                } else {
                    None
                };
                let open_error = open_destination(destination);
                Ok(match updated {
                    Some(catalog) => {
                        Outcome::Moved(catalog, password.into(), destination.clone(), open_error)
                    }
                    None => Outcome::Extracted(destination.clone(), output.warning, open_error),
                })
            }
        }
    }
}

fn resolve_address(engine: &Engine, path: PathBuf, cancel: &Cancellation) -> Result<Outcome> {
    let path = std::path::absolute(&path)?;
    let mut archive = path.clone();
    loop {
        match std::fs::metadata(&archive) {
            Ok(metadata) if metadata.is_dir() && archive == path => {
                return super::filesystem::Directory::read(path).map(Outcome::Directory);
            }
            Ok(metadata) if metadata.is_file() => break,
            Ok(_) => bail!(tr("browser-path-missing")),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                if !archive.pop() {
                    return Err(error.into());
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    let folder = path
        .strip_prefix(&archive)?
        .to_string_lossy()
        .replace('\\', "/");
    match engine.list(&archive, "", cancel) {
        Ok(catalog) => Ok(Outcome::Address(catalog, folder)),
        Err(error) if sevenzip_archive::needs_password(&error) => {
            Ok(Outcome::AddressPassword(Request::Open(archive), folder))
        }
        Err(error) => Err(error),
    }
}

fn run_checksum(
    engine: &Engine,
    paths: &[PathBuf],
    kind: ChecksumKind,
    cancel: &Cancellation,
) -> Result<Outcome> {
    match kind {
        ChecksumKind::Generate => Ok(Outcome::Saved(
            engine.generate_checksum(paths, cancel)?,
            tr("checksum-saved").into(),
        )),
        ChecksumKind::Hash(method) => Ok(checksum_report(engine.checksum(paths, method, cancel)?)),
        ChecksumKind::Verify => Ok(checksum_report(engine.verify_checksums(paths, cancel)?)),
    }
}

fn checksum_report(output: sevenzip_archive::Output) -> Outcome {
    Outcome::Report(
        tr(if output.warning {
            "checksum-warning"
        } else {
            "shell-checksums"
        })
        .into(),
        output.text,
    )
}

fn compose_archive(path: &Path) -> Result<()> {
    platform::mail::compose(path).with_context(|| {
        tf(
            "mail-archive-saved",
            &[("path", path.display().to_string().into())],
        )
    })
}

fn open_destination(path: &Path) -> Option<String> {
    platform::open_file(path).err().map(|error| {
        tf(
            "extract-open-failed",
            &[("error", error.to_string().into())],
        )
    })
}

fn open_entry(
    engine: &Engine,
    catalog: &Catalog,
    path: &str,
    preferences: &Preferences,
    password: &str,
    cancel: &Cancellation,
    progress: Option<Progress>,
) -> Result<Outcome> {
    let root = if preferences.temp_directory.is_empty() {
        std::env::temp_dir()
    } else {
        PathBuf::from(&preferences.temp_directory)
    };
    let directory = tempfile::Builder::new()
        .prefix("7zplus-open-")
        .tempdir_in(root)?;
    let extension = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let all = preferences
        .extract_all
        .split(';')
        .map(|s| s.trim().trim_start_matches("*.").trim_start_matches('.'))
        .any(|s| !s.is_empty() && s.eq_ignore_ascii_case(extension));
    let selection = if all {
        Vec::new()
    } else {
        vec![path.to_owned()]
    };
    let output = engine.extract(
        catalog,
        directory.path(),
        &selection,
        password,
        Overwrite::Replace,
        cancel,
        progress,
    )?;
    if output.warning {
        bail!(tr("integrity-warning"));
    }
    if ["exe", "com", "msi", "bat", "cmd", "ps1", "vbs", "js", "scr"]
        .iter()
        .any(|s| s.eq_ignore_ascii_case(extension))
    {
        let answer = rfd::MessageDialog::new()
            .set_title(tr("browser-open"))
            .set_description(tf("file-run-confirm", &[("path", path.into())]))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if answer != rfd::MessageDialogResult::Yes {
            return Ok(Outcome::Cancelled);
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Ok(Outcome::Cancelled);
    }
    platform::open_file(&directory.path().join(path))?;
    Ok(Outcome::External(directory))
}
