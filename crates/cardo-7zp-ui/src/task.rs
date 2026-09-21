use gpui_kit::Task;
use cardo_7zp_archive::Cancellation;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::progress::ExtractionProgress;

pub(crate) struct TaskState {
    busy: bool,
    status: String,
    cancel: Cancellation,
    task: Option<Task<()>>,
    extraction: Option<ExtractionProgress>,
    temporary_files: Vec<tempfile::TempDir>,
    close_after: bool,
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            busy: false,
            status: String::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            task: None,
            extraction: None,
            temporary_files: Vec::new(),
            close_after: false,
        }
    }
}

impl TaskState {
    pub(crate) fn is_busy(&self) -> bool {
        self.busy
    }
    pub(crate) fn status(&self) -> &str {
        &self.status
    }
    pub(crate) fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
    pub(crate) fn close_after(&self) -> bool {
        self.close_after
    }
    pub(crate) fn set_close_after(&mut self, close: bool) {
        self.close_after = close;
    }

    pub(crate) fn begin(&mut self, label: &str) -> Cancellation {
        self.busy = true;
        self.status = label.to_owned();
        self.cancel = Arc::new(AtomicBool::new(false));
        self.cancel.clone()
    }

    pub(crate) fn attach(&mut self, task: Task<()>) {
        self.task = Some(task);
    }

    pub(crate) fn finish(&mut self) -> Option<ExtractionProgress> {
        self.busy = false;
        self.extraction.take()
    }

    pub(crate) fn release(&mut self) {
        self.task = None;
    }

    pub(crate) fn cancel(&mut self, status: &str) {
        self.cancel.store(true, Ordering::Relaxed);
        self.status = status.to_owned();
    }

    pub(crate) fn retain_file(&mut self, directory: tempfile::TempDir) {
        self.temporary_files.push(directory);
    }

    pub(crate) fn show_extraction(&mut self, progress: ExtractionProgress) {
        self.extraction = Some(progress);
    }

    pub(crate) fn extraction(&self) -> Option<&ExtractionProgress> {
        self.extraction.as_ref()
    }
}
