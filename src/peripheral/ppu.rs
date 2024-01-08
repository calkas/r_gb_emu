use std::convert;

use super::{HardwareAccessible, IoWorkingCycle};

/// # LCD Control Register
#[derive(Default)]
struct LcdControlRegister {
    lcd_enable: bool,
    window_tile_map_area: bool,
    window_enable: bool,
    bg_window_tile_data_area: bool,
    bg_tile_map_area: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_window_enable_priority: bool,
}

impl std::convert::From<u8> for LcdControlRegister {
    fn from(value: u8) -> Self {
        let lcd_enable = (value.rotate_left(7) & 1) == 1;
        let window_tile_map_area = (value.rotate_left(6) & 1) == 1;
        let window_enable = (value.rotate_left(5) & 1) == 1;
        let bg_window_tile_data_area = (value.rotate_left(4) & 1) == 1;
        let bg_tile_map_area = (value.rotate_left(3) & 1) == 1;
        let obj_size = (value.rotate_left(2) & 1) == 1;
        let obj_enable = (value.rotate_left(1) & 1) == 1;
        let bg_window_enable_priority = (value.rotate_left(0) & 1) == 1;

        LcdControlRegister {
            lcd_enable,
            window_tile_map_area,
            window_enable,
            bg_window_tile_data_area,
            bg_tile_map_area,
            obj_size,
            obj_enable,
            bg_window_enable_priority,
        }
    }
}

impl std::convert::From<LcdControlRegister> for u8 {
    fn from(register: LcdControlRegister) -> Self {
        let mut out_value: u8 = 0;
        if register.lcd_enable {
            out_value |= 1_u8.rotate_left(7);
        }
        if register.window_tile_map_area {
            out_value |= 1_u8.rotate_left(6);
        }
        if register.window_enable {
            out_value |= 1_u8.rotate_left(5);
        }
        if register.bg_window_tile_data_area {
            out_value |= 1_u8.rotate_left(4);
        }
        if register.bg_tile_map_area {
            out_value |= 1_u8.rotate_left(3);
        }
        if register.obj_size {
            out_value |= 1_u8.rotate_left(2);
        }
        if register.obj_enable {
            out_value |= 1_u8.rotate_left(1);
        }
        if register.bg_window_enable_priority {
            out_value |= 1_u8.rotate_left(0);
        }
        return out_value;
    }
}

/// # LCD Status Register
#[derive(Default)]
struct LcdStatusRegister {
    // Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    ly_interrupt: bool,
    // Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    mode_m2_interrupt: bool,
    // Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    mode_1_interrupt: bool,
    // Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    mode_0_interrupt: bool,
    // Bit 2 LYC == LY
    same_lyc_ly_flag: bool,
    // Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    //    0: During H-Blank
    //    1: During V-Blank
    //    2: During Searching OAM
    //    3: During Transferring Data to LCD Driver
    ppu_mode: u8,
}

/// # PPU (Picture Processing Unit)
pub struct PictureProcessingUnit {
    lcd_control_register: LcdControlRegister,
    lcd_stat_register: LcdStatusRegister,
}

impl PictureProcessingUnit {
    pub fn new() -> Self {
        PictureProcessingUnit {
            lcd_control_register: LcdControlRegister::default(),
            lcd_stat_register: LcdStatusRegister::default(),
        }
    }
}

impl HardwareAccessible for PictureProcessingUnit {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        todo!()
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        todo!()
    }
}

impl IoWorkingCycle for PictureProcessingUnit {
    fn is_interrupt(&self) -> bool {
        todo!()
    }

    fn reset_interrupt(&mut self) {
        todo!()
    }

    fn next(&mut self, cycles: u32) {
        todo!()
    }
}
