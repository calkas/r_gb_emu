use crate::constants::gb_memory_map::address;
use crate::iommu::IOMMU;

pub static LOAD_OPCODES: [u8; 99] = [
    0x01, 0x02, 0x06, 0x08, 0x0A, 0x0E, 0x11, 0x12, 0x16, 0x1A, 0x1E, 0x21, 0x22, 0x26, 0x2A, 0x2E,
    0x31, 0x32, 0x36, 0x3A, 0x3E, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A,
    0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A,
    0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A,
    0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77, 0x78, 0x79, 0x7A, 0x7B,
    0x7C, 0x7D, 0x7E, 0x7F, 0xE0, 0xE2, 0xEA, 0xF2, 0xF9, 0xFA, 0xC5, 0xD5, 0xE5, 0xF5, 0xC1, 0xD1,
    0xE1, 0xF0, 0xF1,
];
/// # ld
/// LD (load)
pub fn ld(out_reg: &mut u8, value: u8) {
    *out_reg = value;
}

/// # ld_16
/// LD (load) - load 16-bit value
pub fn ld_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8, value: u16) {
    *reg_high_byte = ((value & 0xFF00).rotate_right(8)) as u8;
    *reg_low_byte = (value & 0x00FF) as u8;
}

/// # calculate_address_for_io_port
/// Used for read from io-port n (memory FF00+n)
pub fn calculate_address_for_io_port(val: u8) -> u16 {
    0xFF00 | val as u16
}

/// # hli
/// Used to handle LDI instruction
pub fn hli(reg_h: &mut u8, reg_l: &mut u8) -> u16 {
    let old_hl_val = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
    let res = old_hl_val.wrapping_add(1);

    *reg_h = (res >> 8) as u8;
    *reg_l = (res & 0x00FF) as u8;

    old_hl_val
}
/// # hld
/// Used to handle LDD instruction
pub fn hld(reg_h: &mut u8, reg_l: &mut u8) -> u16 {
    let old_hl_val = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
    let res = old_hl_val.wrapping_sub(1);

    *reg_h = (res >> 8) as u8;
    *reg_l = (res & 0x00FF) as u8;

    old_hl_val
}

/// # push
/// PUSH on stack
pub fn push(stack: &mut IOMMU, reg_sp: &mut u16, value: u16) {
    *reg_sp = reg_sp.wrapping_sub(2);
    if !address::HIGH_RAM.contains(reg_sp) {
        panic!("PUSH operation: Stack overflow SP = {:#06x?}", *reg_sp);
    }
    stack.write_word(*reg_sp, value)
}

/// # pop
/// POP from stack
pub fn pop(stack: &mut IOMMU, reg_sp: &mut u16) -> u16 {
    if !address::HIGH_RAM.contains(reg_sp) {
        panic!("POP operation: Stack overflow SP = {:#06x?}", *reg_sp);
    }
    let value = stack.read_word(*reg_sp);
    *reg_sp = reg_sp.wrapping_add(2);
    value
}

#[cfg(test)]
mod ut {
    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn ld_test() {
        let mut register = Registers::new();
        register.b = 55;
        register.c = 69;

        ld(&mut register.b, register.c);
        assert_eq!(69, register.b);

        ld(&mut register.b, 33);
        assert_eq!(33, register.b);
    }

    #[test]
    fn ld16_test() {
        let mut register = Registers::new();
        register.b = 0x45;
        register.c = 0x33;

        register.d = 0x66;
        register.e = 0x66;

        let exp_val_de = register.get_de();

        ld_16(&mut register.b, &mut register.c, exp_val_de);
        assert_eq!(exp_val_de, register.get_bc());
    }
    #[test]
    fn ldi_test() {
        let mut register = Registers::new();

        register.h = 0x30;
        register.l = 0x20;

        let ret_val = hli(&mut register.h, &mut register.l);
        assert_eq!(0x3020, ret_val);
        assert_eq!(0x30, register.h);
        assert_eq!(0x21, register.l);

        register.h = 0xFF;
        register.l = 0xFF;

        let ret_val = hli(&mut register.h, &mut register.l);
        assert_eq!(0xFFFF, ret_val);
        assert_eq!(0x0, register.h);
        assert_eq!(0x0, register.l);
    }
    #[test]
    fn ldd_test() {
        let mut register = Registers::new();

        register.h = 0x30;
        register.l = 0x20;

        let ret_val = hld(&mut register.h, &mut register.l);
        assert_eq!(0x3020, ret_val);
        assert_eq!(0x30, register.h);
        assert_eq!(0x1F, register.l);

        register.h = 0;
        register.l = 0;

        let ret_val = hld(&mut register.h, &mut register.l);
        assert_eq!(0, ret_val);
        assert_eq!(0xFF, register.h);
        assert_eq!(0xFF, register.l);
    }

    #[test]
    fn stack_test() {
        let mut register = Registers::new();
        let mut iommu = IOMMU::new();
        register.sp = *address::HIGH_RAM.end();

        let mut exp_sp_value = register.sp - 2;

        push(&mut iommu, &mut register.sp, 0x8001);
        assert_eq!(exp_sp_value, register.sp);
        push(&mut iommu, &mut register.sp, 0x8002);
        exp_sp_value -= 2;
        assert_eq!(exp_sp_value, register.sp);
        push(&mut iommu, &mut register.sp, 0x8003);
        exp_sp_value -= 2;
        assert_eq!(exp_sp_value, register.sp);

        assert_eq!(0x8003, pop(&mut iommu, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
        assert_eq!(0x8002, pop(&mut iommu, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
        assert_eq!(0x8001, pop(&mut iommu, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
    }
}
