use crate::cpu_data::FlagsRegister;

/// # rlc
/// RLC (rotate left register) - bit rotate register left (not through the carry flag)
pub fn rlc(flag: &mut FlagsRegister, register_or_value: &mut u8) {
    let msb_bit = (*register_or_value & 0x80).rotate_right(7);
    *register_or_value = ((*register_or_value) << 1 as u8) | msb_bit;

    flag.n = false;
    flag.h = false;
    flag.z = if *register_or_value == 0 { true } else { false };
    flag.c = if msb_bit == 1 { true } else { false };
}

/// # rl
/// RL (rotate left register) - bit rotate register left through the carry flag
pub fn rl(flag: &mut FlagsRegister, register_or_value: &mut u8) {
    let old_carry_val: u8 = if flag.c { 1 } else { 0 };
    let msb_bit = (*register_or_value & 0x80).rotate_right(7);
    *register_or_value = ((*register_or_value) << 1 as u8) | old_carry_val;

    flag.n = false;
    flag.h = false;
    flag.z = if *register_or_value == 0 { true } else { false };
    flag.c = if msb_bit == 1 { true } else { false };
}

/// #rrc
/// RRC (rotate right register) - bit rotate register right (not through the carry flag)
pub fn rrc(flag: &mut FlagsRegister, register_or_value: &mut u8) {
    let lsb_bit = *register_or_value & 0x01;
    *register_or_value = ((*register_or_value) >> 1 as u8) | (0x80 & lsb_bit.rotate_left(7));

    flag.n = false;
    flag.h = false;
    flag.z = if *register_or_value == 0 { true } else { false };
    flag.c = if lsb_bit == 1 { true } else { false };
}

/// # rr
/// RR (rotate right register) - bit rotate register right through the carry flag
pub fn rr(flag: &mut FlagsRegister, register_or_value: &mut u8) {
    let old_carry_val: u8 = if flag.c { 1 } else { 0 };
    let lsb_bit = *register_or_value & 0x01;

    *register_or_value = ((*register_or_value) >> 1 as u8) | old_carry_val.rotate_left(7);

    flag.n = false;
    flag.h = false;
    flag.z = if *register_or_value == 0 { true } else { false };
    flag.c = if lsb_bit == 1 { true } else { false };
}

/// # rlca
/// RLCA (rotate left A register) - bit rotate A register left (not through the carry flag)
pub fn rlca(flag: &mut FlagsRegister, acc: &mut u8) {
    rlc(flag, acc);
    flag.z = false;
}

/// # rla
/// RLA (rotate left A register) - bit rotate A register left through the carry flag
pub fn rla(flag: &mut FlagsRegister, acc: &mut u8) {
    rl(flag, acc);
    flag.z = false;
}

/// #rrca
/// RRCA (rotate right A register) - bit rotate A register right (not through the carry flag)
pub fn rrca(flag: &mut FlagsRegister, acc: &mut u8) {
    rrc(flag, acc);
    flag.z = false;
}

/// # rra
/// RRA (rotate right A register) - bit rotate A register right through the carry flag
pub fn rra(flag: &mut FlagsRegister, acc: &mut u8) {
    rr(flag, acc);
    flag.z = false;
}

#[cfg(test)]
mod ut_test {

    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn rlca_test() {
        let mut register = Registers::new();
        register.a = 0xF2;
        register.flag.z = true;
        register.flag.n = true;
        register.flag.h = true;

        rlca(&mut register.flag, &mut register.a);

        assert_eq!(0xE5, register.a);
        assert!(register.flag.z == false);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == true);
    }

    #[test]
    fn rla_test() {
        let mut register = Registers::new();
        register.a = 0xB5;
        register.flag.z = true;
        register.flag.n = true;
        register.flag.h = true;
        register.flag.c = false;

        rla(&mut register.flag, &mut register.a);

        assert_eq!(0x6A, register.a);
        assert!(register.flag.z == false);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == true);
    }

    #[test]
    fn rrca_test() {
        let mut register = Registers::new();
        register.a = 0xF2;
        register.flag.z = true;
        register.flag.n = true;
        register.flag.h = true;
        register.flag.c = true;

        rrca(&mut register.flag, &mut register.a);

        assert_eq!(0x79, register.a);
        assert!(register.flag.z == false);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == false);
    }

    #[test]
    fn rra_test() {
        let mut register = Registers::new();
        register.a = 0x6A;
        register.flag.z = true;
        register.flag.n = true;
        register.flag.h = true;
        register.flag.c = true;

        rra(&mut register.flag, &mut register.a);

        assert_eq!(0xB5, register.a);
        assert!(register.flag.z == false);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == false);
    }
}
