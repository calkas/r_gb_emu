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

pub mod clock {
    pub const CPU_CLOCK_FREQUENCY: u32 = 4194304;
    pub const CYCLE_SPEED: u32 = CPU_CLOCK_FREQUENCY / 4; // 1_048_576 = 1MHz
}
