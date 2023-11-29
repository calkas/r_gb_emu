pub mod interrupt_controller;
pub mod serial;
pub mod timer;

/// # HardwareAccessible trait
///
///
/// I/O
///
/// The I/O region of the address bus is connected to a lot of peripherals:
///
/// - Video Controller
/// - Sound Controller
/// - D-Pad and Button Inputs
/// - Serial Data Transfer via Link Cable
/// - Timer
pub trait HardwareAccessible {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8;
    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8);
}

pub trait IoWorkingCycle {
    fn is_interrupt(&self) -> bool;
    fn reset_interrupt(&mut self);
    fn run_cycle(&mut self, cycle: u32);
}
