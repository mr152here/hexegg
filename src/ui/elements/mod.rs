pub mod offset_bar;
pub mod text_area;
pub mod byte_area;
pub mod location_bar;
pub mod info_bar;
pub mod user_input;
pub mod message_box;
pub mod separator;

pub trait Element {

    fn x0(&self) -> u16;
    fn set_x0(&mut self, x0: u16);

    fn y0(&self) -> u16;
    fn set_y0(&mut self, y0: u16);

    fn width(&self) -> u16;
    fn set_width(&mut self, w: u16);

    fn height(&self) -> u16;
    fn set_height(&mut self, h: u16);

    fn to_local_coords(&self, col: u16, row: u16) -> Option<(u16, u16)>;
}
