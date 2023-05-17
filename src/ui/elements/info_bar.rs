use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, QueueableCommand};

use crate::cursor::{Cursor, CursorState};
use crate::config::{ColorScheme, Config};
use crate::FileBuffer;

pub struct InfoBar {
    w: u16,
}

impl InfoBar {

    pub fn new(width: u16) -> InfoBar {
        InfoBar { w: width }
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, offset: usize, color_scheme: &ColorScheme, config: &Config) {

        stdout.queue(SetForegroundColor(color_scheme.infobar_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.infobar_bg_color)).unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();

        //generate strings
        let mode = match cursor_state.state() {
            CursorState::Hidden => "-",
            CursorState::Normal => "N",
            CursorState::Text => "T",
            CursorState::Byte => "B",
        };

        let modified = if file_buffers[active_fb_index].is_modified() { "+" } else { "" };
        let file_size = file_buffers[active_fb_index].len();
        let left_size_str = format!(" [{}/{}] {}{}", active_fb_index + 1, file_buffers.len(), modified, file_buffers[active_fb_index].filename());

        let printable = if config.only_printable { "P" } else { "-" };
        let lock = if config.lock_file_buffers { "L" } else { "-" };
        let hl_diff = if config.highlight_diff { "D" } else { "-" };

        let ll = file_buffers[active_fb_index].location_list();
        let right_size_str = if ll.is_empty() {
            format!("({}) {:08X}/{:08X} {:0.1}%  {}{}{}{} [0/0]", offset, offset, file_size, offset as f64 / file_size as f64 * 100.0, mode, printable, lock, hl_diff)
        } else {
            format!("({}) {:08X}/{:08X} {:0.1}%  {}{}{}{} [{}/{}]", offset, offset, file_size, offset as f64 / file_size as f64 * 100.0, mode, printable, lock, hl_diff, ll.current_index()+1, ll.len())
        };

        //chars().count() better represent length of string then number of bytes
        let left_size_str_len = left_size_str.chars().count();

        //handle printing according to terminal size
        if left_size_str_len <= self.w as usize {
            stdout.queue(Print(left_size_str)).unwrap();

            if (left_size_str_len + right_size_str.len()) <= self.w as usize {
                let spacer_len = self.w as usize - left_size_str_len - right_size_str.len();
                stdout.queue(Print(" ".repeat(spacer_len))).unwrap();
                stdout.queue(Print(right_size_str)).unwrap();

            } else {
                stdout.queue(Print(" ".repeat(self.w as usize - left_size_str_len))).unwrap();
            }
        } else {
            stdout.queue(Print(" ".repeat(self.w as usize))).unwrap();
        }
    }
}
