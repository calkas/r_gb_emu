mod constants;
mod cpu;
mod cpu_data;
pub mod emulator_constants;
mod instructions;
mod iommu;
mod peripheral;

use std::cell::RefCell;
use std::rc::Rc;

use cpu::Cpu;
use iommu::IOMMU;
use peripheral::{cartridge::Cartridge, joypad::JoypadInput, ppu::PictureProcessingUnit};

pub struct GameBoyEmulator {
    cartridge: Rc<RefCell<Cartridge>>,
    ppu: Rc<RefCell<PictureProcessingUnit>>,
    pub joypad: Rc<RefCell<JoypadInput>>,
    iommu: Rc<RefCell<IOMMU>>,
    cpu: Cpu,
}

impl GameBoyEmulator {
    pub fn new() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let ppu = Rc::new(RefCell::new(PictureProcessingUnit::new()));
        let joypad = Rc::new(RefCell::new(JoypadInput::default()));
        let iommu = Rc::new(RefCell::new(IOMMU::new(
            cartridge.clone(),
            ppu.clone(),
            joypad.clone(),
        )));

        iommu.borrow_mut().init();

        let mut cpu = Cpu::new(iommu.clone());
        cpu.init();

        Self {
            cartridge,
            ppu,
            joypad,
            iommu,
            cpu,
        }
    }

    pub fn load_cartridge(&mut self, cartridge_path: &str) {
        self.cartridge.borrow_mut().load(cartridge_path);
    }

    pub fn emulation_step(&mut self) -> u32 {
        self.cpu.process()
    }

    /// # get_log
    /// For debug purpose to feed Gameboy Doctor in special format
    pub fn get_log(&mut self) -> String {
        self.cpu.debug_dump_regs().to_uppercase()
    }

    pub fn serial_out(&self) -> String {
        let mut out = String::new();
        for i in self.iommu.borrow_mut().serial.test_out_data.iter() {
            out.push(*i);
        }
        return out;
    }
}
