use super::{
    Cancellation, Catalog, Engine,
    engine::{input_list, list_argument, publish},
};
use anyhow::{Context, Result, bail};
use p7z_core::i18n::tr;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub enum Edit {
    Add(Vec<PathBuf>),
    Delete(Vec<String>),
    Rename { source: String, destination: String },
}

impl Engine {
    pub fn edit(
        &self,
        catalog: &Catalog,
        edit: &Edit,
        password: &str,
        cancel: &Cancellation,
    ) -> Result<Catalog> {
        if !catalog.editable() {
            bail!(tr("archive-read-only"));
        }
        let stage =
            tempfile::tempdir_in(catalog.path.parent().context(tr("destination-required"))?)?;
        let temporary = stage
            .path()
            .join(catalog.path.file_name().context(tr("file-name-missing"))?);
        std::fs::copy(&catalog.path, &temporary)?;
        let mut args = vec![
            match edit {
                Edit::Add(_) => "a",
                Edit::Delete(_) => "d",
                Edit::Rename { .. } => "rn",
            }
            .into(),
            "-sccUTF-8".into(),
            "-scsUTF-8".into(),
            "-spd".into(),
        ];
        if !password.is_empty() {
            args.push(format!("-p{password}").into());
        }
        let inputs = match edit {
            Edit::Add(paths) => {
                if paths.is_empty() {
                    bail!(tr("source-required"));
                }
                Some(input_list(paths)?)
            }
            Edit::Delete(paths) => {
                if paths.is_empty() {
                    bail!(tr("archive-selection-required"));
                }
                let paths = paths.iter().map(PathBuf::from).collect::<Vec<_>>();
                Some(input_list(&paths)?)
            }
            Edit::Rename {
                source,
                destination,
            } => {
                if destination.is_empty()
                    || destination.starts_with('/')
                    || destination.contains(['\\', ':', '\r', '\n'])
                    || destination.split('/').any(|part| {
                        part.is_empty() || part == "." || part == ".." || part.ends_with([' ', '.'])
                    })
                {
                    bail!(tr("archive-name-invalid"));
                }
                if catalog
                    .entries
                    .iter()
                    .any(|entry| entry.path == *destination && entry.path != *source)
                {
                    bail!(tr("archive-entry-exists"));
                }
                None
            }
        };
        if let Some(inputs) = &inputs {
            args.push(list_argument("-i@", inputs.as_ref()));
        }
        args.extend(["--".into(), temporary.as_os_str().into()]);
        if let Edit::Rename {
            source,
            destination,
        } = edit
        {
            args.extend([source.as_str().into(), destination.as_str().into()]);
        }
        let output = self.run(args, cancel)?;
        if output.warning {
            return Err(anyhow::anyhow!(output.text)).context(tr("archive-edit-warning"));
        }
        if self.test(&temporary, password, cancel)?.warning {
            bail!(tr("integrity-warning"));
        }
        let mut updated = self.list(&temporary, password, cancel)?;
        if cancel.load(Ordering::Relaxed) {
            bail!(tr("task-cancelled"));
        }
        publish(&temporary, &catalog.path, true).context(tr("publish-failed"))?;
        updated.path = catalog.path.clone();
        Ok(updated)
    }
}
