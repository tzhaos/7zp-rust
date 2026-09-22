use anyhow::Result;
use cardo_7zp_core::i18n::{tf, tr};
use cardo_7zp_commands::{ACTIONS, CLSID, EXTENSIONS, SHELL_KEY};
use std::path::Path;
use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE},
};

const PROGID: &str = "Cardo.7zplus.Rust.Archive";
const APPLICATION: &str = "7zplus.Rust";
const CAPABILITIES: &str = r"Software\7zplus.Rust\Capabilities";
pub const DEFAULT_APPS_URI: &str = "ms-settings:defaultapps";
const MENU_KEY: &str = r"AllFilesystemObjects\shell\7zplus.Rust";

pub fn register(executable: &Path, dll: &Path) -> Result<()> {
    let preferences = cardo_7zp_core::settings::load_preferences()?;
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let classes = user.create_subkey(r"Software\Classes")?.0;
    let exe = executable.display().to_string();
    // Remove the replaced static verbs so upgrades cannot leave the old universal menu active.
    for path in [r"*\shell\7zplus.Rust", r"Directory\shell\7zplus.Rust"] {
        remove_key(&classes, path)?;
    }
    let server = classes
        .create_subkey(format!(r"CLSID\{CLSID}\InprocServer32"))?
        .0;
    server.set_value("", &dll.to_string_lossy().as_ref())?;
    server.set_value("ThreadingModel", &"Apartment")?;
    let config = user.create_subkey(SHELL_KEY)?.0;
    config.set_value("Executable", &exe)?;
    for key in [
        "archive-open-type",
        "shell-email-menu",
        "extract-each-folder",
    ] {
        config.set_value(key, &tr(key))?;
    }
    for action in ACTIONS {
        use cardo_7zp_commands::Action;
        let label = match action {
            Action::OpenAs(kind) => kind.value().to_owned(),
            Action::Checksum(method) => method.title().to_owned(),
            Action::QuickCompress(..) | Action::ExtractFolder | Action::GenerateChecksum => {
                tf(action.label_key(), &[("name", "{name}".into())])
            }
            _ => tr(action.label_key()).to_owned(),
        };
        config.set_value(action.argument(), &label)?;
    }
    configure(executable, &preferences)?;
    Ok(())
}

pub fn configure(
    executable: &Path,
    preferences: &cardo_7zp_core::settings::Preferences,
) -> Result<()> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let classes = user.create_subkey(r"Software\Classes")?.0;
    let exe = executable.display().to_string();
    let prog = classes.create_subkey(PROGID)?.0;
    prog.set_value("", &tr("archive-association"))?;
    prog.create_subkey("DefaultIcon")?
        .0
        .set_value("", &format!("\"{exe}\",0"))?;
    prog.create_subkey(r"shell\open\command")?
        .0
        .set_value("", &format!("\"{exe}\" --open \"%1\""))?;
    let capabilities = user.create_subkey(CAPABILITIES)?.0;
    capabilities.set_value("ApplicationName", &"7zplus")?;
    capabilities.set_value("ApplicationDescription", &tr("association-description"))?;
    capabilities.set_value("ApplicationIcon", &format!("\"{exe}\",0"))?;
    remove_key(&capabilities, "FileAssociations")?;
    let associations = capabilities.create_subkey("FileAssociations")?.0;
    for extension in EXTENSIONS {
        if preferences
            .associations
            .iter()
            .any(|value| value == extension)
        {
            classes
                .create_subkey(format!(r"{extension}\OpenWithProgids"))?
                .0
                .set_raw_value(
                    PROGID,
                    &winreg::RegValue {
                        vtype: winreg::enums::REG_NONE,
                        bytes: Vec::new(),
                    },
                )?;
            associations.set_value(extension, &PROGID)?;
        } else {
            match classes.open_subkey_with_flags(format!(r"{extension}\OpenWithProgids"), KEY_WRITE)
            {
                Ok(key) => match key.delete_value(PROGID) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
    user.create_subkey(r"Software\RegisteredApplications")?
        .0
        .set_value(APPLICATION, &CAPABILITIES)?;
    if preferences.shell_menu {
        let menu = classes.create_subkey(MENU_KEY)?.0;
        menu.set_value("MUIVerb", &"7zplus")?;
        menu.set_value("Icon", &format!("\"{exe}\",0"))?;
        menu.set_value("ExplorerCommandHandler", &CLSID)?;
        menu.set_value("MultiSelectModel", &"Player")?;
        menu.set_value("CommandStateSync", &"")?;
    } else {
        remove_key(&classes, MENU_KEY)?;
    }
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
    Ok(())
}

pub fn unregister() -> Result<()> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let classes = user.open_subkey_with_flags(r"Software\Classes", KEY_READ | KEY_WRITE)?;
    let command = match classes.open_subkey(format!(r"{PROGID}\shell\open\command")) {
        Ok(key) => key.get_value::<String, _>("")?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let executable = std::env::current_exe()?;
    if command != format!("\"{}\" --open \"%1\"", executable.display()) {
        return Ok(());
    }
    for path in [
        r"*\shell\7zplus.Rust",
        r"Directory\shell\7zplus.Rust",
        MENU_KEY,
    ] {
        remove_key(&classes, path)?;
    }
    remove_key(&classes, &format!(r"CLSID\{CLSID}"))?;
    remove_key(&user, SHELL_KEY)?;
    for extension in EXTENSIONS {
        match classes.open_subkey_with_flags(
            format!(r"{extension}\OpenWithProgids"),
            KEY_READ | KEY_WRITE,
        ) {
            Ok(key) => match key.delete_value(PROGID) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    match user.open_subkey_with_flags(r"Software\RegisteredApplications", KEY_READ | KEY_WRITE) {
        Ok(key) => match key.delete_value(APPLICATION) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    match user.delete_subkey_all(CAPABILITIES) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    classes.delete_subkey_all(PROGID)?;
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    Ok(())
}

fn remove_key(parent: &RegKey, path: &str) -> Result<()> {
    match parent.delete_subkey_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
