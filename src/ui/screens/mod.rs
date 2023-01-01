use crate::cursor::Cursor;
use crate::config::{ColorScheme, Config};
use crate::file_buffer::FileBuffer;

pub mod text_screen;
pub mod byte_screen;

pub trait Screen {

    fn row_size(&self) -> u16;
    fn page_size(&self) -> usize;

    fn inc_row_size(&mut self);
    fn dec_row_size(&mut self);
    
    fn show_offset_bar(&mut self, value: bool);
    fn toggle_offset_bar(&mut self);

    fn show_info_bar(&mut self, value: bool);
    fn toggle_info_bar(&mut self);

    fn show_location_bar(&mut self, value: bool);
    fn toggle_location_bar(&mut self);

    fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, color_scheme: &ColorScheme, config: &Config); 
}
