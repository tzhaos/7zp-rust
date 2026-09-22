use anyhow::{Result, bail};
use std::{os::windows::ffi::OsStrExt, path::PathBuf, ptr};
use windows_sys::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
        DeleteDC, DeleteObject, GdiFlush, SelectObject,
    },
    Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL},
    System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
    UI::{
        Shell::{
            SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES, SHGetFileInfoW,
        },
        WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx, HICON},
    },
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum IconSource {
    Path(PathBuf),
    FileType(String),
    Directory,
}

struct ComApartment(bool);

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

struct ShellIcon(HICON);

impl Drop for ShellIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.0) };
    }
}

pub const ICON_SIZE: u32 = 32;

pub fn load(source: &IconSource) -> Result<Vec<u8>> {
    let result = unsafe { CoInitializeEx(ptr::null(), COINIT_MULTITHREADED as u32) };
    if result < 0 && result != RPC_E_CHANGED_MODE {
        bail!("CoInitializeEx failed: {result:#x}");
    }
    let _apartment = ComApartment(result >= 0);
    let (path, attributes, flags) = match source {
        IconSource::Path(path) => (path.clone(), 0, 0),
        IconSource::Directory => (
            PathBuf::from("folder"),
            FILE_ATTRIBUTE_DIRECTORY,
            SHGFI_USEFILEATTRIBUTES,
        ),
        IconSource::FileType(extension) => (
            PathBuf::from(if extension.is_empty() {
                "file".to_owned()
            } else {
                format!("file.{extension}")
            }),
            FILE_ATTRIBUTE_NORMAL,
            SHGFI_USEFILEATTRIBUTES,
        ),
    };
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let result = unsafe {
        SHGetFileInfoW(
            path.as_ptr(),
            attributes,
            &mut info,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | flags,
        )
    };
    if result == 0 {
        bail!("SHGetFileInfoW failed");
    }
    let icon = ShellIcon(info.hIcon);
    let mut black = draw(icon.0, 0)?;
    let white = draw(icon.0, 255)?;
    // Recover straight alpha from Shell's black/white composites, including mask-based icons.
    for (pixel, light) in black.chunks_exact_mut(4).zip(white.chunks_exact(4)) {
        let alpha = 255 - light[0].saturating_sub(pixel[0]);
        for channel in &mut pixel[..3] {
            *channel = if alpha == 0 {
                0
            } else {
                ((*channel as u32 * 255) / alpha as u32).min(255) as u8
            };
        }
        pixel[3] = alpha;
    }
    Ok(black)
}

fn draw(icon: HICON, background: u8) -> Result<Vec<u8>> {
    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: ICON_SIZE as i32,
        biHeight: -(ICON_SIZE as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..unsafe { std::mem::zeroed() }
    };
    unsafe {
        let dc = CreateCompatibleDC(ptr::null_mut());
        if dc.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        let mut pixels = ptr::null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, ptr::null_mut(), 0);
        if bitmap.is_null() {
            let error = std::io::Error::last_os_error();
            DeleteDC(dc);
            return Err(error.into());
        }
        let previous = SelectObject(dc, bitmap);
        let bytes = std::slice::from_raw_parts_mut(
            pixels.cast::<u8>(),
            (ICON_SIZE * ICON_SIZE * 4) as usize,
        );
        bytes.fill(background);
        let drawn = DrawIconEx(
            dc,
            0,
            0,
            icon,
            ICON_SIZE as i32,
            ICON_SIZE as i32,
            0,
            ptr::null_mut(),
            DI_NORMAL,
        );
        let result = if drawn == 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            GdiFlush();
            Ok(bytes.to_vec())
        };
        SelectObject(dc, previous);
        DeleteObject(bitmap);
        DeleteDC(dc);
        result
    }
}
