use super::constants::gb_memory_map::address::io_hardware_register;
use super::constants::gb_memory_map::{address, memory};
use crate::peripheral::{
    cartridge::Cartridge, interrupt_controller::InterruptController, joypad::JoypadInput,
    ppu::PictureProcessingUnit, serial::SerialDataTransfer, timer::Timer, HardwareAccessible,
    IoWorkingCycle,
};
use std::{cell::RefCell, rc::Rc};
/// # I/O Memory Management
/// Input–output memory management unit
pub struct IOMMU {
    cartridge_rom: Rc<RefCell<Cartridge>>,
    wram: [u8; memory::WRAM_SIZE],
    hram: [u8; memory::HIGH_RAM_SIZE],
    isr_controller: InterruptController,
    pub serial: SerialDataTransfer,
    timer: Timer,
    ppu: Rc<RefCell<PictureProcessingUnit>>,
    joypad: Rc<RefCell<JoypadInput>>,
}

impl IOMMU {
    pub fn new(
        cartridge: Rc<RefCell<Cartridge>>,
        ppu: Rc<RefCell<PictureProcessingUnit>>,
        input_controller: Rc<RefCell<JoypadInput>>,
    ) -> Self {
        IOMMU {
            cartridge_rom: cartridge,
            wram: [memory::DEFAULT_INIT_VALUE; memory::WRAM_SIZE],
            hram: [memory::DEFAULT_INIT_VALUE; memory::HIGH_RAM_SIZE],
            isr_controller: InterruptController::default(),
            serial: SerialDataTransfer::default(),
            timer: Timer::default(),
            ppu,
            joypad: input_controller,
        }
    }
    pub fn init(&mut self) {
        self.write_byte(io_hardware_register::JOYPAD_INPUT, 0xCF);
        self.write_byte(io_hardware_register::SERIAL_DATA, 0);
        self.write_byte(io_hardware_register::SERIAL_CONTROL, 0x7E);
        self.write_byte(io_hardware_register::TIMER_DIV, 0x18);
        self.write_byte(io_hardware_register::TIMER_TIMA, 0);
        self.write_byte(io_hardware_register::TIMER_TMA, 0);
        self.write_byte(io_hardware_register::TIMER_TAC, 0xF8);
        self.write_byte(address::INTF_REGISTER, 0xE1);
        // self.write_byte(0xFF10, 0x80);
        // self.write_byte(0xFF11, 0xBF);
        // self.write_byte(0xFF12, 0xF3);
        // self.write_byte(0xFF13, 0xFF);
        // self.write_byte(0xFF14, 0xBF);
        // self.write_byte(0xFF16, 0x3F);
        // self.write_byte(0xFF17, 0);
        // self.write_byte(0xFF18, 0xFF);
        // self.write_byte(0xFF19, 0xBF);
        // self.write_byte(0xFF1A, 0x7F);
        // self.write_byte(0xFF1B, 0xFF);
        // self.write_byte(0xFF1C, 0x9F);
        // self.write_byte(0xFF1D, 0xFF);
        // self.write_byte(0xFF1E, 0xBF);
        // self.write_byte(0xFF20, 0xFF);
        // self.write_byte(0xFF21, 0);
        // self.write_byte(0xFF22, 0);
        // self.write_byte(0xFF23, 0xBF);
        // self.write_byte(0xFF24, 0x77);
        // self.write_byte(0xFF25, 0xF3);
        // self.write_byte(0xFF26, 0xF1);
        self.write_byte(io_hardware_register::LCD_CONTROL, 0x91);
        self.write_byte(io_hardware_register::LCD_STATUS, 0x81);
        self.write_byte(io_hardware_register::SCY, 0);
        self.write_byte(io_hardware_register::SCX, 0);
        //self.write_byte(io_hardware_register::LY, 0x91);
        self.write_byte(io_hardware_register::LYC, 0);
        self.write_byte(io_hardware_register::OAM_DMA, 0xFF);
        self.write_byte(io_hardware_register::BGP, 0xFC);
        self.write_byte(io_hardware_register::OBP0, 0xFF);
        self.write_byte(io_hardware_register::OBP1, 0xFF);
        self.write_byte(io_hardware_register::WY, 0);
        self.write_byte(io_hardware_register::WX, 0);
        self.write_byte(address::INTE_REGISTER, 0);
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            rom_bank_0_address if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .read_byte_from_hardware_register(rom_bank_0_address)
            }

            rom_bank_n_address if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .read_byte_from_hardware_register(rom_bank_n_address)
            }

            vram_address if address::VIDEO_RAM.contains(&vram_address) => self
                .ppu
                .borrow_mut()
                .read_byte_from_hardware_register(vram_address),

            wram_address_bank_0
                if address::WORKING_RAM_BANK_0.contains(&wram_address_bank_0)
                    | address::ECHO_RAM_BANK_0.contains(&wram_address_bank_0) =>
            {
                self.wram[wram_address_bank_0 as usize & memory::WRAM_ADDRESS_MASK]
            }

            wram_address_bank_1_7
                if address::WORKING_RAM_BANK_1_7.contains(&wram_address_bank_1_7)
                    | address::ECHO_RAM_BANK_1_7.contains(&wram_address_bank_1_7) =>
            {
                self.wram[wram_address_bank_1_7 as usize & memory::WRAM_ADDRESS_MASK | 0x1000]
            }

            voam_address if address::OAM.contains(&voam_address) => self
                .ppu
                .borrow_mut()
                .read_byte_from_hardware_register(voam_address),

            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {
                memory::DEFAULT_INIT_VALUE
            }

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr]
            }

            io_hardware_register::JOYPAD_INPUT => self
                .joypad
                .borrow_mut()
                .read_byte_from_hardware_register(address),

            serial_address if address::HARDWARE_IO_SERIAL.contains(&serial_address) => {
                self.serial.read_byte_from_hardware_register(serial_address)
            }

            timer_address if address::HARDWARE_IO_TIMER.contains(&timer_address) => {
                self.timer.read_byte_from_hardware_register(timer_address)
            }

            graphics_address
                if address::HARDWARE_IO_GRAPHICS_1.contains(&graphics_address)
                    | address::HARDWARE_IO_GRAPHICS_2.contains(&graphics_address) =>
            {
                self.ppu
                    .borrow_mut()
                    .read_byte_from_hardware_register(graphics_address)
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .read_byte_from_hardware_register(address),

            _ => {
                //println!("Reading from unsupported addres: {:#06x?}", address);
                memory::DEFAULT_INIT_VALUE
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            rom_bank_0_address if address::CARTRIDGE_ROM_BANK_0.contains(&rom_bank_0_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .write_byte_to_hardware_register(rom_bank_0_address, data);
            }

            rom_bank_n_address if address::CARTRIDGE_ROM_BANK_1_N.contains(&rom_bank_n_address) => {
                self.cartridge_rom
                    .borrow_mut()
                    .write_byte_to_hardware_register(rom_bank_n_address, data);
            }

            vram_address if address::VIDEO_RAM.contains(&vram_address) => self
                .ppu
                .borrow_mut()
                .write_byte_to_hardware_register(vram_address, data),

            wram_address_bank_0
                if address::WORKING_RAM_BANK_0.contains(&wram_address_bank_0)
                    | address::ECHO_RAM_BANK_0.contains(&wram_address_bank_0) =>
            {
                self.wram[wram_address_bank_0 as usize & memory::WRAM_ADDRESS_MASK] = data;
            }

            wram_address_bank_1_7
                if address::WORKING_RAM_BANK_1_7.contains(&wram_address_bank_1_7)
                    | address::ECHO_RAM_BANK_1_7.contains(&wram_address_bank_1_7) =>
            {
                self.wram[wram_address_bank_1_7 as usize & memory::WRAM_ADDRESS_MASK | 0x1000] =
                    data;
            }

            voam_address if address::OAM.contains(&voam_address) => self
                .ppu
                .borrow_mut()
                .write_byte_to_hardware_register(voam_address, data),

            not_usable_address if address::NOT_USABLE.contains(&not_usable_address) => {}

            hram_address if address::HIGH_RAM.contains(&hram_address) => {
                let adjusted_adr = (hram_address - address::HIGH_RAM.start()) as usize;
                self.hram[adjusted_adr] = data;
            }

            io_hardware_register::JOYPAD_INPUT => self
                .joypad
                .borrow_mut()
                .write_byte_to_hardware_register(address, data),

            serial_address if address::HARDWARE_IO_SERIAL.contains(&serial_address) => self
                .serial
                .write_byte_to_hardware_register(serial_address, data),

            timer_address if address::HARDWARE_IO_TIMER.contains(&timer_address) => {
                self.timer
                    .write_byte_to_hardware_register(timer_address, data);
            }
            //FIXME
            io_hardware_register::OAM_DMA => self.oam_dma_transfer(data),

            graphics_address
                if address::HARDWARE_IO_GRAPHICS_1.contains(&graphics_address)
                    | address::HARDWARE_IO_GRAPHICS_2.contains(&graphics_address) =>
            {
                self.ppu
                    .borrow_mut()
                    .write_byte_to_hardware_register(graphics_address, data)
            }

            address::INTF_REGISTER | address::INTE_REGISTER => self
                .isr_controller
                .write_byte_to_hardware_register(address, data),

            _ => {
                // println!(
                //     "Writing to unsupported addres: {:#06x?} data = {:#06x?}",
                //     address, data
                // );
            }
        }
    }

    pub fn process(&mut self, cycles: u32) {
        self.ppu.borrow_mut().next_to(cycles);

        //  * 0 V-Blank
        self.isr_controller.intf.v_blank = self.ppu.borrow_mut().vblank_interrupt_req;
        self.ppu.borrow_mut().vblank_interrupt_req = false;

        //  * 1 LCD
        self.isr_controller.intf.lcd = self.ppu.borrow_mut().lcd_interrupt_req;
        self.ppu.borrow_mut().lcd_interrupt_req = false;

        //  * 2 Timer
        self.timer.next_to(cycles);
        self.isr_controller.intf.timer = self.timer.interrupt_req;
        self.timer.interrupt_req = false;

        //  * 3 Serial Link
        self.isr_controller.intf.serial_link = self.serial.interrupt_req;
        self.serial.interrupt_req = false;

        //  * 4 Joypad
        self.isr_controller.intf.joypad = self.joypad.borrow_mut().interrupt_req;
        self.joypad.borrow_mut().interrupt_req = false;
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low_byte_val = self.read_byte(address) as u16;
        let high_byte_val = self.read_byte(address + 1) as u16;
        high_byte_val.rotate_left(8) | low_byte_val
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low_byte_val = (value & 0x00FF) as u8;
        let high_byte_val = (value & 0xFF00).rotate_right(8) as u8;
        self.write_byte(address, low_byte_val);
        self.write_byte(address + 1, high_byte_val);
    }

    fn oam_dma_transfer(&mut self, hi_source_address: u8) {
        let base_source_address = (hi_source_address as u16).rotate_left(8);
        let base_destination_address = *address::OAM.start();
        for i in 0..memory::VOAM_SIZE as u16 {
            let source_byte = self.read_byte(base_source_address + i);
            self.write_byte(base_destination_address + i, source_byte);
        }
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn little_endianness_test() {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let ppu = Rc::new(RefCell::new(PictureProcessingUnit::new()));
        let joypad = Rc::new(RefCell::new(JoypadInput::default()));
        let mut iommu = IOMMU::new(cartridge.clone(), ppu.clone(), joypad.clone());

        iommu.write_byte(*address::HIGH_RAM.start(), 0xCD);
        iommu.write_byte(*address::HIGH_RAM.start() + 1, 0xAB);
        let actual_value = iommu.read_word(*address::HIGH_RAM.start());

        assert_eq!(0xABCD, actual_value);
    }
    #[test]
    fn read_write_to_memory_map_test() {
        const EXP_STORED_VALUE: u8 = 0xCD;
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let ppu = Rc::new(RefCell::new(PictureProcessingUnit::new()));
        let joypad = Rc::new(RefCell::new(JoypadInput::default()));
        let mut iommu = IOMMU::new(cartridge.clone(), ppu.clone(), joypad.clone());

        // [0xFEA0 - 0xFEFF] Not Usable
        iommu.write_byte(*address::NOT_USABLE.start(), EXP_STORED_VALUE);
        assert_eq!(
            memory::DEFAULT_INIT_VALUE,
            iommu.read_byte(*address::NOT_USABLE.start())
        );

        // [0xFF80 - 0xFFFE] High RAM (HRAM)
        iommu.write_byte(*address::HIGH_RAM.start() + 1, EXP_STORED_VALUE);
        assert_eq!(
            EXP_STORED_VALUE,
            iommu.read_byte(*address::HIGH_RAM.start() + 1)
        );
    }

    #[test]
    fn read_write_to_io_register_test() {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let ppu = Rc::new(RefCell::new(PictureProcessingUnit::new()));
        let joypad = Rc::new(RefCell::new(JoypadInput::default()));
        let mut iommu = IOMMU::new(cartridge.clone(), ppu.clone(), joypad.clone());
        iommu.write_byte(address::io_hardware_register::SERIAL_DATA, 0xAA);

        assert_eq!(
            0xAA,
            iommu.read_byte(address::io_hardware_register::SERIAL_DATA)
        );
    }
}
