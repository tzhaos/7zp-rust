use anyhow::{Result, bail};
use std::{os::windows::ffi::OsStrExt, path::Path};

pub fn open_directory(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()?;
    Ok(true)
}
use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

pub fn open_file(path: &Path) -> Result<()> {
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let operation: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    if result <= 32 {
        bail!(p7z_core::i18n::tf(
            "file-open-failed",
            &[("code", result.to_string().into())]
        ));
    }
    Ok(())
}
