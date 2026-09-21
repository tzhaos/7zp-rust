use crate::*;
use gpui_kit::component::{h_flex, v_flex};

#[derive(Clone, Copy)]
enum Choice {
    Replace,
    Skip,
    Rename,
    ReplaceAll,
    SkipAll,
    RenameAll,
    Cancel,
}

impl Workspace {
    pub(super) fn conflict_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(Modal::Conflict {
            conflicts, index, ..
        }) = self.dialogs.current()
        else {
            return div().into_any_element();
        };
        let Some(conflict) = conflicts.get(*index) else {
            return div().into_any_element();
        };
        let path = conflict.destination.display().to_string();
        let incoming = format!(
            "{}\n{}",
            crate::format::size_text(conflict.incoming_size),
            tf(
                "extract-conflict-modified",
                &[("time", conflict.incoming_modified.as_str().into())],
            ),
        );
        let existing = format!(
            "{}\n{}",
            crate::format::size_text(conflict.existing_size),
            tf(
                "extract-conflict-modified",
                &[("time", conflict.existing_modified.as_str().into())],
            ),
        );
        v_flex()
            .child(
                v_flex()
                    .px(px(24.))
                    .py(px(18.))
                    .gap(px(12.))
                    .child(tr("extract-conflict-exists"))
                    .child(path_strip("conflict-path", path, cx))
                    .child(tr("extract-conflict-incoming"))
                    .child(incoming)
                    .child(tr("extract-conflict-existing"))
                    .child(existing),
            )
            .child(
                v_flex()
                    .px(px(16.))
                    .py(px(12.))
                    .gap(px(8.))
                    .child(h_flex().gap(px(8.)).children([
                        self.conflict_button("conflict-replace", tr("conflict-replace"), Choice::Replace, cx),
                        self.conflict_button("conflict-skip", tr("conflict-skip"), Choice::Skip, cx),
                        self.conflict_button("conflict-rename", tr("conflict-rename"), Choice::Rename, cx),
                    ]))
                    .child(h_flex().gap(px(8.)).children([
                        self.conflict_button(
                            "conflict-replace-all",
                            tr("conflict-replace-all"),
                            Choice::ReplaceAll,
                            cx,
                        ),
                        self.conflict_button(
                            "conflict-skip-all",
                            tr("conflict-skip-all"),
                            Choice::SkipAll,
                            cx,
                        ),
                        self.conflict_button(
                            "conflict-rename-all",
                            tr("conflict-rename-all"),
                            Choice::RenameAll,
                            cx,
                        ),
                        self.conflict_button("conflict-cancel", tr("cancel"), Choice::Cancel, cx),
                    ])),
            )
            .into_any_element()
    }

    fn conflict_button(
        &self,
        id: &'static str,
        label: &'static str,
        choice: Choice,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        command(id, label)
            .flex_1()
            .h(px(36.))
            .on_click(cx.listener(move |this, _, _, cx| this.apply_conflict(choice, cx)))
    }

    fn apply_conflict(&mut self, choice: Choice, cx: &mut Context<Self>) {
        if matches!(choice, Choice::Cancel) {
            self.dialogs.take();
            self.tasks.set_close_after(false);
            self.extract_follow.clear();
            cx.notify();
            return;
        }
        let overwrite = match choice {
            Choice::Replace | Choice::ReplaceAll => Overwrite::Replace,
            Choice::Skip | Choice::SkipAll => Overwrite::Skip,
            Choice::Rename | Choice::RenameAll => Overwrite::RenameIncoming,
            Choice::Cancel => return,
        };
        let all = matches!(
            choice,
            Choice::ReplaceAll | Choice::SkipAll | Choice::RenameAll
        );
        if let Some(Modal::Conflict {
            conflicts,
            index,
            decisions,
            ..
        }) = self.dialogs.current_mut()
        {
            let start = *index;
            if all {
                for conflict in conflicts.iter().skip(start) {
                    decisions.push((conflict.entry.clone(), overwrite));
                }
                *index = conflicts.len();
            } else if let Some(conflict) = conflicts.get(start) {
                decisions.push((conflict.entry.clone(), overwrite));
                *index += 1;
            }
        }
        let done = matches!(
            self.dialogs.current(),
            Some(Modal::Conflict {
                index,
                conflicts,
                ..
            }) if *index >= conflicts.len()
        );
        if done {
            self.launch_conflict_plan(cx);
        } else {
            cx.notify();
        }
    }

    fn launch_conflict_plan(&mut self, cx: &mut Context<Self>) {
        let Some(Modal::Conflict {
            catalog,
            selected,
            parent,
            folder,
            open_after,
            password,
            conflicts,
            decisions,
            ..
        }) = self.dialogs.take()
        else {
            return;
        };
        let conflicted: std::collections::BTreeSet<_> =
            conflicts.into_iter().map(|conflict| conflict.entry).collect();
        let plain: Vec<String> = catalog
            .entries
            .iter()
            .filter(|entry| {
                !entry.directory
                    && selected_entry(&entry.path, &selected)
                    && !conflicted.contains(&entry.path)
            })
            .map(|entry| entry.path.clone())
            .collect();
        let mut replace = Vec::new();
        let mut rename = Vec::new();
        for (entry, mode) in decisions {
            match mode {
                Overwrite::Replace => replace.push(entry),
                Overwrite::RenameIncoming => rename.push(entry),
                _ => {}
            }
        }
        let mut batches = Vec::new();
        if !plain.is_empty() {
            batches.push((Overwrite::Skip, plain));
        }
        if !replace.is_empty() {
            batches.push((Overwrite::Replace, replace));
        }
        if !rename.is_empty() {
            batches.push((Overwrite::RenameIncoming, rename));
        }
        if batches.is_empty() {
            self.tasks.set_close_after(false);
            self.message = Some(tr("extract-conflict-skipped").into());
            cx.notify();
            return;
        }
        let last = batches.len() - 1;
        let mut requests: Vec<_> = batches
            .into_iter()
            .enumerate()
            .map(|(index, (overwrite, paths))| Request::Extract {
                catalog: catalog.clone(),
                selected: paths,
                parent: parent.clone(),
                folder: folder.clone(),
                overwrite,
                open_after: open_after && index == last,
            })
            .collect();
        let first = requests.remove(0);
        self.extract_follow = requests
            .into_iter()
            .map(|request| (request, password.clone()))
            .collect();
        self.execute(first, password, cx);
    }
}

fn selected_entry(path: &str, selected: &[String]) -> bool {
    selected.is_empty()
        || selected
            .iter()
            .any(|item| path == item || path.starts_with(&format!("{item}/")))
}
