mod constants;
mod cpu;
mod cpu_data;
mod instructions;
mod iommu;
mod peripheral;
use cpu::Cpu;

fn main() {
    println!("\x1b[94m=========================\n..::Gameboy Emulator::..\n=========================\x1b[0m");
    let mut cpu = Cpu::new();
    cpu.load_program(&[0x3C]);
    cpu.process();
    println!(
        "\x1b[96m=========================\n      ..::END::..      \n=========================\x1b[0m"
    );
}
