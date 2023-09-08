pub mod load {}

pub mod arithmetic {
    use crate::cpu_data::Flags;
    use crate::cpu_data::Registers;

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
            cpu_data.set_flag(Flags::C);
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

pub mod logic {}
pub mod rotate_and_shift {}
pub mod single_bit_operation {}
pub mod cpu_control {}
pub mod jump {}
