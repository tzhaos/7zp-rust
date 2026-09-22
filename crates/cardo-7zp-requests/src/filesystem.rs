use anyhow::Result;
use cardo_7zp_engine::Entry;
use cardo_7zp_core::i18n::tf;
use std::{
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
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
    pub incoming_size: u64,
    pub incoming_modified: String,
    pub existing_size: u64,
    pub existing_modified: String,
}

pub fn name_conflicts(
    catalog: &cardo_7zp_engine::Catalog,
    selected: &[String],
    destination: &Path,
) -> Vec<NameConflict> {
    catalog
        .entries
        .iter()
        .filter(|entry| !entry.directory && entry_selected(&entry.path, selected))
        .filter_map(|entry| {
            let path = destination.join(&entry.path);
            let metadata = std::fs::metadata(&path).ok()?;
            if metadata.is_dir() {
                return None;
            }
            Some(NameConflict {
                entry: entry.path.clone(),
                destination: path,
                incoming_size: entry.size.unwrap_or(0),
                incoming_modified: entry.modified.clone(),
                existing_size: metadata.len(),
                existing_modified: local_timestamp(metadata.last_write_time())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

fn entry_selected(path: &str, selected: &[String]) -> bool {
    selected.is_empty()
        || selected
            .iter()
            .any(|item| path == item || path.starts_with(&format!("{item}/")))
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
