use super::constants::gb_memory_map::{address, memory};
use crate::peripheral::{
    cartridge::Cartridge, interrupt_controller::InterruptController, serial::SerialDataTransfer,
    timer::Timer, HardwareAccessible, IoWorkingCycle,
};
use std::{cell::RefCell, rc::Rc};
/// # I/O Memory Management
/// Input–output memory management unit
pub struct IOMMU {
    cartridge_rom: Rc<RefCell<Cartridge>>,
    wram: [u8; memory::WRAM_SIZE],
    hram: [u8; memory::HIGH_RAM_SIZE],
    isr_controller: InterruptController,
    pub serial: SerialDataTransfer,
    timer: Timer,
}

impl IOMMU {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        IOMMU {
            cartridge_rom: cartridge,
            wram: [memory::DEFAULT_INIT_VALUE; memory::WRAM_SIZE],
            hram: [memory::DEFAULT_INIT_VALUE; memory::HIGH_RAM_SIZE],
            isr_controller: InterruptController::default(),
            serial: SerialDataTransfer::default(),
            timer: Timer::default(),
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            rom_bank_0_address if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .read_byte_from_hardware_register(rom_bank_0_address)
            }

            rom_bank_n_address if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .read_byte_from_hardware_register(rom_bank_n_address)
            }

            wram_address_bank_0
                if address::WORKING_RAM_BANK_0.contains(&wram_address_bank_0)
                    | address::ECHO_RAM_BANK_0.contains(&wram_address_bank_0) =>
            {
                self.wram[wram_address_bank_0 as usize & memory::WRAM_ADDRESS_MASK]
            }

            wram_address_bank_1_7
                if address::WORKING_RAM_BANK_1_7.contains(&wram_address_bank_1_7)
                    | address::ECHO_RAM_BANK_1_7.contains(&wram_address_bank_1_7) =>
            {
                self.wram[wram_address_bank_1_7 as usize & memory::WRAM_ADDRESS_MASK]
            }

            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {
                memory::DEFAULT_INIT_VALUE
            }

            0xFF44 => 0x90, //Hardcode LCD
            0xFF4C..=0xFF7F => panic!("[IMMU ERROR][READ]: CGB not implemented"),

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr]
            }

            serial_address if address::HARDWARE_IO_SERIAL.contains(&serial_address) => {
                self.serial.read_byte_from_hardware_register(serial_address)
            }

            timer_address if address::HARDWARE_IO_TIMER.contains(&timer_address) => {
                self.timer.read_byte_from_hardware_register(timer_address)
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .read_byte_from_hardware_register(address),

            _ => {
                println!("Reading from unsupported addres: {:#06x?}", address);
                memory::DEFAULT_INIT_VALUE
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            rom_bank_0_address if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .write_byte_to_hardware_register(rom_bank_0_address, data);
            }

            rom_bank_n_address if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .write_byte_to_hardware_register(rom_bank_n_address, data);
            }

            wram_address_bank_0
                if address::WORKING_RAM_BANK_0.contains(&wram_address_bank_0)
                    | address::ECHO_RAM_BANK_0.contains(&wram_address_bank_0) =>
            {
                self.wram[wram_address_bank_0 as usize & memory::WRAM_ADDRESS_MASK] = data;
            }

            wram_address_bank_1_7
                if address::WORKING_RAM_BANK_1_7.contains(&wram_address_bank_1_7)
                    | address::ECHO_RAM_BANK_1_7.contains(&wram_address_bank_1_7) =>
            {
                self.wram[wram_address_bank_1_7 as usize & memory::WRAM_ADDRESS_MASK] = data;
            }

            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {}

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr] = data;
            }

            serial_address if address::HARDWARE_IO_SERIAL.contains(&serial_address) => self
                .serial
                .write_byte_to_hardware_register(serial_address, data),

            timer_address if address::HARDWARE_IO_TIMER.contains(&timer_address) => {
                self.timer
                    .write_byte_to_hardware_register(timer_address, data);
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .write_byte_to_hardware_register(address, data),

            _ => {
                println!(
                    "Writing to unsupported addres: {:#06x?} data = {:#06x?}",
                    address, data
                );
            }
        }
    }

    pub fn process(&mut self, cycles: u32) {
        //  * 0 V-Blank

        //  * 1 LCD

        //  * 2 Timer
        self.timer.next(cycles);
        self.isr_controller.intf.timer = self.timer.is_interrupt();
        self.timer.reset_interrupt();

        //  * 3 Serial Link
        self.serial.next(cycles);
        self.isr_controller.intf.serial_link = self.serial.is_interrupt();
        self.serial.reset_interrupt();

        //  * 4 Joypad
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low_byte_val = self.read_byte(address) as u16;
        let high_byte_val = self.read_byte(address + 1) as u16;
        high_byte_val.rotate_left(8) | low_byte_val
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low_byte_val = (value & 0x00FF) as u8;
        let high_byte_val = (value & 0xFF00).rotate_right(8) as u8;
        self.write_byte(address, low_byte_val);
        self.write_byte(address + 1, high_byte_val);
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn little_endianness_test() {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let mut iommu = IOMMU::new(cartridge.clone());

        iommu.write_byte(*address::HIGH_RAM.start(), 0xCD);
        iommu.write_byte(*address::HIGH_RAM.start() + 1, 0xAB);
        let actual_value = iommu.read_word(*address::HIGH_RAM.start());

        assert_eq!(0xABCD, actual_value);
    }
    #[test]
    fn read_write_to_memory_map_test() {
        const EXP_STORED_VALUE: u8 = 0xCD;
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let mut iommu = IOMMU::new(cartridge.clone());

        // [0xFEA0 - 0xFEFF] Not Usable
        iommu.write_byte(*address::NOT_USABLE.start(), EXP_STORED_VALUE);
        assert_eq!(
            memory::DEFAULT_INIT_VALUE,
            iommu.read_byte(*address::NOT_USABLE.start())
        );

        // [0xFF80 - 0xFFFE] High RAM (HRAM)
        iommu.write_byte(*address::HIGH_RAM.start() + 1, EXP_STORED_VALUE);
        assert_eq!(
            EXP_STORED_VALUE,
            iommu.read_byte(*address::HIGH_RAM.start() + 1)
        );
    }

    #[test]
    fn read_write_to_io_register_test() {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let mut iommu = IOMMU::new(cartridge.clone());
        iommu.write_byte(address::SERIAL_DATA_REGISTER, 0xAA);

        assert_eq!(0xAA, iommu.read_byte(address::SERIAL_DATA_REGISTER));
    }
}
