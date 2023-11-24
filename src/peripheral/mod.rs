pub mod interrupt_controller;
pub mod serial;

/// # Hardware access
/// - Cartridge space.
/// - WRAM and Display RAM.
/// - I/O (joypad, audio, graphics and LCD)
/// - Interrupt controls.
pub trait HardwareAccessible {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8;
    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8);
}
