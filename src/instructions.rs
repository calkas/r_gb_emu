pub mod load {}

pub mod arithmetic_logic {
    use crate::cpu_data::{self, FlagsRegister};

    fn was_half_carry(reg: u8, value: u8) -> bool {
        (((reg & 0x0F) + (value & 0x0F)) & 0xF0) == 0x10
    }

    fn was_half_carry_for_16_bits(reg: u16, value: u16) -> bool {
        (((reg & 0x0FFF) + (value & 0x0FFF)) & 0x100) == 0x100
    }

    pub fn add(flag: &mut FlagsRegister, acc: &mut u8, value: u8, carry_value: u8) {
        flag.n = false;

        let (new_value, did_overflow) = acc.overflowing_add(value + carry_value);
        flag.c = false;
        flag.z = false;
        flag.h = false;

        if did_overflow {
            flag.c = true;
        }

        if new_value == 0 {
            flag.z = true;
        }

        if was_half_carry(*acc, value + carry_value) {
            flag.h = true;
        }
        *acc = new_value;
    }

    pub fn add_hl(flag: &mut FlagsRegister, reg_h: &mut u8, reg_l: &mut u8, reg_16_value: u16) {
        flag.n = false;

        let hl_reg_value = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
        let (new_value, did_overflow) = hl_reg_value.overflowing_add(reg_16_value);

        flag.h = false;
        flag.c = false;

        if did_overflow {
            flag.c = true;
        }

        if was_half_carry_for_16_bits(hl_reg_value, reg_16_value) {
            flag.h = true;
        }

        *reg_h = ((new_value & 0xFF00).rotate_right(8)) as u8;
        *reg_l = (new_value & 0x00FF) as u8;
    }

    pub fn add_sp(flag: &mut FlagsRegister, reg_sp: &mut u16, value: i8) {
        flag.z = false;
        flag.n = false;

        let coverted_value = value as i8 as i16 as u16;
        let (new_value, did_overflow) = reg_sp.overflowing_add(coverted_value);

        flag.h = false;
        flag.c = false;

        if did_overflow {
            flag.c = true;
        }

        if was_half_carry_for_16_bits(*reg_sp, coverted_value) {
            flag.h = true;
        }

        *reg_sp = new_value;
    }

    pub fn adc(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        let carry_val = if flag.c == true { 1 } else { 0 };
        add(flag, acc, value, carry_val);
    }

    pub fn sub(flag: &mut FlagsRegister, acc: &mut u8, value: u8, carry_value: u8) {
        flag.n = true;

        let (new_value, did_overflow) = acc.overflowing_sub(value + carry_value);

        flag.c = false;
        flag.z = false;
        flag.h = false;

        if did_overflow {
            flag.c = true;
        }

        if new_value == 0 {
            flag.z = true;
        }

        if was_half_carry(*acc, value + carry_value) {
            flag.h = true;
        }
        *acc = new_value;
    }

    pub fn sbc(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        let carry_val = if flag.c == true { 1 } else { 0 };
        sub(flag, acc, value, carry_val);
    }

    pub fn and(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        flag.n = false;
        flag.h = true;
        flag.c = false;

        *acc &= value;
        flag.z = false;

        if *acc == 0 {
            flag.z = true;
        }
    }

    pub fn xor(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        flag.n = false;
        flag.h = false;
        flag.c = false;

        *acc ^= value;
        flag.z = false;

        if *acc == 0 {
            flag.z = true;
        }
    }

    pub fn or(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        flag.n = false;
        flag.h = false;
        flag.c = false;

        *acc |= value;
        flag.z = false;

        if *acc == 0 {
            flag.z = true;
        }
    }

    pub fn cp(flag: &mut FlagsRegister, acc: &mut u8, value: u8) {
        let saved_acc = *acc;
        sub(flag, acc, value, 0);
        *acc = saved_acc;
    }

    pub fn inc(flag: &mut FlagsRegister, reg_or_data: &mut u8) {
        flag.n = false;

        let result = *reg_or_data + 1;

        flag.z = false;
        flag.h = false;

        if result == 0 {
            flag.z = true;
        }

        if was_half_carry(*reg_or_data, 1) {
            flag.h = true;
        }

        *reg_or_data = result;
    }

    pub fn inc_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8) {
        let mut reg_value = (*reg_high_byte as u16).rotate_left(8) | (*reg_low_byte as u16);
        reg_value += 1;

        *reg_high_byte = ((reg_value & 0xFF00).rotate_right(8)) as u8;
        *reg_low_byte = (reg_value & 0x00FF) as u8;
    }

    //Flagi do poprawy halfcarry

    pub fn dec(flag: &mut FlagsRegister, reg_or_data: &mut u8) {
        flag.n = true;

        let result = *reg_or_data - 1;

        flag.z = false;
        flag.h = false;

        if result == 0 {
            flag.z = true;
        }

        if was_half_carry(*reg_or_data, 1) {
            flag.h = true;
        }

        *reg_or_data = result;
    }
}
#[cfg(test)]
mod arithmetic_logic_ut {

    use super::arithmetic_logic::*;
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
        assert!(register.flag.h == true);
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
        assert!(register.flag.h == false);
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
        assert!(register.flag.h == true);
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
        assert!(register.flag.h == true);
        assert!(register.flag.z == true);
        assert!(register.flag.c == false);
        assert!(register.flag.n == true);
    }

    #[test]
    fn inc_test() {
        let mut register = Registers::new();
        register.flag.n = true;
        register.b = 7;

        inc(&mut register.flag, &mut register.b);

        assert_eq!(8, register.b);
        assert!(register.flag.h == false);
        assert!(register.flag.z == false);
        assert!(register.flag.c == false);
        assert!(register.flag.n == false);

        let mut val: u8 = 99;

        inc(&mut register.flag, &mut val);

        assert_eq!(100, val);
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
    }
}

pub mod rotate_and_shift {}
pub mod single_bit_operation {}
pub mod cpu_control {}
pub mod jump {}
