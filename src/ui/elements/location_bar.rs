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
}

impl LocationBar {

    pub fn new(x: u16, y: u16, w: u16, h: u16) -> LocationBar {
        LocationBar { x, y, w, h }
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, location_list: &LocationList, color_scheme: &ColorScheme) {

        let mut current_fg_color = color_scheme.location_list_fg_color;
        let mut current_bg_color = color_scheme.location_list_bg_color;
        let mut loc_iter = location_list.iter();
        let mut start_from_index: usize = 0;
        let selection_index = location_list.current_index();
        let width = self.w as usize;
        let height = self.h as usize;

        //skip rows that can't be displayed
        if selection_index >= height {
            loc_iter.nth(selection_index - height);
            start_from_index = selection_index - height + 1;
        }

        stdout.queue(SetForegroundColor(current_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(current_bg_color)).unwrap();

        for row in 0..self.h {

            stdout.queue(cursor::MoveTo(self.x, self.y + row)).unwrap();

            //select color
            let new_fg_color: Color;
            let new_bg_color: Color;
            
            if selection_index == start_from_index + row as usize && !location_list.is_empty() {
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
            if let Some((_, s)) = loc_iter.next() {
                print_string = s.bytes()
                    .enumerate()
                    .map_while(|(i, b)| (i < width).then_some(b as char) )
                    .collect();

                //fill the rest with empty line if string is too short
                print_string.push_str(&EMPTY_LINE[0..width-print_string.len()]);

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
}
