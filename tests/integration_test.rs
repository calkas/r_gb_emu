use r_gb_emu::cpu::Cpu;
use r_gb_emu::iommu::IOMMU;
use r_gb_emu::peripheral::cartridge::Cartridge;
use r_gb_emu::peripheral::joypad::JoypadInput;
use r_gb_emu::peripheral::ppu::PictureProcessingUnit;
use r_gb_emu::GameBoyEmulator;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::rc::Rc;

#[test]
fn cpu_instruction_behavior_test() {
    let mut gameboy = GameBoyEmulator::new();
    gameboy.load_cartridge("roms/07-jr,jp,call,ret,rst.gb");

    let mut sum_of_cycles = 0;

    let path = Path::new("cpu_log.log");

    let _ = fs::remove_file(path);

    let mut output = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", path.display(), why),
        Ok(file) => file,
    };

    for _ in 1..100000 {
        let _ = output.write(gameboy.get_log().as_bytes());
        sum_of_cycles += gameboy.emulation_step();
    }

    println!("cycles {}", sum_of_cycles);
    println!("Test output: {}", gameboy.serial_out());

    // //63802933 * 4
    // while sum_of_cycles < 200 {
    //     sum_of_cycles += gameboy.emulation_step();
    // }
}
