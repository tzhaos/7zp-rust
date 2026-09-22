use crate::{
    Request,
    filesystem::{NameConflict, name_conflicts},
};
use anyhow::{Result, bail};
use cardo_7zp_core::i18n::tr;
use cardo_7zp_engine::{Cancellation, Catalog, Overwrite};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone)]
pub struct Extraction {
    pub catalog: Catalog,
    pub selected: Vec<String>,
    pub parent: PathBuf,
    pub folder: String,
    pub open_after: bool,
}

pub struct ExtractionPlan {
    pub extraction: Extraction,
    pub conflicts: Vec<NameConflict>,
}

impl Extraction {
    pub fn prepare(self, cancel: &Cancellation) -> Result<ExtractionPlan> {
        if !valid_folder(&self.folder) {
            bail!(tr("folder-name-invalid"));
        }
        let conflicts = name_conflicts(
            &self.catalog,
            &self.selected,
            &self.parent.join(&self.folder),
            cancel,
        )?;
        Ok(ExtractionPlan {
            extraction: self,
            conflicts,
        })
    }

    pub fn request(
        &self,
        selected: Vec<String>,
        overwrite: Overwrite,
        open_after: bool,
    ) -> Request {
        Request::Extract {
            catalog: self.catalog.clone(),
            selected,
            parent: self.parent.clone(),
            folder: self.folder.clone(),
            overwrite,
            open_after,
        }
    }
}

pub fn valid_folder(folder: &str) -> bool {
    folder != "." && folder != ".." && !folder.contains(['/', '\\', ':'])
}

pub fn entry_selected(path: &str, selected: &[String]) -> bool {
    selected.is_empty()
        || selected.iter().any(|item| {
            path == item
                || path
                    .strip_prefix(item)
                    .is_some_and(|tail| tail.starts_with('/'))
        })
}

impl ExtractionPlan {
    pub fn requests(self, decisions: Vec<(String, Overwrite)>) -> Vec<Request> {
        let extraction = self.extraction;
        let conflicted: BTreeSet<_> = self
            .conflicts
            .into_iter()
            .map(|conflict| conflict.entry)
            .collect();
        // Selecting a directory recursively would reintroduce its conflicted descendants.
        // Keep only empty directory entries alongside the individually selected files.
        let nonempty: BTreeSet<_> = extraction
            .catalog
            .entries
            .iter()
            .flat_map(|entry| {
                entry
                    .path
                    .match_indices('/')
                    .map(|(index, _)| entry.path[..index].to_owned())
            })
            .collect();
        let plain: Vec<_> = extraction
            .catalog
            .entries
            .iter()
            .filter(|entry| {
                entry_selected(&entry.path, &extraction.selected)
                    && !conflicted.contains(&entry.path)
                    && (!entry.directory || !nonempty.contains(&entry.path))
            })
            .map(|entry| entry.path.clone())
            .collect();
        let mut replace = Vec::new();
        let mut rename = Vec::new();
        for (path, mode) in decisions {
            match mode {
                Overwrite::Replace => replace.push(path),
                Overwrite::RenameIncoming => rename.push(path),
                _ => {}
            }
        }
        let mut batches: Vec<_> = [
            (Overwrite::Skip, plain),
            (Overwrite::Replace, replace),
            (Overwrite::RenameIncoming, rename),
        ]
        .into_iter()
        .filter(|(_, paths)| !paths.is_empty())
        .collect();
        let last = batches.len().saturating_sub(1);
        batches
            .drain(..)
            .enumerate()
            .map(|(index, (mode, paths))| {
                extraction.request(paths, mode, extraction.open_after && index == last)
            })
            .collect()
    }
}
