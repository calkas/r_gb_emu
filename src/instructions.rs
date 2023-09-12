pub mod load {}

pub mod arithmetic {
    use crate::cpu_data::Flags;
    use crate::cpu_data::Registers;

    pub fn get_reg_8bit_value(opcode:u8, regs: &Registers) -> u8 {
        let reg_id = opcode & 7;
        match reg_id {
            0 => regs.b,
            1 => regs.c,
            2 => regs.d,
            3 => regs.e,
            4 => regs.h,
            5 => regs.l,
            7 => regs.a,
            _ => 0,
        }
    }

    fn was_half_carry(reg: u8, value: u8) -> bool {
        ((reg & 0x0F) + (value & 0x0F)) & 0xF0 == 0x10
    }

    pub fn add(cpu_data: &mut Registers, value: u8, carry_value: u8) {
        cpu_data.unset_flag(Flags::N);

        let (new_value, did_overflow) = cpu_data.a.overflowing_add(value + carry_value);

        cpu_data.unset_flag(Flags::C);
        cpu_data.unset_flag(Flags::Z);
        cpu_data.unset_flag(Flags::H);

        if did_overflow {
            cpu_data.set_flag(Flags::C);
        }

        if new_value == 0 {
            cpu_data.set_flag(Flags::Z);
        }

        if was_half_carry(cpu_data.a, value + carry_value) {
            cpu_data.set_flag(Flags::H);
        }

        cpu_data.a = new_value;
    }
    pub fn add_hl(cpu_data: &mut Registers, value: u8) {}
    pub fn add_sp(cpu_data: &mut Registers, value: u8) {}

    pub fn adc(cpu_data: &mut Registers, value: u8) {
        let carry_val = if cpu_data.is_flag_set(Flags::C) { 1 } else { 0 };
        add(cpu_data, value, carry_val);
    }
}
#[cfg(test)]
mod arithmetic_add_adc_ut {

    use super::arithmetic::*;
    use crate::cpu_data::Flags;
    use crate::cpu_data::Registers;

    #[test]
    fn add_half_carry_flag_test() {
        let mut registers = Registers::new();
        registers.a = 0x6C;
        registers.set_flag(Flags::N);

        add(&mut registers, 0x2E, 0);

        assert_eq!(0x9A, registers.a);
        assert!(registers.is_flag_set(Flags::H));
        assert!(!registers.is_flag_set(Flags::Z));
        assert!(!registers.is_flag_set(Flags::C));
        assert!(!registers.is_flag_set(Flags::N));
    }

    #[test]
    fn add_carry_half_carry_and_zero_flag_test() {
        let mut registers = Registers::new();
        registers.a = 0xFF;
        registers.set_flag(Flags::N);

        add(&mut registers, 1, 0);

        assert_eq!(0x00, registers.a);
        assert!(registers.is_flag_set(Flags::H));
        assert!(registers.is_flag_set(Flags::Z));
        assert!(registers.is_flag_set(Flags::C));
        assert!(!registers.is_flag_set(Flags::N));
    }

    #[test]
    fn adc_carry_flag_set_test() {
        let mut registers = Registers::new();
        registers.a = 0x3D;
        registers.set_flag(Flags::N);

        add(&mut registers, 0x42, 1);

        assert_eq!(0x80, registers.a);
        assert!(registers.is_flag_set(Flags::H));
        assert!(!registers.is_flag_set(Flags::Z));
        assert!(!registers.is_flag_set(Flags::C));
        assert!(!registers.is_flag_set(Flags::N));
    }



}

pub mod logic {}
pub mod rotate_and_shift {}
pub mod single_bit_operation {}
pub mod cpu_control {}
pub mod jump {}
