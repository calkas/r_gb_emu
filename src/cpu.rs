use super::cpu_data::Registers;
use super::iommu::IOMMU;
use crate::{
    instructions::{self, arithmetic_logic, load, single_bit_operation},
    iommu::{STACK_SIZE, WRAM_SIZE},
};
/// # DMG-CPU
/// 8-bit 8080-like Sharp CPU
pub struct Cpu {
    pub register: Registers,
    pub cycle: u32,
    pub iommu: IOMMU,
}
impl Cpu {
    pub fn new() -> Self {
        return Cpu {
            register: Registers::new(),
            cycle: 0,
            iommu: IOMMU::new(),
        };
    }

    pub fn load_program(&mut self, program: &[u8]) {
        //Temporary solution
        assert!(program.len() < WRAM_SIZE - STACK_SIZE);
        for (index, byte) in program.iter().enumerate() {
            self.iommu.write_byte(index as u16, *byte);
        }
        //Set stack pointer
        self.register.sp = 0xFFFE;
    }

    pub fn process(&mut self) {
        let opcode = self.fetch_byte();
        if self.is_prefix_instruction(opcode) {
            let opcode = self.fetch_byte();
            self.execute_prefix_instruction(opcode);
        } else {
            self.execute(opcode);
        }

        self.dump_regs();
    }

    fn is_prefix_instruction(&self, opcode: u8) -> bool {
        opcode == 0xCB
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.iommu.read_byte(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let low_byte = self.fetch_byte();
        let high_byte = self.fetch_byte();
        (high_byte as u16).rotate_left(8) | (low_byte as u16)
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
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0x86 => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::add(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycle += 8;
            }
            0xC6 => {
                let val = self.fetch_byte();
                arithmetic_logic::add(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycle += 8;
            }
            0xE8 => {
                let val = self.fetch_byte() as i8;
                arithmetic_logic::add_sp(&mut self.register.flag, &mut self.register.sp, val);
                self.cycle += 16;
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
                self.cycle += 4;
            }
            0xCE => {
                let val = self.fetch_byte();
                arithmetic_logic::adc(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0x8E => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::adc(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0x96 => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::sub(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycle += 8;
            }
            0xD6 => {
                let val = self.fetch_byte();
                arithmetic_logic::sub(&mut self.register.flag, &mut self.register.a, val, 0);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0x9E => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::sbc(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0xDE => {
                let val = self.fetch_byte();
                arithmetic_logic::sbc(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0xA6 => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::and(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0xE6 => {
                let val = self.fetch_byte();
                arithmetic_logic::and(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0xAE => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::xor(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0xEE => {
                let val = self.fetch_byte();
                arithmetic_logic::xor(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0xB6 => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::or(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0xF6 => {
                let val = self.fetch_byte();
                arithmetic_logic::or(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
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
                self.cycle += 4;
            }
            0xBE => {
                let val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::cp(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }
            0xFE => {
                let val = self.fetch_byte();
                arithmetic_logic::cp(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }

            // .::INC operation::.
            0x03 => {
                arithmetic_logic::inc_16(&mut self.register.b, &mut self.register.c);
                self.cycle += 8;
            }
            0x13 => {
                arithmetic_logic::inc_16(&mut self.register.d, &mut self.register.e);
                self.cycle += 8;
            }
            0x23 => {
                arithmetic_logic::inc_16(&mut self.register.h, &mut self.register.l);
                self.cycle += 8;
            }
            0x33 => {
                let mut sp_low: u8 = self.register.sp as u8;
                let mut sp_high: u8 = ((self.register.sp & 0xFF00) >> 8) as u8;
                arithmetic_logic::inc_16(&mut sp_high, &mut sp_low);

                self.register.sp = (sp_high as u16).rotate_left(8) | (sp_low as u16);
                self.cycle += 8;
            }
            0x04 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.b);
                self.cycle += 4;
            }
            0x0C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.c);
                self.cycle += 4;
            }
            0x14 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.d);
                self.cycle += 4;
            }
            0x1C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.e);
                self.cycle += 4;
            }
            0x24 => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.h);
                self.cycle += 4;
            }
            0x2C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.l);
                self.cycle += 4;
            }
            0x3C => {
                arithmetic_logic::inc(&mut self.register.flag, &mut self.register.a);
                self.cycle += 4;
            }
            0x34 => {
                let mut val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::inc(&mut self.register.flag, &mut val);
                self.iommu.write_byte(self.register.get_hl(), val);
                self.cycle += 12;
            }

            // .::DEC operation::.
            0x0B => {
                arithmetic_logic::dec_16(&mut self.register.b, &mut self.register.c);
                self.cycle += 8;
            }
            0x1B => {
                arithmetic_logic::dec_16(&mut self.register.d, &mut self.register.e);
                self.cycle += 8;
            }
            0x2B => {
                arithmetic_logic::dec_16(&mut self.register.h, &mut self.register.l);
                self.cycle += 8;
            }
            0x3B => {
                let mut sp_low: u8 = self.register.sp as u8;
                let mut sp_high: u8 = ((self.register.sp & 0xFF00) >> 8) as u8;
                arithmetic_logic::dec_16(&mut sp_high, &mut sp_low);

                self.register.sp = (sp_high as u16).rotate_left(8) | (sp_low as u16);
                self.cycle += 8;
            }
            0x05 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.b);
                self.cycle += 4;
            }
            0x0D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.c);
                self.cycle += 4;
            }
            0x15 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.d);
                self.cycle += 4;
            }
            0x1D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.e);
                self.cycle += 4;
            }
            0x25 => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.h);
                self.cycle += 4;
            }
            0x2D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.l);
                self.cycle += 4;
            }
            0x3D => {
                arithmetic_logic::dec(&mut self.register.flag, &mut self.register.a);
                self.cycle += 4;
            }
            0x35 => {
                let mut val = self.iommu.read_byte(self.register.get_hl());
                arithmetic_logic::dec(&mut self.register.flag, &mut val);
                self.iommu.write_byte(self.register.get_hl(), val);
                self.cycle += 12;
            }

            // .::DAA operation::.
            0x27 => {
                arithmetic_logic::daa(&mut self.register.flag, &mut self.register.a);
                self.cycle += 4;
            }

            // .::CPL operation::.
            0x2F => {
                arithmetic_logic::cpl(&mut self.register.flag, &mut self.register.a);
                self.cycle += 4;
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
                self.cycle += 12;
            }
            _ => panic!("arithmetic_logic opcode not supported"),
        }
    }

    fn load_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            0x01 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.b, &mut self.register.c, value);
                self.cycle += 12;
            }
            0x02 => {
                let value = self.iommu.read_byte(self.register.get_bc());
                load::ld(&mut self.register.a, value);
                self.cycle += 8;
            }
            0x06 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.b, value);
                self.cycle += 8;
            }
            0x08 => {
                let address = self.fetch_word();
                self.iommu.write_word(address, self.register.sp);
                self.cycle += 20;
            }
            0x0A => {
                let value = self.iommu.read_byte(self.register.get_bc());
                load::ld(&mut self.register.b, value);
                self.cycle += 8;
            }
            0x0E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.c, value);
                self.cycle += 8;
            }
            0x11 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.d, &mut self.register.e, value);
                self.cycle += 12;
            }
            0x12 => {
                let address = self.register.get_de();
                self.iommu.write_byte(address, self.register.a);
                self.cycle += 8;
            }
            0x16 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.d, value);
                self.cycle += 8;
            }
            0x1A => {
                let value = self.iommu.read_byte(self.register.get_de());
                load::ld(&mut self.register.a, value);
                self.cycle += 8;
            }
            0x1E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.e, value);
                self.cycle += 8;
            }
            0x21 => {
                let value = self.fetch_word();
                load::ld_16(&mut self.register.h, &mut self.register.l, value);
                self.cycle += 12;
            }
            0x22 => {
                let address = load::hli(&mut self.register.h, &mut self.register.l);
                self.iommu.write_byte(address, self.register.a);
                self.cycle += 8;
            }
            0x26 => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.h, value);
                self.cycle += 8;
            }
            0x2A => {
                let address = load::hli(&mut self.register.h, &mut self.register.l);
                self.register.a = self.iommu.read_byte(address);
                self.cycle += 8;
            }
            0x2E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.l, value);
                self.cycle += 8;
            }
            0x31 => {
                self.register.sp = self.fetch_word();
                self.cycle += 12;
            }
            0x32 => {
                let address = load::hld(&mut self.register.h, &mut self.register.l);
                self.iommu.write_byte(address, self.register.a);
                self.cycle += 8;
            }
            0x36 => {
                let value = self.fetch_byte();
                self.iommu.write_byte(self.register.get_hl(), value);
                self.cycle += 12;
            }
            0x3A => {
                let address = load::hld(&mut self.register.h, &mut self.register.l);
                self.register.a = self.iommu.read_byte(address);
                self.cycle += 8;
            }
            0x3E => {
                let value = self.fetch_byte();
                load::ld(&mut self.register.a, value);
                self.cycle += 8;
            }
            // LD B, r
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x47 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x47],
                    opcode,
                );
                load::ld(&mut self.register.b, register_value);
                self.cycle += 4;
            }
            0x46 => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.b, value);
                self.cycle += 8;
            }
            // LD C, r
            0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4F],
                    opcode,
                );
                load::ld(&mut self.register.c, register_value);
                self.cycle += 4;
            }
            0x4E => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.c, value);
                self.cycle += 8;
            }
            // LD D, r
            0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x57 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57],
                    opcode,
                );
                load::ld(&mut self.register.d, register_value);
                self.cycle += 4;
            }
            0x56 => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.d, value);
                self.cycle += 8;
            }
            // LD E, r
            0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5F],
                    opcode,
                );
                load::ld(&mut self.register.e, register_value);
                self.cycle += 4;
            }
            0x5E => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.e, value);
                self.cycle += 8;
            }

            // LD H, r
            0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x67 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67],
                    opcode,
                );
                load::ld(&mut self.register.h, register_value);
                self.cycle += 4;
            }
            0x66 => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.h, value);
                self.cycle += 8;
            }
            // LD L, r
            0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6F],
                    opcode,
                );
                load::ld(&mut self.register.h, register_value);
                self.cycle += 4;
            }
            0x6E => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.e, value);
                self.cycle += 8;
            }
            // LD(HL), reg
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77],
                    opcode,
                );
                let address = self.register.get_hl();
                self.iommu.write_byte(address, register_value);
                self.cycle += 8;
            }
            // LD A, r
            0x78 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7F => {
                let register_value = self.register.get_reg_value_from_opcode_range(
                    &[0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7F],
                    opcode,
                );
                load::ld(&mut self.register.a, register_value);
                self.cycle += 4;
            }
            0x7E => {
                let value = self.iommu.read_byte(self.register.get_hl());
                load::ld(&mut self.register.a, value);
                self.cycle += 8;
            }
            //write to io-port C
            0xE2 => {
                let port_address = load::calculate_address_for_io_port(self.register.c);
                self.iommu.write_byte(port_address, self.register.a);
                self.cycle += 8;
            }
            0xEA => {
                let address = self.fetch_word();
                self.iommu.write_byte(address, self.register.a);
                self.cycle += 16;
            }
            // read from io-port C
            0xF2 => {
                let port_address = load::calculate_address_for_io_port(self.register.c);
                self.register.a = self.iommu.read_byte(port_address);
                self.cycle += 8;
            }
            //0xF8 in ADD instructions
            0xF9 => {
                self.register.sp = self.register.get_hl();
                self.cycle += 8;
            }
            0xFA => {
                let address = self.fetch_word();
                self.register.a = self.iommu.read_byte(address);
                self.cycle += 16;
            }
            // PUSH
            0xC5 | 0xD5 | 0xE5 => {
                let reg_val = self
                    .register
                    .get_reg16_value_from_opcode_array(&[0xC5, 0xD5, 0xE5], opcode);
                load::push(&mut self.iommu, &mut self.register.sp, reg_val);
                self.cycle += 16;
            }
            //AF
            0xF5 => {
                let reg_val = self.register.get_af();
                load::push(&mut self.iommu, &mut self.register.sp, reg_val);
                self.cycle += 16;
            }
            // POP
            0xC1 => {
                let value = load::pop(&mut self.iommu, &mut self.register.sp);
                self.register.set_bc(value);
                self.cycle += 12;
            }
            0xD1 => {
                let value = load::pop(&mut self.iommu, &mut self.register.sp);
                self.register.set_de(value);
                self.cycle += 12;
            }
            0xE1 => {
                let value = load::pop(&mut self.iommu, &mut self.register.sp);
                self.register.set_hl(value);
                self.cycle += 12;
            }
            0xF1 => {
                let value = load::pop(&mut self.iommu, &mut self.register.sp);
                self.register.set_af(value);
                self.cycle += 12;
            }
            _ => panic!("load opcode not supported"),
        }
    }

    fn rotate_and_shift_operation_dispatcher(&mut self, opcode: u8) {
        match opcode {
            _ => panic!("rotate and shift opcode not supported"),
        }
    }

    fn single_bit_operation_dispatcher(&mut self, opcode: u8) {
        match opcode {
            0x40 => {}
            _ => panic!("single bit opcode not supported"),
        }
    }

    fn execute_prefix_instruction(&mut self, opcode: u8) {}

    fn execute(&mut self, opcode: u8) {
        if instructions::is_supported(opcode, &arithmetic_logic::ARITHMETIC_LOGIC_OPCODES) {
            self.arithmetic_logic_instruction_dispatcher(opcode);
        } else if instructions::is_supported(opcode, &load::LOAD_OPCODES) {
            self.load_instruction_dispatcher(opcode);
        } else if instructions::is_supported(
            opcode,
            &single_bit_operation::SINGLE_BIT_OPERATION_OPCODES,
        ) {
            self.single_bit_operation_dispatcher(opcode);
        } else {
            panic!("Instruction not supported!");
        }
    }
    fn dump_regs(&self) {
        println!(
            "A = {}\nF.z = {}, F.n = {}, F.h = {}, F.c = {}\nB = {}\nC = {}\nD = {}\nE = {}\nH = {}\nL = {}",
            self.register.a,
            self.register.flag.z as u8,
            self.register.flag.n as u8,
            self.register.flag.h as u8,
            self.register.flag.c as u8,
            self.register.b,
            self.register.c,
            self.register.d,
            self.register.e,
            self.register.h,
            self.register.l
        );
    }
}

#[cfg(test)]
mod cpu_ut {

    //use super::*;
    #[test]
    fn arithmetic_logic_opcode_check() {}
}
