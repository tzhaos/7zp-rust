use gpui_kit::{
    component::{Theme, ThemeMode},
    *,
};

#[derive(Clone, Copy)]
pub struct Palette {
    pub surface: u32,
    pub panel: u32,
    pub title: u32,
    pub text: u32,
    pub muted: u32,
    pub border: u32,
    pub hover: u32,
    pub selected: u32,
    pub selected_hover: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub on_accent: u32,
    pub success: u32,
    pub danger: u32,
}

pub struct ThemeStyle {
    pub mode: ThemeMode,
    pub palette: Palette,
    pub font_family: SharedString,
    pub font_size: Pixels,
    pub radius: Pixels,
    pub radius_lg: Pixels,
}

impl ThemeStyle {
    pub fn apply(self, window: Option<&mut Window>, cx: &mut App) {
        Theme::change(self.mode, window, cx);
        let p = self.palette;
        let theme = Theme::global_mut(cx);
        theme.font_family = self.font_family;
        theme.font_size = self.font_size;
        theme.radius = self.radius;
        theme.radius_lg = self.radius_lg;
        let c = &mut theme.colors;
        c.background = rgb(p.surface).into();
        c.foreground = rgb(p.text).into();
        c.border = rgb(p.border).into();
        c.input = c.border;
        c.accent = rgb(p.hover).into();
        c.accent_foreground = c.foreground;
        c.muted = rgb(p.panel).into();
        c.muted_foreground = rgb(p.muted).into();
        c.popover = c.background;
        c.popover_foreground = c.foreground;
        c.button = c.background;
        c.button_foreground = c.foreground;
        c.button_hover = rgb(p.hover).into();
        c.button_active = rgb(p.selected).into();
        c.primary = rgb(p.accent).into();
        c.primary_hover = rgb(p.accent_hover).into();
        c.primary_active = c.primary_hover;
        c.primary_foreground = rgb(p.on_accent).into();
        c.button_primary = c.primary;
        c.button_primary_hover = c.primary_hover;
        c.button_primary_active = c.primary_active;
        c.button_primary_foreground = c.primary_foreground;
        c.secondary = rgb(p.hover).into();
        c.secondary_foreground = c.foreground;
        c.secondary_hover = c.border;
        c.secondary_active = c.border;
        c.ring = c.primary;
        c.caret = c.primary;
        c.list = c.background;
        c.list_head = c.muted;
        c.list_hover = c.button_hover;
        c.list_active = c.button_active;
        c.list_active_border = c.primary;
        c.switch = c.border;
        c.switch_thumb = rgb(0xffffff).into();
        c.selection = rgb(p.selected).into();
        c.scrollbar = rgba(0x00000000).into();
        c.scrollbar_thumb = rgba((p.muted << 8) | 0xb0).into();
        c.scrollbar_thumb_hover = rgba((p.text << 8) | 0xe6).into();
        theme.scrollbar_mode = gpui_kit::component::scroll::ScrollbarMode::Always;
        // Components and the base theme must see the same resolved tokens.
        theme.tokens = theme.colors.into();
        Theme::sync_base(cx);
        cx.refresh_windows();
    }
}
