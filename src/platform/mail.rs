use crate::i18n::{tf, tr};
use anyhow::{Context, Result, bail};
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::{
    Foundation::FreeLibrary,
    System::{
        LibraryLoader::{GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW},
        Mapi::{MAPI_DIALOG, MAPI_LOGON_UI, MAPI_USER_ABORT, MapiFileDescW, MapiMessageW},
    },
};

/// Open the configured Simple MAPI client's compose dialog, with an attachment.
pub fn compose(attachment: &Path) -> Result<()> {
    let library: Vec<_> = "mapi32.dll".encode_utf16().chain(Some(0)).collect();
    let module = unsafe {
        LoadLibraryExW(
            library.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    if module.is_null() {
        bail!(tr("mail-unavailable"));
    }
    let result = (|| {
        type SendMail = unsafe extern "system" fn(usize, usize, *mut MapiMessageW, u32, u32) -> u32;
        let address = unsafe { GetProcAddress(module, c"MAPISendMailW".as_ptr().cast()) }
            .context(tr("mail-unavailable"))?;
        // The signature and pointer lifetimes follow mapi.h; the call is synchronous.
        let send: SendMail = unsafe { std::mem::transmute(address) };
        let mut path: Vec<_> = attachment
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut name: Vec<_> = attachment
            .file_name()
            .unwrap_or_default()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut file = MapiFileDescW {
            ulReserved: 0,
            flFlags: 0,
            nPosition: u32::MAX,
            lpszPathName: path.as_mut_ptr(),
            lpszFileName: name.as_mut_ptr(),
            lpFileType: std::ptr::null_mut(),
        };
        let mut message: MapiMessageW = unsafe { std::mem::zeroed() };
        message.lpszSubject = name.as_mut_ptr();
        message.nFileCount = 1;
        message.lpFiles = &mut file;
        let code = unsafe { send(0, 0, &mut message, MAPI_DIALOG | MAPI_LOGON_UI, 0) };
        if code != 0 && code != MAPI_USER_ABORT {
            bail!(tf("mail-failed", &[("code", code.into())]));
        }
        Ok(())
    })();
    unsafe {
        FreeLibrary(module);
    }
    result
}
