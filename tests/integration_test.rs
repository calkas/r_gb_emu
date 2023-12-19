use r_gb_emu::cpu::Cpu;
use r_gb_emu::iommu::IOMMU;
use r_gb_emu::peripheral::cartridge::Cartridge;
use r_gb_emu::peripheral::joypad::JoypadInput;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn cpu_instruction_behavior_test() {
    let cartridge = Rc::new(RefCell::new(Cartridge::default()));
    let joypad = Rc::new(RefCell::new(JoypadInput::default()));

    cartridge.borrow_mut().load("roms/cpu_instrs.gb");

    let iommu = Rc::new(RefCell::new(IOMMU::new(cartridge.clone(), joypad.clone())));

    let mut cpu = Cpu::new(iommu.clone());
    cpu.init();

    let mut sum_of_cycles = 0;

    //69905
    while sum_of_cycles < 20000 {
        sum_of_cycles += cpu.process();
    }

    for i in iommu.borrow_mut().serial.test_out_data.iter() {
        print!("{}", *i)
    }
}
