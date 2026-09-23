use crate::*;

impl Workspace {
    pub(crate) fn apply_conflict(&mut self, overwrite: Overwrite, cx: &mut Context<Self>) {
        if let Some(Modal::Conflict {
            plan,
            index,
            decisions,
            repeat,
            ..
        }) = self.dialogs.current_mut()
        {
            if *repeat {
                decisions.extend(
                    plan.conflicts
                        .iter()
                        .skip(*index)
                        .map(|conflict| (conflict.entry.clone(), overwrite)),
                );
                *index = plan.conflicts.len();
            } else if let Some(conflict) = plan.conflicts.get(*index) {
                decisions.push((conflict.entry.clone(), overwrite));
                *index += 1;
            }
        }
        if matches!(self.dialogs.current(), Some(Modal::Conflict { plan, index, .. }) if *index >= plan.conflicts.len())
        {
            self.launch_conflict_plan(cx);
        } else {
            cx.notify();
        }
    }

    fn launch_conflict_plan(&mut self, cx: &mut Context<Self>) {
        let Some(Modal::Conflict {
            plan,
            password,
            decisions,
            ..
        }) = self.dialogs.take()
        else {
            return;
        };
        self.dialogs.close_prompt(cx);
        self.extract_follow = plan
            .requests(decisions)
            .into_iter()
            .map(|request| (request, password.clone()))
            .collect();
        if let Some((request, password)) = self.extract_follow.pop_front() {
            self.execute(request, password, cx);
        } else {
            self.tasks.set_close_after(false);
            self.tasks.set_close_archive(false);
            self.notify_message(tr("extract-conflict-skipped").into());
            cx.notify();
        }
    }
}
