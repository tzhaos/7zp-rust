use anyhow::{Context, Result, bail};
use zip_commands::{ACTIONS, CLSID, EXTENSIONS, SHELL_KEY};
use zip_core::i18n::{tf, tr};
use cardo_runtime::registry::{Ownership, ownership, remove_key, remove_value, string};
use std::path::Path;
use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const PROGID: &str = "Cardo.Plus7z.Archive";
const APPLICATION: &str = "Plus7z";
const CAPABILITIES: &str = r"Software\Plus7z\Capabilities";
pub const DEFAULT_APPS_URI: &str = "ms-settings:defaultapps";
const MENU_KEY: &str = r"AllFilesystemObjects\shell\Plus7z";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct UpdateRegistration {
    pub dll: std::path::PathBuf,
    pub version: Option<String>,
}

pub(crate) fn update_registration(directory: &Path) -> Result<Option<UpdateRegistration>> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let Some(registered) = string(&user, r"Software\Plus7z", "InstallDir")? else {
        return Ok(None);
    };
    let installed = match std::fs::canonicalize(&registered) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect installation {registered}"));
        }
    };
    if installed != std::fs::canonicalize(directory)? {
        return Ok(None);
    }
    let executable =
        string(&user, SHELL_KEY, "Executable")?.context(tr("update-ownership-error"))?;
    if std::fs::canonicalize(executable)? != std::fs::canonicalize(directory.join("p7z.exe"))? {
        bail!(tr("update-ownership-error"));
    }
    let dll = string(
        &user,
        &format!(r"Software\Classes\CLSID\{CLSID}\InprocServer32"),
        "",
    )?
    .context(tr("update-ownership-error"))?;
    let dll = std::path::PathBuf::from(dll);
    if !std::fs::canonicalize(&dll)?.starts_with(installed) {
        bail!(tr("update-ownership-error"));
    }
    Ok(Some(UpdateRegistration {
        dll,
        version: string(
            &user,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z",
            "DisplayVersion",
        )?,
    }))
}

pub(crate) fn restore_update_registration(
    directory: &Path,
    previous: &UpdateRegistration,
) -> Result<()> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let installed = string(&user, r"Software\Plus7z", "InstallDir")?
        .context(tr("update-ownership-error"))?;
    let directory = std::fs::canonicalize(directory)?;
    if std::fs::canonicalize(installed)? != directory {
        bail!(tr("update-ownership-error"));
    }
    if let Some(executable) = string(&user, SHELL_KEY, "Executable")? {
        if std::fs::canonicalize(executable)?
            != std::fs::canonicalize(directory.join("p7z.exe"))?
        {
            bail!(tr("update-ownership-error"));
        }
    }
    if let Some(dll) = string(
        &user,
        &format!(r"Software\Classes\CLSID\{CLSID}\InprocServer32"),
        "",
    )? {
        let dll = std::path::PathBuf::from(dll);
        let parent = dll.parent().context(tr("update-ownership-error"))?;
        if !std::fs::canonicalize(parent)?.starts_with(&directory) {
            bail!(tr("update-ownership-error"));
        }
    }
    register(&directory.join("p7z.exe"), &previous.dll)?;
    let key = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z";
    if let Some(version) = &previous.version {
        user.open_subkey_with_flags(key, winreg::enums::KEY_SET_VALUE)?
            .set_value("DisplayVersion", version)?;
    } else {
        remove_value(&user, key, "DisplayVersion")?;
    }
    Ok(())
}

pub fn register(executable: &Path, dll: &Path) -> Result<()> {
    let dll = std::path::absolute(dll)?;
    if !extension_belongs_to(&dll, executable) {
        bail!(tr("registration-extension-location"));
    }
    if !std::fs::metadata(&dll)
        .with_context(|| format!("Cannot inspect Explorer extension {}", dll.display()))?
        .is_file()
    {
        bail!("Explorer extension is not a file: {}", dll.display());
    }
    register_files(executable, &dll).with_context(|| {
        format!(
            "Cannot register Windows integration for {}",
            executable.display()
        )
    })
}

fn register_files(executable: &Path, dll: &Path) -> Result<()> {
    let preferences = zip_core::settings::load_preferences()?;
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let classes = user.create_subkey(r"Software\Classes")?.0;
    let exe = executable.display().to_string();
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
        use zip_commands::Action;
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
    write_configuration(executable, &preferences)?;
    Ok(())
}

pub fn configure(
    executable: &Path,
    preferences: &zip_core::settings::Preferences,
) -> Result<()> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let (association, shell) = owners(&user, executable)?;
    if association == Ownership::Other || shell == Ownership::Other {
        bail!(tr("registration-other-installation"));
    }
    if preferences.shell_menu {
        let server = string(
            &user,
            &format!(r"Software\Classes\CLSID\{CLSID}\InprocServer32"),
            "",
        )?;
        let executable_owner = string(&user, SHELL_KEY, "Executable")?;
        if shell != Ownership::Current || server.is_none() || executable_owner.is_none() {
            bail!(tr("registration-shell-required"));
        }
    }
    write_configuration(executable, preferences).with_context(|| {
        format!(
            "Cannot configure Windows integration for {}",
            executable.display()
        )
    })
}

fn write_configuration(
    executable: &Path,
    preferences: &zip_core::settings::Preferences,
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
    capabilities.set_value("ApplicationName", &"Plus7z")?;
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
            remove_value(&classes, &format!(r"{extension}\OpenWithProgids"), PROGID)?;
        }
    }
    user.create_subkey(r"Software\RegisteredApplications")?
        .0
        .set_value(APPLICATION, &CAPABILITIES)?;
    if preferences.shell_menu {
        let menu = classes.create_subkey(MENU_KEY)?.0;
        menu.set_value("MUIVerb", &"Plus7z")?;
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
    let executable = std::env::current_exe()?;
    let (association, shell) = owners(&user, &executable)?;
    if shell == Ownership::Current {
        remove_key(&user, &format!(r"Software\Classes\{MENU_KEY}"))?;
        remove_key(&user, &format!(r"Software\Classes\CLSID\{CLSID}"))?;
        // Keep the executable identity until dependent entries have been removed.
        remove_key(&user, SHELL_KEY)?;
    }
    if association == Ownership::Current {
        for extension in EXTENSIONS {
            remove_value(
                &user,
                &format!(r"Software\Classes\{extension}\OpenWithProgids"),
                PROGID,
            )?;
        }
        remove_value(&user, r"Software\RegisteredApplications", APPLICATION)?;
        remove_key(&user, CAPABILITIES)?;
        remove_key(&user, &format!(r"Software\Classes\{PROGID}"))?;
    }
    if association == Ownership::Other || shell == Ownership::Other {
        tracing::info!("Preserved registration owned by another installation");
    }
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

fn owners(user: &RegKey, executable: &Path) -> Result<(Ownership, Ownership)> {
    let exe = executable.display().to_string();
    let icon = format!("\"{exe}\",0");
    let association = ownership(
        user,
        &[
            (
                &format!(r"Software\Classes\{PROGID}\shell\open\command"),
                "",
                &format!("\"{exe}\" --open \"%1\""),
            ),
            (
                &format!(r"Software\Classes\{PROGID}\DefaultIcon"),
                "",
                &icon,
            ),
            (CAPABILITIES, "ApplicationIcon", &icon),
        ],
    )?;
    let mut shell = ownership(
        user,
        &[
            (SHELL_KEY, "Executable", &exe),
            (&format!(r"Software\Classes\{MENU_KEY}"), "Icon", &icon),
        ],
    )?;
    if let Some(server) = string(
        user,
        &format!(r"Software\Classes\CLSID\{CLSID}\InprocServer32"),
        "",
    )? {
        if !extension_belongs_to(Path::new(&server), executable) {
            shell = Ownership::Other;
        } else if shell == Ownership::Missing {
            shell = Ownership::Current;
        }
    }
    Ok((association, shell))
}

fn extension_belongs_to(library: &Path, executable: &Path) -> bool {
    let Some(directory) = executable.parent() else {
        return false;
    };
    let mut components = library.components();
    directory.components().all(|part| {
        components
            .next()
            .is_some_and(|candidate| candidate.as_os_str().eq_ignore_ascii_case(part.as_os_str()))
    }) && components.next().is_some()
        && !library
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
}
