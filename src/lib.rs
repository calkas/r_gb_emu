mod constants;
mod cpu;
mod cpu_data;
pub mod emulator_constants;
mod instructions;
mod iommu;
mod peripheral;

use cpu::Cpu;
use emulator_constants::GameBoyKeys;
use iommu::IOMMU;
use peripheral::{cartridge::Cartridge, joypad::JoypadInput, ppu::PictureProcessingUnit};
use std::cell::RefCell;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::{thread, time};

pub struct GameBoyEmulator {
    cartridge: Rc<RefCell<Cartridge>>,
    ppu: Rc<RefCell<PictureProcessingUnit>>,
    pub joypad: Rc<RefCell<JoypadInput>>,
    iommu: Rc<RefCell<IOMMU>>,
    cpu: Cpu,
}

impl Default for GameBoyEmulator {
    fn default() -> Self {
        Self::new()
    }
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

    /// # load_cartridge
    pub fn load_cartridge(&mut self, cartridge_path: &str) -> std::io::Result<()> {
        let path = Path::new(cartridge_path);
        self.cartridge.borrow_mut().load(path)?;
        Ok(())
    }

    /// # show_cartridge_status
    pub fn show_cartridge_status(&self) {
        self.cartridge.borrow_mut().show_status();
    }

    /// # get_cartridge_name
    pub fn get_cartridge_name(&self) -> String {
        self.cartridge.borrow_mut().name.clone()
    }

    /// # emulate_step
    /// One cpu step
    pub fn emulate_step(&mut self) -> u32 {
        // 0,000000238 * cycle
        self.cpu.process()
    }

    /// # emulate_frame
    /// A frame consists of 154 scanlines. A dot = 4194304 Hhz
    /// Frame 1/4194304 (0,000000238) * 456 * 154 = 0,016742706 = 16,74 ms <--60 fps
    pub fn emulate_frame(&mut self, frame_buffer: &mut [u32]) {
        let start_time_of_emulation_frame = time::Instant::now();
        const NUMBER_OF_CYCLES_PER_FRAME: u32 = 456 * 154;
        let mut sum_of_processed_cycles: u32 = 0;

        while sum_of_processed_cycles < NUMBER_OF_CYCLES_PER_FRAME {
            sum_of_processed_cycles += self.emulate_step();
        }

        let mut frame_pixel_id: usize = 0;

        for ppu_pixel in self.ppu.borrow_mut().out_frame_buffer.iter() {
            for color in ppu_pixel.iter() {
                let blue = u32::from(color[0]).rotate_left(16);
                let green = u32::from(color[1]).rotate_left(8);
                let red = u32::from(color[2]);
                let alpha = 0xFF000000;
                frame_buffer[frame_pixel_id] = alpha | blue | green | red;
                frame_pixel_id += 1;
            }
        }
        let end_of_processed_time = start_time_of_emulation_frame.elapsed();

        if end_of_processed_time < time::Duration::from_micros(16743) {
            let sleeping_time = time::Duration::from_micros(16743) - end_of_processed_time;
            thread::sleep(sleeping_time);
        }

        let fps = 1.0 / start_time_of_emulation_frame.elapsed().as_secs_f32();
        print!("\rFPS:{:?} ", fps);
        let _ = std::io::stdout().flush();
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
        out
    }
}
