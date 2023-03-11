use crate::file_buffer::FileBuffer;
use crate::location_list::LocationList;
use crate::ui::screens::Screen;
use crate::ui::elements::Element;
use crate::ui::elements::info_bar::InfoBar;
use crate::ui::elements::location_bar::LocationBar;
use crate::ui::elements::offset_bar::OffsetBar;
use crate::ui::elements::text_area::TextArea;
use crate::ui::elements::separator::Separator;
use crate::config::{ColorScheme, Config, ScreenSettings};
use crate::cursor::Cursor;

pub struct TextScreen {
    w: u16,
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

    pub fn new(w: u16, h: u16, screen_settings: &ScreenSettings) -> TextScreen {
        Self::create_layout(w, h, screen_settings.data_area_width, screen_settings.show_info_bar, screen_settings.show_offset_bar, screen_settings.show_location_bar, screen_settings.location_bar_width)
    }

    fn create_layout(w: u16, h: u16, data_area_width: u16, show_info_bar: bool, show_offset_bar: bool, show_location_bar: bool, location_bar_width: u16) -> TextScreen {
        let y0 = show_info_bar as u16;
        let new_h = h - y0;

        let ib = InfoBar::new(w);
        let ob = OffsetBar::new(0, y0, new_h);

        let ob_width = if show_offset_bar { ob.width() } else { 0 };
        let lb_width = if show_location_bar { location_bar_width } else { 0 };

        let ls = Separator::new(ob_width, y0, 1, new_h);

        let ta_max_width = w - ob_width - lb_width - ls.width() - 1; //-1 => minimum for rs.width()
        let ta_width = std::cmp::min(data_area_width, ta_max_width);
        let ta = TextArea::new(ob_width + ls.width(), y0, ta_width, new_h);

        let rs_width = w - ob_width - ls.width() - ta_width - lb_width;
        let rs = Separator::new(ta.x0() + ta_width, y0, rs_width, new_h);
        let lb = LocationBar::new(w - location_bar_width, y0, location_bar_width, new_h);

        TextScreen {
            w, h,
            info_bar: ib,
            offset_bar: ob,
            left_separator: ls,
            right_separator: rs,
            text_area: ta,
            location_bar: lb,
            max_text_area_width: ta_max_width,
            show_info_bar,
            show_offset_bar,
            show_location_bar
        }
    }
}

impl Screen for TextScreen {

    fn screen_name(&self) -> &'static str {
        "text_screen"
    }

    fn row_size(&self) -> u16 {
        self.text_area.width()
    }

    fn num_of_rows(&self) -> u16 {
        self.text_area.height()
    }

    fn page_size(&self) -> usize {
        self.text_area.page_size()
    }
    
    fn inc_row_size(&mut self) {
        if self.text_area.width() < self.max_text_area_width {
            *self = Self::create_layout(self.w, self.h, self.text_area.width() + 1, self.show_info_bar, self.show_offset_bar, self.show_location_bar, self.location_bar.width());
        }
    }

    fn dec_row_size(&mut self) {
        if self.text_area.width() > 1 {
            *self = Self::create_layout(self.w, self.h, self.text_area.width() - 1, self.show_info_bar, self.show_offset_bar, self.show_location_bar, self.location_bar.width());
        }
    }

    fn toggle_info_bar(&mut self) {
        let ta_width = if self.text_area.width() == self.max_text_area_width { u16::MAX } else { self.text_area.width() };
        *self = Self::create_layout(self.w, self.h, ta_width, !self.show_info_bar, self.show_offset_bar, self.show_location_bar, self.location_bar.width());
    }

    fn toggle_offset_bar(&mut self) {
        let ta_width = if self.text_area.width() == self.max_text_area_width { u16::MAX } else { self.text_area.width() };
        *self = Self::create_layout(self.w, self.h, ta_width, self.show_info_bar, !self.show_offset_bar, self.show_location_bar, self.location_bar.width());
    }

    fn toggle_location_bar(&mut self) {
        let ta_width = if self.text_area.width() == self.max_text_area_width { u16::MAX } else { self.text_area.width() };
        *self = Self::create_layout(self.w, self.h, ta_width, self.show_info_bar, self.show_offset_bar, !self.show_location_bar, self.location_bar.width());
    }

    fn show_location_bar(&mut self, value: bool) {
        if value != self.show_location_bar {
            *self = Self::create_layout(self.w, self.h, self.text_area.width(), self.show_info_bar, self.show_offset_bar, value, self.location_bar.width());
        }
    }

    fn is_over_location_bar(&self, col: u16, row: u16) -> bool {
        self.show_location_bar && self.location_bar.contains_position(col, row)
    }

    fn is_over_data_area(&self, col: u16, row: u16) -> bool {
        self.text_area.contains_position(col, row)
    }

    fn location_list_index(&self, col: u16, row: u16, location_list: &LocationList) -> Option<usize> {
        if let Some((_, row)) = self.location_bar.to_local_position(col, row) {
            return self.location_bar.location_list_index(row, location_list)
        }
        None
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
