pub const CONTROL_HEIGHT: f32 = 32.;
pub const CONTROL_RADIUS: f32 = 4.;
pub const ICON_BUTTON_SIZE: f32 = 30.;
pub const TOOLBAR_HEIGHT: f32 = 94.;
pub const TOOL_HEIGHT: f32 = 80.;
pub const TOOL_ICON_HEIGHT: f32 = 56.;
pub const TOOL_MIN_WIDTH: f32 = 66.;
pub const TOOL_MAX_WIDTH: f32 = 72.;
pub const TOOL_GAP: f32 = 5.;
pub const TOOLBAR_PADDING: f32 = 12.;
pub const ARTWORK_SIZE: f32 = 48.;
pub const PANEL_RADIUS: f32 = 12.;
pub const CONTENT_TOP_GAP: f32 = 12.;

pub mod file_table {
    pub const SELECTION_WIDTH: f32 = 52.;
    pub const SIZE_WIDTH: f32 = 108.;
    pub const TYPE_WIDTH: f32 = 120.;
    pub const MODIFIED_WIDTH: f32 = 166.;
    pub const HEADER_HEIGHT: f32 = 36.;
}

pub mod settings {
    pub const CONTENT_PADDING: f32 = 24.;
    pub const BACK_BUTTON_GAP: f32 = 12.;
    pub const NAV_GUTTER: f32 = super::ICON_BUTTON_SIZE + BACK_BUTTON_GAP - CONTENT_PADDING;
    pub const TITLE_SIZE: f32 = 24.;
    pub const SECTION_TITLE_SIZE: f32 = 14.;
    pub const SECTION_GAP: f32 = 32.;
    pub const HEADING_GAP: f32 = 16.;
}

pub mod popup {
    pub const OUTER_INSET: f32 = 12.;
    pub const TITLE_HEIGHT: f32 = 44.;
    pub const TITLE_TEXT: f32 = 14.;
    pub const ACTION_MIN_WIDTH: f32 = 80.;
    pub const PADDING: f32 = 20.;
    pub const GAP: f32 = 16.;
    pub const FIELD_GAP: f32 = 8.;
    pub const CARD_PADDING: f32 = 16.;
    pub const FOOTER_PADDING: f32 = 12.;
    pub const BODY_TEXT: f32 = 13.;
    pub const LINE_HEIGHT: f32 = 20.;
}

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

pub mod titlebar {
    pub const HEIGHT: f32 = 36.;
    pub const MENU_MAX_WIDTH: f32 = 320.;
    pub const CONTROL_WIDTH: f32 = 42.;
    pub const GRIP_WIDTH: f32 = 28.;
    pub const GRIP_DOT_SIZE: f32 = 2.;
    pub const GRIP_GAP: f32 = 3.;
    pub const GRIP_OPACITY: f32 = 0.4;
}
