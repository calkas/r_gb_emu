use super::constants::gb_memory_map::{address, memory};
use crate::peripheral::{interrupt_controller::InterruptController, HardwareAccessible};
/// # I/O Memory Management
/// Input–output memory management unit
pub struct IOMMU<'a> {
    hram: [u8; memory::HIGH_RAM_SIZE],
    temp_memory: [u8; 0x10000], // Temporary solution For now all 64kB is available
    isr_controller: InterruptController,
    io_registers: Vec<Option<&'a mut dyn HardwareAccessible>>,
}

impl<'a> IOMMU<'a> {
    pub fn new() -> Self {
        IOMMU {
            hram: [memory::DEFAULT_INIT_VALUE; memory::HIGH_RAM_SIZE],
            temp_memory: [memory::DEFAULT_INIT_VALUE; 0x10000],
            isr_controller: InterruptController::new(),
            io_registers: vec![],
        }
    }
    pub fn add_to_io_register(&mut self, io_reg: &'a mut dyn HardwareAccessible) {
        self.io_registers.push(Some(io_reg));
    }

    pub fn load_value_from_io_register(&self, io_address: u16) -> u8 {
        let mut io_ret_val = memory::DEFAULT_INIT_VALUE;
        for io_device in self.io_registers.iter() {
            io_ret_val = io_device
                .as_deref()
                .unwrap()
                .read_byte_from_hardware_register(io_address);
        }
        io_ret_val
    }

    pub fn store_value_to_io_register(&mut self, io_address: u16, io_data: u8) {
        for io_device in self.io_registers.iter_mut() {
            io_device
                .as_deref_mut()
                .unwrap()
                .write_byte_to_hardware_register(io_address, io_data);
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {
                memory::DEFAULT_INIT_VALUE
            }

            io_address if address::HARDWARE_IO_REGISTERS_1.contains(&io_address) => {
                self.load_value_from_io_register(io_address)
            }

            io_address if address::HARDWARE_IO_REGISTERS_2.contains(&io_address) => {
                self.load_value_from_io_register(io_address)
            }

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr]
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .read_byte_from_hardware_register(address),

            _ => self.temp_memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {}

            io_address if address::HARDWARE_IO_REGISTERS_1.contains(&io_address) => {
                self.store_value_to_io_register(io_address, data);
            }

            io_address if address::HARDWARE_IO_REGISTERS_2.contains(&io_address) => {
                self.store_value_to_io_register(io_address, data);
            }

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr] = data;
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .write_byte_to_hardware_register(address, data),

            _ => self.temp_memory[address as usize] = data,
        }
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
    use crate::peripheral::serial::SerialDataTransfer;

    #[test]
    fn little_endianness_test() {
        let mut iommu = IOMMU::new();

        iommu.write_byte(*address::WORKING_RAM_BANK_0.start(), 0xCD);
        iommu.write_byte(*address::WORKING_RAM_BANK_0.start() + 1, 0xAB);
        let actual_value = iommu.read_word(*address::WORKING_RAM_BANK_0.start());

        assert_eq!(0xABCD, actual_value);
    }
    #[test]
    fn read_write_to_memory_map_test() {
        const EXP_STORED_VALUE: u8 = 0xCD;
        let mut iommu = IOMMU::new();

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
        let mut iommu = IOMMU::new();

        let mut serial = SerialDataTransfer::new();

        iommu.add_to_io_register(&mut serial);
        iommu.write_byte(address::SERIAL_DATA_REGISTER, 0xAA);

        assert_eq!(0xAA, iommu.read_byte(address::SERIAL_DATA_REGISTER));
    }
}
