use super::cpu_data::Registers;

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

    pub fn process(&self) {
        let opcode = self.fetch_instruction();
        self.decode();
        self.execute();
    }

    fn fetch_instruction(&self) -> u8 {
        self.memory[self.register.pc as usize]
    }

    fn decode(&self) {}

    fn execute(&self) {}
}
