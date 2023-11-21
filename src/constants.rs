#![allow(dead_code)]

/// GameBoy Memory Map
pub mod address {
    use std::ops::RangeInclusive;

    pub const CARTRIDGE_ROM: RangeInclusive<u16> = 0x0000..=0x7FFF;
    pub const VIDEO_RAM: RangeInclusive<u16> = 0x8000..=0x9FFF;
    pub const CARTRIDGE_RAM: RangeInclusive<u16> = 0xA000..=0xBFFF;
    pub const WORKING_RAM_BANK_0: RangeInclusive<u16> = 0xC000..=0xCFFF;
    pub const WORKING_RAM_BANK_1_7: RangeInclusive<u16> = 0xD000..=0xDFFF;
    pub const ECHO_RAM: RangeInclusive<u16> = 0xE000..=0xFDFF;
    pub const OAM: RangeInclusive<u16> = 0xFE00..=0xFE9F;
    pub const NOT_USABLE: RangeInclusive<u16> = 0xFEA0..=0xFEFF;
    pub const HARDWARE_IO_REGISTERS: RangeInclusive<u16> = 0xFF00..=0xFF7F;
    pub const HIGH_RAM: RangeInclusive<u16> = 0xFF80..=0xFFFE;
    pub const INTE_REGISTER: u16 = 0xFFFF;

    pub const INTF_REGISTER: u16 = 0xFF0F;
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
