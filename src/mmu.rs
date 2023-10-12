const WRAM_SIZE: usize = 0xFFFF;

/// # MMU
/// - Working RAM 8 KiB
/// - Video RAM 8 KiB
pub struct MMU {
    wram: [u8; WRAM_SIZE], // For now all 64kB is available
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            wram: [0xFF; WRAM_SIZE],
        }
    }

    fn get_memory(&self, address: u16) -> &[u8] {
        match address {
            _ => &self.wram,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        todo!();
    }
    pub fn write_byte(&self, address: u16, value: u8) {
        todo!();
    }
    pub fn read_word(&self, address: u16) -> u16 {
        todo!();
    }
    pub fn write_word(&self, address: u16, value: u16) {
        todo!();
    }
}
