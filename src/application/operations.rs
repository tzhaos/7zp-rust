use crate::{
    archive::{Cancellation, Catalog, Edit, Engine, Overwrite, Progress},
    i18n::{tf, tr},
    platform,
    settings::Preferences,
};
use anyhow::{Context, Result, bail};
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
    Cancelled,
}

impl Request {
    pub fn label(&self) -> &'static str {
        tr(match self {
            Self::Comment(..) | Self::Edit(..) => "archive-editing",
            Self::OpenEntry { .. } | Self::Open(_) | Self::OpenAs(..) => "opening",
            Self::QuickCompress { .. } => "creating",
            Self::Extract { .. } | Self::Transfer { .. } => "extracting",
            Self::Check(_) => "checking",
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
        let engine = Engine::bundled()?;
        match self.perform(&engine, &password, cancel, progress) {
            Err(error) if crate::archive::needs_password(&error) => Ok(Outcome::Password(self)),
            result => result,
        }
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
