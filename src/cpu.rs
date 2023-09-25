use super::cpu_data::Registers;
use crate::instructions;

pub struct Cpu {
    pub register: Registers,
    pub cycle: u32,
    pub memory: [u8; 0xFFFF],
}
impl Cpu {
    pub fn new() -> Self {
        return Cpu {
            register: Registers::new(),
            cycle: 0,
            memory: [0xFF; 0xFFFF],
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
        self.register.pc += 1;
        byte
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            // .::ADD operation::.
            0x09 | 0x19 | 0x29 | 0x39 => {
                let val = instructions::arithmetic::get_reg_16bit_value(opcode, &self.register);
                instructions::arithmetic::add_hl(&mut self.register, val);
                self.cycle += 8;
            }
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x87 => {
                let val = instructions::arithmetic::get_reg_8bit_value(opcode, &self.register);
                instructions::arithmetic::add(&mut self.register, val, 0);
                self.cycle += 4;
            }
            0x86 => {
                let val = self.read_byte(self.register.get_hl());
                instructions::arithmetic::add(&mut self.register, val, 0);
                self.cycle += 8;
            }
            0xC6 => {
                let val = self.fetch_byte();
                instructions::arithmetic::add(&mut self.register, val, 0);
                self.cycle += 8;
            }
            0xE8 => {
                let val = self.fetch_byte() as i8;
                instructions::arithmetic::add_sp(&mut self.register, val);
                self.cycle += 16;
            }
            // .::ADC operation::.
            0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8F => {
                let val = instructions::arithmetic::get_reg_8bit_value(opcode, &self.register);
                instructions::arithmetic::adc(&mut self.register, val);
                self.cycle += 4;
            }
            0xCE => {
                let val = self.fetch_byte();
                instructions::arithmetic::adc(&mut self.register, val);
                self.cycle += 8;
            }
            0x8E => {
                let val = self.read_byte(self.register.get_hl());
                instructions::arithmetic::adc(&mut self.register, val);
                self.cycle += 8;
            }
            // .::SUB operation::.
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x97 => {
                let val = instructions::arithmetic::get_reg_8bit_value(opcode, &self.register);
                instructions::arithmetic::sub(&mut self.register, val, 0);
                self.cycle += 4;
            }
            0x96 => {
                let val = self.read_byte(self.register.get_hl());
                instructions::arithmetic::sub(&mut self.register, val, 0);
                self.cycle += 8;
            }
            0xD6 => {
                let val = self.fetch_byte();
                instructions::arithmetic::sub(&mut self.register, val, 0);
                self.cycle += 8;
            }

            // .::SBC operation::.
            0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9F => {
                let val = instructions::arithmetic::get_reg_8bit_value(opcode, &self.register);
                instructions::arithmetic::sbc(&mut self.register, val);
                self.cycle += 4;
            }
            0x9E => {
                let val = self.read_byte(self.register.get_hl());
                instructions::arithmetic::sbc(&mut self.register, val);
                self.cycle += 8;
            }
            0xDE => {
                let val = self.fetch_byte();
                instructions::arithmetic::sbc(&mut self.register, val);
                self.cycle += 8;
            }
            _ => println!("Nothing"),
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
            "A = {}\nF = {}\nB = {}\nC = {}\nD = {}E = {}\nH = {}\nL = {}",
            self.register.a,
            self.register.f,
            self.register.b,
            self.register.c,
            self.register.d,
            self.register.e,
            self.register.h,
            self.register.l
        );
    }
}
