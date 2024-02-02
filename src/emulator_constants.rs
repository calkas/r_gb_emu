#[derive(PartialEq, Clone, Copy)]
pub enum GameBoyKeys {
    Right,
    Left,
    Down,
    Up,
    A,
    B,
    Select,
    Start,
}

pub mod resolution {
    pub const SCREEN_W: usize = 160;
    pub const SCREEN_H: usize = 144;
}
