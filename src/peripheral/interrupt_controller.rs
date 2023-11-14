use super::HardwareAccessible;

/// # InterruptRegister
///
/// Bit position:
///  * 0 V-Blank
///  * 1 LCD
///  * 2 Timer
///  * 3 Serial Link
///  * 4 Joypad
#[derive(Clone, Copy, Default)]
pub struct InterruptRegister {
    pub joypad: bool,
    pub serial_link: bool,
    pub timer: bool,
    pub lcd: bool,
    pub v_blank: bool,
}

impl std::convert::From<u8> for InterruptRegister {
    fn from(value: u8) -> InterruptRegister {
        let joypad = (value.rotate_right(4) & 1) == 1;
        let serial_link = (value.rotate_right(3) & 1) == 1;
        let timer = (value.rotate_right(2) & 1) == 1;
        let lcd = (value.rotate_right(1) & 1) == 1;
        let v_blank = (value & 1) == 1;
        InterruptRegister {
            joypad,
            serial_link,
            timer,
            lcd,
            v_blank,
        }
    }
}

impl std::convert::From<InterruptRegister> for u8 {
    fn from(flag: InterruptRegister) -> u8 {
        let mut ret_val_flag: u8 = 0;

        if flag.joypad {
            ret_val_flag |= 1_u8.rotate_left(4);
        }
        if flag.serial_link {
            ret_val_flag |= 1_u8.rotate_left(3);
        }
        if flag.timer {
            ret_val_flag |= 1_u8.rotate_left(2);
        }
        if flag.lcd {
            ret_val_flag |= 1_u8.rotate_left(1);
        }
        if flag.v_blank {
            ret_val_flag |= 1;
        }
        ret_val_flag
    }
}
pub struct InterruptController {
    //IF register
    pub intf: InterruptRegister,
    //IE register
    pub inte: InterruptRegister,
}
impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            intf: InterruptRegister::default(),
            inte: InterruptRegister::default(),
        }
    }
}

impl HardwareAccessible for InterruptController {
    fn read_byte_from_hardware_register(&self, address: usize) -> u8 {
        match address {
            0xFF0F => InterruptRegister::into(self.intf),
            0xFFFF => InterruptRegister::into(self.inte),
            _ => panic!(
                "Read - This address [{}] is not for interrupt controller",
                address
            ),
        }
    }
    fn write_byte_to_hardware_register(&mut self, address: usize, data: u8) {
        match address {
            0xFF0F => self.intf = InterruptRegister::from(data),
            0xFFFF => self.inte = InterruptRegister::from(data),
            _ => panic!(
                "Write - This address [{}] is not for interrupt controller",
                address
            ),
        }
    }
}

#[cfg(test)]
mod ut {

    use super::*;
    #[test]
    fn ie_if_flag_test() {
        let mut isr = InterruptController::new();

        //IF
        isr.write_byte_to_hardware_register(0xFF0F, 0x19);
        assert_eq!(0x19, isr.read_byte_from_hardware_register(0xFF0F));
        assert!(isr.intf.joypad == true);
        assert!(isr.intf.serial_link == true);
        assert!(isr.intf.timer == false);
        assert!(isr.intf.lcd == false);
        assert!(isr.intf.v_blank == true);

        //IE
        isr.inte.lcd = true;
        isr.inte.v_blank = true;

        assert_eq!(0x03, isr.read_byte_from_hardware_register(0xFFFF));
        isr.write_byte_to_hardware_register(0xFFFF, 0x1F);
        assert_eq!(0x1F, isr.read_byte_from_hardware_register(0xFFFF));
        assert!(isr.inte.joypad == true);
        assert!(isr.inte.serial_link == true);
        assert!(isr.inte.timer == true);
        assert!(isr.inte.lcd == true);
        assert!(isr.inte.v_blank == true);
    }
}
