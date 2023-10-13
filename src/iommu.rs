const WRAM_SIZE: usize = 0xFFFF;

/// # I/O Memory Management
/// - Working RAM 8 KiB
/// - Video RAM 8 KiB
pub struct IOMMU {
    wram: [u8; WRAM_SIZE], // For now all 64kB is available
}

impl IOMMU {
    pub fn new() -> Self {
        IOMMU {
            wram: [0xFF; WRAM_SIZE],
        }
    }

    fn read_byte_from_device(&self, address: usize) -> u8 {
        match address {
            _ => self.wram[address],
        }
    }

    fn write_byte_to_device(&mut self, address: usize, data: u8) {
        match address {
            _ => self.wram[address] = data,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.read_byte_from_device(address as usize)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.write_byte_to_device(address as usize, value);
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
