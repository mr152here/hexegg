use crate::file_buffer::FileBuffer;
use crate::ui::screens::Screen;
use crate::ui::elements::Element;
use crate::ui::elements::info_bar::InfoBar;
use crate::ui::elements::location_bar::LocationBar;
use crate::ui::elements::offset_bar::OffsetBar;
use crate::ui::elements::text_area::TextArea;
use crate::ui::elements::separator::Separator;
use crate::config::{ColorScheme, Config};
use crate::cursor::Cursor;

pub struct TextScreen {
    h: u16,
    info_bar: InfoBar,
    offset_bar: OffsetBar,
    left_separator: Separator,
    right_separator: Separator,
    text_area: TextArea,
    location_bar: LocationBar,
    max_text_area_width: u16,
    show_info_bar: bool,
    show_offset_bar: bool,
    show_location_bar: bool,
}

impl TextScreen {
    pub fn new(w: u16, h: u16) -> TextScreen {
        let ib = InfoBar::new(w);
        let ob = OffsetBar::new(0, ib.height(), h-ib.height());
        let ls = Separator::new(ob.width(), ib.height(), 1, h-ib.height());
        let lb = LocationBar::new(w-8, ib.height(), 8, h-ib.height());
        let rs = Separator::new(w-1, ib.height(), 1, h-ib.height());
        let ta = TextArea::new(ob.width() + ls.width(), ib.height(), rs.x0()-ls.x0()-1, h-ib.height());
        let max_ta_width = ta.width();

        TextScreen {
            h,
            info_bar: ib,
            offset_bar: ob,
            left_separator: ls,
            right_separator: rs,
            text_area: ta,
            location_bar: lb,
            max_text_area_width: max_ta_width,
            show_info_bar: true,
            show_offset_bar: true,
            show_location_bar: false
        }
    }
}

impl Screen for TextScreen {

    fn row_size(&self) -> u16 {
        self.text_area.width()
    }

    fn page_size(&self) -> usize {
        self.text_area.page_size()
    }
    
    fn inc_row_size(&mut self) {
        if self.text_area.width() < self.max_text_area_width {
            self.text_area.set_width(self.text_area.width() + 1);
            self.right_separator.set_width(self.right_separator.width() - 1);
            self.right_separator.set_x0(self.right_separator.x0() + 1);
        }
    }

    fn dec_row_size(&mut self) {
        if self.text_area.width() > 1 {
            self.text_area.set_width(self.text_area.width() - 1);
            self.right_separator.set_width(self.right_separator.width() + 1);
            self.right_separator.set_x0(self.right_separator.x0() - 1);
        }
    }

    fn show_info_bar(&mut self, value: bool) {
        let y0 = value as u16; 
        let h = self.h - y0;
        self.offset_bar.set_y0(y0);
        self.offset_bar.set_height(h);
        self.left_separator.set_y0(y0);
        self.left_separator.set_height(h);
        self.right_separator.set_y0(y0);
        self.right_separator.set_height(h);
        self.text_area.set_y0(y0);
        self.text_area.set_height(h);
        self.location_bar.set_y0(y0);
        self.location_bar.set_height(h);
        self.show_info_bar = value;
    }

    fn toggle_info_bar(&mut self) {
        self.show_info_bar(!self.show_info_bar);
    }

    fn show_offset_bar(&mut self, value: bool) {
        if value != self.show_offset_bar{
        let obw = self.offset_bar.width();
            if value {
                self.left_separator.set_x0(self.left_separator.x0() + obw);
                self.text_area.set_x0(self.text_area.x0() + obw);
                self.text_area.set_width(self.text_area.width() - obw);
                self.max_text_area_width -= obw;
            } else {
                self.left_separator.set_x0(self.left_separator.x0() -obw);
                self.text_area.set_x0(self.text_area.x0() - obw);
                self.text_area.set_width(self.text_area.width() + obw);
                self.max_text_area_width += obw;
            }
            self.show_offset_bar = value;
        }
    }

    fn toggle_offset_bar(&mut self) {
        self.show_offset_bar(!self.show_offset_bar);
    }

    fn show_location_bar(&mut self, value: bool) {
        if value != self.show_location_bar {
            let lbw = self.location_bar.width();

            if value {
                self.right_separator.set_x0(self.right_separator.x0() - lbw);
                self.text_area.set_width(self.text_area.width() - lbw);
                self.max_text_area_width -= lbw;
            } else {
                self.right_separator.set_x0(self.right_separator.x0() + lbw);
                self.text_area.set_width(self.text_area.width() + lbw);
                self.max_text_area_width += lbw;
            }

            self.show_location_bar = value;
        }
    }

    fn toggle_location_bar(&mut self) {
        self.show_location_bar(!self.show_location_bar);
    }

    fn is_over_location_bar(&self, col: u16, row: u16) -> bool {
        self.show_location_bar && self.location_bar.contains_position(col, row)
    }

    fn is_over_data_area(&self, col: u16, row: u16) -> bool {
        self.text_area.contains_position(col, row)
    }

    fn screen_coord_to_file_offset(&self, init_offset: usize, column: u16, row: u16) -> Option<usize> {
        if let Some((loc_col, loc_row)) = self.text_area.to_local_position(column, row) {
            let w = self.text_area.width() as usize;
            return Some(init_offset + (loc_row as usize * w) + loc_col as usize);
        }
        None
    }
    
    fn draw(&self, stdout: &mut std::io::Stdout, file_buffers: &[FileBuffer], active_fb_index: usize, cursor_state: &Cursor, color_scheme: &ColorScheme, config: &Config) {
        let offset = file_buffers[active_fb_index].position();

        if self.show_info_bar {
            let o = if cursor_state.is_visible() { cursor_state.position() } else { offset };
            self.info_bar.draw(stdout, file_buffers, active_fb_index, cursor_state, o, color_scheme, config);
        }

        if self.show_offset_bar {
            let step = self.row_size();
            self.offset_bar.draw(stdout, offset, step, color_scheme);
        }

        self.left_separator.draw(stdout, color_scheme);
        self.text_area.draw(stdout, file_buffers, active_fb_index, cursor_state, color_scheme, config);
        self.right_separator.draw(stdout, color_scheme);

        if self.show_location_bar {
            self.location_bar.draw(stdout, file_buffers[active_fb_index].location_list(), color_scheme);
        }
    }
}
