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
#[derive(PartialEq)]
enum BankMode {
    Rom,
    Ram,
}

struct CartridgeController {
    cart_type: CartridgeType,
    bank_mode: BankMode,
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
            cart_type: CartridgeType::RomOnly,
            bank_mode: BankMode::Rom,
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
        self.cart_type = match code {
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
        let catridge_type = match self.cart_type {
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

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            rom: Default::default(),
            ram: Default::default(),
            controller: CartridgeController::new(),
        }
    }
}

impl Cartridge {
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

        if self.controller.cart_type != CartridgeType::RomOnly {
            self.ram.fill(memory::DEFAULT_INIT_VALUE);
            self.ram.reserve(self.controller.ram_size);
        }

        self.controller.print_status_data();
    }

    fn ram_bank_enable_request(&mut self, address: u16, data: u8) {
        if self.controller.cart_type == CartridgeType::Mbc2 && address & 0x100 != 0 {
            return;
        }
        self.controller.is_ram_enable = data & 0x0F == 0x0A;
    }

    fn mbc2_rom_bank_change(&mut self, data: u8) {
        let mut reg_data = data & 0x0F;
        if reg_data == 0 {
            reg_data = 0x01;
        }
        self.controller.current_rom_bank = reg_data as usize;
    }

    fn mbc1_rom_bank_change_step_1(&mut self, data: u8) {
        let mut reg_data_lo_5bits = data & 0x1F;
        if reg_data_lo_5bits == 0 {
            reg_data_lo_5bits = 0x01;
        }
        self.controller.current_rom_bank &= 0xE0; // Clear 0-5 bits
        self.controller.current_rom_bank |= reg_data_lo_5bits as usize;
    }

    fn mbc1_rom_bank_change_step_2(&mut self, data: u8) {
        let mut reg_data_hi_3bits = data & 0xE0;
        if reg_data_hi_3bits == 0 {
            reg_data_hi_3bits = 0x01;
        }
        self.controller.current_rom_bank &= 0x1F; // Clear 6-8 bits
        self.controller.current_rom_bank |= reg_data_hi_3bits as usize;
    }

    fn mbc1_ram_bank_change(&mut self, data: u8) {
        //You cannot change RAM Banks in MBC2 as that has external ram on the cartridge.
        self.controller.current_ram_bank = (data & 0x03) as usize;
    }

    fn mbc1_rom_ram_mode_change(&mut self, data: u8) {
        let reg_mod_data = data & 1;
        self.controller.bank_mode = if reg_mod_data == 0 {
            BankMode::Rom
        } else {
            BankMode::Ram
        };

        if self.controller.bank_mode == BankMode::Rom {
            self.controller.current_ram_bank = 0;
        }
    }

    fn bank_handling(&mut self, address: u16, data: u8) {
        match address {
            // do RAM enabling
            0x0000..=0x1FFF => {
                self.ram_bank_enable_request(address, data);
            }

            // do ROM bank change
            0x2000..=0x3FFF => {
                match self.controller.cart_type {
                    CartridgeType::RomOnly => (),
                    CartridgeType::Mbc1 => self.mbc1_rom_bank_change_step_1(data),
                    CartridgeType::Mbc2 => self.mbc2_rom_bank_change(data),
                };
            }

            // do ROM or RAM bank change
            0x4000..=0x5FFF => {
                match self.controller.bank_mode {
                    BankMode::Rom => self.mbc1_rom_bank_change_step_2(data),
                    BankMode::Ram => self.mbc1_ram_bank_change(data),
                };
            }

            // do ROM/RAM Mode change
            0x6000..=0x7FFF => {
                self.mbc1_rom_ram_mode_change(data);
            }
            _ => (),
        }
    }

    fn write_to_ram(&mut self, address: u16, data: u8) {
        if !self.controller.is_ram_enable {
            return;
        }

        let mut new_ram_address = (address - *address::CARTRIDGE_RAM.start()) as usize;
        new_ram_address += self.controller.current_ram_bank * 0x2000;
        self.ram[new_ram_address] = data;
    }
}

impl HardwareAccessible for Cartridge {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            rom_bank_0_adr if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0_adr) => {
                self.rom[rom_bank_0_adr as usize]
            }

            rom_bank_n_adr if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n_adr) => {
                let mut new_rom_address =
                    (rom_bank_n_adr - *address::CARTRIDGE_ROM_BANK_1_N.start()) as usize;
                new_rom_address += self.controller.current_rom_bank * 0x4000;
                self.rom[new_rom_address]
            }

            ram_bank_adr if address::CARTRIDGE_RAM.contains(&ram_bank_adr) => {
                if self.controller.is_ram_enable {
                    let mut new_ram_address =
                        (ram_bank_adr - *address::CARTRIDGE_RAM.start()) as usize;
                    new_ram_address += self.controller.current_ram_bank * 0x2000;
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
        if self.controller.cart_type == CartridgeType::RomOnly {
            //Nothing to write
            return;
        }
        match address {
            rom_address
                if address::CARTRIDGE_ROM_BANK_0.contains(&rom_address)
                    | address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_address) =>
            {
                self.bank_handling(rom_address, data);
            }

            ram_address if address::CARTRIDGE_RAM.contains(&ram_address) => {
                self.write_to_ram(ram_address, data);
            }

            _ => panic!(
                "[CARTRIDGE ERROR][Write] Unsupported address: [0x{:02x}]",
                address
            ),
        }
    }
}
