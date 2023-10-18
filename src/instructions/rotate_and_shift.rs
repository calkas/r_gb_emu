use crate::cpu_data::FlagsRegister;

/// # rlca
/// RLCA (rotate left A register) - bit rotate A register left (not through the carry flag)
pub fn rlca(flag: &mut FlagsRegister, acc: &mut u8) {
    let bit_for_rotate = (*acc & 0x80).rotate_right(7);
    *acc = (*acc).rotate_left(1) | bit_for_rotate;

    flag.z = false;
    flag.n = false;
    flag.h = false;

    flag.c = if bit_for_rotate == 1 { true } else { false };
}

/// # rla
/// RLA (rotate left A register) - bit rotate A register left through the carry flag
pub fn rla(flag: &mut FlagsRegister, acc: &mut u8) {
    todo!();
}

/// #rrca
/// RRCA (rotate right A register) - bit rotate A register right (not through the carry flag)
pub fn rrca(flag: &mut FlagsRegister, acc: &mut u8) {
    todo!();
}

/// # rra
/// RRA (rotate right A register) - bit rotate A register right through the carry flag
pub fn rra(flag: &mut FlagsRegister, acc: &mut u8) {
    todo!();
}
