use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Store {
    directory: PathBuf,
}

impl Store {
    pub fn for_app(directory_name: &str) -> Result<Self> {
        Ok(Self {
            directory: dirs::config_local_dir()
                .context("Cannot locate the user's local configuration directory")?
                .join(directory_name),
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let path = self.directory.join(name);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
        }
    }

    pub fn read_text(&self, name: &str) -> Result<Option<String>> {
        self.read(name)?
            .map(|bytes| {
                String::from_utf8(bytes).with_context(|| {
                    format!("Invalid UTF-8 in {}", self.directory.join(name).display())
                })
            })
            .transpose()
    }

    pub fn read_json<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        self.read(name)?
            .map(|bytes| {
                serde_json::from_slice(&bytes).with_context(|| {
                    format!("Invalid JSON in {}", self.directory.join(name).display())
                })
            })
            .transpose()
    }

    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let path = self.directory.join(name);
        (|| -> Result<()> {
            std::fs::create_dir_all(&self.directory)?;
            // Persist within the same directory so an interrupted write cannot truncate settings.
            let mut file = tempfile::NamedTempFile::new_in(&self.directory)?;
            file.write_all(bytes)?;
            file.as_file().sync_all()?;
            file.persist(&path)?;
            Ok(())
        })()
        .with_context(|| format!("Cannot save {}", path.display()))
    }

    pub fn write_json(&self, name: &str, value: &impl Serialize) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(value)
            .with_context(|| format!("Cannot serialize {}", self.directory.join(name).display()))?;
        self.write(name, &bytes)
    }
}
