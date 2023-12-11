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
    current_rom_bank: usize,
    number_of_rom_banks: u16,
    is_ram_enable: bool,
    ram_size: usize,
    current_ram_bank: usize,
    number_of_ram_banks: u16,
}

impl CartridgeController {
    pub fn new() -> Self {
        Self {
            bank_mode: CartridgeType::RomOnly,
            rom_size: 0,
            current_rom_bank: 1, // As ROM Bank 0 is fixed into memory address 0x0-0x3FFF. The declaration to specify which ROM bank is currently loaded into internal memory address 0x4000-0x7FFF
            number_of_rom_banks: 0,
            is_ram_enable: false,
            ram_size: 0,
            current_ram_bank: 0,
            number_of_ram_banks: 0,
        }
    }

    pub fn determine_cartridge_type(&mut self, code: u8) {
        self.bank_mode = match code {
            0 => CartridgeType::RomOnly,
            1..=3 => CartridgeType::Mbc1,
            5..=6 => CartridgeType::Mbc2,
            _ => panic!("[CARTRIDGE ERROR] Unsupported Bank mode: [0x{:02x}]", code),
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
            _ => panic!("[CARTRIDGE ERROR] Unsupported Rom bank: [0x{:02x}]", code),
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
            _ => panic!("[CARTRIDGE ERROR] Unsupported Ram bank: [0x{:02x}]", code),
        };

        let bank_size: usize = 0x2000; // 8 KiB
        self.ram_size = bank_size * self.number_of_ram_banks as usize;
    }

    pub fn print_status_data(&self) {
        let catridge_type = match self.bank_mode {
            CartridgeType::RomOnly => "Rom only",
            CartridgeType::Mbc1 => "MBC1",
            CartridgeType::Mbc2 => "MBC2",
        };
        println!("Cartridge Type: {}", catridge_type);
        println!(
            "ROM Size: {}, Banks: {}",
            self.rom_size, self.number_of_rom_banks
        );
        println!(
            "RAM Size: {}, Banks: {}",
            self.ram_size, self.number_of_ram_banks
        );
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
            panic!("[CARTRIDGE ERROR] Couldn't read the Rom file.");
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

        self.controller.print_status_data();
    }
}

impl HardwareAccessible for Cartridge {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            rom_bank_0 if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0) => {
                self.rom[rom_bank_0 as usize]
            }

            rom_bank_n if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n) => {
                let mut new_rom_address =
                    (rom_bank_n - *address::CARTRIDGE_ROM_BANK_1_N.start()) as usize;
                new_rom_address = new_rom_address + (self.controller.current_rom_bank * 0x4000);
                self.rom[new_rom_address]
            }

            ram_bank if address::CARTRIDGE_RAM.contains(&ram_bank) => {
                if self.controller.is_ram_enable {
                    let mut new_ram_address = (ram_bank - *address::CARTRIDGE_RAM.start()) as usize;
                    new_ram_address = new_ram_address + (self.controller.current_ram_bank * 0x2000);
                    self.ram[new_ram_address]
                } else {
                    memory::DEFAULT_INIT_VALUE
                }
            }

            _ => panic!(
                "[CARTRIDGE ERROR][Read] Unsupported address: [0x{:02x}]",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        if self.controller.bank_mode == CartridgeType::RomOnly {
            //Nothing to write
            return;
        }

        match address {
            _ => panic!(
                "[CARTRIDGE ERROR][Write] Unsupported address: [0x{:02x}]",
                address
            ),
        }
    }
}
