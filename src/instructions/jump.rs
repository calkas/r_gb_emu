use super::load::pop;
use super::load::push;
use crate::iommu::IOMMU;

/// # jump_to
/// jump to nn, PC=nn
/// jump to HL, PC=HL
fn jump_to(program_counter: &mut u16, new_address: u16) {
    *program_counter = new_address;
}

/// # conditional_jump
/// conditional jump if nz,z,nc,c flag is set
fn conditional_jump(should_jump: bool, program_counter: &mut u16, new_address: u16) {
    if should_jump {
        jump_to(program_counter, new_address);
    }
}

/// # relative_jump
/// relative jump to nn (PC=PC+8-bit signed)
fn relative_jump(program_counter: &mut u16, address_offset: i8) {
    *program_counter = ((*program_counter as u32 as i32) + (address_offset as i32)) as u16;
}

/// # conditional_relative_jump
/// conditional relative jump if nz,z,nc,c flag is set
fn conditional_relative_jump(should_jump: bool, program_counter: &mut u16, address_offset: i8) {
    if should_jump {
        relative_jump(program_counter, address_offset);
    }
}

/// # call
/// call to nn, SP=SP-2, (SP)=PC, PC=nn
fn call(program_counter: &mut u16, address: u16, stack: &mut IOMMU, reg_sp: &mut u16) {
    let next_pc = *program_counter + 2;
    push(stack, reg_sp, next_pc);
    jump_to(program_counter, address);
}

/// # conditional_call
/// conditional call if nz,z,nc,c flag is set
fn conditional_call(
    should_call: bool,
    program_counter: &mut u16,
    address: u16,
    stack: &mut IOMMU,
    reg_sp: &mut u16,
) {
    if should_call {
        call(program_counter, address, stack, reg_sp);
    }
}

/// # ret
/// return, PC=(SP), SP=SP+2
fn ret(program_counter: &mut u16, stack: &mut IOMMU, reg_sp: &mut u16) {
    let old_pc = pop(stack, reg_sp);
    *program_counter = old_pc;
}

/// # conditional_ret
/// conditional return if nz,z,nc,c flag is set
fn conditional_ret(
    should_return: bool,
    program_counter: &mut u16,
    stack: &mut IOMMU,
    reg_sp: &mut u16,
) {
    if should_return {
        ret(program_counter, stack, reg_sp);
    }
}

#[cfg(test)]
mod ut {
    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn jump_to_test() {
        let mut register = Registers::new();
        register.pc = 0xAAAA;

        jump_to(&mut register.pc, 0xBBCC);

        assert_eq!(0xBBCC, register.pc);
    }

    #[test]
    fn conditional_jump_test() {
        let mut register = Registers::new();
        register.pc = 0xAAAA;

        // flag true
        conditional_jump(true, &mut register.pc, 0xBBCC);
        assert_eq!(0xBBCC, register.pc);

        // flag false
        conditional_jump(false, &mut register.pc, 0xDDEE);
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
    fn conditional_relative_jump_test() {
        let mut register = Registers::new();
        register.pc = 1000;

        //flag true
        conditional_relative_jump(true, &mut register.pc, -100);
        assert_eq!(900, register.pc);

        //flag false
        conditional_relative_jump(false, &mut register.pc, -123);
        assert_eq!(900, register.pc);
    }

    #[test]
    fn call_and_return_test() {
        let mut register = Registers::new();
        const PROGRAM_COUNTER_POINT_TO_CALL_INSTR: u16 = 1234;
        register.pc = PROGRAM_COUNTER_POINT_TO_CALL_INSTR;
        register.sp = 0xFFFE;

        let mut iomm = IOMMU::new();

        call(&mut register.pc, 500, &mut iomm, &mut register.sp);
        assert_eq!(500, register.pc);

        ret(&mut register.pc, &mut iomm, &mut register.sp);
        assert_eq!(PROGRAM_COUNTER_POINT_TO_CALL_INSTR + 2, register.pc);
    }

    #[test]
    fn conditional_call_and_return_test() {
        let mut register = Registers::new();
        const PROGRAM_COUNTER_POINT_TO_CALL_INSTR: u16 = 1234;
        register.pc = PROGRAM_COUNTER_POINT_TO_CALL_INSTR;
        register.sp = 0xFFFE;

        let mut iomm = IOMMU::new();

        conditional_call(false, &mut register.pc, 500, &mut iomm, &mut register.sp);
        assert_eq!(PROGRAM_COUNTER_POINT_TO_CALL_INSTR, register.pc);

        conditional_call(true, &mut register.pc, 500, &mut iomm, &mut register.sp);
        assert_eq!(500, register.pc);

        conditional_ret(false, &mut register.pc, &mut iomm, &mut register.sp);
        assert_eq!(500, register.pc);

        conditional_ret(true, &mut register.pc, &mut iomm, &mut register.sp);
        assert_eq!(PROGRAM_COUNTER_POINT_TO_CALL_INSTR + 2, register.pc);
    }
}
