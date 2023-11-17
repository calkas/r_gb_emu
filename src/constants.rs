#![allow(dead_code)]

pub mod address {
    pub const INTF_REGISTER: u16 = 0xFF0F;
    pub const INTE_REGISTER: u16 = 0xFFFF;
}

/// ISR_ADDRESS
///
/// * [0] => Bit 0 Vblank        Priority = 1
/// * [1] => Bit 1 LCD Status    Priority = 2
/// * [2] => Bit 2 Timer         Priority = 3
/// * [3] => Bit 3 Serial Link   Priority = 4
/// * [4] => Bit 4 Joypad        Priority = 5
pub mod isr_adress {
    pub const V_BLANK: u16 = 0x0040;
    pub const LCD_STATUS: u16 = 0x0048;
    pub const TIMER: u16 = 0x0050;
    pub const SERIAL_LINK: u16 = 0x0058;
    pub const JOYPAD: u16 = 0x0060;
}
