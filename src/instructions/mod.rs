pub mod arithmetic_logic;
pub mod load;
pub mod rotate_and_shift;
pub mod single_bit_operation;
// pub mod cpu_control;
// pub mod jump;

pub fn is_supported(looking_opcode: u8, opcode_array: &[u8]) -> bool {
    let result = opcode_array
        .iter()
        .position(|&element| looking_opcode == element);

    if result.is_some() {
        return true;
    }
    false
}
