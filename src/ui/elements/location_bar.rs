use crossterm::style::{Color, Print, SetForegroundColor, SetBackgroundColor};
use crossterm::{QueueableCommand, cursor};
use crate::location_list::LocationList;
use crate::config::ColorScheme;
use crate::ui::elements::Element;

const EMPTY_LINE: &str = "                                                                               ";

pub struct LocationBar {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    display_from: usize
}

impl LocationBar {

    pub fn new(x: u16, y: u16, w: u16, h: u16) -> LocationBar {
        LocationBar { x, y, w, h, display_from: 0 }
    }

    pub fn location_list_index(&self, row: u16, location_list: &LocationList) -> Option<usize> {
        let clicked = self.display_from + row as usize;
        (clicked < location_list.len()).then_some(clicked)
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout, location_list: &LocationList, color_scheme: &ColorScheme) {

        let mut current_fg_color = color_scheme.location_list_fg_color;
        let mut current_bg_color = color_scheme.location_list_bg_color;
        let selection_index = location_list.current_index();
        let width = self.w as usize;
        let height = self.h as usize;

        if selection_index < self.display_from {
            self.display_from = selection_index;

        } else if selection_index > self.display_from + height - 1 {
            self.display_from = selection_index - height + 1;
        }

        stdout.queue(SetForegroundColor(current_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(current_bg_color)).unwrap();

        for row in 0..self.h {

            stdout.queue(cursor::MoveTo(self.x, self.y + row)).unwrap();

            //select color
            let new_fg_color: Color;
            let new_bg_color: Color;
            
            if selection_index == self.display_from + row as usize && !location_list.is_empty() {
                new_fg_color = color_scheme.location_list_cursor_fg_color;
                new_bg_color = color_scheme.location_list_cursor_bg_color;
            } else {
                new_fg_color = color_scheme.location_list_fg_color;
                new_bg_color = color_scheme.location_list_bg_color;
            }

            if current_fg_color != new_fg_color {
                stdout.queue(SetForegroundColor(new_fg_color)).unwrap();
                current_fg_color = new_fg_color;
            }
            
            if current_bg_color != new_bg_color {
                stdout.queue(SetBackgroundColor(new_bg_color)).unwrap();
                current_bg_color = new_bg_color;
            }

            //print strings from location list. Limit it size to self width
            let mut print_string: String;

            if let Some(location) = location_list.get(self.display_from + row as usize) {
                print_string = location.name.chars()
                    .enumerate()
                    .map_while(|(i, c)| (i < width).then_some(c))
                    .collect();

                //fill the rest with empty line if string is not long enought
                let char_len = print_string.chars().count();
                print_string.push_str(&EMPTY_LINE[0..width-char_len]);

            } else {
                print_string = EMPTY_LINE[0..width].to_owned();
            }

            stdout.queue(Print(print_string)).unwrap();
        }
    }
}

impl Element for LocationBar {

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
