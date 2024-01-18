use self::lcd_monochrome::PaletteRegister;
use self::lcd_monochrome::{Color, PalleteMode};
use self::sprite::Attribute;

use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::{
    gb_memory_map::{address, address::io_hardware_register, memory},
    resolution,
};

mod graphics {
    pub const MAX_NUMBER_OF_SPRITES: u16 = 40;
    pub const MAX_SPRITES_PER_LINE: usize = 10;
}

/// # LCD Control Register
#[derive(Clone, Copy, Default)]
struct LcdControlRegister {
    lcd_enable: bool,                // LCD Display Enable (0=Off, 1=On)
    window_tile_map_area: bool,      // Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF
    window_enable: bool,             // Window Display Enable (0=Off, 1=On)
    bg_window_tile_data_area: bool,  // BG & Window Tile Data Select (0=8800-97FF, 1=8000-8FFF)
    bg_tile_map_area: bool,          // BG Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    obj_size: bool,                  // OBJ (Sprite) Size (0=8x8, 1=8x16)
    obj_enable: bool,                // OBJ (Sprite) Display Enable (0=Off, 1=On)
    bg_window_enable_priority: bool, // BG Display (for CGB see below) (0=Off, 1=On)
}

impl LcdControlRegister {
    pub fn get_window_tile_map_base_address(&self) -> u16 {
        if self.window_tile_map_area {
            return 0x9C00;
        }
        0x9800
    }

    pub fn get_tile_data_base_address(&self) -> u16 {
        if self.bg_window_tile_data_area {
            return 0x8000;
        }
        0x8800
    }

    pub fn get_bg_tile_map_base_address(&self) -> u16 {
        if self.bg_tile_map_area {
            return 0x9C00;
        }
        0x9800
    }

    pub fn get_sprite_high_size(&self) -> u8 {
        if self.obj_size {
            return 16;
        }
        8
    }
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
        out_value
    }
}

/// # LCD Status Register
#[derive(Clone, Copy, Default)]
struct LcdStatusRegister {
    ly_interrupt: bool,     // Bit 6 - LYC=LY Coincidence Interrupt
    mode_2_interrupt: bool, // Bit 5 - Mode 2 OAM Interrupt
    mode_1_interrupt: bool, // Bit 4 - Mode 1 V-Blank Interrupt
    mode_0_interrupt: bool, // Bit 3 - Mode 0 H-Blank Interrupt
    lyc_flag: bool,         // Bit 2 - LYC == LY
    // Bit 1-0 - Mode Flag
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

mod lcd_monochrome {
    #[derive(PartialEq)]
    pub enum PalleteMode {
        BgPallete,
        ObjPallete,
    }

    #[derive(PartialEq, Debug)]
    pub enum Color {
        White = 0xff,
        LightGray = 0xc0,
        DarkGray = 0x60,
        Black = 0x00,
    }

    pub struct PaletteRegister {
        pub data: u8,
        mode: PalleteMode,
    }

    impl PaletteRegister {
        pub fn new(mode: PalleteMode) -> Self {
            Self { data: 0, mode }
        }
        pub fn get_color(&self, color_id: u8) -> Color {
            let color_value = self.data.rotate_right(2 * color_id as u32) & 0x03;
            match color_value {
                0x00 => Color::White,
                0x01 => Color::LightGray,
                0x02 => Color::DarkGray,
                _ => Color::Black,
            }
        }
        pub fn is_transparent(&self, color: Color) -> bool {
            self.mode == PalleteMode::ObjPallete && color == Color::White
        }
    }
}

mod fsm {
    #[derive(Clone, Copy)]
    pub enum PpuState {
        OamScanMode2 = 2,
        DrawingPixelsMode3 = 3,
        HBlankMode0 = 0,
        VBlankMode1 = 1,
    }

    impl PpuState {
        pub fn new() -> Self {
            Self::OamScanMode2
        }
        pub fn next(self) -> Self {
            match self {
                Self::OamScanMode2 => Self::DrawingPixelsMode3,
                Self::DrawingPixelsMode3 => Self::HBlankMode0,
                Self::HBlankMode0 => Self::VBlankMode1,
                Self::VBlankMode1 => Self::OamScanMode2,
            }
        }
    }
}

mod sprite {
    pub struct Attribute {
        pub priority: bool, // Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
        pub yflip: bool,    // Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
        pub xflip: bool,    // Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
        pub dmg_palette: bool, // Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    }

    impl From<u8> for Attribute {
        fn from(value: u8) -> Self {
            Self {
                priority: (value.rotate_left(7) & 1) == 1,
                yflip: (value.rotate_left(6) & 1) == 1,
                xflip: (value.rotate_left(5) & 1) == 1,
                dmg_palette: (value.rotate_left(4) & 1) == 1,
            }
        }
    }

    //Object Attribute Memory (OAM)
    pub struct Sprite {
        pub attribute: Attribute, // Byte 3 — Attributes/Flags
        pub tile_index: u8,       // Byte 2 — Tile Index
        pub x_position: u8,       // Byte 1 — X Position
        pub y_position: u8,       // Byte 0 — Y Position
    }
}

/// # PPU (Picture Processing Unit)
/// On Gameboy Classic there's only one way to initialize VRAM - manually copy data with CPU instructions. This is done in bootstrap ROM process:
pub struct PictureProcessingUnit {
    //..::Memory::..
    vram: [u8; memory::VRAM_SIZE],
    voam: [u8; memory::VOAM_SIZE],
    //..::Registers::..
    lcd_control_register: LcdControlRegister,
    lcd_stat_register: LcdStatusRegister,
    scy_register: u8,
    scx_register: u8,
    ly_register: u8,
    lyc_register: u8,
    bgp_register: PaletteRegister,
    obp0_register: PaletteRegister,
    obp1_register: PaletteRegister,
    wy_register: u8,
    wx_register: u8,
    pub vblank_interrupt_req: bool,
    pub lcd_interrupt_req: bool,
    //..::Internal::..
    ppu_fsm: fsm::PpuState,
    internal_scan_line_counter: u32,
    sprite_buffer: Vec<sprite::Sprite>,
    //..::Out::..
    pub out_frame_buffer: [u8; resolution::SCREEN_W * resolution::SCREEN_W * 3],
}

impl PictureProcessingUnit {
    pub fn new() -> Self {
        PictureProcessingUnit {
            //..::Memory::..
            vram: [memory::DEFAULT_INIT_VALUE; memory::VRAM_SIZE],
            voam: [memory::DEFAULT_INIT_VALUE; memory::VOAM_SIZE],
            //..::Registers::..
            lcd_control_register: LcdControlRegister::default(),
            lcd_stat_register: LcdStatusRegister::default(),
            scy_register: 0,
            scx_register: 0,
            ly_register: 0,
            lyc_register: 0,
            bgp_register: PaletteRegister::new(PalleteMode::BgPallete),
            obp0_register: PaletteRegister::new(PalleteMode::ObjPallete),
            obp1_register: PaletteRegister::new(PalleteMode::ObjPallete),
            wy_register: 0,
            wx_register: 0,
            vblank_interrupt_req: false,
            lcd_interrupt_req: false,
            //..::Internal::..
            ppu_fsm: fsm::PpuState::new(),
            internal_scan_line_counter: 0,
            sprite_buffer: Vec::new(),
            //..::Out::..
            out_frame_buffer: [memory::DEFAULT_INIT_VALUE;
                resolution::SCREEN_W * resolution::SCREEN_W * 3],
        }
    }

    fn get_sprite_from_oam(&mut self, sprite_id: u16) -> sprite::Sprite {
        let oam_base_address = *address::OAM.start();
        // sprite occupies 4 bytes in the sprite attributes table
        let sprite_index = sprite_id * 4;
        let y_position = (self.read_byte_from_hardware_register(oam_base_address + sprite_index)
            as i8
            - 16) as u8;
        let x_position = (self.read_byte_from_hardware_register(oam_base_address + sprite_index + 1)
            as i8
            - 8) as u8;
        let tile_index = self.read_byte_from_hardware_register(oam_base_address + sprite_index + 2);

        let raw_attribute =
            self.read_byte_from_hardware_register(oam_base_address + sprite_index + 3);

        sprite::Sprite {
            attribute: Attribute::from(raw_attribute),
            tile_index,
            x_position,
            y_position,
        }
    }

    pub fn check_lyc_ly_comparison(&mut self) {
        if self.ly_register == self.lyc_register {
            self.lcd_stat_register.lyc_flag = true;
            self.lcd_stat_register.ly_interrupt = true;
        }
    }

    fn sprite_search(&mut self) {
        self.sprite_buffer.clear();

        let line = self.ly_register;
        let sprite_high_size = self.lcd_control_register.get_sprite_high_size();

        for sprite_id in 0..graphics::MAX_NUMBER_OF_SPRITES {
            if self.sprite_buffer.len() == graphics::MAX_SPRITES_PER_LINE as usize {
                break;
            }

            let sprite = self.get_sprite_from_oam(sprite_id);

            // The sprite is not on the screen at this line.
            if line < sprite.y_position || line >= sprite.y_position + sprite_high_size {
                continue;
            }

            self.sprite_buffer.push(sprite);
        }
    }
}

impl HardwareAccessible for PictureProcessingUnit {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            vram_address if address::VIDEO_RAM.contains(&vram_address) => {
                let address = (vram_address - *address::VIDEO_RAM.start()) as usize;
                self.vram[address]
            }
            voam_address if address::OAM.contains(&voam_address) => {
                let address = (voam_address - *address::OAM.start()) as usize;
                self.voam[address]
            }
            io_hardware_register::LCD_CONTROL => {
                LcdControlRegister::into(self.lcd_control_register)
            }
            io_hardware_register::LCD_STATUS => LcdStatusRegister::into(self.lcd_stat_register),
            io_hardware_register::SCY => self.scy_register,
            io_hardware_register::SCX => self.scx_register,
            io_hardware_register::LY => self.ly_register,
            io_hardware_register::LYC => self.lyc_register,
            io_hardware_register::BGP => self.bgp_register.data,
            io_hardware_register::OBP0 => self.obp0_register.data,
            io_hardware_register::OBP1 => self.obp1_register.data,
            io_hardware_register::WY => self.wy_register,
            io_hardware_register::WX => self.wx_register,
            _ => panic!("[PPU ERROR][Read] Unsupported address: [{:#06x?}]", address),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            vram_address if address::VIDEO_RAM.contains(&vram_address) => {
                let address = (vram_address - *address::VIDEO_RAM.start()) as usize;
                self.vram[address] = data
            }
            voam_address if address::OAM.contains(&voam_address) => {
                let address = (voam_address - *address::OAM.start()) as usize;
                self.voam[address] = data
            }
            io_hardware_register::LCD_CONTROL => {
                self.lcd_control_register = LcdControlRegister::from(data)
            }
            io_hardware_register::LCD_STATUS => {
                self.lcd_stat_register = LcdStatusRegister::from(data)
            }
            io_hardware_register::SCY => self.scy_register = data,
            io_hardware_register::SCX => self.scx_register = data,
            io_hardware_register::LY => self.ly_register = data,
            io_hardware_register::LYC => self.lyc_register = data,
            io_hardware_register::BGP => self.bgp_register.data = data,
            io_hardware_register::OBP0 => self.obp0_register.data = data,
            io_hardware_register::OBP1 => self.obp1_register.data = data,
            io_hardware_register::WY => self.wy_register = data,
            io_hardware_register::WX => self.wx_register = data,
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

        self.internal_scan_line_counter += cycles;

        let state = self.ppu_fsm;

        // Mode_2 (80 dots) + Mode_3 (172 dots) + Mode_0 (204 dots) = 456 dots
        match state {
            fsm::PpuState::OamScanMode2 => {
                self.lcd_stat_register.ppu_mode = fsm::PpuState::OamScanMode2 as u8;

                if self.internal_scan_line_counter >= 80 {
                    self.sprite_search();
                    self.internal_scan_line_counter -= 80;
                    self.ppu_fsm.next();
                }
            }
            fsm::PpuState::DrawingPixelsMode3 => {
                self.lcd_stat_register.ppu_mode = fsm::PpuState::DrawingPixelsMode3 as u8;

                if self.internal_scan_line_counter >= 172 {
                    self.internal_scan_line_counter -= 172;
                    self.ppu_fsm.next();
                }
            }
            fsm::PpuState::HBlankMode0 => {
                self.lcd_stat_register.ppu_mode = fsm::PpuState::HBlankMode0 as u8;

                if self.internal_scan_line_counter >= 204 {
                    self.internal_scan_line_counter -= 204;

                    self.ly_register = (self.ly_register + 1) % 154;

                    if self.ly_register >= 144 {
                        //vblank
                        self.ppu_fsm.next();
                    } else {
                        //go back to sprite search for next line
                        self.ppu_fsm = fsm::PpuState::OamScanMode2;
                    }
                }
            }
            fsm::PpuState::VBlankMode1 => {
                self.lcd_stat_register.ppu_mode = fsm::PpuState::VBlankMode1 as u8;
                //Duration 4560 dots (10 scanlines)
                if self.internal_scan_line_counter >= 4560 {
                    self.internal_scan_line_counter -= 4560;
                    self.ppu_fsm.next();
                }
            }
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

    #[test]
    fn lcd_monochrome_color_palette_test() {
        let mut palette_reg = PaletteRegister::new(PalleteMode::ObjPallete);

        //Set following colors
        // [3] White, [2] LightGray, [1] DarkGray, [0] Black
        // 00011011
        palette_reg.data = 0x1B;

        let exp_color: [Color; 4] = [
            Color::Black,
            Color::DarkGray,
            Color::LightGray,
            Color::White,
        ];

        for color_id in 0..=3 {
            assert_eq!(
                exp_color[color_id as usize],
                palette_reg.get_color(color_id)
            );
        }
        assert!(palette_reg.is_transparent(Color::White) == true);
        assert!(palette_reg.is_transparent(Color::Black) == false);
    }
}
