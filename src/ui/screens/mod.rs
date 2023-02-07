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

    fn is_over_location_bar(&self, col: u16, row: u16) -> bool;
    fn is_over_data_area(&self, col: u16, row: u16) -> bool;
    fn screen_coord_to_file_offset(&self, init_offset: usize, column: u16, row: u16) -> Option<usize>;

    fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, color_scheme: &ColorScheme, config: &Config); 
}
