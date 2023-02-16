use crossterm::style::{Print, SetForegroundColor, SetBackgroundColor};
use crossterm::QueueableCommand;

use crate::config::{ColorScheme, Config};
use crate::file_buffer::FileBuffer;
use crate::cursor::Cursor;
use crate::ui::elements::Element;

//lookup table for fast converting byte to string.
pub const BYTE_TO_STR_TABLE: &str= "000102030405060708090A0B0C0D0E0F\
                                    101112131415161718191A1B1C1D1E1F\
                                    202122232425262728292A2B2C2D2E2F\
                                    303132333435363738393A3B3C3D3E3F\
                                    404142434445464748494A4B4C4D4E4F\
                                    505152535455565758595A5B5C5D5E5F\
                                    606162636465666768696A6B6C6D6E6F\
                                    707172737475767778797A7B7C7D7E7F\
                                    808182838485868788898A8B8C8D8E8F\
                                    909192939495969798999A9B9C9D9E9F\
                                    A0A1A2A3A4A5A6A7A8A9AAABACADAEAF\
                                    B0B1B2B3B4B5B6B7B8B9BABBBCBDBEBF\
                                    C0C1C2C3C4C5C6C7C8C9CACBCCCDCECF\
                                    D0D1D2D3D4D5D6D7D8D9DADBDCDDDEDF\
                                    E0E1E2E3E4E5E6E7E8E9EAEBECEDEEEF\
                                    F0F1F2F3F4F5F6F7F8F9FAFBFCFDFEFF";

//undefined bytes (out of range) are represented as this str
const UNDEFINED_BYTE: &str = "??";

//function that convert byte to "str"
fn byte_to_str(byte: u8) -> &'static str {
    let index = 2*byte as usize;
    &BYTE_TO_STR_TABLE[index..index+2]
}


pub struct WordArea {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    bytes_per_row: u16,
}


impl WordArea {

    pub fn new(x: u16, y: u16, w: u16, h: u16) -> WordArea {
        WordArea {
            x, y, w, h,
            bytes_per_row: (w + 1) / 5 * 2,
        }
    }

    pub fn row_size(&self) -> u16 {
        self.bytes_per_row
    }

    pub fn page_size(&self) -> usize {
        self.bytes_per_row as usize * self.h as usize
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, color_scheme: &ColorScheme, config: &Config) {

        let file_buffer = &file_buffers[active_fb_index];
        let offset = file_buffer.position();
        let mut counter: usize = 0;
        let mut current_fg_color = color_scheme.fg_color;
        let mut current_bg_color = color_scheme.bg_color;
        let mut second_byte = false;

        stdout.queue(SetForegroundColor(current_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(current_bg_color)).unwrap();


        for y in self.y..self.y + self.h {
            stdout.queue(crossterm::cursor::MoveTo(self.x, y)).unwrap();

            for x in 0..self.bytes_per_row {
                let byte = file_buffer.get(offset + counter);

                //select correct colors
                let (new_fg_color, new_bg_color) = if cursor_state.is_visible() && cursor_state.position() == offset + counter {
                    (color_scheme.cursor_fg_color, color_scheme.cursor_bg_color)

                } else {
                    let new_fg_color = if file_buffer.is_patched(offset + counter) {
                        color_scheme.patch_fg_color

                    } else if config.highlight_diff && byte.is_some() && file_buffers.iter().any(|fb| if let Some(b) = fb.get(offset + counter) { b != byte.unwrap() } else { true }) {
                        color_scheme.diff_fg_color

                    } else {
                        color_scheme.fg_color
                    };

                    let new_bg_color = if file_buffer.is_selected(offset + counter) {
                        color_scheme.selection_bg_color

                    } else if let Some((_,_,color)) = file_buffer.get_highlight(offset + counter) {
                        color

                    } else {
                        color_scheme.bg_color
                    };
                    (new_fg_color, new_bg_color)
                };

                if new_bg_color != current_bg_color {
                    stdout.queue(SetBackgroundColor(new_bg_color)).unwrap();
                    current_bg_color = new_bg_color;
                }

                if new_fg_color != current_fg_color {
                    stdout.queue(SetForegroundColor(new_fg_color)).unwrap();
                    current_fg_color = new_fg_color;
                }

                let s = byte.map_or(UNDEFINED_BYTE, |b| byte_to_str(b));
                stdout.queue(Print(s)).unwrap();

                //reset color to the colorscheme fg/bg color before printing deliminer
                if current_fg_color != color_scheme.fg_color {
                    stdout.queue(SetForegroundColor(color_scheme.fg_color)).unwrap();
                    current_fg_color = color_scheme.fg_color;
                }

                if current_bg_color != color_scheme.bg_color {
                    stdout.queue(SetBackgroundColor(color_scheme.bg_color)).unwrap();
                    current_bg_color = color_scheme.bg_color;
                }

                //print free space if it is not the last number in the row
                if second_byte && x != self.bytes_per_row - 1 {
                    stdout.queue(Print(" ")).unwrap();
                }
                second_byte = !second_byte;
                counter += 1;
            }

            //fill the rest of the row with empty space
            let fill_width = self.w - self.bytes_per_row / 2 * 5 + 1;
            stdout.queue(Print(" ".repeat(fill_width as usize))).unwrap();
        }
    }
}

impl Element for WordArea {

    fn x0(&self) -> u16 {
        self.x
    }

    fn set_x0(&mut self, x0: u16) {
        self.x = x0;
    }

    fn y0(&self) -> u16 {
        self.y
    }

    fn set_y0(&mut self, y0: u16) {
        self.y = y0;
    }

    fn width(&self) -> u16 {
        self.w
    }

    fn set_width(&mut self, w: u16) {
        self.w = w;
        self.bytes_per_row = (w + 1) / 5 * 2;
    }

    fn height(&self) -> u16 {
        self.h
    }

    fn set_height(&mut self, h: u16) {
        self.h = h;
    }

    fn contains_position(&self, col: u16, row: u16) -> bool {
        let x1 = self.x + self.w;
        let y1 = self.y + self.h;
        self.x <= col && self.y <= row && x1 > col && y1 > row
    }

    fn to_local_position(&self, col: u16, row: u16) -> Option<(u16, u16)> {
        self.contains_position(col, row).then_some((2 * (col - self.x) / 5, row - self.y))
    }
}
