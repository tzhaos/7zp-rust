use anyhow::Result;
use sevenzip_archive::Entry;
use sevenzip_core::i18n::tf;
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
