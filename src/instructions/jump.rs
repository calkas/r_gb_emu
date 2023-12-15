use super::load::pop;
use super::load::push;
use crate::iommu::IOMMU;

pub static JUMP_OPCODES: [u8; 30] = [
    0x18, 0x20, 0x28, 0x30, 0x38, 0xC0, 0xC2, 0xC3, 0xC4, 0xC7, 0xC8, 0xC9, 0xCA, 0xCC, 0xCD, 0xCF,
    0xD0, 0xD2, 0xD4, 0xD7, 0xD8, 0xD9, 0xDA, 0xDC, 0xDF, 0xE7, 0xE9, 0xEF, 0xF7, 0xFF,
];
pub static RESET_VECTOR_ADDRESS: [u16; 8] = [
    0x0000, 0x0008, 0x0010, 0x0018, 0x0020, 0x0028, 0x0030, 0x0038,
];

/// # jump_to
/// jump to nn, PC=nn
/// jump to HL, PC=HL
pub fn jump_to(program_counter: &mut u16, new_address: u16) {
    *program_counter = new_address;
}

/// # relative_jump
/// relative jump to nn (PC=PC+8-bit signed)
pub fn relative_jump(program_counter: &mut u16, address_offset: i8) {
    *program_counter = ((*program_counter as u32 as i32) + (address_offset as i32)) as u16;
}

/// # call
/// call to nn, SP=SP-2, (SP)=PC, PC=nn
pub fn call(program_counter: &mut u16, address: u16, stack: &mut IOMMU, reg_sp: &mut u16) {
    let next_pc = *program_counter + 2;
    push(stack, reg_sp, next_pc);
    jump_to(program_counter, address);
}

/// # ret
/// return, PC=(SP), SP=SP+2
pub fn ret(program_counter: &mut u16, stack: &mut IOMMU, reg_sp: &mut u16) {
    let old_pc = pop(stack, reg_sp);
    *program_counter = old_pc;
}

/// # rst
/// reset, call to 0x0000, 0x0008, 0x0010, 0x0018, 0x0020, 0x0028, 0x0030, 0x0038
pub fn rst(rst_index: usize, program_counter: &mut u16, stack: &mut IOMMU, reg_sp: &mut u16) {
    push(stack, reg_sp, *program_counter);
    jump_to(program_counter, RESET_VECTOR_ADDRESS[rst_index]);
}

#[cfg(test)]
mod ut {
    use super::*;
    use crate::{cpu_data::Registers, peripheral::cartridge::Cartridge};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn jump_to_test() {
        let mut register = Registers::new();
        register.pc = 0xAAAA;

        jump_to(&mut register.pc, 0xBBCC);

        assert_eq!(0xBBCC, register.pc);
    }

    #[test]
    fn relative_jump_test() {
        let mut register = Registers::new();
        register.pc = 1234;

        relative_jump(&mut register.pc, -100);
        assert_eq!(1134, register.pc);

        relative_jump(&mut register.pc, 120);
        assert_eq!(1254, register.pc);
    }

    #[test]
    fn call_and_return_test() {
        let mut register = Registers::new();
        const PROGRAM_COUNTER_POINT_TO_CALL_INSTR: u16 = 1234;
        register.pc = PROGRAM_COUNTER_POINT_TO_CALL_INSTR;
        register.sp = 0xFFFE;

        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let mut iommu = IOMMU::new(cartridge.clone());

        call(&mut register.pc, 500, &mut iommu, &mut register.sp);
        assert_eq!(500, register.pc);

        ret(&mut register.pc, &mut iommu, &mut register.sp);
        assert_eq!(PROGRAM_COUNTER_POINT_TO_CALL_INSTR + 2, register.pc);
    }

    #[test]
    fn rst_test() {
        let mut register = Registers::new();
        register.pc = 0xAAAA;
        register.sp = 0xFFFE;

        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        let mut iommu = IOMMU::new(cartridge.clone());

        for index in 0..RESET_VECTOR_ADDRESS.len() {
            rst(index, &mut register.pc, &mut iommu, &mut register.sp);
            assert_eq!(RESET_VECTOR_ADDRESS[index], register.pc);
        }
    }
}
