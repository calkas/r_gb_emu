const WRAM_SIZE: usize = 0x8000;
const HRAM_SIZE: usize = 0x7F;

/// # MMU
/// - Working RAM 8 KiB
/// - Video RAM 8 KiB
pub struct MMU {
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            wram: [0xFF; WRAM_SIZE],
            hram: [0xFF; HRAM_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        todo!();
    }
}
