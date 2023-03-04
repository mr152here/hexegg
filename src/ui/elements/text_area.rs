use crossterm::style::{Print, SetForegroundColor, SetBackgroundColor};
use crossterm::QueueableCommand;

use crate::config::{ColorScheme, Config};
use crate::file_buffer::FileBuffer;
use crate::cursor::Cursor;
use crate::ui::elements::Element;

//lookup table for converting byte to UTF8 cp437 charset representation (except x00 and xFF which are replaced by \u{0020} and \u{237D})
const BYTE_TO_UTF8_TABLE: [&str; 256] = [
     //   0,         1,         2,         3,         4,         5,         6,         7,         8,         9,         A,         B,         C,         D,         E,         F
     "\u{0020}","\u{263A}","\u{263B}","\u{2665}","\u{2666}","\u{2663}","\u{2660}","\u{2022}","\u{25D8}","\u{25CB}","\u{25D9}","\u{2642}","\u{2640}","\u{266A}","\u{266B}","\u{263C}",
     "\u{25BA}","\u{25C4}","\u{2195}","\u{203C}","\u{00B6}","\u{00A7}","\u{25AC}","\u{21A8}","\u{2191}","\u{2193}","\u{2192}","\u{2190}","\u{221F}","\u{2194}","\u{25B2}","\u{25BC}",
     "\u{0020}","\u{0021}","\u{0022}","\u{0023}","\u{0024}","\u{0025}","\u{0026}","\u{0027}","\u{0028}","\u{0029}","\u{002A}","\u{002B}","\u{002C}","\u{002D}","\u{002E}","\u{002F}", 
     "\u{0030}","\u{0031}","\u{0032}","\u{0033}","\u{0034}","\u{0035}","\u{0036}","\u{0037}","\u{0038}","\u{0039}","\u{003A}","\u{003B}","\u{003C}","\u{003D}","\u{003E}","\u{003F}",
     "\u{0040}","\u{0041}","\u{0042}","\u{0043}","\u{0044}","\u{0045}","\u{0046}","\u{0047}","\u{0048}","\u{0049}","\u{004A}","\u{004B}","\u{004C}","\u{004D}","\u{004E}","\u{004F}", 
     "\u{0050}","\u{0051}","\u{0052}","\u{0053}","\u{0054}","\u{0055}","\u{0056}","\u{0057}","\u{0058}","\u{0059}","\u{005A}","\u{005B}","\u{005C}","\u{005D}","\u{005E}","\u{005F}",
     "\u{0060}","\u{0061}","\u{0062}","\u{0063}","\u{0064}","\u{0065}","\u{0066}","\u{0067}","\u{0068}","\u{0069}","\u{006A}","\u{006B}","\u{006C}","\u{006D}","\u{006E}","\u{006F}", 
     "\u{0070}","\u{0071}","\u{0072}","\u{0073}","\u{0074}","\u{0075}","\u{0076}","\u{0077}","\u{0078}","\u{0079}","\u{007A}","\u{007B}","\u{007C}","\u{007D}","\u{007E}","\u{2302}",
     "\u{00C7}","\u{00FC}","\u{00E9}","\u{00E2}","\u{00E4}","\u{00E0}","\u{00E5}","\u{00E7}","\u{00EA}","\u{00EB}","\u{00E8}","\u{00EF}","\u{00EE}","\u{00EC}","\u{00C4}","\u{00C5}", 
     "\u{00C9}","\u{00E6}","\u{00C6}","\u{00F4}","\u{00F6}","\u{00F2}","\u{00FB}","\u{00F9}","\u{00FF}","\u{00D6}","\u{00DC}","\u{00A2}","\u{00A3}","\u{00A5}","\u{20A7}","\u{0192}",
     "\u{00E1}","\u{00ED}","\u{00F3}","\u{00FA}","\u{00F1}","\u{00D1}","\u{00AA}","\u{00BA}","\u{00BF}","\u{2310}","\u{00AC}","\u{00BD}","\u{00BC}","\u{00A1}","\u{00AB}","\u{00BB}", 
     "\u{2591}","\u{2592}","\u{2593}","\u{2502}","\u{2524}","\u{2561}","\u{2562}","\u{2556}","\u{2555}","\u{2563}","\u{2551}","\u{2557}","\u{255D}","\u{255C}","\u{255B}","\u{2510}",
     "\u{2514}","\u{2534}","\u{252C}","\u{251C}","\u{2500}","\u{253C}","\u{255E}","\u{255F}","\u{255A}","\u{2554}","\u{2569}","\u{2566}","\u{2560}","\u{2550}","\u{256C}","\u{2567}", 
     "\u{2568}","\u{2564}","\u{2565}","\u{2559}","\u{2558}","\u{2552}","\u{2553}","\u{256B}","\u{256A}","\u{2518}","\u{250C}","\u{2588}","\u{2584}","\u{258C}","\u{2590}","\u{2580}",
     "\u{03B1}","\u{00DF}","\u{0393}","\u{03C0}","\u{03A3}","\u{03C3}","\u{00B5}","\u{03C4}","\u{03A6}","\u{0398}","\u{03A9}","\u{03B4}","\u{221E}","\u{03C6}","\u{03B5}","\u{2229}", 
     "\u{2261}","\u{00B1}","\u{2265}","\u{2264}","\u{2320}","\u{2321}","\u{00F7}","\u{2248}","\u{00B0}","\u{2219}","\u{00B7}","\u{221A}","\u{207F}","\u{00B2}","\u{25A0}","\u{237D}"
     ];

//how to print byte that doesn't exits in file or are hidden
const UNDEFINED_BYTE: &str = "?";
const HIDDEN_BYTE: &str = " ";

//function that convert byte to "str"
fn byte_to_utf8_str(byte: u8) -> &'static str {
    BYTE_TO_UTF8_TABLE[ byte as usize ]
}

//returns true if byte is in printable range
fn is_printable(byte: u8) -> bool {
    (0x20..=0x7E).contains(&byte)
}

pub struct TextArea {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

impl TextArea {

    pub fn new(x: u16, y: u16, w: u16, h: u16) -> TextArea {
        TextArea { x, y, w, h, }
    }

    pub fn page_size(&self) -> usize {
        self.w as usize * self.h as usize
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, color_scheme: &ColorScheme, config: &Config) {
        
        let file_buffer = &file_buffers[active_fb_index];
        let offset = file_buffer.position();
        let mut current_fg_color = color_scheme.fg_color;
        let mut current_bg_color = color_scheme.bg_color;
        stdout.queue(SetForegroundColor(current_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(current_bg_color)).unwrap();

        let mut counter: usize = 0;
        for y in self.y .. self.y + self.h {
            stdout.queue(crossterm::cursor::MoveTo(self.x,y)).unwrap();

            for _ in 0..self.w {
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

                    } else if let Some(color) = file_buffer.highlight_list().color(offset + counter) {
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

                let s = match byte {
                    Some(b) if config.only_printable => if is_printable(b) { byte_to_utf8_str(b) } else { HIDDEN_BYTE },
                    Some(b) => byte_to_utf8_str(b),
                    None => UNDEFINED_BYTE,
                };
                stdout.queue(Print(s)).unwrap();
                counter += 1;
            }
        }
    }
}

impl Element for TextArea {

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
        self.contains_position(col, row).then_some((col - self.x, row - self.y))
    }
}
