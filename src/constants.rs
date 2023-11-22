#![allow(dead_code)]

/// # GameBoy Memory Map
///
/// [0x0000 - 0x3FFF] 16 KiB ROM bank 00
///
/// [0x4000 - 0x7FFF] 16 KiB ROM Bank 01~NN
///
/// [0x8000 - 0x9FFF] 8 KiB Video RAM (VRAM)
///
/// [0xA000 - 0xBFFF] 8 KiB External RAM
///
/// [0xC000 - 0xCFFF] 4 KiB Work RAM (WRAM)
///
/// [0xD000 - 0xDFFF] 4 KiB Work RAM (WRAM)
///
/// [0xE000 - 0xFDFF] Mirror of C000~DDFF (ECHO RAM)
///
/// [0xFE00 - 0xFE9F] Object attribute memory (OAM)
///
/// [0xFEA0 - 0xFEFF] Not Usable
///
/// [0xFF00 - 0xFF7F] I/O Registers
///
/// [0xFF80 - 0xFFFE] High RAM (HRAM)
///
/// [0xFFFF - 0xFFFF] Interrupt Enable register (IE)
pub mod gb_memory_map {
    pub mod address {
        use std::ops::RangeInclusive;

        pub const CARTRIDGE_ROM_BANK_0: RangeInclusive<u16> = 0x0000..=0x3FFF;
        pub const CARTRIDGE_ROM_BANK_1_N: RangeInclusive<u16> = 0x4000..=0x7FFF;
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

    pub mod memory {
        pub const INIT_VALUE: u8 = 0xFF;
        pub const CARTRIDGE_ROM_SIZE: usize = 0x8000;
        pub const HIGH_RAM_SIZE: usize = 0x7F;
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
}
