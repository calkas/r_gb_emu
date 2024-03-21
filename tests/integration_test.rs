use r_gb_emu::GameBoyEmulator;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// This is test for gameboy doctor
#[test]
fn cpu_instruction_behavior_test() {
    // let mut gameboy = GameBoyEmulator::new();
    // gameboy.load_cartridge("roms/08-misc instrs.gb");

    // let mut sum_of_cycles = 0;

    // let path = Path::new("cpu_log.log");

    // let _ = fs::remove_file(path);

    // let mut output = match File::create(path) {
    //     Err(why) => panic!("couldn't create {}: {}", path.display(), why),
    //     Ok(file) => file,
    // };

    // for _ in 1..400000 {
    //     let _ = output.write(gameboy.get_log().as_bytes());
    //     sum_of_cycles += gameboy.emulation_step();
    // }

    // println!("cycles {}", sum_of_cycles);
    // println!("Test output: {}", gameboy.serial_out());
}

#[test]
fn cpu_08_misc_instrs_test() {
    let mut gameboy = GameBoyEmulator::new();
    gameboy.load_cartridge("roms/08-misc instrs.gb").unwrap();

    let exp_test_result = String::from("08-misc instrs\n\n\nPassed\n");

    for _ in 1..400000 {
        let _ = gameboy.emulate_step();
    }

    assert_eq!(exp_test_result, gameboy.serial_out());
}

#[test]
fn cpu_07_jr_jp_call_ret_rst_test() {
    let mut gameboy = GameBoyEmulator::new();
    gameboy
        .load_cartridge("roms/07-jr,jp,call,ret,rst.gb")
        .unwrap();

    let exp_test_result = String::from("07-jr,jp,call,ret,rst\n\n\nPassed\n");

    for _ in 1..400000 {
        let _ = gameboy.emulate_step();
    }

    assert_eq!(exp_test_result, gameboy.serial_out());
}
