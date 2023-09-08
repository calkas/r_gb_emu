mod cpu;
mod cpu_data;
mod instructions;

use cpu::Cpu;
use cpu_data::Flags;

// fn is_half_carry(reg: u8, value: u8) -> bool {
//     ((reg & 0x0F) + (value & 0x0F)) & 0xF0 == 0x10
// }

// fn is_zero(new_value: u16) -> bool {
//     new_value == 0
// }

fn main() {
    println!("..::CPU 8080 Emulator::..");
    println!("..:: End ::..");
}
