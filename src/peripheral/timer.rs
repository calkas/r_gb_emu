use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::address;

mod timer_setup {
    pub const DIV_CLOCK_DIV: u32 = 255;
    pub const TIMA_CLOCK_DIV_0: u32 = 1024;
    pub const TIMA_CLOCK_DIV_1: u32 = 16;
    pub const TIMA_CLOCK_DIV_2: u32 = 64;
    pub const TIMA_CLOCK_DIV_3: u32 = 256;
}

#[derive(Clone, Copy, Default)]
struct TimerControlRegister {
    pub clock_enable: bool,
    pub clock_select: u8,
}

impl TimerControlRegister {
    pub fn get_tima_clock_div(&self) -> u32 {
        match self.clock_select {
            0 => timer_setup::TIMA_CLOCK_DIV_0, // 4096 Hz
            1 => timer_setup::TIMA_CLOCK_DIV_1, // 262144 Hz
            2 => timer_setup::TIMA_CLOCK_DIV_2, // 65536 Hz
            _ => timer_setup::TIMA_CLOCK_DIV_3, // 16384 Hz
        }
    }
}

impl std::convert::From<u8> for TimerControlRegister {
    fn from(value: u8) -> TimerControlRegister {
        let enable = (value.rotate_right(2) & 1) == 1;
        let clock_select = value & 0x03;
        TimerControlRegister {
            clock_enable: enable,
            clock_select,
        }
    }
}

impl std::convert::From<TimerControlRegister> for u8 {
    fn from(control_reg: TimerControlRegister) -> u8 {
        let mut ret_reg_val = 0xF8;
        if control_reg.clock_enable {
            ret_reg_val |= 1_u8.rotate_left(2);
        }
        ret_reg_val |= control_reg.clock_select;
        ret_reg_val
    }
}

#[derive(Default)]
pub struct Timer {
    div_counter_register: u8,           // DIV
    tima_counter_register: u8,          // TIMA
    modulo_register: u8,                // TMA
    tac_register: TimerControlRegister, // TAC
    pub interrupt_req: bool,
    internal_div_counter: u32,
    internal_tima_counter: u32,
}

impl Timer {
    fn div_counter_update(&mut self, cycles: u32) {
        self.internal_div_counter += cycles;
        // It counts up at a frequency of 16382 Hz which means every 256 CPU clock cycles
        // the divider register needs to increment.
        if self.internal_div_counter >= timer_setup::DIV_CLOCK_DIV {
            self.div_counter_register = self.div_counter_register.wrapping_add(1);
            self.internal_div_counter = 0;
        }
    }

    fn tima_counter_update(&mut self, cycles: u32) {
        self.internal_tima_counter += cycles;
        if self.internal_tima_counter >= self.tac_register.get_tima_clock_div() {
            let (new_value, did_overflow) = self.tima_counter_register.overflowing_add(1);
            if did_overflow {
                self.tima_counter_register = self.modulo_register;
                self.interrupt_req = true;
            } else {
                self.tima_counter_register = new_value;
            }
            self.internal_tima_counter = 0;
        }
    }
}

impl HardwareAccessible for Timer {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            address::io_hardware_register::TIMER_DIV => self.div_counter_register,
            address::io_hardware_register::TIMER_TIMA => self.tima_counter_register,
            address::io_hardware_register::TIMER_TMA => self.modulo_register,
            address::io_hardware_register::TIMER_TAC => {
                TimerControlRegister::into(self.tac_register)
            }
            _ => panic!(
                "[TIMER ERROR][Read] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            address::io_hardware_register::TIMER_DIV => self.div_counter_register = 0,
            address::io_hardware_register::TIMER_TIMA => self.tima_counter_register = data,
            address::io_hardware_register::TIMER_TMA => self.modulo_register = data,
            address::io_hardware_register::TIMER_TAC => {
                self.tac_register = TimerControlRegister::from(data)
            }
            _ => panic!(
                "[TIMER ERROR][Write] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }
}

impl IoWorkingCycle for Timer {
    fn next_to(&mut self, cycle: u32) {
        self.div_counter_update(cycle);

        if self.tac_register.clock_enable {
            self.tima_counter_update(cycle);
        }
    }
}

#[cfg(test)]
mod ut {
    use super::*;
    #[test]
    fn tac_register_test() {
        let mut tac = TimerControlRegister::default();
        tac = TimerControlRegister::from(4);
        assert!(tac.clock_enable == true);
        assert_eq!(0, tac.clock_select);
        assert_eq!(timer_setup::TIMA_CLOCK_DIV_0, tac.get_tima_clock_div());

        let reg_val: u8 = TimerControlRegister::into(tac);
        assert_eq!(0xFC, reg_val);

        tac = TimerControlRegister::from(0xFB);
        assert!(tac.clock_enable == false);
        assert_eq!(3, tac.clock_select);
        assert_eq!(timer_setup::TIMA_CLOCK_DIV_3, tac.get_tima_clock_div());

        let reg_val: u8 = TimerControlRegister::into(tac);
        assert_eq!(0xFB, reg_val);
    }

    #[test]
    fn div_counter_test() {
        let mut timer = Timer::default();
        // the tima is not enable
        timer.next_to(256);
        assert_eq!(1, timer.div_counter_register);
        assert_eq!(0, timer.tima_counter_register);

        assert_eq!(
            1,
            timer.read_byte_from_hardware_register(address::io_hardware_register::TIMER_DIV)
        );

        // overflow
        for _ in 0..255 {
            timer.next_to(256);
        }
        assert_eq!(0, timer.div_counter_register);
    }

    #[test]
    fn tima_counter_test() {
        let mut timer = Timer::default();

        //Turn on tima counter
        timer.write_byte_to_hardware_register(address::io_hardware_register::TIMER_TAC, 4);

        assert_eq!(
            0xFC,
            timer.read_byte_from_hardware_register(address::io_hardware_register::TIMER_TAC)
        );

        //Default div is 1024
        timer.next_to(1024);
        assert_eq!(1, timer.tima_counter_register);

        assert_eq!(
            1,
            timer.read_byte_from_hardware_register(address::io_hardware_register::TIMER_TIMA)
        );

        // overflow
        for _ in 0..255 {
            timer.next_to(1024);
        }
        assert_eq!(0, timer.tima_counter_register);
        assert!(timer.interrupt_req == true);
    }
}
