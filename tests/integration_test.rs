use r_gb_emu::cpu::Cpu;
use r_gb_emu::iommu::IOMMU;
use r_gb_emu::peripheral::cartridge::Cartridge;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn cpu_instruction_behavior_test() {
    let cartridge = Rc::new(RefCell::new(Cartridge::default()));

    cartridge.borrow_mut().load("roms/cpu_instrs.gb");

    let iommu = Rc::new(RefCell::new(IOMMU::new(cartridge.clone())));

    let mut cpu = Cpu::new(iommu.clone());
    cpu.init();

    let mut sum_of_cycles = 0;

    //69905
    while sum_of_cycles < 1000000 {
        sum_of_cycles += cpu.process();
    }

    for i in iommu.borrow_mut().serial.get_test_buff() {
        print!("{}", *i)
    }
}
