use super::cpu_data::Registers;
use crate::instructions::arithmetic_logic;

pub struct Cpu {
    pub register: Registers,
    pub cycle: u32,
    pub memory: [u8; 0xFFFF],
    pub stack: [u8; 0xFFFF], //temporary solution
}
impl Cpu {
    pub fn new() -> Self {
        return Cpu {
            register: Registers::new(),
            cycle: 0,
            memory: [0xFF; 0xFFFF],
            stack: [0xFF; 0xFFFF],
        };
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (index, byte) in program.iter().enumerate() {
            self.memory[index] = *byte;
        }
    }

    pub fn process(&mut self) {
        let opcode = self.fetch_byte();
        self.execute(opcode);
        self.dump_regs();
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.register.pc);
        self.register.pc = self.register.pc.wrapping_add(1);
        byte
    }

    fn get_reg_value_from_opcode_range(&self, opcode_array: &[u8], opcode: u8) -> u8 {
        assert!(opcode_array.len() == 7);
        let mut reg_id: usize = 0xFF;
        for (id, element) in opcode_array.iter().enumerate() {
            if opcode == *element {
                reg_id = id;
            }
        }

        match reg_id {
            0 => self.register.b,
            1 => self.register.c,
            2 => self.register.d,
            3 => self.register.e,
            4 => self.register.h,
            5 => self.register.l,
            6 => self.register.a,
            _ => panic!("opcode does not exist in looking array"),
        }
    }

    fn get_reg16_value_from_opcode_array(&self, opcode_array: &[u8], opcode: u8) -> u16 {
        assert!(opcode_array.len() == 4);
        let mut reg_id: usize = 0xFF;
        for (id, element) in opcode_array.iter().enumerate() {
            if opcode == *element {
                reg_id = id;
            }
        }

        match reg_id {
            0 => self.register.get_bc(),
            1 => self.register.get_de(),
            2 => self.register.get_hl(),
            3 => self.register.sp,
            _ => panic!("opcode does not exist in looking array"),
        }
    }

    fn arithmetic_logic_instruction_dispatcher(&mut self, opcode: u8) {
        match opcode {
            // .::ADD operation::.
            0x09 | 0x19 | 0x29 | 0x39 => {
                let value =
                    self.get_reg16_value_from_opcode_array(&[0x09, 0x19, 0x29, 0x39], opcode);
                arithmetic_logic::add_hl(
                    &mut self.register.flag,
                    &mut self.register.h,
                    &mut self.register.l,
                    value,
                );
                self.cycle += 8;
            }
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x87 => {
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
                arithmetic_logic::adc(&mut self.register.flag, &mut self.register.a, val);
                self.cycle += 8;
            }

            // .::SUB operation::.
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x97 => {
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let register_value = self.get_reg_value_from_opcode_range(
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
                let val = self.read_byte(self.register.get_hl());
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
                let mut val = self.read_byte(self.register.get_hl());
                arithmetic_logic::inc(&mut self.register.flag, &mut val);
                self.write_byte(self.register.get_hl(), val);
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
                let mut val = self.read_byte(self.register.get_hl());
                arithmetic_logic::dec(&mut self.register.flag, &mut val);
                self.write_byte(self.register.get_hl(), val);
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

    fn load_instruction_dispatcher(&mut self, opcode: u8) {}

    fn execute(&mut self, opcode: u8) {
        if arithmetic_logic::is_supported_instruction(opcode) {
            self.arithmetic_logic_instruction_dispatcher(opcode);
        } else {
            panic!("Instruction not supported!");
        }
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
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

    use super::*;
    use crate::instructions::arithmetic_logic;
    #[test]
    fn arithmetic_logic_opcode_check() {}
}
