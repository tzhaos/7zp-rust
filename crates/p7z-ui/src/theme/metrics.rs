pub const CONTROL_HEIGHT: f32 = 32.;
pub const ICON_BUTTON_SIZE: f32 = 30.;
pub const TOOLBAR_HEIGHT: f32 = 94.;
pub const TOOL_HEIGHT: f32 = 80.;
pub const TOOL_ICON_HEIGHT: f32 = 56.;
pub const TOOL_MIN_WIDTH: f32 = 66.;
pub const TOOL_MAX_WIDTH: f32 = 72.;
pub const TOOL_GAP: f32 = 5.;
pub const TOOLBAR_PADDING: f32 = 12.;
pub const ARTWORK_SIZE: f32 = 48.;
pub const CONTENT_TOP_GAP: f32 = 12.;

pub mod file_table {
    pub const SELECTION_WIDTH: f32 = 52.;
    pub const SIZE_WIDTH: f32 = 108.;
    pub const TYPE_WIDTH: f32 = 120.;
    pub const MODIFIED_WIDTH: f32 = 166.;
    pub const HEADER_HEIGHT: f32 = 36.;
}

pub use cardo_ui::metrics::settings;

pub use cardo_ui::metrics::popup;

pub mod home {
    pub const MAX_WIDTH: f32 = 760.;
    pub const PADDING: f32 = 24.;
    pub const ACTION_GAP: f32 = 12.;
    pub const ACTION_ICON_SIZE: f32 = 17.;
    pub const HISTORY_GAP: f32 = 8.;
    pub const HISTORY_LIMIT: usize = 8;
    pub const HISTORY_MAX_WIDTH: f32 = 600.;
    pub const HISTORY_ROW_HEIGHT: f32 = 40.;
    pub const HISTORY_ROW_PADDING: f32 = 8.;
    pub const HISTORY_REMOVE_WIDTH: f32 = 38.;
}

pub use cardo_ui::metrics::titlebar;
