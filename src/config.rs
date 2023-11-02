use std::collections::HashMap;
use crossterm::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub highlight_diff: bool,
    pub only_printable: bool,
    pub lock_file_buffers: bool,
    pub screen_paging_size: ScreenPagingSize,
    pub mouse_enabled: bool,
    pub mouse_scroll_type: ScreenPagingSize,
    pub mouse_scroll_size: usize,
    pub esc_to_quit: bool,
    pub clear_screen_on_exit: bool,
    pub highlight_style: HighlightStyle,
    pub active_color_scheme: String,
    pub yank_to_program: Vec<String>,
    pub stdin_input: StdinInput,
    aliases: Vec<(String, String)>,
    pub default_screen: String,
    screen_settings: Vec<ScreenSettings>,
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

#[derive(Deserialize)]
pub struct ScreenSettings {
    pub name: String,
    pub enabled: bool,
    pub data_area_width: u16,
    pub location_bar_width: u16,
    pub show_info_bar: bool,
    pub show_offset_bar: bool,
    pub show_location_bar: bool
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

#[derive(Deserialize)]
pub enum StdinInput {
    Pipe,
    Always,
    Never
}

impl Config {

    pub fn color_scheme(&self, name: &str) -> Option<&ColorScheme> {
        self.color_scheme.iter().find(|cs| cs.name.eq(name))
    }

    pub fn screen_settings(&self, name: &str) -> Option<&ScreenSettings> {
        self.screen_settings.iter().find(|s| s.name.eq(name))
    }

    pub fn aliases(&self) -> HashMap<String, String> {
        self.aliases.iter()
            .map(|(k,v)| (k.clone(), v.clone()))
            .collect::<HashMap<String, String>>()
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

impl ScreenPagingSize {

    pub fn from_str(style: &str) -> Option<ScreenPagingSize> {
        match style {
            "Byte" => Some(ScreenPagingSize::Byte),
            "Row" => Some(ScreenPagingSize::Row),
            "Page" => Some(ScreenPagingSize::Page),
            _ => None
        }
    }
}
