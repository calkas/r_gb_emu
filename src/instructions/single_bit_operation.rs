use crate::cpu_data::FlagsRegister;

pub static SINGLE_BIT_OPERATION_OPCODES: [u8; 1] = [0x0];

/// # bit
/// bit test - test to see if a specific bit of a specific register is set
pub fn bit(flag: &mut FlagsRegister, register_or_value: u8, bit_number: u8) {
    flag.n = false;
    flag.h = true;

    let is_bit_set = (register_or_value & 1_u8.rotate_left(bit_number as u32)) != 0;
    flag.z = !is_bit_set;
}

/// # set
/// (bit set) - set a specific bit of a specific register to 1
pub fn set(register_or_value: &mut u8, bit_number: u8) {
    *register_or_value |= 1_u8.rotate_left(bit_number as u32);
}

/// # res - RESET
/// (bit reset) - set a specific bit of a specific register to 0
pub fn res(register_or_value: &mut u8, bit_number: u8) {
    *register_or_value &= !1_u8.rotate_left(bit_number as u32);
}

#[cfg(test)]
mod single_bit_operation_ut {
    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn bit_check_test() {
        let mut register = Registers::new();
        register.flag.n = true;
        register.flag.h = false;

        register.a = 0x08;
        bit(&mut register.flag, register.a, 3);

        assert!(register.flag.n == false);
        assert!(register.flag.h == true);
        assert!(register.flag.c == false);
        assert!(register.flag.z == false);

        bit(&mut register.flag, register.a, 2);
        assert!(register.flag.z == true);
    }
    #[test]
    fn bit_set_reset_test() {
        let mut register = Registers::new();

        set(&mut register.a, 3);
        assert_eq!(0x08, register.a);

        set(&mut register.a, 4);
        assert_eq!(0x18, register.a);

        res(&mut register.a, 3);
        assert_eq!(0x10, register.a);

        res(&mut register.a, 4);
        assert_eq!(0, register.a);
    }
}
