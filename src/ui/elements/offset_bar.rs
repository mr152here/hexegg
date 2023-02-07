use crossterm::style::{Print, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, QueueableCommand};

use crate::config::ColorScheme;
use crate::ui::elements::Element;

pub struct OffsetBar {
    x: u16,
    y: u16,
    h: u16
}

impl OffsetBar {
    pub fn new(x: u16, y: u16, h: u16) -> OffsetBar {
        OffsetBar { x, y, h }
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout, offset: usize, step: u16, color_scheme: &ColorScheme) {

        stdout.queue(SetForegroundColor(color_scheme.offsetbar_fg_color)).unwrap();
        stdout.queue(SetBackgroundColor(color_scheme.offsetbar_bg_color)).unwrap();
        for i in 0..self.h {
            let o = offset + (i * step) as usize;
            stdout.queue(cursor::MoveTo(self.x, self.y + i)).unwrap();
            stdout.queue(Print(format!("{:08X}:", o))).unwrap();
        }
    }
}

impl Element for OffsetBar {

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
        9
    }

    fn set_width(&mut self, _w: u16) {
    }

    fn height(&self) -> u16 {
        self.h
    }

    fn set_height(&mut self, h: u16) {
        self.h = h;
    }

    fn contains_position(&self, col: u16, row: u16) -> bool {
        let x1 = self.x + 9;
        let y1 = self.y + self.h;
        self.x <= col && self.y <= row && x1 > col && y1 > row
    }

    fn to_local_position(&self, _col: u16, _row: u16) -> Option<(u16, u16)> {
        None
    }
}
