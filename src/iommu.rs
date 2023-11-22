use super::constants::gb_memory_map::address;
use super::constants::gb_memory_map::memory;
use crate::peripheral::{interrupt_controller::InterruptController, HardwareAccessible};
/// # I/O Memory Management
/// Input–output memory management unit
pub struct IOMMU {
    stack: [u8; memory::HIGH_RAM_SIZE],
    temp_memory: [u8; 0x10000], // Temporary solution For now all 64kB is available
    isr_controller: InterruptController,
}

impl IOMMU {
    pub fn new() -> Self {
        IOMMU {
            stack: [memory::INIT_VALUE; memory::HIGH_RAM_SIZE],
            temp_memory: [memory::INIT_VALUE; 0x10000],
            isr_controller: InterruptController::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            stack_adr if address::HIGH_RAM.contains(&stack_adr) => {
                let converted_address = (address - address::HIGH_RAM.start()) as usize;
                self.stack[converted_address]
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .read_byte_from_hardware_register(address),

            _ => self.temp_memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            stack_adr if address::HIGH_RAM.contains(&stack_adr) => {
                let converted_address = (address - address::HIGH_RAM.start()) as usize;
                self.stack[converted_address] = data;
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
mod mmu_ut {
    use super::*;

    #[test]
    fn write_read_test() {
        //Little Endianness
        let mut mmu = IOMMU::new();
        const WORKING_RAM_START_ADDRESS: u16 = 0xC000;
        const EXP_VALUE: u16 = 0xABCD;

        mmu.write_word(WORKING_RAM_START_ADDRESS, EXP_VALUE);
        let actual_value = mmu.read_word(WORKING_RAM_START_ADDRESS);

        assert_eq!(EXP_VALUE, actual_value);
    }
}
