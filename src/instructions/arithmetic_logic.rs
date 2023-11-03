#![allow(dead_code)]

use crate::cpu_data::FlagsRegister;

pub static ARITHMETIC_LOGIC_OPCODES: [u8; 104] = [
    0x03, 0x04, 0x05, 0x09, 0x0b, 0x0c, 0x0d, 0x13, 0x14, 0x15, 0x19, 0x1b, 0x1c, 0x1d, 0x23, 0x24,
    0x25, 0x27, 0x29, 0x2b, 0x2c, 0x2d, 0x2f, 0x33, 0x34, 0x35, 0x39, 0x3b, 0x3c, 0x3d, 0x80, 0x81,
    0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91,
    0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1,
    0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1,
    0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc6, 0xce,
    0xd6, 0xde, 0xe6, 0xe8, 0xee, 0xf6, 0xf8, 0xfe,
];

fn half_carry_on_addition(a: u8, b: u8) -> bool {
    (((a & 0x0F) + (b & 0x0F)) & 0xF0) == 0x10
}

fn half_carry_on_addition_16(a: u16, b: u16) -> bool {
    (((a & 0x0FFF) + (b & 0x0FFF)) & 0x100) == 0x100
}

fn half_carry_on_subtration(a: u8, b: u8) -> bool {
    (a & 0x0F) < (b & 0x0F)
}

fn half_carry_on_subtration_16(a: u16, b: u16) -> bool {
    (a & 0x00FF) < (b & 0x00FF)
}
/// # add
/// ADD operation
pub fn add(flag: &mut FlagsRegister, acc: &mut u8, value: u8, carry_value: u8) {
    let (new_value, did_overflow) = acc.overflowing_add(value + carry_value);
    flag.c = false;
    flag.z = false;
    flag.h = false;
    flag.n = false;

    if did_overflow {
        flag.c = true;
    }

    if new_value == 0 {
        flag.z = true;
    }

    if half_carry_on_addition(*acc, value + carry_value) {
        flag.h = true;
    }
    *acc = new_value;
}
/// # add_hl
/// ADDHL (add to HL) - just like ADD except that the target is added to the HL register
pub fn add_hl(flag: &mut FlagsRegister, reg_h: &mut u8, reg_l: &mut u8, reg_16_value: u16) {
    let hl_reg_value = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
    let (new_value, did_overflow) = hl_reg_value.overflowing_add(reg_16_value);

    flag.h = false;
    flag.c = false;
    flag.n = false;

    if did_overflow {
        flag.c = true;
    }

    if half_carry_on_addition_16(hl_reg_value, reg_16_value) {
        flag.h = true;
    }

    *reg_h = ((new_value & 0xFF00).rotate_right(8)) as u8;
    *reg_l = (new_value & 0x00FF) as u8;
}

/// # add_sp
/// perform add operation value to sp register
pub fn add_sp(flag: &mut FlagsRegister, reg_sp: &mut u16, value: i8) {
    let coverted_value = value as i16 as u16;
    let (new_value, did_overflow) = reg_sp.overflowing_add(coverted_value);

    flag.h = false;
    flag.c = false;
    flag.z = false;
    flag.n = false;

    if did_overflow {
        flag.c = true;
    }

    if half_carry_on_addition_16(*reg_sp, coverted_value) {
        flag.h = true;
    }

    *reg_sp = new_value;
}

/// # adc
/// ADC (add with carry) - just like ADD except that the value of the carry flag is also added to the number
pub fn adc(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    let carry_val = if flag.c { 1 } else { 0 };
    add(flag, acc, value, carry_val);
}

/// # sub
/// SUB (subtract) - subtract the value stored in a specific register with the value in the A register
pub fn sub(flag: &mut FlagsRegister, acc: &mut u8, value: u8, carry_value: u8) {
    let (new_value, did_overflow) = acc.overflowing_sub(value + carry_value);

    flag.c = false;
    flag.z = false;
    flag.h = false;
    flag.n = true;

    if did_overflow {
        flag.c = true;
    }

    if new_value == 0 {
        flag.z = true;
    }

    if half_carry_on_subtration(*acc, value + carry_value) {
        flag.h = true;
    }
    *acc = new_value;
}

/// # sbc
/// SBC (subtract with carry) - just like ADD except that the value of the carry flag is also subtracted from the number
pub fn sbc(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    let carry_val = if flag.c { 1 } else { 0 };
    sub(flag, acc, value, carry_val);
}

/// # and
/// AND (logical and) - do a bitwise and on the value in a specific register and the value in the A register
pub fn and(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    *acc &= value;
    flag.z = false;
    flag.n = false;
    flag.h = true;
    flag.c = false;

    if *acc == 0 {
        flag.z = true;
    }
}

/// # xor
/// XOR (logical xor) - do a bitwise xor on the value in a specific register and the value in the A register
pub fn xor(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    *acc ^= value;
    flag.z = false;
    flag.n = false;
    flag.h = false;
    flag.c = false;

    if *acc == 0 {
        flag.z = true;
    }
}

/// # or
/// OR (logical or) - do a bitwise or on the value in a specific register and the value in the A register
pub fn or(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    *acc |= value;
    flag.z = false;
    flag.n = false;
    flag.h = false;
    flag.c = false;

    if *acc == 0 {
        flag.z = true;
    }
}

/// # cp
/// CP (compare) - just like SUB except the result of the subtraction is not stored back into A
pub fn cp(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
    let saved_acc = *acc;
    sub(flag, acc, value, 0);
    *acc = saved_acc;
}

/// # inc
/// INC (increment) - increment the value in a specific register by 1
pub fn inc(flag: &mut FlagsRegister, reg_or_data: &mut u8) {
    let result = reg_or_data.wrapping_add(1);

    flag.z = false;
    flag.h = false;
    flag.n = false;

    if result == 0 {
        flag.z = true;
    }

    if half_carry_on_addition(*reg_or_data, 1) {
        flag.h = true;
    }

    *reg_or_data = result;
}

/// # inc_16
/// INC (increment) - increment the 16-bit value in a specific register by 1
pub fn inc_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8) {
    let mut reg_value = (*reg_high_byte as u16).rotate_left(8) | (*reg_low_byte as u16);
    reg_value = reg_value.wrapping_add(1);

    *reg_high_byte = ((reg_value & 0xFF00).rotate_right(8)) as u8;
    *reg_low_byte = (reg_value & 0x00FF) as u8;
}

/// # dec
/// DEC (decrement) - decrement the value in a specific register by 1
pub fn dec(flag: &mut FlagsRegister, reg_or_data: &mut u8) {
    let result = reg_or_data.wrapping_sub(1);

    flag.z = false;
    flag.h = false;
    flag.n = true;

    if result == 0 {
        flag.z = true;
    }

    if half_carry_on_subtration(*reg_or_data, 1) {
        flag.h = true;
    }

    *reg_or_data = result;
}

/// # dec_16
/// DEC (decrement) - decrement the 16-bit value in a specific register by 1
pub fn dec_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8) {
    let mut reg_value = (*reg_high_byte as u16).rotate_left(8) | (*reg_low_byte as u16);
    reg_value = reg_value.wrapping_sub(1);

    *reg_high_byte = ((reg_value & 0xFF00).rotate_right(8)) as u8;
    *reg_low_byte = (reg_value & 0x00FF) as u8;
}

/// # daa
/// DAA - decimal adjust A
pub fn daa(flag: &mut FlagsRegister, acc: &mut u8) {
    let mut a = *acc;

    let mut correction: u8 = if flag.c { 0x60 } else { 0x00 };

    if flag.h {
        correction |= 0x06;
    }

    if !flag.h {
        if a & 0x0F > 0x09 {
            correction |= 0x06;
        };
        if a > 0x99 {
            correction |= 0x60;
        };
        a = a.wrapping_add(correction);
    } else {
        a = a.wrapping_sub(correction);
    }

    flag.z = false;
    flag.c = false;
    flag.h = false;

    if a == 0 {
        flag.z = true;
    }

    if correction >= 0x60 {
        flag.c = true;
    }
    *acc = a;
}

/// # cpl
/// CPL (complement) - toggle every bit of the A register
pub fn cpl(flag: &mut FlagsRegister, acc: &mut u8) {
    *acc ^= 0xFF;
    flag.h = true;
    flag.n = true;
}

/// # ld_hl
/// HL = SP +/- dd ; dd is 8-bit signed number
pub fn ld_hl(flag: &mut FlagsRegister, reg_h: &mut u8, reg_l: &mut u8, sp_reg: u16, value: i8) {
    let reg_hl = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);

    let coverted_value = value as i16 as u16;
    let (new_value, did_overflow) = reg_hl.overflowing_add(coverted_value + sp_reg);

    flag.h = false;
    flag.c = false;
    flag.z = false;
    flag.n = false;

    if did_overflow {
        flag.c = true;
    }

    if half_carry_on_addition_16(reg_hl, coverted_value + sp_reg) {
        flag.h = true;
    }

    *reg_h = ((new_value & 0xFF00).rotate_right(8)) as u8;
    *reg_l = (new_value & 0x00FF) as u8;
}

#[cfg(test)]
mod ut {
    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn add_half_carry_flag_test() {
        let mut register = Registers::new();
        register.a = 0x6C;
        register.flag.n = true;

        add(&mut register.flag, &mut register.a, 0x2E, 0);

        assert_eq!(0x9A, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn add_carry_half_carry_and_zero_flag_test() {
        let mut register = Registers::new();
        register.a = 0xFF;
        register.flag.n = true;

        add(&mut register.flag, &mut register.a, 1, 0);

        assert_eq!(0x00, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == true);
        assert!(register.flag.c == true);
        assert!(register.flag.n == false);
    }

    #[test]
    fn adc_carry_flag_set_test() {
        let mut register = Registers::new();
        register.a = 0x3D;
        register.flag.n = true;
        register.flag.c = true;

        adc(&mut register.flag, &mut register.a, 0x42);

        assert_eq!(0x80, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn add_hl_half_carry_test() {
        let mut register = Registers::new();
        register.b = 0x01;
        register.c = 0x80;

        register.h = 0x01;
        register.l = 0x80;

        register.flag.n = true;

        let bc_reg_val = register.get_bc();

        add_hl(
            &mut register.flag,
            &mut register.h,
            &mut register.l,
            bc_reg_val,
        );

        assert_eq!(0x300, register.get_hl());
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn add_sp_with_carry_test() {
        let mut register = Registers::new();
        register.sp = 0x81;

        register.flag.n = true;
        register.flag.z = true;

        add_sp(&mut register.flag, &mut register.sp, -1);

        assert_eq!(0x80, register.sp);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == true);
        assert!(register.flag.n == false);
    }

    #[test]
    fn sub_the_same_value_test() {
        let mut register = Registers::new();
        register.a = 0x3E;

        sub(&mut register.flag, &mut register.a, 0x3E, 0);

        assert_eq!(0, register.a);
        assert!(register.flag.h == false);
        assert!(register.flag.z == true);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }
    #[test]
    fn sub_overflow_test() {
        let mut register = Registers::new();
        register.a = 16;

        sub(&mut register.flag, &mut register.a, 18, 0);

        assert_eq!(254, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == true);
        assert!(register.flag.n == true);
    }

    #[test]
    fn sbc_carry_flag_set_test() {
        let mut register = Registers::new();
        register.a = 77;
        register.flag.c = true;

        sbc(&mut register.flag, &mut register.a, 7);

        assert_eq!(69, register.a);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }

    #[test]
    fn and_test() {
        let mut register = Registers::new();
        register.flag.c = true;
        register.flag.n = true;
        register.a = 0xFC;

        and(&mut register.flag, &mut register.a, 0x0F);

        assert_eq!(0xC, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }
    #[test]
    fn xor_test() {
        let mut register = Registers::new();
        register.flag.c = true;
        register.flag.n = true;
        register.flag.h = true;
        register.a = 0xFC;

        xor(&mut register.flag, &mut register.a, 0xAC);

        assert_eq!(0x50, register.a);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }
    #[test]
    fn or_test() {
        let mut register = Registers::new();
        register.flag.c = true;
        register.flag.n = true;
        register.flag.h = true;
        register.a = 0x8D;

        or(&mut register.flag, &mut register.a, 0xA6);

        assert_eq!(0xAF, register.a);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn cp_test() {
        let mut register = Registers::new();
        register.a = 0x3E;

        cp(&mut register.flag, &mut register.a, 0x3E);

        assert_eq!(0x3E, register.a);
        assert!(register.flag.h == false); // false for z80 for 8080 true
        assert!(register.flag.z == true);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }

    #[test]
    fn inc_overflow_test() {
        let mut register = Registers::new();
        register.flag.n = true;
        register.b = 0xFF;

        inc(&mut register.flag, &mut register.b);

        assert_eq!(0, register.b);
        assert!(register.flag.h == true);
        assert!(register.flag.z == true);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);

        let mut val: u8 = 99;

        inc(&mut register.flag, &mut val);

        assert_eq!(100, val);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn inc16_test() {
        let mut register = Registers::new();

        register.b = 0x01;
        register.c = 0x10;

        inc_16(&mut register.b, &mut register.c);

        assert_eq!(0x111, register.get_bc());
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);

        //INC sp
        register.sp = 0xAABB;

        let mut sp_low: u8 = register.sp as u8;
        let mut sp_high: u8 = ((register.sp & 0xFF00) >> 8) as u8;
        inc_16(&mut sp_high, &mut sp_low);

        let sp_update = (sp_high as u16).rotate_left(8) | (sp_low as u16);
        register.sp = sp_update;
        assert_eq!(0xAABC, sp_update);
        assert_eq!(0xAABC, register.sp);
    }

    #[test]
    fn dec_overflow_test() {
        let mut register = Registers::new();
        register.flag.n = true;
        register.b = 0;

        dec(&mut register.flag, &mut register.b);

        assert_eq!(255, register.b);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);

        let mut val: u8 = 99;

        dec(&mut register.flag, &mut val);

        assert_eq!(98, val);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }

    #[test]
    fn dec16_test() {
        let mut register = Registers::new();

        register.b = 0x01;
        register.c = 0x10;

        dec_16(&mut register.b, &mut register.c);

        assert_eq!(0x10F, register.get_bc());
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);
    }

    #[test]
    fn daa_test() {
        let mut register = Registers::new();

        //Performing decimal addition
        //    85  1000 0101   0x85
        // +  36  0011 0110   0x36
        // = 121 BCD

        //1) First add 0x85 + 0x36
        register.a = 0x85;

        add(&mut register.flag, &mut register.a, 0x36, 0);

        assert_eq!(0xBB, register.a);

        //2) Perform daa correction
        daa(&mut register.flag, &mut register.a);

        assert_eq!(0x21, register.a);
        //1 carry 21 = 121 BCD
        assert!(register.flag.c == true);
    }

    #[test]
    fn cpl_test() {
        let mut register = Registers::new();

        register.a = 0xFF;

        cpl(&mut register.flag, &mut register.a);

        assert_eq!(0, register.a);
        assert!(register.flag.h == true);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }

    #[test]
    fn ld_hl_test() {
        let mut register = Registers::new();

        register.h = 0x01;
        register.l = 0;

        register.sp = 30;

        ld_hl(
            &mut register.flag,
            &mut register.h,
            &mut register.l,
            register.sp,
            2,
        );

        assert_eq!(0x120, register.get_hl());
    }
}
