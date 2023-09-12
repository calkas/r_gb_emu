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
        let opcode = self.fetch_instruction();
        self.execute(opcode);
        self.register.pc += 1;
    }

    fn fetch_instruction(&self) -> u8 {
        self.memory[self.register.pc as usize]
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 |0x85 | 0x87 => {
                let val = instructions::arithmetic::get_reg_8bit_value(opcode, &self.register);
                instructions::arithmetic::add(&mut self.register, val, 0);
                self.cycle += 4;
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


}
