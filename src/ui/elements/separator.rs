use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, QueueableCommand};

use crate::config::ColorScheme;
use crate::ui::elements::Element;

pub struct Separator {
    x: u16,
    y: u16,
    w: u16,
    h: u16
}


impl Separator {

    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Separator {
        Separator{ x, y, w, h }
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, color_scheme: &ColorScheme) {

        stdout.queue(SetForegroundColor(color_scheme.fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.bg_color)).unwrap();

        for i in 0..self.h {
            stdout.queue(cursor::MoveTo(self.x, self.y + i)).unwrap();
            stdout.queue(Print(" ".repeat(self.w as usize))).unwrap();
        }
    }
}


impl Element for Separator {

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

    fn to_local_coords(&self, _col: u16, _row: u16) -> Option<(u16, u16)> {
        None
    }
}
