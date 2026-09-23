use zip_commands::{
    ACTIONS, Action, ArchiveFormat, CLSID_VALUE, REQUEST_PREFIX, Request, SHELL_KEY, archive_name,
    extract_folder, may_extract,
};
use std::{
    cell::Cell,
    ffi::c_void,
    path::PathBuf,
    process::Command as Process,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::{
            CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_INVALIDARG, E_NOTIMPL,
            E_POINTER, S_FALSE, S_OK,
        },
        System::{
            Com::{CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl},
            SystemServices::{SFGAO_FILESYSTEM, SFGAO_FOLDER, SFGAO_STREAM},
        },
        UI::Shell::{
            ECF_DEFAULT, ECF_HASSUBCOMMANDS, ECF_SEPARATORBEFORE, ECS_ENABLED, ECS_HIDDEN,
            IEnumExplorerCommand, IEnumExplorerCommand_Impl, IExplorerCommand,
            IExplorerCommand_Impl, IShellItemArray, SHStrDupW, SIGDN_FILESYSPATH,
        },
    },
    core::{
        BOOL, Error, GUID, HRESULT, IUnknown, Interface, PCWSTR, PWSTR, Ref, Result, implement,
    },
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

static OBJECTS: AtomicUsize = AtomicUsize::new(0);
static LOCKS: AtomicUsize = AtomicUsize::new(0);

struct ModuleGuard;
impl ModuleGuard {
    fn new() -> Self {
        OBJECTS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}
impl Drop for ModuleGuard {
    fn drop(&mut self) {
        OBJECTS.fetch_sub(1, Ordering::Release);
    }
}

struct Configuration {
    executable: PathBuf,
    titles: Vec<String>,
    open_types_title: String,
    email_title: String,
    extract_each_title: String,
}
impl Configuration {
    fn load() -> Result<Arc<Self>> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(SHELL_KEY)?;
        Ok(Arc::new(Self {
            executable: PathBuf::from(key.get_value::<String, _>("Executable")?),
            open_types_title: key.get_value("archive-open-type")?,
            email_title: key.get_value("shell-email-menu")?,
            extract_each_title: key.get_value("extract-each-folder")?,
            titles: ACTIONS
                .iter()
                .map(|action| key.get_value(action.argument()))
                .collect::<std::io::Result<_>>()?,
        }))
    }
}

struct Selection {
    paths: Vec<PathBuf>,
    all_archives: bool,
    first_directory: bool,
}
impl Selection {
    fn read(items: Ref<'_, IShellItemArray>) -> Result<Self> {
        let items = items.as_ref().ok_or_else(|| Error::from(E_INVALIDARG))?;
        let count = unsafe { items.GetCount()? };
        let mut paths = Vec::with_capacity(count as usize);
        let mut all_archives = count > 0;
        let mut first_directory = false;
        for index in 0..count {
            let item = unsafe { items.GetItemAt(index)? };
            let flags =
                unsafe { item.GetAttributes(SFGAO_FILESYSTEM | SFGAO_FOLDER | SFGAO_STREAM)? };
            if !flags.contains(SFGAO_FILESYSTEM) {
                return Err(E_INVALIDARG.into());
            }
            let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
            let text = unsafe { name.to_string() };
            unsafe { CoTaskMemFree(Some(name.0.cast())) };
            let path = PathBuf::from(text?);
            let directory = flags.contains(SFGAO_FOLDER) && !flags.contains(SFGAO_STREAM);
            if index == 0 {
                first_directory = directory;
            }
            all_archives &= !directory && may_extract(&path);
            paths.push(path);
        }
        Ok(Self {
            paths,
            all_archives,
            first_directory,
        })
    }
}

#[derive(Clone, Copy)]
enum Node {
    Root,
    OpenTypes,
    Email,
    Hashes,
    Action(usize),
}
impl Node {
    fn action(action: Action) -> Self {
        Self::Action(ACTIONS.iter().position(|value| *value == action).unwrap())
    }

    fn children(self) -> Vec<Self> {
        if matches!(self, Self::Root) {
            return vec![
                Self::action(Action::Open),
                Self::OpenTypes,
                Self::action(Action::Extract),
                Self::action(Action::ExtractHere),
                Self::action(Action::ExtractFolder),
                Self::action(Action::Check),
                Self::action(Action::Compress),
                Self::action(Action::QuickCompress(ArchiveFormat::Zip, false)),
                Self::action(Action::QuickCompress(ArchiveFormat::SevenZip, false)),
                Self::Email,
                Self::Hashes,
            ];
        }
        if matches!(self, Self::Email) {
            return vec![
                Self::action(Action::CompressEmail),
                Self::action(Action::QuickCompress(ArchiveFormat::Zip, true)),
                Self::action(Action::QuickCompress(ArchiveFormat::SevenZip, true)),
            ];
        }
        let mut nodes = Vec::new();
        for (index, action) in ACTIONS.iter().enumerate() {
            let belongs = match self {
                Self::OpenTypes => matches!(action, Action::OpenAs(_)),
                Self::Hashes => matches!(
                    action,
                    Action::Checksum(_) | Action::GenerateChecksum | Action::VerifyChecksum
                ),
                _ => false,
            };
            if belongs {
                nodes.push(Self::Action(index));
            }
        }
        nodes
    }
}

fn shell_string(value: &str) -> Result<PWSTR> {
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    unsafe { SHStrDupW(PCWSTR(wide.as_ptr())) }
}

#[implement(IExplorerCommand)]
struct ArchiveCommand {
    node: Node,
    config: Arc<Configuration>,
    _module: ModuleGuard,
}
impl ArchiveCommand {
    fn interface(node: Node, config: Arc<Configuration>) -> IExplorerCommand {
        Self {
            node,
            config,
            _module: ModuleGuard::new(),
        }
        .into()
    }
}
impl IExplorerCommand_Impl for ArchiveCommand_Impl {
    fn GetTitle(&self, items: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        let index = match self.node {
            Node::Root => return shell_string("7zplus"),
            Node::Hashes => return shell_string("CRC SHA"),
            Node::OpenTypes => return shell_string(&self.config.open_types_title),
            Node::Email => return shell_string(&self.config.email_title),
            Node::Action(index) => index,
        };
        let template = &self.config.titles[index];
        let name = match ACTIONS[index] {
            Action::QuickCompress(_, _) | Action::GenerateChecksum => {
                let selection = Selection::read(items)?;
                if selection.paths.is_empty() {
                    return Err(E_INVALIDARG.into());
                }
                let hash = ACTIONS[index] == Action::GenerateChecksum;
                let extension = match ACTIONS[index] {
                    Action::QuickCompress(format, _) => format.value(),
                    _ => "sha256",
                };
                format!(
                    "{}.{extension}",
                    archive_name(&selection.paths, selection.first_directory, hash)
                )
            }
            Action::ExtractFolder => {
                let selection = Selection::read(items)?;
                if selection.paths.len() == 1 {
                    extract_folder(&selection.paths[0])
                } else {
                    return shell_string(&self.config.extract_each_title);
                }
            }
            _ => return shell_string(template),
        };
        shell_string(&template.replace("{name}", &name.replace('&', "&&")))
    }
    fn GetIcon(&self, _: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        if matches!(self.node, Node::Root) {
            shell_string(&format!("\"{}\",0", self.config.executable.display()))
        } else {
            Err(E_NOTIMPL.into())
        }
    }
    fn GetToolTip(&self, _: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        Err(E_NOTIMPL.into())
    }
    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(GUID::from_u128(
            CLSID_VALUE
                + match self.node {
                    Node::Root => 0,
                    Node::OpenTypes => 1000,
                    Node::Hashes => 1001,
                    Node::Email => 1002,
                    Node::Action(index) => index as u128 + 1,
                },
        ))
    }
    fn GetState(&self, items: Ref<'_, IShellItemArray>, _: BOOL) -> Result<u32> {
        let Ok(selection) = Selection::read(items) else {
            return Ok(ECS_HIDDEN.0 as u32);
        };
        let available = match self.node {
            Node::Root | Node::Hashes | Node::Email => !selection.paths.is_empty(),
            Node::OpenTypes => {
                Action::Open.available(selection.paths.len(), selection.all_archives)
            }
            Node::Action(index) => {
                ACTIONS[index].available(selection.paths.len(), selection.all_archives)
            }
        };
        Ok(if available {
            ECS_ENABLED.0
        } else {
            ECS_HIDDEN.0
        } as u32)
    }
    fn Invoke(&self, items: Ref<'_, IShellItemArray>, _: Ref<'_, IBindCtx>) -> Result<()> {
        let Node::Action(index) = self.node else {
            return Err(E_NOTIMPL.into());
        };
        let selection = Selection::read(items)?;
        let action = ACTIONS[index];
        if !action.available(selection.paths.len(), selection.all_archives) {
            return Err(E_INVALIDARG.into());
        }
        // Pass one complete selection without per-file launches or command-line length limits.
        let mut file = tempfile::Builder::new()
            .prefix(REQUEST_PREFIX)
            .suffix(".json")
            .tempfile()?;
        serde_json::to_writer(
            file.as_file_mut(),
            &Request {
                action,
                paths: selection.paths,
            },
        )
        .map_err(|error| Error::new(E_FAIL, error.to_string()))?;
        let path = file
            .into_temp_path()
            .keep()
            .map_err(|error| Error::from(error.error))?;
        if let Err(error) = Process::new(&self.config.executable)
            .arg("--shell-request")
            .arg(&path)
            .spawn()
        {
            let _ = std::fs::remove_file(path);
            return Err(error.into());
        }
        Ok(())
    }
    fn GetFlags(&self) -> Result<u32> {
        let flags = if matches!(self.node, Node::Action(_)) {
            ECF_DEFAULT.0
        } else {
            ECF_HASSUBCOMMANDS.0
        };
        let separator = match self.node {
            Node::Email => true,
            Node::Action(index) => {
                matches!(
                    ACTIONS[index],
                    Action::Extract | Action::Compress | Action::GenerateChecksum
                )
            }
            _ => false,
        };
        Ok((flags | if separator { ECF_SEPARATORBEFORE.0 } else { 0 }) as u32)
    }
    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        if matches!(self.node, Node::Action(_)) {
            return Err(E_NOTIMPL.into());
        }
        Ok(CommandEnumerator::interface(
            self.config.clone(),
            self.node,
            0,
        ))
    }
}

#[implement(IEnumExplorerCommand, Agile = false)]
struct CommandEnumerator {
    config: Arc<Configuration>,
    parent: Node,
    children: Vec<Node>,
    position: Cell<usize>,
    _module: ModuleGuard,
}
impl CommandEnumerator {
    fn interface(
        config: Arc<Configuration>,
        parent: Node,
        position: usize,
    ) -> IEnumExplorerCommand {
        Self {
            config,
            parent,
            children: parent.children(),
            position: Cell::new(position),
            _module: ModuleGuard::new(),
        }
        .into()
    }
}
impl IEnumExplorerCommand_Impl for CommandEnumerator_Impl {
    fn Next(
        &self,
        count: u32,
        output: *mut Option<IExplorerCommand>,
        fetched: *mut u32,
    ) -> HRESULT {
        if output.is_null() || (count != 1 && fetched.is_null()) {
            return E_POINTER;
        }
        let start = self.position.get();
        let end = start
            .saturating_add(count as usize)
            .min(self.children.len());
        for index in start..end {
            // COM provides uninitialized output slots; write transfers ownership to the caller.
            unsafe {
                output
                    .add(index - start)
                    .write(Some(ArchiveCommand::interface(
                        self.children[index],
                        self.config.clone(),
                    )))
            };
        }
        self.position.set(end);
        if !fetched.is_null() {
            unsafe { fetched.write((end - start) as u32) };
        }
        if end - start == count as usize {
            S_OK
        } else {
            S_FALSE
        }
    }
    fn Skip(&self, count: u32) -> Result<()> {
        let next = self.position.get().saturating_add(count as usize);
        self.position.set(next.min(self.children.len()));
        if next <= self.children.len() {
            Ok(())
        } else {
            Err(Error::from_hresult(S_FALSE))
        }
    }
    fn Reset(&self) -> Result<()> {
        self.position.set(0);
        Ok(())
    }
    fn Clone(&self) -> Result<IEnumExplorerCommand> {
        Ok(CommandEnumerator::interface(
            self.config.clone(),
            self.parent,
            self.position.get(),
        ))
    }
}

#[implement(IClassFactory)]
struct Factory {
    _module: ModuleGuard,
}
impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        outer: Ref<'_, IUnknown>,
        iid: *const GUID,
        output: *mut *mut c_void,
    ) -> Result<()> {
        if iid.is_null() || output.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { output.write(std::ptr::null_mut()) };
        if outer.as_ref().is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let command = ArchiveCommand::interface(Node::Root, Configuration::load()?);
        unsafe { command.query(iid, output).ok() }
    }
    fn LockServer(&self, lock: BOOL) -> Result<()> {
        if lock.as_bool() {
            LOCKS.fetch_add(1, Ordering::Relaxed);
        } else {
            LOCKS.fetch_sub(1, Ordering::Release);
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
/// # Safety
/// The caller must provide valid COM GUID pointers and an output interface slot.
pub unsafe extern "system" fn DllGetClassObject(
    class: *const GUID,
    iid: *const GUID,
    output: *mut *mut c_void,
) -> HRESULT {
    if class.is_null() || iid.is_null() || output.is_null() {
        return E_POINTER;
    }
    unsafe { output.write(std::ptr::null_mut()) };
    if unsafe { *class } != GUID::from_u128(CLSID_VALUE) {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = Factory {
        _module: ModuleGuard::new(),
    }
    .into();
    unsafe { factory.query(iid, output) }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if OBJECTS.load(Ordering::Acquire) == 0 && LOCKS.load(Ordering::Acquire) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
