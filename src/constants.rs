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
        pub const ECHO_RAM_BANK_0: RangeInclusive<u16> = 0xE000..=0xEFFF;
        pub const ECHO_RAM_BANK_1_7: RangeInclusive<u16> = 0xF000..=0xFDFF;
        pub const OAM: RangeInclusive<u16> = 0xFE00..=0xFE9F;
        pub const NOT_USABLE: RangeInclusive<u16> = 0xFEA0..=0xFEFF;

        pub const HARDWARE_IO_SERIAL: RangeInclusive<u16> = 0xFF01..=0xFF02;
        pub const HARDWARE_IO_TIMER: RangeInclusive<u16> = 0xFF04..=0xFF07;

        //pub const HARDWARE_IO_REGISTERS_1: RangeInclusive<u16> = 0xFF00..=0xFF0E;
        //pub const HARDWARE_IO_REGISTERS_2: RangeInclusive<u16> = 0xFF10..=0xFF7F;
        pub const HIGH_RAM: RangeInclusive<u16> = 0xFF80..=0xFFFE;

        pub mod cartridge_header {
            //CARTRIDGE HEADER
            pub const ENTRY_POINT: u16 = 0x0100;
            pub const CARTRIDGE_TYPE: u16 = 0x0147;
            pub const ROM_SIZE: u16 = 0x0148;
            pub const RAM_SIZE: u16 = 0x0149;
            pub const HEADER_CHECKSUM: u16 = 0x014D;
        }
        //IO_HARDWARE_REGISTERS
        pub mod io_hardware_register {
            pub const JOYPAD_INPUT: u16 = 0xFF00;

            pub const SERIAL_DATA: u16 = 0xFF01;
            pub const SERIAL_CONTROL: u16 = 0xFF02;

            pub const TIMER_DIV: u16 = 0xFF04;
            pub const TIMER_TIMA: u16 = 0xFF05;
            pub const TIMER_TMA: u16 = 0xFF06;
            pub const TIMER_TAC: u16 = 0xFF07;
        }
        pub const INTF_REGISTER: u16 = 0xFF0F;
        pub const INTE_REGISTER: u16 = 0xFFFF;
    }

    pub mod memory {
        pub const DEFAULT_INIT_VALUE: u8 = 0;
        pub const HIGH_RAM_SIZE: usize = 0x7F;
        pub const WRAM_SIZE: usize = 0x2000; // Temporary solution when CGB will be support
        pub const WRAM_ADDRESS_MASK: usize = 0x0FFF;
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

pub mod clock {
    pub const CPU_CLOCK_FREQUENCY: u32 = 4194304;
    pub const CPU_MAX_CYCLES: u32 = 69905;
}
