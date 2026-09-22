use anyhow::{Context, Result};
use std::io::ErrorKind;
use winreg::{RegKey, enums::KEY_WRITE};

/// Ownership is established by existing registration values, not by key presence.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    Missing,
    Current,
    Other,
}

pub fn string(root: &RegKey, path: &str, name: &str) -> Result<Option<String>> {
    let read = || root.open_subkey(path)?.get_value::<String, _>(name);
    match read() {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Cannot read registry {path} [{name}]")),
    }
}

/// Every present identity must match; conflicting partial registrations are never removed.
pub fn ownership(root: &RegKey, identities: &[(&str, &str, &str)]) -> Result<Ownership> {
    let mut state = Ownership::Missing;
    for &(path, name, expected) in identities {
        if let Some(actual) = string(root, path, name)? {
            if !actual.eq_ignore_ascii_case(expected) {
                return Ok(Ownership::Other);
            }
            state = Ownership::Current;
        }
    }
    Ok(state)
}

pub fn remove_key(root: &RegKey, path: &str) -> Result<()> {
    match root.delete_subkey_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("Cannot remove registry key {path}")),
    }
}

pub fn remove_value(root: &RegKey, path: &str, name: &str) -> Result<()> {
    let remove = || {
        root.open_subkey_with_flags(path, KEY_WRITE)?
            .delete_value(name)
    };
    match remove() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("Cannot remove registry value {path} [{name}]"))
        }
    }
}
