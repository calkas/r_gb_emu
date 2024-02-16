use self::fsm::PpuState;
use self::lcd_monochrome::{Color, PaletteRegister, PalleteMode, Pixel2bpp};
use self::sprite::{Attribute, Sprite};
use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::{address, address::io_hardware_register, memory};
use crate::emulator_constants::resolution;

mod graphics {
    pub const MAX_NUMBER_OF_SPRITES: u16 = 40;
    pub const MAX_SPRITES_PER_LINE: usize = 10;
}

/// # LCD Control Register
#[derive(Clone, Copy, Default)]
struct LcdControlRegister {
    lcd_enable: bool,               // LCD Display Enable (0=Off, 1=On)
    window_tile_map_area: bool,     // Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF
    window_enable: bool,            // Window Display Enable (0=Off, 1=On)
    bg_window_tile_data_area: bool, // BG & Window Tile Data Select (0=8800-97FF, 1=8000-8FFF)
    bg_tile_map_area: bool,         // BG Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    obj_size: bool,                 // OBJ (Sprite) Size (0=8x8, 1=8x16)
    obj_enable: bool,               // OBJ (Sprite) Display Enable (0=Off, 1=On)
    bg_and_window_enable: bool,     // BG Display (for CGB see below) (0=Off, 1=On)
}

impl LcdControlRegister {
    //----------------- MAPS -----------------
    pub fn get_window_tile_map_base_address(&self) -> u16 {
        if self.window_tile_map_area {
            return 0x9C00;
        }
        0x9800
    }

    pub fn get_bg_tile_map_base_address(&self) -> u16 {
        if self.bg_tile_map_area {
            return 0x9C00;
        }
        0x9800
    }

    //-------------- TILE DATA ---------------

    pub fn get_tile_data_base_address(&self) -> u16 {
        if self.bg_window_tile_data_area {
            return 0x8000;
        }
        0x8800
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
            lcd_enable: (value.rotate_right(7) & 1) == 1,
            window_tile_map_area: (value.rotate_right(6) & 1) == 1,
            window_enable: (value.rotate_right(5) & 1) == 1,
            bg_window_tile_data_area: (value.rotate_right(4) & 1) == 1,
            bg_tile_map_area: (value.rotate_right(3) & 1) == 1,
            obj_size: (value.rotate_right(2) & 1) == 1,
            obj_enable: (value.rotate_right(1) & 1) == 1,
            bg_and_window_enable: (value.rotate_right(0) & 1) == 1,
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
        if register.bg_and_window_enable {
            out_value |= 1_u8.rotate_left(0);
        }
        out_value
    }
}

/// # LCD Status Register
/// When the LCD status changes its mode to either Mode 0, 1 or 2
/// then this can cause an LCD Interupt Request to happen.
/// Bits 3, 4 and 5 of the LCD Status register (0xFF41) are interupt enabled flags
/// (the same as the Interupt Enabled Register 0xFFFF).
/// These bits are set by the game not the emulator !!!
#[derive(Clone, Copy, Default)]
struct LcdStatusRegister {
    enable_ly_interrupt: bool,     // Bit 6 - LYC=LY Coincidence Interrupt
    enable_mode_2_interrupt: bool, // Bit 5 - Mode 2 OAM Interrupt
    enable_mode_1_interrupt: bool, // Bit 4 - Mode 1 V-Blank Interrupt
    enable_mode_0_interrupt: bool, // Bit 3 - Mode 0 H-Blank Interrupt
    lyc_flag: bool,                // Bit 2 - LYC == LY
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
            enable_ly_interrupt: (value.rotate_right(6) & 1) == 1,
            enable_mode_2_interrupt: (value.rotate_right(5) & 1) == 1,
            enable_mode_1_interrupt: (value.rotate_right(4) & 1) == 1,
            enable_mode_0_interrupt: (value.rotate_right(3) & 1) == 1,
            lyc_flag: (value.rotate_right(2) & 1) == 1,
            ppu_mode: value & 0x03,
        }
    }
}

impl std::convert::From<LcdStatusRegister> for u8 {
    fn from(register: LcdStatusRegister) -> Self {
        let mut out_value: u8 = 0;
        if register.enable_ly_interrupt {
            out_value |= 1_u8.rotate_left(6);
        }
        if register.enable_mode_2_interrupt {
            out_value |= 1_u8.rotate_left(5);
        }
        if register.enable_mode_1_interrupt {
            out_value |= 1_u8.rotate_left(4);
        }
        if register.enable_mode_0_interrupt {
            out_value |= 1_u8.rotate_left(3);
        }
        if register.lyc_flag {
            out_value |= 1_u8.rotate_left(2);
        }
        out_value |= register.ppu_mode;
        out_value
    }
}

mod lcd_monochrome {
    #[derive(Clone, Copy, PartialEq)]
    pub enum PalleteMode {
        BgPallete,
        ObjPallete,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Color {
        White = 0xff,
        LightGray = 0xc0,
        DarkGray = 0x60,
        Black = 0x00,
    }

    impl Color {
        pub fn rgb(self) -> (u8, u8, u8) {
            (self as u8, self as u8, self as u8)
        }
    }

    #[derive(Clone, Copy)]
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

    #[derive(Clone, Copy)]
    pub struct Pixel2bpp {
        pub low_byte: u8,
        pub high_byte: u8,
        pub pixel_bit_activation: u8,
    }

    impl Pixel2bpp {
        pub fn get_color_id(&mut self) -> u8 {
            let color_bit_low = self
                .low_byte
                .rotate_right(7 - self.pixel_bit_activation as u32)
                & 0x1;
            let color_bit_high = self
                .high_byte
                .rotate_right(7 - self.pixel_bit_activation as u32)
                & 0x1;
            (color_bit_high.rotate_left(1)) | color_bit_low
        }
    }
}

mod fsm {
    #[derive(Clone, Copy, PartialEq, Debug)]
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
        pub priority: bool, // Bit7 OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
        pub yflip: bool,    // Bit6 Y flip          (0=Normal, 1=Vertically mirrored)
        pub xflip: bool,    // Bit5 X flip          (0=Normal, 1=Horizontally mirrored)
        pub dmg_palette: bool, // Bit4 Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
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
    ppu_fsm: PpuState,
    internal_scan_line_counter: u32,
    internal_window_line_counter: u8,
    sprite_buffer: Vec<Sprite>,
    //..::Out::..
    pub out_frame_buffer: [[[u8; 3]; resolution::SCREEN_W]; resolution::SCREEN_H],
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
            ppu_fsm: PpuState::new(),
            internal_scan_line_counter: 0,
            internal_window_line_counter: 0,
            sprite_buffer: Vec::new(),
            //..::Out::..
            out_frame_buffer: [[[0xFF; 3]; resolution::SCREEN_W]; resolution::SCREEN_H],
        }
    }

    fn get_tile_data_address(&self, tile_number: u8) -> u16 {
        let base_title_address = self.lcd_control_register.get_tile_data_base_address();

        let title_offset = if base_title_address == 0x8000 {
            i16::from(tile_number)
        } else {
            i16::from(tile_number as i8) + 128
        } as u16
            * 16;

        base_title_address + title_offset
    }

    fn get_sprite_from_oam(&mut self, sprite_id: u16) -> Sprite {
        let oam_base_address = *address::OAM.start();
        // sprite occupies 4 bytes in the sprite attributes table
        let sprite_index = sprite_id * 4;
        let y = self.read_byte_from_hardware_register(oam_base_address + sprite_index) as i16 - 16;
        let x =
            self.read_byte_from_hardware_register(oam_base_address + sprite_index + 1) as i16 - 8;
        let tile_index = self.read_byte_from_hardware_register(oam_base_address + sprite_index + 2)
            & if self.lcd_control_register.obj_size {
                0xFE
            } else {
                0xFF
            };

        let raw_attribute =
            self.read_byte_from_hardware_register(oam_base_address + sprite_index + 3);

        Sprite {
            attribute: Attribute::from(raw_attribute),
            tile_index,
            x_position: x as u8,
            y_position: y as u8,
        }
    }

    fn sprite_search(&mut self) {
        self.sprite_buffer.clear();

        let line = self.ly_register;
        let sprite_high_size = self.lcd_control_register.get_sprite_high_size();

        for sprite_id in 0..graphics::MAX_NUMBER_OF_SPRITES {
            if self.sprite_buffer.len() == graphics::MAX_SPRITES_PER_LINE {
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

    fn get_tile_pixel(&mut self, cursor_x: u8, cursor_y: u8, tile_map_address: u16) -> Pixel2bpp {
        // 32x32 grid of 8x8 pixel tiles
        let tile_grid_map_row_num = cursor_y / 8;
        let tile_grid_map_col_num = cursor_x / 8;
        let tile_coordinates = tile_grid_map_row_num as u16 * 32 + tile_grid_map_col_num as u16;

        let tile_number =
            self.read_byte_from_hardware_register(tile_map_address + tile_coordinates);

        let tile_data_address = self.get_tile_data_address(tile_number);

        // The Gameboy displays its graphics using 8x8-pixel tiles.
        // As the name 2BPP implies, it takes exactly two bits to store the information about a single pixel.
        // There are 64 total pixels in a single tile (8x8 pixels).
        // Therefore, exactly 128 bits, or 16 bytes, are required to fully represent a single tile
        // Eg.
        // [Row0][Row1][Row2][Row3][Row4][Row5][Row6][Row7]
        //  7C 7C 00 C6 C6 00 00 FE C6 C6 00 C6 C6 00 00 00 <-One title

        //Pixel coordinates in the local 8x8 tile.
        let pixel_row_num = cursor_y % 8;
        let pixel_col_num = cursor_x % 8;

        // multiply by 2 because every row of 8 pixels is 2 bytes of data.
        let tile_pixel_row_index = tile_data_address + (pixel_row_num as u16 * 2);

        let low_byte = self.read_byte_from_hardware_register(tile_pixel_row_index);
        let high_byte = self.read_byte_from_hardware_register(tile_pixel_row_index + 1);

        Pixel2bpp {
            low_byte,
            high_byte,
            pixel_bit_activation: pixel_col_num,
        }
    }

    fn draw_background_scanline(&mut self) {
        let tile_map_address = self.lcd_control_register.get_bg_tile_map_base_address();
        let bg_cursor_y: u8 = self.ly_register.wrapping_add(self.scy_register);
        //  -------> [x]
        // |
        // |   [row, col]
        // |
        // V [y]

        for screen_col in 0..resolution::SCREEN_W as u8 {
            let bg_cursor_x = screen_col.wrapping_add(self.scx_register);
            let mut pixel = self.get_tile_pixel(bg_cursor_x, bg_cursor_y, tile_map_address);
            let color_id = pixel.get_color_id();

            let color = self.bgp_register.get_color(color_id);
            self.out_frame_buffer[self.ly_register as usize][screen_col as usize] =
                [color.rgb().0, color.rgb().1, color.rgb().2];

            // self.out_frame_buffer[self.ly_register as usize]
            //     [self.scx_register as usize + screen_col as usize] =
            //     [color.rgb().0, color.rgb().1, color.rgb().2];
        }
    }

    fn draw_window_scanline(&mut self) {
        if !self.lcd_control_register.window_enable || self.ly_register != self.wy_register {
            return;
        }
        let tile_map_address = self.lcd_control_register.get_window_tile_map_base_address();

        // The window keeps an internal line counter that’s functionally similar to LY,
        // and increments alongside it. However, it only gets incremented when the window is visible.
        let win_cursor_y = self.internal_window_line_counter;

        for screen_col in 0..resolution::SCREEN_W as u8 {
            let win_cursor_x: i16 = 0 - (self.wx_register as i16 - 7) + screen_col as i16;
            if win_cursor_x < 0 {
                continue;
            }

            let mut pixel = self.get_tile_pixel(win_cursor_x as u8, win_cursor_y, tile_map_address);
            let color_id = pixel.get_color_id();

            let color = self.bgp_register.get_color(color_id);
            //Todo screen_col or win_cursor_x
            self.out_frame_buffer[self.ly_register as usize][screen_col as usize] =
                [color.rgb().0, color.rgb().1, color.rgb().2];
        }

        self.internal_window_line_counter += 1;
    }

    fn draw_sprite_scanline(&mut self) {
        let sprite_high = self.lcd_control_register.get_sprite_high_size();
        let line = self.ly_register;

        for sprite in self.sprite_buffer.iter() {
            let sprite_y = if sprite.attribute.yflip {
                (sprite_high as i16 - 1 - (line as i16 - sprite.y_position as i16)) as u16
            } else {
                (line as i16 - sprite.y_position as i16) as u16
            };

            let sprite_data_address =
                *address::VIDEO_RAM.start() + (sprite.tile_index as u16 * 16) + (sprite_y * 2);

            let low_byte = self.read_byte_from_hardware_register(sprite_data_address);
            let high_byte = self.read_byte_from_hardware_register(sprite_data_address + 1);

            let pallete = if sprite.attribute.dmg_palette {
                self.obp0_register
            } else {
                self.obp1_register
            };

            // Walk through each pixel to be drawn.
            for pixel_col in 0..8_u8 {
                if sprite.x_position as u16 + pixel_col as u16 >= resolution::SCREEN_W as u16 {
                    continue;
                }

                // Check  BG vs. OBJ priority
                if !self.lcd_control_register.bg_and_window_enable
                    && sprite.attribute.priority
                    && self.out_frame_buffer[line as usize]
                        [(sprite.x_position + pixel_col) as usize]
                        == [0, 0, 0]
                {
                    continue;
                }

                // Number of pixel (0-7) of this row of the sprite. Might be horizontally flipped.
                let pixel_num = if sprite.attribute.xflip {
                    7 - pixel_col
                } else {
                    pixel_col
                };

                let mut pixel = Pixel2bpp {
                    low_byte,
                    high_byte,
                    pixel_bit_activation: pixel_num,
                };

                let color_id = pixel.get_color_id();
                let color = pallete.get_color(color_id);

                if pallete.is_transparent(color) {
                    continue;
                }

                self.out_frame_buffer[line as usize][(sprite.x_position + pixel_col) as usize] =
                    [color.rgb().0, color.rgb().1, color.rgb().2];
            }
        }
    }

    fn enter_to_new_mode(&mut self, ppu_state: u8, interrupt_needed: bool) {
        if self.lcd_stat_register.ppu_mode != ppu_state {
            self.lcd_stat_register.ppu_mode = ppu_state;

            if ppu_state == PpuState::VBlankMode1 as u8 {
                self.vblank_interrupt_req = true;
            }
            self.lcd_interrupt_req = interrupt_needed;
        }
    }

    fn check_conincidence_flag(&mut self) {
        if self.ly_register == self.lyc_register {
            self.lcd_stat_register.lyc_flag = true;
            if self.lcd_stat_register.enable_ly_interrupt {
                self.lcd_interrupt_req = true;
            }
        } else {
            self.lcd_stat_register.lyc_flag = false;
        }
    }

    fn draw_scanline(&mut self) {
        if self.lcd_control_register.bg_and_window_enable {
            self.draw_background_scanline();
            self.draw_window_scanline();
        }

        if self.lcd_control_register.obj_enable {
            self.draw_sprite_scanline();
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

        // ..:: One scanline ::..
        // Mode_2 (80 dots) + Mode_3 (172 dots) + Mode_0 (204 dots) = 456 dots
        // ..:: All scanlines done "One frame" ::..
        // Mode_2 (80 dots) + Mode_3 (172 dots) + Mode_0 (204 dots) + Mode_1 (10 *456) = 70224 dosts
        match state {
            PpuState::OamScanMode2 => {
                self.enter_to_new_mode(
                    PpuState::OamScanMode2 as u8,
                    self.lcd_stat_register.enable_mode_2_interrupt,
                );
                if self.internal_scan_line_counter >= 80 {
                    self.sprite_search();
                    self.internal_scan_line_counter -= 80;
                    self.ppu_fsm = self.ppu_fsm.next();
                }
            }
            PpuState::DrawingPixelsMode3 => {
                self.enter_to_new_mode(PpuState::DrawingPixelsMode3 as u8, false);

                if self.internal_scan_line_counter >= 172 {
                    self.internal_scan_line_counter -= 172;
                    self.ppu_fsm = self.ppu_fsm.next();
                }
            }
            PpuState::HBlankMode0 => {
                self.enter_to_new_mode(
                    PpuState::HBlankMode0 as u8,
                    self.lcd_stat_register.enable_mode_0_interrupt,
                );

                if self.internal_scan_line_counter >= 204 {
                    self.internal_scan_line_counter -= 204;

                    self.check_conincidence_flag();

                    if self.ly_register < 144 {
                        self.draw_scanline();
                        self.ly_register += 1;
                    }

                    if self.ly_register >= 144 {
                        //go to vblank
                        self.ppu_fsm = self.ppu_fsm.next();
                    } else {
                        //go back to sprite search for next line
                        self.ppu_fsm = PpuState::OamScanMode2;
                    }
                }
            }
            PpuState::VBlankMode1 => {
                self.enter_to_new_mode(
                    PpuState::VBlankMode1 as u8,
                    self.lcd_stat_register.enable_mode_1_interrupt,
                );

                // Reset window internal state counter.
                self.internal_window_line_counter = 0;

                //Duration 4560 dots (10 scanlines)
                if self.internal_scan_line_counter >= 456 {
                    self.internal_scan_line_counter -= 456;
                    self.ly_register += 1;
                    if self.ly_register == 154 {
                        self.ly_register = 0;
                        self.ppu_fsm = self.ppu_fsm.next();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod uint_test {
    use super::*;

    #[test]
    fn ppu_mode_one_scanline_done_hblank_test() {
        let mut ppu = PictureProcessingUnit::new();

        ppu.lcd_control_register.lcd_enable = true;

        //Case: Test one scanline Hblank 456 dots
        //[Mode 2]->[Mode 3]->[Mode 0]->[Mode 2]

        //Switch to Mode 2
        ppu.next_to(0);
        assert_eq!(PpuState::OamScanMode2, ppu.ppu_fsm);
        assert_eq!(2, ppu.lcd_stat_register.ppu_mode);
        ppu.next_to(80);

        //Switch to Mode 3
        ppu.next_to(0);
        assert_eq!(PpuState::DrawingPixelsMode3, ppu.ppu_fsm);
        assert_eq!(3, ppu.lcd_stat_register.ppu_mode);
        ppu.next_to(172);

        //Switch to Mode 0
        ppu.next_to(0);
        assert_eq!(PpuState::HBlankMode0, ppu.ppu_fsm);
        assert_eq!(ppu.lcd_stat_register.ppu_mode, 0);

        ppu.lyc_register = 0;
        ppu.lcd_stat_register.enable_ly_interrupt = true;
        ppu.next_to(204);
        assert_eq!(1, ppu.ly_register);
        assert!(ppu.lcd_interrupt_req == true);
        assert!(ppu.lcd_stat_register.lyc_flag == true);

        //Switch to Mode 2
        ppu.next_to(0);
        assert_eq!(PpuState::OamScanMode2, ppu.ppu_fsm);
        assert_eq!(2, ppu.lcd_stat_register.ppu_mode);
    }

    #[test]

    fn ppu_mode_all_scanlines_done_vblank_test() {
        let mut ppu = PictureProcessingUnit::new();

        ppu.lcd_control_register.lcd_enable = true;
        ppu.ly_register = 143;

        //Case: Test all scanlines done vblank 70224 dots "one frame"
        //[Mode 2]->[Mode 3]->[Mode 0]->[Mode 1]->[Mode 2]

        //Switch to Mode 2
        ppu.next_to(0);
        assert_eq!(PpuState::OamScanMode2, ppu.ppu_fsm);
        assert_eq!(2, ppu.lcd_stat_register.ppu_mode);
        ppu.next_to(80);

        //Switch to Mode 3
        ppu.next_to(0);
        assert_eq!(PpuState::DrawingPixelsMode3, ppu.ppu_fsm);
        assert_eq!(3, ppu.lcd_stat_register.ppu_mode);
        ppu.next_to(172);

        //Switch to Mode 0
        ppu.next_to(0);
        assert_eq!(PpuState::HBlankMode0, ppu.ppu_fsm);
        assert_eq!(ppu.lcd_stat_register.ppu_mode, 0);
        ppu.next_to(204);

        //Switch to Mode 1
        ppu.next_to(0);
        assert_eq!(PpuState::VBlankMode1, ppu.ppu_fsm);
        assert_eq!(ppu.lcd_stat_register.ppu_mode, 1);
        assert!(ppu.vblank_interrupt_req == true);

        //Wait 10 scanlines
        for _ in 0..10 {
            ppu.next_to(456);
        }

        assert_eq!(0, ppu.ly_register);

        //Switch to Mode 2
        ppu.next_to(0);
        assert_eq!(PpuState::OamScanMode2, ppu.ppu_fsm);
        assert_eq!(2, ppu.lcd_stat_register.ppu_mode);
    }
    #[test]
    fn lcd_control_register_convert_test() {
        let mut register = LcdControlRegister::from(0x91);
        assert!(register.lcd_enable == true);
        assert!(register.window_tile_map_area == false);
        assert!(register.window_enable == false);
        assert!(register.bg_window_tile_data_area == true);
        assert!(register.bg_tile_map_area == false);
        assert!(register.obj_size == false);
        assert!(register.obj_enable == false);
        assert!(register.bg_and_window_enable == true);

        register.obj_size = true;
        register.obj_enable = true;
        assert_eq!(151 as u8, LcdControlRegister::into(register));
    }

    #[test]
    fn lcd_stat_register_convert_test() {
        let mut register = LcdStatusRegister::from(0x2B);

        assert!(register.enable_ly_interrupt == false);
        assert!(register.enable_mode_2_interrupt == true);
        assert!(register.enable_mode_1_interrupt == false);
        assert!(register.enable_mode_0_interrupt == true);
        assert!(register.lyc_flag == false);
        assert!(register.ppu_mode == 3);

        register.enable_ly_interrupt = true;
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
