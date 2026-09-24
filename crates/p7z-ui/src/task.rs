use crate::progress::ExtractionProgress;
use cardo_ui::task::TaskHandle;
use p7z_engine::Cancellation;

pub(crate) struct TaskState {
    state: cardo_runtime::task::TaskState<ExtractionProgress>,
    task: Option<TaskHandle>,
    temporary_files: Vec<tempfile::TempDir>,
    close_after: bool,
    close_archive: bool,
    extraction_warning: bool,
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            state: Default::default(),
            task: None,
            temporary_files: Vec::new(),
            close_after: false,
            close_archive: false,
            extraction_warning: false,
        }
    }
}

impl TaskState {
    pub(crate) fn reset_extraction_result(&mut self) {
        self.extraction_warning = false;
    }

    pub(crate) fn record_extraction_warning(&mut self, warning: bool) -> bool {
        self.extraction_warning |= warning;
        self.extraction_warning
    }
    pub(crate) fn is_busy(&self) -> bool {
        self.state.is_busy()
    }
    pub(crate) fn status(&self) -> &str {
        self.state.status()
    }
    pub(crate) fn cancelled(&self) -> bool {
        self.state.cancelled()
    }
    pub(crate) fn close_after(&self) -> bool {
        self.close_after
    }
    pub(crate) fn set_close_after(&mut self, close: bool) {
        self.close_after = close;
    }
    pub(crate) fn close_archive(&self) -> bool {
        self.close_archive
    }
    pub(crate) fn set_close_archive(&mut self, close: bool) {
        self.close_archive = close;
    }

    pub(crate) fn begin(&mut self, label: &str) -> Cancellation {
        self.state.begin(label)
    }

    pub(crate) fn attach(&mut self, task: TaskHandle) {
        self.task = Some(task);
    }

    pub(crate) fn finish(&mut self) -> Option<ExtractionProgress> {
        self.state.finish()
    }

    pub(crate) fn release(&mut self) {
        self.task = None;
    }

    pub(crate) fn cancel(&mut self, status: &str) {
        self.state.cancel(status);
    }

    pub(crate) fn retain_file(&mut self, directory: tempfile::TempDir) {
        self.temporary_files.push(directory);
    }

    pub(crate) fn show_extraction(&mut self, progress: ExtractionProgress) {
        self.state.set_progress(progress);
    }

    pub(crate) fn extraction(&self) -> Option<&ExtractionProgress> {
        self.state.progress()
    }
}
