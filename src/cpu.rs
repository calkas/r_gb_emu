use super::constants::gb_memory_map::{address, isr_adress};
use super::cpu_data::{ControlFlags, FlagsRegister, Registers};
use super::iommu::IOMMU;
use crate::instructions::{
    self, arithmetic_logic, cpu_control, jump, load, rotate_and_shift, single_bit_operation,
};
use crate::peripheral::interrupt_controller::InterruptRegister;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
/// # DMG-CPU
/// 8-bit 8080-like Sharp CPU
pub struct Cpu {
    register: Registers,
    cycles: u32,
    control: ControlFlags,
    iommu: Rc<RefCell<IOMMU>>,
}
impl Cpu {
    pub fn new(iommu: Rc<RefCell<IOMMU>>) -> Self {
        Cpu {
            register: Registers::default(),
            cycles: 0,
            control: ControlFlags::default(),
            iommu,
        }
    }

    pub fn init(&mut self) {
        self.register.a = 0x01;
        self.register.flag = FlagsRegister::from(0xB0);
        self.register.b = 0x00;
        self.register.c = 0x13;
        self.register.d = 0x00;
        self.register.e = 0xD8;
        self.register.h = 0x01;
        self.register.l = 0x4D;
        self.register.sp = *address::HIGH_RAM.end();
        self.register.pc = address::cartridge_header::ENTRY_POINT;
    }

    pub fn process(&mut self) -> u32 {
        //todo HALT handling support

        self.interrupt_handling();
        let opcode = self.fetch_byte();

        //self.debug_dump_processing_instruction(opcode);

        if self.is_prefix_instruction(opcode) {
            let opcode = self.fetch_byte();
            //self.debug_dump_processing_instruction(opcode);
            self.execute_cbprefixed_instruction(opcode);
        } else {
            self.execute(opcode);
        }

        self.iommu.borrow_mut().process(self.cycles);
        //self.debug_dump_regs();
        self.cycles
    }

    fn is_prefix_instruction(&self, opcode: u8) -> bool {
        opcode == 0xCB
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.iommu.borrow_mut().read_byte(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let low_byte = self.fetch_byte();
        let high_byte = self.fetch_byte();
        (high_byte as u16).rotate_left(8) | (low_byte as u16)
    }

    fn interrupt_handling(&mut self) {
        if !self.control.ime {
            return;
        }

        let intf = self.iommu.borrow_mut().read_byte(address::INTF_REGISTER);
        let inte = self.iommu.borrow_mut().read_byte(address::INTE_REGISTER);

        if inte & intf != 0 {
            self.control.ime = false;
            load::push(
                &mut self.iommu.borrow_mut(),
                &mut self.register.sp,
                self.register.pc,
            );
            let mut isr_reg = InterruptRegister::from(intf);
            if isr_reg.v_blank {
                self.register.pc = isr_adress::V_BLANK;
                isr_reg.v_blank = false;
            } else if isr_reg.lcd {
                self.register.pc = isr_adress::LCD_STATUS;
                isr_reg.lcd = false;
            } else if isr_reg.timer {
                self.register.pc = isr_adress::TIMER;
                isr_reg.timer = false;
            } else if isr_reg.serial_link {
                self.register.pc = isr_adress::SERIAL_LINK;
                isr_reg.serial_link = false;
            } else if isr_reg.joypad {
                self.register.pc = isr_adress::JOYPAD;
                isr_reg.joypad = false;
            } else { //Error
            }
            let mask: u8 = InterruptRegister::into(isr_reg);
            let reg_val_for_reset_isr = mask & intf;
            self.iommu
                .borrow_mut()
                .write_byte(address::INTF_REGISTER, reg_val_for_reset_isr);
            self.cycles = 16;
        }
    }

    fn arithmetic_logic_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            // .::ADD operation::.
            0x09 | 0x19 | 0x29 | 0x39 => {
                let value = self
                    .register
                    .get_reg16_value_from_opcode_array(&[0x09, 0x19, 0x29, 0x39], opcode);
                arithmetic_logic::add_hl(
                    &mut self.register.flag,
                    &mut self.register.h,
                    &mut self.register.l,
                    value,
                );
                self.cycles = 8;
            }
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x87 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x87],
                    opcode,
                );
                arithmetic_logic::add(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                    0,
                );
                self.cycles = 4;
            }
            0x86 => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::add(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycles = 8;
            }
            0xC6 => {
                let val = self.fetch_byte();
                arithmetic_logic::add(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycles = 8;
            }
            0xE8 => {
                let val = self.fetch_byte() as i8;
                arithmetic_logic::add_sp(&mut self.register.flag, &mut self.register.sp, val);
                self.cycles = 16;
            }

            // .::ADC operation::.
            0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8F],
                    opcode,
                );
                arithmetic_logic::adc(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0xCE => {
                let val = self.fetch_byte();
                arithmetic_logic::adc(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0x8E => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::adc(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::SUB operation::.
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x97 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x97],
                    opcode,
                );
                arithmetic_logic::sub(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                    0,
                );
                self.cycles = 4;
            }
            0x96 => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::sub(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycles = 8;
            }
            0xD6 => {
                let val = self.fetch_byte();
                arithmetic_logic::sub(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycles = 8;
            }

            // .::SBC operation::.
            0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9F],
                    opcode,
                );
                arithmetic_logic::sbc(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0x9E => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::sbc(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0xDE => {
                let val = self.fetch_byte();
                arithmetic_logic::sbc(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::AND operation::.
            0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA7 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA7],
                    opcode,
                );
                arithmetic_logic::and(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0xA6 => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::and(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0xE6 => {
                let val = self.fetch_byte();
                arithmetic_logic::and(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::XOR operation::.
            0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAF => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAF],
                    opcode,
                );
                arithmetic_logic::xor(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0xAE => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::xor(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0xEE => {
                let val = self.fetch_byte();
                arithmetic_logic::xor(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::OR operation::.
            0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB7 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB7],
                    opcode,
                );
                arithmetic_logic::xor(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0xB6 => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::or(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0xF6 => {
                let val = self.fetch_byte();
                arithmetic_logic::or(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::CP operation::.
            0xB8 | 0xB9 | 0xBA | 0xBB | 0xBC | 0xBD | 0xBF => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF],
                    opcode,
                );
                arithmetic_logic::cp(
                    &mut self.register.flag,
                    &mut self.register.a,
                    register_value,
                );
                self.cycles = 4;
            }
            0xBE => {
                let val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::cp(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }
            0xFE => {
                let val = self.fetch_byte();
                arithmetic_logic::cp(&mut self.register.flag, &mut self.register.a, val);
                self.cycles = 8;
            }

            // .::INC operation::.
            0x03 => {
                arithmetic_logic::inc_16(&mut self.register.b, &mut self.register.c);
                self.cycles = 8;
            }
            0x13 => {
                arithmetic_logic::inc_16(&mut self.register.d, &mut self.register.e);
                self.cycles = 8;
            }
            0x23 => {
                arithmetic_logic::inc_16(&mut self.register.h, &mut self.register.l);
                self.cycles = 8;
            }
            0x33 => {
                let mut sp_low: u8 = self.register.sp as u8;
                let mut sp_high: u8 = ((self.register.sp & 0xFF00) >> 8) as u8;
                arithmetic_logic::inc_16(&mut sp_high, &mut sp_low);

                self.register.sp = (sp_high as u16).rotate_left(8) | (sp_low as u16);
                self.cycles = 8;
            }
            0x04 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.b);
                self.cycles = 4;
            }
            0x0C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.c);
                self.cycles = 4;
            }
            0x14 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.d);
                self.cycles = 4;
            }
            0x1C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.e);
                self.cycles = 4;
            }
            0x24 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.h);
                self.cycles = 4;
            }
            0x2C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.l);
                self.cycles = 4;
            }
            0x3C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            0x34 => {
                let mut val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::inc(&mut self.register.flag, &mut val);
                self.iommu
                    .borrow_mut()
                    .write_byte(self.register.get_hl(), val);
                self.cycles = 12;
            }

            // .::DEC operation::.
            0x0B => {
                arithmetic_logic::dec_16(&mut self.register.b, &mut self.register.c);
                self.cycles = 8;
            }
            0x1B => {
                arithmetic_logic::dec_16(&mut self.register.d, &mut self.register.e);
                self.cycles = 8;
            }
            0x2B => {
                arithmetic_logic::dec_16(&mut self.register.h, &mut self.register.l);
                self.cycles = 8;
            }
            0x3B => {
                let mut sp_low: u8 = self.register.sp as u8;
                let mut sp_high: u8 = ((self.register.sp & 0xFF00) >> 8) as u8;
                arithmetic_logic::dec_16(&mut sp_high, &mut sp_low);

                self.register.sp = (sp_high as u16).rotate_left(8) | (sp_low as u16);
                self.cycles = 8;
            }
            0x05 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.b);
                self.cycles = 4;
            }
            0x0D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.c);
                self.cycles = 4;
            }
            0x15 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.d);
                self.cycles = 4;
            }
            0x1D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.e);
                self.cycles = 4;
            }
            0x25 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.h);
                self.cycles = 4;
            }
            0x2D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.l);
                self.cycles = 4;
            }
            0x3D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            0x35 => {
                let mut val = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                arithmetic_logic::dec(&mut self.register.flag, &mut val);
                self.iommu
                    .borrow_mut()
                    .write_byte(self.register.get_hl(), val);
                self.cycles = 12;
            }

            // .::DAA operation::.
            0x27 => {
                arithmetic_logic::daa(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }

            // .::CPL operation::.
            0x2F => {
                arithmetic_logic::cpl(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }

            // .::LD HL operation::.
            0xF8 => {
                let val = self.fetch_byte() as i8;
                arithmetic_logic::ld_hl(
                    &mut self.register.flag,
                    &mut self.register.h,
                    &mut self.register.l,
                    self.register.sp,
                    val,
                );
                self.cycles = 12;
            }
            _ => panic!("arithmetic_logic opcode [{:#02x?}] not supported!", opcode),
        }
    }

    fn load_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            0x01 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.b, &mut self.register.c, value);
                self.cycles = 12;
            }
            0x02 => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_bc());
                load::ld(&mut self.register.a, value);
                self.cycles = 8;
            }
            0x06 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.b, value);
                self.cycles = 8;
            }
            0x08 => {
                let address = self.fetch_word();
                self.iommu
                    .borrow_mut()
                    .write_word(address, self.register.sp);
                self.cycles = 20;
            }
            0x0A => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_bc());
                load::ld(&mut self.register.b, value);
                self.cycles = 8;
            }
            0x0E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.c, value);
                self.cycles = 8;
            }
            0x11 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.d, &mut self.register.e, value);
                self.cycles = 12;
            }
            0x12 => {
                let address = self.register.get_de();
                self.iommu.borrow_mut().write_byte(address, self.register.a);
                self.cycles = 8;
            }
            0x16 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.d, value);
                self.cycles = 8;
            }
            0x1A => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_de());
                load::ld(&mut self.register.a, value);
                self.cycles = 8;
            }
            0x1E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.e, value);
                self.cycles = 8;
            }
            0x21 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.h, &mut self.register.l, value);
                self.cycles = 12;
            }
            0x22 => {
                let address = load::hli(&mut self.register.h, &mut self.register.l);
                self.iommu.borrow_mut().write_byte(address, self.register.a);
                self.cycles = 8;
            }
            0x26 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.h, value);
                self.cycles = 8;
            }
            0x2A => {
                let address = load::hli(&mut self.register.h, &mut self.register.l);
                self.register.a = self.iommu.borrow_mut().read_byte(address);
                self.cycles = 8;
            }
            0x2E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.l, value);
                self.cycles = 8;
            }
            0x31 => {
                self.register.sp = self.fetch_word();
                self.cycles = 12;
            }
            0x32 => {
                let address = load::hld(&mut self.register.h, &mut self.register.l);
                self.iommu.borrow_mut().write_byte(address, self.register.a);
                self.cycles = 8;
            }
            0x36 => {
                let value = self.fetch_byte();
                self.iommu
                    .borrow_mut()
                    .write_byte(self.register.get_hl(), value);
                self.cycles = 12;
            }
            0x3A => {
                let address = load::hld(&mut self.register.h, &mut self.register.l);
                self.register.a = self.iommu.borrow_mut().read_byte(address);
                self.cycles = 8;
            }
            0x3E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.a, value);
                self.cycles = 8;
            }
            // LD B, r
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x47 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x47],
                    opcode,
                );
                load::ld(&mut self.register.b, register_value);
                self.cycles = 4;
            }
            0x46 => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.b, value);
                self.cycles = 8;
            }
            // LD C, r
            0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4F],
                    opcode,
                );
                load::ld(&mut self.register.c, register_value);
                self.cycles = 4;
            }
            0x4E => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.c, value);
                self.cycles = 8;
            }
            // LD D, r
            0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x57 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57],
                    opcode,
                );
                load::ld(&mut self.register.d, register_value);
                self.cycles = 4;
            }
            0x56 => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.d, value);
                self.cycles = 8;
            }
            // LD E, r
            0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5F],
                    opcode,
                );
                load::ld(&mut self.register.e, register_value);
                self.cycles = 4;
            }
            0x5E => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.e, value);
                self.cycles = 8;
            }

            // LD H, r
            0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x67 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67],
                    opcode,
                );
                load::ld(&mut self.register.h, register_value);
                self.cycles = 4;
            }
            0x66 => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.h, value);
                self.cycles = 8;
            }
            // LD L, r
            0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6F],
                    opcode,
                );
                load::ld(&mut self.register.h, register_value);
                self.cycles = 4;
            }
            0x6E => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.e, value);
                self.cycles = 8;
            }
            // LD(HL), reg
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77],
                    opcode,
                );
                let address = self.register.get_hl();
                self.iommu.borrow_mut().write_byte(address, register_value);
                self.cycles = 8;
            }
            // LD A, r
            0x78 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7F],
                    opcode,
                );
                load::ld(&mut self.register.a, register_value);
                self.cycles = 4;
            }
            0x7E => {
                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                load::ld(&mut self.register.a, value);
                self.cycles = 8;
            }
            // write to io-port n (memory FF00+n)
            0xE0 => {
                let address = self.fetch_byte();
                let port_address = load::calculate_address_for_io_port(address);
                self.iommu
                    .borrow_mut()
                    .write_byte(port_address, self.register.a);
                self.cycles = 12;
            }

            //write to io-port C
            0xE2 => {
                let port_address = load::calculate_address_for_io_port(self.register.c);
                self.iommu
                    .borrow_mut()
                    .write_byte(port_address, self.register.a);
                self.cycles = 8;
            }
            0xEA => {
                let address = self.fetch_word();
                self.iommu.borrow_mut().write_byte(address, self.register.a);
                self.cycles = 16;
            }
            //read from io-port n (memory FF00+n)
            0xF0 => {
                let address = self.fetch_byte();
                let port_address = load::calculate_address_for_io_port(address);
                self.register.a = self.iommu.borrow_mut().read_byte(port_address);
                self.cycles = 12;
            }
            // read from io-port C
            0xF2 => {
                let port_address = load::calculate_address_for_io_port(self.register.c);
                self.register.a = self.iommu.borrow_mut().read_byte(port_address);
                self.cycles = 8;
            }
            //0xF8 in ADD instructions
            0xF9 => {
                self.register.sp = self.register.get_hl();
                self.cycles = 8;
            }
            0xFA => {
                let address = self.fetch_word();
                self.register.a = self.iommu.borrow_mut().read_byte(address);
                self.cycles = 16;
            }
            // PUSH
            0xC5 | 0xD5 | 0xE5 => {
                let reg_val = self
                    .register
                    .get_reg16_value_from_opcode_array(&[0xC5, 0xD5, 0xE5], opcode);
                load::push(&mut self.iommu.borrow_mut(), &mut self.register.sp, reg_val);
                self.cycles = 16;
            }
            //AF
            0xF5 => {
                let reg_val = self.register.get_af();
                load::push(&mut self.iommu.borrow_mut(), &mut self.register.sp, reg_val);
                self.cycles = 16;
            }
            // POP
            0xC1 => {
                let value = load::pop(&mut self.iommu.borrow_mut(), &mut self.register.sp);
                self.register.set_bc(value);
                self.cycles = 12;
            }
            0xD1 => {
                let value = load::pop(&mut self.iommu.borrow_mut(), &mut self.register.sp);
                self.register.set_de(value);
                self.cycles = 12;
            }
            0xE1 => {
                let value = load::pop(&mut self.iommu.borrow_mut(), &mut self.register.sp);
                self.register.set_hl(value);
                self.cycles = 12;
            }
            0xF1 => {
                let value = load::pop(&mut self.iommu.borrow_mut(), &mut self.register.sp);
                self.register.set_af(value);
                self.cycles = 12;
            }
            _ => panic!("load opcode [{:#02x?}] not supported!", opcode),
        }
    }

    fn accumulator_rotate_and_shift_operation_for_dispatcher(&mut self, opcode: u8) {
        match opcode {
            0x07 => {
                rotate_and_shift::rlca(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            0x17 => {
                rotate_and_shift::rla(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            0x0F => {
                rotate_and_shift::rrca(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            0x1F => {
                rotate_and_shift::rra(&mut self.register.flag, &mut self.register.a);
                self.cycles = 4;
            }
            _ => panic!(
                "acc rotate and shift opcode [{:#02x?}] not supported!",
                opcode
            ),
        }
    }
    fn rotate_and_shift_operation_dispatcher(&mut self, opcode: u8) {
        match opcode {
            //RLC
            0x00 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x01 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x02 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x03 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x04 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x05 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x06 => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::rlc(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x07 => {
                rotate_and_shift::rlc(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //RRC
            0x08 => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x09 => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x0A => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x0B => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x0C => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x0D => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x0E => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::rrc(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x0F => {
                rotate_and_shift::rrc(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            // RL
            0x11 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x12 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x13 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x14 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x10 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x15 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x16 => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::rl(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x17 => {
                rotate_and_shift::rl(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //RR
            0x18 => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x19 => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x1A => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x1B => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x1C => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x1D => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x1E => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::rr(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x1F => {
                rotate_and_shift::rr(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //SLA
            0x20 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x21 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x22 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x23 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x24 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x25 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x26 => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::sla(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x27 => {
                rotate_and_shift::sla(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //SRA
            0x28 => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x29 => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x2A => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x2B => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x2C => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x2D => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x2E => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::sra(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x2F => {
                rotate_and_shift::sra(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //Swap
            0x30 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x31 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x32 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x33 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x34 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x35 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x36 => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::swap(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x37 => {
                rotate_and_shift::swap(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }

            //SRL
            0x38 => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.b);
                self.cycles = 8;
            }
            0x39 => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.c);
                self.cycles = 8;
            }
            0x3A => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.d);
                self.cycles = 8;
            }
            0x3B => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.e);
                self.cycles = 8;
            }
            0x3C => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.h);
                self.cycles = 8;
            }
            0x3D => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.l);
                self.cycles = 8;
            }
            0x3E => {
                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                rotate_and_shift::srl(&mut self.register.flag, &mut value);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }
            0x3F => {
                rotate_and_shift::srl(&mut self.register.flag, &mut self.register.a);
                self.cycles = 8;
            }
            _ => panic!("rotate and shift opcode [{:#02x?}] not supported!", opcode),
        }
    }
    fn single_bit_operation_dispatcher(&mut self, opcode: u8) {
        match opcode {
            // BIT
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x47 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x47],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 0);
                self.cycles = 8;
            }
            0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4F],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 1);
                self.cycles = 8;
            }
            0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x57 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 2);
                self.cycles = 8;
            }
            0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5F],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 3);
                self.cycles = 8;
            }
            0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x67 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 4);
                self.cycles = 8;
            }
            0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6F],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 5);
                self.cycles = 8;
            }
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 6);
                self.cycles = 8;
            }
            0x78 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7F],
                    opcode,
                );
                single_bit_operation::bit(&mut self.register.flag, register_value, 7);
                self.cycles = 8;
            }
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x76 | 0x7E => {
                let opcode_with_bit_pos: [u8; 8] = [0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x76, 0x7E];
                let bit_number = opcode_with_bit_pos
                    .iter()
                    .position(|&x| x == opcode)
                    .unwrap() as u8;

                let value = self.iommu.borrow_mut().read_byte(self.register.get_hl());
                single_bit_operation::bit(&mut self.register.flag, value, bit_number);
                self.cycles = 12;
            }

            // RES
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x87 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x87],
                    opcode,
                );
                single_bit_operation::res(register_address, 0);
                self.cycles = 8;
            }
            0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8F => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8F],
                    opcode,
                );
                single_bit_operation::res(register_address, 1);
                self.cycles = 8;
            }
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x97 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x97],
                    opcode,
                );
                single_bit_operation::res(register_address, 2);
                self.cycles = 8;
            }
            0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9F => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9F],
                    opcode,
                );
                single_bit_operation::res(register_address, 3);
                self.cycles = 8;
            }
            0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA7],
                    opcode,
                );
                single_bit_operation::res(register_address, 4);
                self.cycles = 8;
            }
            0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAF],
                    opcode,
                );
                single_bit_operation::res(register_address, 5);
                self.cycles = 8;
            }
            0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB7],
                    opcode,
                );
                single_bit_operation::res(register_address, 6);
                self.cycles = 8;
            }
            0xB9 | 0xBA | 0xBB | 0xB8 | 0xBC | 0xBD | 0xBF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xB9, 0xBA, 0xBB, 0xB8, 0xBC, 0xBD, 0xBF],
                    opcode,
                );
                single_bit_operation::res(register_address, 7);
                self.cycles = 8;
            }
            0x86 | 0x8E | 0x96 | 0x9E | 0xA6 | 0xAE | 0xB6 | 0xBE => {
                let opcode_with_bit_pos: [u8; 8] = [0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE];
                let bit_number = opcode_with_bit_pos
                    .iter()
                    .position(|&x| x == opcode)
                    .unwrap() as u8;

                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                single_bit_operation::res(&mut value, bit_number);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }

            // SET
            0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC4 | 0xC5 | 0xC7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC7],
                    opcode,
                );
                single_bit_operation::set(register_address, 0);
                self.cycles = 8;
            }
            0xC8 | 0xC9 | 0xCA | 0xCB | 0xCC | 0xCD | 0xCF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCF],
                    opcode,
                );
                single_bit_operation::set(register_address, 1);
                self.cycles = 8;
            }
            0xD0 | 0xD1 | 0xD2 | 0xD3 | 0xD4 | 0xD5 | 0xD7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD7],
                    opcode,
                );
                single_bit_operation::set(register_address, 2);
                self.cycles = 8;
            }
            0xD8 | 0xD9 | 0xDA | 0xDB | 0xDC | 0xDD | 0xDF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDF],
                    opcode,
                );
                single_bit_operation::set(register_address, 3);
                self.cycles = 8;
            }
            0xE0 | 0xE1 | 0xE2 | 0xE3 | 0xE4 | 0xE5 | 0xE7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE7],
                    opcode,
                );
                single_bit_operation::set(register_address, 4);
                self.cycles = 8;
            }
            0xE8 | 0xE9 | 0xEA | 0xEB | 0xEC | 0xED | 0xEF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEF],
                    opcode,
                );
                single_bit_operation::set(register_address, 5);
                self.cycles = 8;
            }
            0xF0 | 0xF1 | 0xF2 | 0xF3 | 0xF4 | 0xF5 | 0xF7 => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF7],
                    opcode,
                );
                single_bit_operation::set(register_address, 6);
                self.cycles = 8;
            }
            0xF8 | 0xF9 | 0xFA | 0xFB | 0xFC | 0xFD | 0xFF => {
                let register_address = self.register.get_reg_address_from_opcode_range(
                    &[0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFF],
                    opcode,
                );
                single_bit_operation::set(register_address, 7);
                self.cycles = 8;
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                let opcode_with_bit_pos: [u8; 8] = [0xC6, 0xCE, 0xD6, 0xDE, 0xE6, 0xEE, 0xF6, 0xFE];
                let bit_number = opcode_with_bit_pos
                    .iter()
                    .position(|&x| x == opcode)
                    .unwrap() as u8;

                let address = self.register.get_hl();
                let mut value = self.iommu.borrow_mut().read_byte(address);
                single_bit_operation::set(&mut value, bit_number);
                self.iommu.borrow_mut().write_byte(address, value);
                self.cycles = 16;
            }

            _ => panic!("single bit opcode [{:#02x?}] not supported!", opcode),
        }
    }
    fn jump_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            //JR
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                let mut conditional_relative_jump: HashMap<u8, bool> = HashMap::new();
                conditional_relative_jump.insert(0x18, true); //Always jump
                conditional_relative_jump.insert(0x20, !self.register.flag.z); //JR NZ
                conditional_relative_jump.insert(0x28, self.register.flag.z); //JR Z
                conditional_relative_jump.insert(0x30, !self.register.flag.c); //JR NC
                conditional_relative_jump.insert(0x38, self.register.flag.c); //JR C

                if *conditional_relative_jump.get(&opcode).unwrap() {
                    let offset = self.fetch_byte() as i8;
                    jump::relative_jump(&mut self.register.pc, offset);
                    self.cycles = 12;
                } else {
                    self.register.pc += 1;
                    self.cycles = 8;
                }
            }
            //JP
            0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA => {
                let mut conditional_jump: HashMap<u8, bool> = HashMap::new();
                conditional_jump.insert(0xC2, !self.register.flag.z); //JP NZ
                conditional_jump.insert(0xC3, true); //Always jump
                conditional_jump.insert(0xCA, self.register.flag.z); //JP Z
                conditional_jump.insert(0xD2, !self.register.flag.c); //JP NC
                conditional_jump.insert(0xDA, self.register.flag.c); //JP C

                if *conditional_jump.get(&opcode).unwrap() {
                    let address = self.fetch_word();
                    jump::jump_to(&mut self.register.pc, address);
                    self.cycles = 16;
                } else {
                    self.register.pc += 2;
                    self.cycles = 12;
                }
            }
            //JP HL
            0xE9 => {
                let address = self.register.get_hl();
                jump::jump_to(&mut self.register.pc, address);
                self.cycles = 4;
            }
            // CALL
            0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC => {
                let mut conditional_call: HashMap<u8, bool> = HashMap::new();
                conditional_call.insert(0xC4, !self.register.flag.z); //CALL NZ
                conditional_call.insert(0xCC, self.register.flag.z); //CALL Z
                conditional_call.insert(0xCD, true); //Always call
                conditional_call.insert(0xD4, !self.register.flag.c); //CALL NC
                conditional_call.insert(0xDC, self.register.flag.c); //CALL C

                if *conditional_call.get(&opcode).unwrap() {
                    let address = self.fetch_word();
                    jump::call(
                        &mut self.register.pc,
                        address,
                        &mut self.iommu.borrow_mut(),
                        &mut self.register.sp,
                    );
                    self.cycles = 24;
                } else {
                    self.register.pc += 2;
                    self.cycles = 12;
                }
            }

            // RET
            0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 => {
                let mut conditional_ret: HashMap<u8, bool> = HashMap::new();
                conditional_ret.insert(0xC0, !self.register.flag.z); //RET NZ
                conditional_ret.insert(0xC8, self.register.flag.z); //RET Z
                conditional_ret.insert(0xC9, true); //Always RET
                conditional_ret.insert(0xD0, !self.register.flag.c); //RET NC
                conditional_ret.insert(0xD8, self.register.flag.c); //RET C

                if *conditional_ret.get(&opcode).unwrap() {
                    jump::ret(
                        &mut self.register.pc,
                        &mut self.iommu.borrow_mut(),
                        &mut self.register.sp,
                    );
                    if opcode == 0xC9 {
                        self.cycles = 16;
                    } else {
                        self.cycles = 20;
                    }
                } else {
                    self.cycles = 8;
                }
            }

            //RETI
            0xD9 => {
                jump::ret(
                    &mut self.register.pc,
                    &mut self.iommu.borrow_mut(),
                    &mut self.register.sp,
                );
                // return and enable interrupts (IME=1)
                self.control.ime = true;
                self.cycles = 16;
            }

            //RST
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let mut reset_map: HashMap<u8, usize> = HashMap::new();
                reset_map.insert(0xC7, 0);
                reset_map.insert(0xCF, 1);
                reset_map.insert(0xD7, 2);
                reset_map.insert(0xDF, 3);
                reset_map.insert(0xE7, 4);
                reset_map.insert(0xEF, 5);
                reset_map.insert(0xF7, 6);
                reset_map.insert(0xFF, 7);

                let reset_index = *reset_map.get(&opcode).unwrap();
                jump::rst(
                    reset_index,
                    &mut self.register.pc,
                    &mut self.iommu.borrow_mut(),
                    &mut self.register.sp,
                );

                self.cycles = 16;
            }

            _ => panic!("Jump opcode [{:#02x?}] not supported!", opcode),
        }
    }

    fn cpu_control_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            0x00 => {
                //NOP
                self.cycles = 4;
            }
            0x10 => {
                //STOP
                self.cycles = 4;
            }
            0x37 => {
                cpu_control::scf(&mut self.register.flag);
                self.cycles = 4;
            }
            0x3F => {
                cpu_control::ccf(&mut self.register.flag);
                self.cycles = 4;
            }
            0x76 => {
                cpu_control::halt(&mut self.control);
                self.cycles = 4;
            }
            0xF3 => {
                cpu_control::di(&mut self.control);
                self.cycles = 4;
            }
            0xFB => {
                cpu_control::ei(&mut self.control);
                self.cycles = 4;
            }
            _ => panic!("CPU control opcode [{:#02x?}] not supported!", opcode),
        }
    }

    fn execute_cbprefixed_instruction(&mut self, opcode: u8) {
        if instructions::is_supported(opcode, &single_bit_operation::SINGLE_BIT_OPERATION_OPCODES) {
            self.single_bit_operation_dispatcher(opcode);
        } else if instructions::is_supported(
            opcode,
            &rotate_and_shift::ROTATE_SHIFT_OPERATION_OPCODES,
        ) {
            self.rotate_and_shift_operation_dispatcher(opcode);
        } else {
            self.debug_dump_regs();
            panic!("CBPrefixed Instruction [{:#02x?}] not supported!", opcode);
        }
    }

    fn execute(&mut self, opcode: u8) {
        if instructions::is_supported(opcode, &arithmetic_logic::ARITHMETIC_LOGIC_OPCODES) {
            self.arithmetic_logic_instruction_dispatcher(opcode);
        } else if instructions::is_supported(opcode, &load::LOAD_OPCODES) {
            self.load_instruction_dispatcher(opcode);
        } else if instructions::is_supported(
            opcode,
            &rotate_and_shift::ACC_ROTATE_SHIFT_OPERATION_OPCODES,
        ) {
            self.accumulator_rotate_and_shift_operation_for_dispatcher(opcode);
        } else if instructions::is_supported(opcode, &jump::JUMP_OPCODES) {
            self.jump_instruction_dispatcher(opcode);
        } else if instructions::is_supported(opcode, &cpu_control::CPU_CONTROL_OPCODES) {
            self.cpu_control_instruction_dispatcher(opcode);
        } else {
            self.debug_dump_regs();
            panic!("Instruction [{:#02x?}] not supported!", opcode);
        }
    }

    // ------------------------ DEBUG ------------------------
    fn debug_dump_processing_instruction(&self, opcode: u8) {
        print!("\x1b[93m*** Processing Opcode: [{:#02x?}]", opcode);
        if instructions::is_supported(opcode, &arithmetic_logic::ARITHMETIC_LOGIC_OPCODES) {
            print!("[Arithmetic]\x1b[0m\n");
        } else if instructions::is_supported(opcode, &load::LOAD_OPCODES) {
            print!("[Load]\x1b[0m\n");
        } else if instructions::is_supported(
            opcode,
            &rotate_and_shift::ACC_ROTATE_SHIFT_OPERATION_OPCODES,
        ) {
            print!("[Rotate and shift]\x1b[0m\n");
        } else if instructions::is_supported(opcode, &jump::JUMP_OPCODES) {
            print!("[Jump]\x1b[0m\n");
        } else if instructions::is_supported(opcode, &cpu_control::CPU_CONTROL_OPCODES) {
            print!("[Cpu control]\x1b[0m\n");
        } else if instructions::is_supported(
            opcode,
            &single_bit_operation::SINGLE_BIT_OPERATION_OPCODES,
        ) {
            print!("[0xCB][Bit operation]\x1b[0m\n");
        } else if instructions::is_supported(
            opcode,
            &rotate_and_shift::ROTATE_SHIFT_OPERATION_OPCODES,
        ) {
            print!("[0xCB][Rotate and shift]\x1b[0m\n");
        }
    }

    fn debug_dump_regs(&self) {
        let mem_byte_0 = self.iommu.borrow_mut().read_byte(self.register.pc);
        let mem_byte_1 = self
            .iommu
            .borrow_mut()
            .read_byte(self.register.pc.wrapping_add(1));
        let mem_byte_2 = self
            .iommu
            .borrow_mut()
            .read_byte(self.register.pc.wrapping_add(2));
        let mem_byte_3 = self
            .iommu
            .borrow_mut()
            .read_byte(self.register.pc.wrapping_add(3));
        println!(
            "A: {:#04x?} F: [z:{}, n:{}, h:{}, c:{}], BC: {:#06x?}, DE: {:#06x?}, HL: {:#06x?}, PC: {:#06x?}, SP: {:#06x?}, PCMEM: {:#04x?}, {:#04x?}, {:#04x?}, {:#04x?}",
            self.register.a,
            self.register.flag.z as u8,
            self.register.flag.n as u8,
            self.register.flag.h as u8,
            self.register.flag.c as u8,
            self.register.get_bc(),
            self.register.get_de(),
            self.register.get_hl(),
            self.register.pc,
            self.register.sp,
            mem_byte_0,
            mem_byte_1,
            mem_byte_2,
            mem_byte_3
        );
    }
}
