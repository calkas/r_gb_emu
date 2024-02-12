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
use emulator_constants::resolution;
use emulator_constants::GameBoyKeys;
use iommu::IOMMU;
use peripheral::{cartridge::Cartridge, joypad::JoypadInput, ppu::PictureProcessingUnit};
use std::{thread, time};

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

    pub fn emulate_step(&mut self) -> u32 {
        self.cpu.process() // 0,000000238 * cycle
    }
    /// A frame consists of 154 scanlines. A dot = 4194304 Hhz
    /// Frame 1/4194304 (0,000000238) * 456 * 154 = 0,016742706 = 16,74 ms <--60 fps
    /// &mut [u32; resolution::SCREEN_W * resolution::SCREEN_H]
    pub fn emulate_frame(&mut self, frame_buffer: &mut [u32]) {
        let start_time_of_emulation_frame = time::Instant::now();
        const NUMBER_OF_CYCLES_PER_FRAME: u32 = 70224;
        let mut sum_of_processed_cycles: u32 = 0;

        while sum_of_processed_cycles < NUMBER_OF_CYCLES_PER_FRAME {
            sum_of_processed_cycles += self.emulate_step();
        }

        println!("Duration {:?}", start_time_of_emulation_frame.elapsed());

        let mut i: usize = 0;

        for pixel in self.ppu.borrow_mut().out_frame_buffer.iter() {
            for v in pixel.iter() {
                let b = u32::from(v[0]) << 16;
                let g = u32::from(v[1]) << 8;
                let r = u32::from(v[2]);
                let a = 0xff00_0000;

                frame_buffer[i] = a | b | g | r;
                i += 1;
            }
        }

        let frame_millis = time::Duration::from_millis(16);

        //std::thread::sleep(frame_millis);
    }

    pub fn button_pressed(&mut self, key: GameBoyKeys) {
        self.joypad.borrow_mut().key_pressed(key);
    }

    pub fn button_released(&mut self, key: GameBoyKeys) {
        self.joypad.borrow_mut().key_released(key);
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
