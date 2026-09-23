use anyhow::{Result, bail};
use zip_core::i18n::tr;
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
};
use windows_sys::Win32::{
    Foundation::{BOOL, HWND, LPARAM, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        QueryFullProcessImageNameW, WaitForSingleObject,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindowVisible, PostMessageW,
        WM_CLOSE,
    },
};

struct ApplicationWindows {
    executable: String,
    windows: Vec<(HWND, OwnedHandle)>,
}

unsafe extern "system" fn find_window(window: HWND, state: LPARAM) -> BOOL {
    if unsafe { IsWindowVisible(window) } == 0 || !unsafe { GetWindow(window, GW_OWNER) }.is_null()
    {
        return 1;
    }
    let state = unsafe { &mut *(state as *mut ApplicationWindows) };
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, &mut process_id) };
    let process = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            process_id,
        )
    };
    if process.is_null() {
        return 1;
    }
    let process = unsafe { OwnedHandle::from_raw_handle(process) };
    let mut path = vec![0u16; 32768];
    let mut length = path.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(process.as_raw_handle(), 0, path.as_mut_ptr(), &mut length)
    } != 0
        && String::from_utf16_lossy(&path[..length as usize])
            .eq_ignore_ascii_case(&state.executable)
    {
        state.windows.push((window, process));
    }
    1
}

pub fn close_application(directory: &Path) -> Result<()> {
    let mut state = ApplicationWindows {
        executable: directory.join("p7z.exe").to_string_lossy().into_owned(),
        windows: Vec::new(),
    };
    if unsafe { EnumWindows(Some(find_window), &mut state as *mut _ as LPARAM) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    for (window, _) in &state.windows {
        unsafe { PostMessageW(*window, WM_CLOSE, 0, 0) };
    }
    // WM_CLOSE lets the workspace refuse to exit while an archive task is running.
    for (_, process) in state.windows {
        match unsafe { WaitForSingleObject(process.as_raw_handle(), 5000) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => bail!(tr("maintenance-app-busy")),
            _ => return Err(std::io::Error::last_os_error().into()),
        }
    }
    Ok(())
}
