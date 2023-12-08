use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::HardwareAccessible;

use crate::constants::gb_memory_map::address;
use crate::constants::gb_memory_map::memory;

#[derive(PartialEq)]
enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc2,
}

struct CartridgeController {
    bank_mode: CartridgeType,
    rom_size: usize,
    number_of_rom_banks: u16,
    ram_size: usize,
    number_of_ram_banks: u16,
}

impl CartridgeController {
    pub fn new() -> Self {
        Self {
            bank_mode: CartridgeType::RomOnly,
            rom_size: 0,
            number_of_rom_banks: 0,
            ram_size: 0,
            number_of_ram_banks: 0,
        }
    }

    pub fn determine_cartridge_type(&mut self, code: u8) {
        self.bank_mode = match code {
            0 => CartridgeType::RomOnly,
            1..=3 => CartridgeType::Mbc1,
            5..=6 => CartridgeType::Mbc2,
            _ => panic!("[CARTRIDGE ERROR] Unsupported bank mode: [0x{:02x}]", code),
        }
    }
    pub fn calculate_rom_size(&mut self, code: u8) {
        self.number_of_rom_banks = match code {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x07 => 256,
            0x08 => 512,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => panic!("[CARTRIDGE ERROR] Unsupported rom bank: [0x{:02x}]", code),
        };
        let bank_size: usize = 0x4000; // 16 KiB
        self.rom_size = bank_size * self.number_of_rom_banks as usize;
    }

    pub fn calculate_ram_size(&mut self, code: u8) {
        self.number_of_ram_banks = match code {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => panic!("[CARTRIDGE ERROR] Unsupported ram bank: [0x{:02x}]", code),
        };

        let bank_size: usize = 0x2000; // 8 KiB
        self.ram_size = bank_size * self.number_of_ram_banks as usize;
    }
}

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    controller: CartridgeController,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: vec![],
            ram: vec![],
            controller: CartridgeController::new(),
        }
    }

    pub fn load(&mut self, cartridge_path: &str) {
        let path = Path::new(cartridge_path);

        let mut cartridge_file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!(
                "[CARTRIDGE ERROR] Couldn't open {}: {}",
                path.display(),
                why
            ),
        };

        if cartridge_file.read_to_end(&mut self.rom).is_err() {
            panic!("[CARTRIDGE ERROR] Couldn't read the rom file.");
        }

        self.controller
            .determine_cartridge_type(self.rom[address::cartridge_header::CARTRIDGE_TYPE as usize]);

        self.controller
            .calculate_rom_size(self.rom[address::cartridge_header::ROM_SIZE as usize]);

        self.controller
            .calculate_ram_size(self.rom[address::cartridge_header::RAM_SIZE as usize]);

        if self.controller.bank_mode != CartridgeType::RomOnly {
            self.ram.fill(memory::DEFAULT_INIT_VALUE);
            self.ram.reserve(self.controller.ram_size);
        }
    }
}

impl HardwareAccessible for Cartridge {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        if self.rom.is_empty() {
            panic!("No cartridge has been loaded");
        }

        match address {
            address::cartridge_header::CARTRIDGE_TYPE => {
                self.rom[address::cartridge_header::CARTRIDGE_TYPE as usize]
            }

            address::cartridge_header::ROM_SIZE => {
                self.rom[address::cartridge_header::ROM_SIZE as usize]
            }

            address::cartridge_header::RAM_SIZE => {
                self.rom[address::cartridge_header::RAM_SIZE as usize]
            }
            _ => panic!(
                "Read - This address [{:#02x?}] is not for Cartridge header",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        todo!()
    }
}
