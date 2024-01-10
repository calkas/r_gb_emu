use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::{address, memory};

/// # LCD Control Register
#[derive(Clone, Copy, Default)]
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
        Self {
            lcd_enable: (value.rotate_left(7) & 1) == 1,
            window_tile_map_area: (value.rotate_left(6) & 1) == 1,
            window_enable: (value.rotate_left(5) & 1) == 1,
            bg_window_tile_data_area: (value.rotate_left(4) & 1) == 1,
            bg_tile_map_area: (value.rotate_left(3) & 1) == 1,
            obj_size: (value.rotate_left(2) & 1) == 1,
            obj_enable: (value.rotate_left(1) & 1) == 1,
            bg_window_enable_priority: (value.rotate_left(0) & 1) == 1,
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
#[derive(Clone, Copy, Default)]
struct LcdStatusRegister {
    // Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    ly_interrupt: bool,
    // Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    mode_2_interrupt: bool,
    // Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    mode_1_interrupt: bool,
    // Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    mode_0_interrupt: bool,
    // Bit 2 LYC == LY
    lyc_flag: bool,
    // Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    //    0: During H-Blank
    //    1: During V-Blank
    //    2: During Searching OAM
    //    3: During Transferring Data to LCD Driver
    ppu_mode: u8,
}

impl std::convert::From<u8> for LcdStatusRegister {
    fn from(value: u8) -> Self {
        Self {
            ly_interrupt: (value.rotate_left(6) & 1) == 1,
            mode_2_interrupt: (value.rotate_left(5) & 1) == 1,
            mode_1_interrupt: (value.rotate_left(4) & 1) == 1,
            mode_0_interrupt: (value.rotate_left(3) & 1) == 1,
            lyc_flag: (value.rotate_left(2) & 1) == 1,
            ppu_mode: value & 0x03,
        }
    }
}

impl std::convert::From<LcdStatusRegister> for u8 {
    fn from(register: LcdStatusRegister) -> Self {
        let mut out_value: u8 = 0;
        if register.ly_interrupt {
            out_value |= 1_u8.rotate_left(6);
        }
        if register.mode_2_interrupt {
            out_value |= 1_u8.rotate_left(5);
        }
        if register.mode_1_interrupt {
            out_value |= 1_u8.rotate_left(4);
        }
        if register.mode_0_interrupt {
            out_value |= 1_u8.rotate_left(3);
        }
        if register.lyc_flag {
            out_value |= 1_u8.rotate_left(2);
        }
        out_value |= register.ppu_mode;
        return out_value;
    }
}

/// # PPU (Picture Processing Unit)
/// On Gameboy Classic there's only one way to initialize VRAM - manually copy data with CPU instructions. This is done in bootstrap ROM process:
pub struct PictureProcessingUnit {
    vram: [u8; memory::VRAM_SIZE],
    voam: [u8; memory::VOAM_SIZE],
    lcd_control_register: LcdControlRegister,
    lcd_stat_register: LcdStatusRegister,
    scy_register: u8,
    scx_register: u8,
    ly_register: u8,
    lyc_register: u8,
}

impl PictureProcessingUnit {
    pub fn new() -> Self {
        PictureProcessingUnit {
            vram: [memory::DEFAULT_INIT_VALUE; memory::VRAM_SIZE],
            voam: [memory::DEFAULT_INIT_VALUE; memory::VOAM_SIZE],
            lcd_control_register: LcdControlRegister::default(),
            lcd_stat_register: LcdStatusRegister::default(),
            scy_register: 0,
            scx_register: 0,
            ly_register: 0,
            lyc_register: 0,
        }
    }
}

impl HardwareAccessible for PictureProcessingUnit {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            vram_address if address::VIDEO_RAM.contains(&vram_address) => {
                self.vram[vram_address as usize]
            }
            voam_address if address::OAM.contains(&voam_address) => {
                self.voam[voam_address as usize]
            }
            address::io_hardware_register::LCD_CONTROL => {
                LcdControlRegister::into(self.lcd_control_register)
            }
            address::io_hardware_register::LCD_STATUS => {
                LcdStatusRegister::into(self.lcd_stat_register)
            }
            address::io_hardware_register::SCY => self.scy_register,
            address::io_hardware_register::SCX => self.scx_register,
            address::io_hardware_register::LY => self.ly_register,
            address::io_hardware_register::LYC => self.lyc_register,

            _ => panic!("[PPU ERROR][Read] Unsupported address: [{:#06x?}]", address),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            vram_address if address::VIDEO_RAM.contains(&vram_address) => {
                self.vram[vram_address as usize] = data
            }
            voam_address if address::OAM.contains(&voam_address) => {
                self.voam[voam_address as usize] = data
            }
            address::io_hardware_register::LCD_CONTROL => {
                self.lcd_control_register = LcdControlRegister::from(data)
            }
            address::io_hardware_register::LCD_STATUS => {
                self.lcd_stat_register = LcdStatusRegister::from(data)
            }
            address::io_hardware_register::SCY => self.scy_register = data,
            address::io_hardware_register::SCX => self.scx_register = data,
            address::io_hardware_register::LY => self.ly_register = data,
            address::io_hardware_register::LYC => self.lyc_register = data,
            _ => panic!(
                "[PPU ERROR][Write] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }
}

impl IoWorkingCycle for PictureProcessingUnit {
    fn next_to(&mut self, cycles: u32) {
        if !self.lcd_control_register.lcd_enable {
            return;
        }
    }
}

#[cfg(test)]
mod uint_test {
    use super::*;
    #[test]
    fn lcd_control_register_convert_test() {
        let mut register = LcdControlRegister::from(0xAA);
        assert!(register.lcd_enable == true);
        assert!(register.window_tile_map_area == false);
        assert!(register.window_enable == true);
        assert!(register.bg_window_tile_data_area == false);
        assert!(register.bg_tile_map_area == true);
        assert!(register.obj_size == false);
        assert!(register.obj_enable == true);
        assert!(register.bg_window_enable_priority == false);

        register.window_tile_map_area = true;
        register.bg_window_tile_data_area = true;
        assert_eq!(0xFA as u8, LcdControlRegister::into(register));
    }

    #[test]
    fn lcd_stat_register_convert_test() {
        let mut register = LcdStatusRegister::from(0x2B);

        assert!(register.ly_interrupt == false);
        assert!(register.mode_2_interrupt == true);
        assert!(register.mode_1_interrupt == false);
        assert!(register.mode_0_interrupt == true);
        assert!(register.lyc_flag == false);
        assert!(register.ppu_mode == 3);

        register.ly_interrupt = true;
        register.ppu_mode = 2;
        assert_eq!(0x6A as u8, LcdStatusRegister::into(register));
    }
}
