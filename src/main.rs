mod constants;
mod cpu;
mod cpu_data;
mod instructions;
mod iommu;
mod peripheral;

fn main() {
    println!("\x1b[94m=========================\n..::Gameboy Emulator::..\n=========================\x1b[0m");

    println!(
        "\x1b[96m=========================\n      ..::END::..      \n=========================\x1b[0m"
    );
}
