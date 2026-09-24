use crate::*;
impl Workspace {
    pub(crate) fn titlebar(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement + use<> {
        cardo_ui::chrome::titlebar(self.window_title(), self.menubar(cx), self.tasks.is_busy() || self.settings_busy(cx), [tr("window-minimize").into(), tr("window-maximize").into(), tr("window-close").into()], icon, window, cx)
    }

    fn window_title(&self) -> String {
        let view = self.browser.view();
        let path = view
            .catalog
            .as_ref()
            .map(|catalog| &catalog.path)
            .or_else(|| view.directory.as_ref().map(|directory| &directory.path));
        let Some(path) = path else {
            return "Plus7z".to_owned();
        };
        let name = path
            .file_name()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        format!("{name} - Plus7z")
    }
}
