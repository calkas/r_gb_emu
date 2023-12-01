use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::clock::timer;
use crate::constants::gb_memory_map::address;

#[derive(Clone, Copy, Default)]
struct TimerControlRegister {
    pub clock_enable: bool,
    pub clock_select: u8,
}

impl TimerControlRegister {
    pub fn get_tima_clock_div(&self) -> u32 {
        match self.clock_select {
            0 => timer::TIMA_CLOCK_DIV_0, // 4096 Hz
            1 => timer::TIMA_CLOCK_DIV_1, // 262144 Hz
            2 => timer::TIMA_CLOCK_DIV_2, // 65536 Hz
            _ => timer::TIMA_CLOCK_DIV_3, // 16384 Hz
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
pub struct Timer {
    div_counter_register: u8,           // DIV
    tima_counter_register: u8,          // TIMA
    modulo_register: u8,                // TMA
    tac_register: TimerControlRegister, // TAC
    interrupt_req: bool,
    internal_div_counter: u32,
    internal_tima_counter: u32,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter_register: 0,
            tima_counter_register: 0,
            modulo_register: 0,
            tac_register: TimerControlRegister::default(),
            interrupt_req: false,
            internal_div_counter: 0,
            internal_tima_counter: 0,
        }
    }

    fn div_counter_update(&mut self, cycles: u32) {
        self.internal_div_counter += cycles;
        // It counts up at a frequency of 16382 Hz which means every 256 CPU clock cycles
        // the divider register needs to increment.
        if self.internal_div_counter >= timer::DIV_CLOCK_DIV {
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
            address::TIMER_DIV_REGISTER => self.div_counter_register,
            address::TIMER_TIMA_REGISTER => self.tima_counter_register,
            address::TIMER_TMA_REGISTER => self.modulo_register,
            address::TIMER_TAC_REGISTER => TimerControlRegister::into(self.tac_register),
            _ => panic!("Read - This address [{:#02x?}] is not for Timer", address),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            address::TIMER_DIV_REGISTER => self.div_counter_register = 0,
            address::TIMER_TIMA_REGISTER => self.tima_counter_register = data,
            address::TIMER_TMA_REGISTER => self.modulo_register = data,
            address::TIMER_TAC_REGISTER => self.tac_register = TimerControlRegister::from(data),
            _ => panic!("Write - This address [{:#02x?}] is not for Timer", address),
        }
    }
}

impl IoWorkingCycle for Timer {
    fn is_interrupt(&self) -> bool {
        self.interrupt_req
    }

    fn reset_interrupt(&mut self) {
        self.interrupt_req = false
    }

    fn next(&mut self, cycle: u32) {
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
        assert_eq!(timer::TIMA_CLOCK_DIV_0, tac.get_tima_clock_div());

        let reg_val: u8 = TimerControlRegister::into(tac);
        assert_eq!(0xFC, reg_val);

        tac = TimerControlRegister::from(0xFB);
        assert!(tac.clock_enable == false);
        assert_eq!(3, tac.clock_select);
        assert_eq!(timer::TIMA_CLOCK_DIV_3, tac.get_tima_clock_div());

        let reg_val: u8 = TimerControlRegister::into(tac);
        assert_eq!(0xFB, reg_val);
    }

    #[test]
    fn div_counter_test() {
        let mut timer = Timer::new();
        // the tima is not enable
        timer.next(256);
        assert_eq!(1, timer.div_counter_register);
        assert_eq!(0, timer.tima_counter_register);

        assert_eq!(
            1,
            timer.read_byte_from_hardware_register(address::TIMER_DIV_REGISTER)
        );

        // overflow
        for _ in 0..255 {
            timer.next(256);
        }
        assert_eq!(0, timer.div_counter_register);
    }

    #[test]
    fn tima_counter_test() {
        let mut timer = Timer::new();

        //Turn on tima counter
        timer.write_byte_to_hardware_register(address::TIMER_TAC_REGISTER, 4);

        assert_eq!(
            0xFC,
            timer.read_byte_from_hardware_register(address::TIMER_TAC_REGISTER)
        );

        //Default div is 1024
        timer.next(1024);
        assert_eq!(1, timer.tima_counter_register);

        assert_eq!(
            1,
            timer.read_byte_from_hardware_register(address::TIMER_TIMA_REGISTER)
        );

        // overflow
        for _ in 0..255 {
            timer.next(1024);
        }
        assert_eq!(0, timer.tima_counter_register);
        assert!(timer.is_interrupt() == true);
    }
}
