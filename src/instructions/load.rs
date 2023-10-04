pub fn ld(out_reg: &mut u8, value: u8) {
    *out_reg = value;
}

pub fn ld_16(reg_high_byte: &mut u8, reg_low_byte: &mut u8, value: u16) {
    *reg_high_byte = ((value & 0xFF00).rotate_right(8)) as u8;
    *reg_low_byte = (value & 0x00FF) as u8;
}
