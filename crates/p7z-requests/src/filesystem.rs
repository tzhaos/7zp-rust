use anyhow::{Context, Result, bail};
use p7z_core::i18n::{tf, tr};
use p7z_engine::Entry;
use std::{
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

#[derive(Clone, Copy)]
pub struct SourceInfo {
    pub directory: bool,
    pub file: bool,
    pub size: u64,
}

pub fn inspect_source(path: &Path) -> Result<SourceInfo, String> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(SourceInfo {
            directory: metadata.is_dir(),
            file: metadata.is_file(),
            size: metadata.len(),
        }),
        Err(error) => Err(tf(
            "source-read-error",
            &[
                ("path", path.to_string_lossy().as_ref().into()),
                ("error", error.to_string().into()),
            ],
        )),
    }
}

pub fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    std::fs::copy(source, destination)?;
    Ok(())
}

#[derive(Clone)]
pub struct NameConflict {
    pub entry: String,
    pub destination: PathBuf,
    pub incoming_size: Option<u64>,
    pub incoming_modified: String,
    pub existing_size: u64,
    pub existing_modified: String,
}

pub fn name_conflicts(
    catalog: &p7z_engine::Catalog,
    selected: &[String],
    destination: &Path,
    cancel: &p7z_engine::Cancellation,
) -> Result<Vec<NameConflict>> {
    let mut conflicts = Vec::new();
    for entry in catalog
        .entries
        .iter()
        .filter(|entry| crate::extraction::entry_selected(&entry.path, selected))
    {
        if cancel.load(Ordering::Relaxed) {
            bail!(tr("extract-cancelled"));
        }
        let path = destination.join(&entry.path);
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    tf(
                        "extract-target",
                        &[("path", path.to_string_lossy().as_ref().into())],
                    )
                });
            }
        };
        if entry.directory != metadata.is_dir() {
            bail!(tf(
                "extract-type-conflict",
                &[("path", path.to_string_lossy().as_ref().into())]
            ));
        }
        if entry.directory {
            continue;
        }
        let existing_modified = local_timestamp(metadata.last_write_time()).with_context(|| {
            tf(
                "extract-target",
                &[("path", path.to_string_lossy().as_ref().into())],
            )
        })?;
        conflicts.push(NameConflict {
            entry: entry.path.clone(),
            destination: path,
            incoming_size: entry.size,
            incoming_modified: entry.modified.clone(),
            existing_size: metadata.len(),
            existing_modified,
        });
    }
    Ok(conflicts)
}
use windows_sys::Win32::{
    Foundation::{FILETIME, SYSTEMTIME},
    Storage::FileSystem::FileTimeToLocalFileTime,
    System::Time::FileTimeToSystemTime,
};

pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
}

impl Directory {
    pub fn read(path: PathBuf) -> Result<Self> {
        let path = std::path::absolute(path)?;
        let entries = std::fs::read_dir(&path)?
            .map(|item| {
                let item = item?;
                let metadata = item.metadata()?;
                Ok(Entry {
                    path: item.path().to_string_lossy().into_owned(),
                    name: item.file_name().to_string_lossy().into_owned(),
                    directory: metadata.is_dir(),
                    size: (!metadata.is_dir()).then_some(metadata.len()),
                    modified: local_timestamp(metadata.last_write_time())?,
                    link: metadata.is_symlink(),
                    encrypted: false,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { path, entries })
    }
}

fn local_timestamp(timestamp: u64) -> Result<String> {
    let filetime = FILETIME {
        dwLowDateTime: timestamp as u32,
        dwHighDateTime: (timestamp >> 32) as u32,
    };
    let mut local: FILETIME = unsafe { std::mem::zeroed() };
    let mut date: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        if FileTimeToLocalFileTime(&filetime, &mut local) == 0
            || FileTimeToSystemTime(&local, &mut date) == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.wYear, date.wMonth, date.wDay, date.wHour, date.wMinute
    ))
}
