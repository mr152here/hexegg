use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Copy, Clone)]
pub enum CursorState {
    Hidden,
    Text,
    Byte,
}

pub struct Cursor {
    position: usize,
    state: CursorState,
    ho_part: bool
}

impl Cursor {

    pub fn new(position: usize, state: CursorState) -> Cursor {
        Cursor {
            position,
            state,
            ho_part: true
        }
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
        self.ho_part = true;
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_state(&mut self, state: CursorState) {
        self.state = state;
        self.ho_part = true;
    }

    pub fn state(&self) -> CursorState {
        self.state
    }

    pub fn ho_byte_part(&self) -> bool {
        self.ho_part
    }

    pub fn set_ho_byte_part(&mut self, value: bool) {
        self.ho_part = value;
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self.state, CursorState::Hidden)
    }

    pub fn is_text(&self) -> bool {
        matches!(self.state, CursorState::Text)
    }

    pub fn is_byte(&self) -> bool {
        matches!(self.state, CursorState::Byte)
    }
}


impl Add<usize> for Cursor {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            position: self.position + rhs,
            state: self.state,
            ho_part: true
        }
    }
}

impl AddAssign<usize> for Cursor {

    fn add_assign(&mut self, rhs: usize) {
        self.position += rhs;
        self.ho_part = true;
    }
}

impl Sub<usize> for Cursor {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self {
            position: self.position - rhs.clamp(0, self.position),
            state: self.state,
            ho_part: true
        }
    }
}

impl SubAssign<usize> for Cursor {

    fn sub_assign(&mut self, rhs: usize) {
        self.position -= rhs.clamp(0, self.position);
        self.ho_part = true;
    }
}
