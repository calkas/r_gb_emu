pub fn ld(out_reg: &mut u8, value: u8) {
    *out_reg = value;
}

pub fn ld_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8, value: u16) {
    *reg_high_byte = ((value & 0xFF00).rotate_right(8)) as u8;
    *reg_low_byte = (value & 0x00FF) as u8;
}

pub fn calculate_address_for_io_port(val: u8) -> u16 {
    0xFF00 | val as u16
}

//ldi
pub fn hli(reg_h: &mut u8, reg_l: &mut u8) -> u16 {
    let reg_hl_val = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
    *reg_h = (((reg_hl_val + 1) & 0xFF00).rotate_right(8)) as u8;
    *reg_l = ((reg_hl_val + 1) & 0x00FF) as u8;
    reg_hl_val
}
//ldd
pub fn hld(reg_h: &mut u8, reg_l: &mut u8) -> u16 {
    let reg_hl_val = (*reg_h as u16).rotate_left(8) | (*reg_l as u16);
    *reg_h = (((reg_hl_val - 1) & 0xFF00).rotate_right(8)) as u8;
    *reg_l = ((reg_hl_val - 1) & 0x00FF) as u8;
    reg_hl_val
}

pub fn push(stack: &mut [u8], reg_sp: &mut u16, value: u16) {
    let low_byte_val = (value & 0x00FF) as u8;
    let high_byte_val = (value & 0xFF00).rotate_right(8) as u8;

    stack[*reg_sp as usize] = high_byte_val;
    *reg_sp = reg_sp.wrapping_sub(1);
    stack[*reg_sp as usize] = low_byte_val;
    *reg_sp = reg_sp.wrapping_sub(1);
}

fn pop(stack: &mut [u8], reg_sp: &mut u16) -> u16 {
    if *reg_sp == 0xFFFE {
        panic!("Error pop stack operation");
    }
    *reg_sp += 1;
    let low_byte_val = stack[*reg_sp as usize];
    *reg_sp += 1;
    let high_byte_val = stack[*reg_sp as usize];
    (high_byte_val as u16).rotate_left(8) | (low_byte_val as u16)
}

#[cfg(test)]
mod load_ut {
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

        let exp_ret_hl_val = (register.h as u16).rotate_left(8) | (register.l as u16);

        let ret_val = hli(&mut register.h, &mut register.l);
        assert_eq!(exp_ret_hl_val, ret_val);
        assert_eq!(0x30, register.h);
        assert_eq!(0x21, register.l);
    }
    #[test]
    fn ldd_test() {
        let mut register = Registers::new();

        register.h = 0x30;
        register.l = 0x20;

        let exp_ret_hl_val = (register.h as u16).rotate_left(8) | (register.l as u16);

        let ret_val = hld(&mut register.h, &mut register.l);
        assert_eq!(exp_ret_hl_val, ret_val);
        assert_eq!(0x30, register.h);
        assert_eq!(0x1F, register.l);
    }

    #[test]
    fn stack_test() {
        let mut stack: [u8; 0xFFFF] = [0xFF; 0xFFFF];
        let mut register = Registers::new();

        let mut exp_sp_value = register.sp - 2;

        push(&mut stack, &mut register.sp, 0x8001);
        assert_eq!(exp_sp_value, register.sp);
        push(&mut stack, &mut register.sp, 0x8002);
        exp_sp_value -= 2;
        assert_eq!(exp_sp_value, register.sp);
        push(&mut stack, &mut register.sp, 0x8003);
        exp_sp_value -= 2;
        assert_eq!(exp_sp_value, register.sp);

        assert_eq!(0x8003, pop(&mut stack, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
        assert_eq!(0x8002, pop(&mut stack, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
        assert_eq!(0x8001, pop(&mut stack, &mut register.sp));
        exp_sp_value += 2;
        assert_eq!(exp_sp_value, register.sp);
    }
}
