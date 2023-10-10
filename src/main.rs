mod cpu;
mod cpu_data;
mod instructions;
mod mmu;

use cpu::Cpu;

fn main() {
    println!("..::CPU 8080 Emulator::..");
    let mut cpu = Cpu::new();
    cpu.load_program(&[0x3C]);
    cpu.process();
    println!("..:: End ::..");
}
