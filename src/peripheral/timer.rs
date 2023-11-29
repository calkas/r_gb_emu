use super::HardwareAccessible;
use crate::constants::gb_memory_map::address;

pub struct Timer {
    divider_register: u8,
    counter: u8, // TIMA
    modulo: u8,  // TMA
    control: u8, // TAC
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider_register: 0,
            counter: 0,
            modulo: 0,
            control: 0,
        }
    }
}

impl HardwareAccessible for Timer {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            address::TIMER_DIV_REGISTER => 0xFF,
            address::TIMER_TIMA_REGISTER => 0xFF,
            address::TIMER_TMA_REGISTER => 0xFF,
            address::TIMER_TAC_REGISTER => 0xFF,
            _ => panic!("Read - This address [{:#02x?}] is not for Timer", address),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            address::TIMER_DIV_REGISTER => {}
            address::TIMER_TIMA_REGISTER => {}
            address::TIMER_TMA_REGISTER => {}
            address::TIMER_TAC_REGISTER => {}
            _ => panic!("Write - This address [{:#02x?}] is not for Timer", address),
        }
    }

    fn run_cycle(&mut self, cycle: u32) {}
}
