use crossterm::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub highlight_diff: bool,
    pub only_printable: bool,
    pub lock_file_buffers: bool,
    pub screen_paging_size: ScreenPagingSize,
    pub clear_screen_on_exit: bool,
    pub highlight_style: HighlightStyle,
    pub active_color_scheme: String,
    color_scheme: Vec<ColorScheme>
}

#[derive(Deserialize, Clone, Copy)]
pub enum HighlightStyle {
    None,
    Solid,
    RandomDark,
    RandomLight,
    RandomAnsi
}

#[derive(Deserialize)]
pub enum ScreenPagingSize {
    Byte,
    Row,
    Page
}

#[derive(Deserialize, Clone)]
pub struct ColorScheme {
    pub name: String,
    pub fg_color: Color,
    pub bg_color: Color,
    pub error_fg_color: Color,
    pub error_bg_color: Color,
    pub cursor_fg_color: Color,
    pub cursor_bg_color: Color,
    pub patch_fg_color: Color,
    pub selection_bg_color: Color,
    pub highlight_bg_color: Color,
    pub diff_fg_color: Color,

    pub infobar_fg_color: Color,
    pub infobar_bg_color: Color,
    pub offsetbar_fg_color: Color,
    pub offsetbar_bg_color: Color,
    pub location_list_fg_color: Color,
    pub location_list_bg_color: Color,
    pub location_list_cursor_fg_color: Color,
    pub location_list_cursor_bg_color: Color
}

impl Config {

    pub fn color_scheme(&self, name: &str) -> Option<&ColorScheme> {
        self.color_scheme.iter().find(|cs| cs.name.eq(name))
    }
}

impl HighlightStyle {

    pub fn from_str(style: &str) -> Option<HighlightStyle> {
        match style {
            "None" => Some(HighlightStyle::None),
            "Solid" => Some(HighlightStyle::Solid),
            "RandomDark" => Some(HighlightStyle::RandomDark),
            "RandomLight" => Some(HighlightStyle::RandomLight),
            "RandomAnsi" => Some(HighlightStyle::RandomAnsi),
            _ => None
        }
    }
}

impl ScreenPagingSize{

    pub fn from_str(style: &str) -> Option<ScreenPagingSize> {
        match style {
            "Byte" => Some(ScreenPagingSize::Byte),
            "Row" => Some(ScreenPagingSize::Row),
            "Page" => Some(ScreenPagingSize::Page),
            _ => None
        }
    }
}
