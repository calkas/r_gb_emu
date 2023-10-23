#[derive(Clone, Copy)]
pub struct FlagsRegister {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

impl FlagsRegister {
    pub fn new() -> Self {
        FlagsRegister {
            z: false,
            n: false,
            h: false,
            c: false,
        }
    }
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        let mut ret_val_flag: u8 = 0;

        if flag.z {
            ret_val_flag |= 1_u8.rotate_left(7);
        }
        if flag.n {
            ret_val_flag |= 1_u8.rotate_left(6);
        }
        if flag.h {
            ret_val_flag |= 1_u8.rotate_left(5);
        }
        if flag.c {
            ret_val_flag |= 1_u8.rotate_left(4);
        }
        ret_val_flag
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(flag_raw_value: u8) -> FlagsRegister {
        let z = (flag_raw_value.rotate_right(7) & 1) == 1;
        let n = (flag_raw_value.rotate_right(6) & 1) == 1;
        let h = (flag_raw_value.rotate_right(5) & 1) == 1;
        let c = (flag_raw_value.rotate_right(4) & 1) == 1;

        FlagsRegister { z, n, h, c }
    }
}

pub struct Registers {
    pub a: u8,
    pub flag: FlagsRegister,

    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            flag: FlagsRegister::new(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
        }
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16).rotate_left(8) | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16).rotate_left(8) | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16).rotate_left(8) | (self.l as u16)
    }

    pub fn get_af(&self) -> u16 {
        let flag_value: u8 = FlagsRegister::into(self.flag);
        (self.a as u16).rotate_left(8) | (flag_value as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00).rotate_right(8)) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00).rotate_right(8)) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00).rotate_right(8)) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00).rotate_right(8)) as u8;
        let raw_flag_value = (value & 0x00FF) as u8;
        self.flag = FlagsRegister::from(raw_flag_value);
    }

    pub fn get_reg_value_from_opcode_range(&self, opcode_array: &[u8], opcode: u8) -> u8 {
        assert!(opcode_array.len() == 7);
        let mut reg_id: usize = 0xFF;
        for (id, element) in opcode_array.iter().enumerate() {
            if opcode == *element {
                reg_id = id;
            }
        }
        match reg_id {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.a,
            _ => panic!("opcode does not exist in looking array"),
        }
    }

    pub fn get_reg_address_from_opcode_range(
        &mut self,
        opcode_array: &[u8],
        opcode: u8,
    ) -> &mut u8 {
        assert!(opcode_array.len() == 7);
        let mut reg_id: usize = 0xFF;
        for (id, element) in opcode_array.iter().enumerate() {
            if opcode == *element {
                reg_id = id;
            }
        }
        match reg_id {
            0 => &mut self.b,
            1 => &mut self.c,
            2 => &mut self.d,
            3 => &mut self.e,
            4 => &mut self.h,
            5 => &mut self.l,
            6 => &mut self.a,
            _ => panic!("opcode does not exist in looking array"),
        }
    }

    pub fn get_reg16_value_from_opcode_array(&self, opcode_array: &[u8], opcode: u8) -> u16 {
        assert!(opcode_array.len() == 4);
        let mut reg_id: usize = 0xFF;
        for (id, element) in opcode_array.iter().enumerate() {
            if opcode == *element {
                reg_id = id;
            }
        }

        match reg_id {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.sp,
            _ => panic!("opcode does not exist in looking array"),
        }
    }
}

#[cfg(test)]
mod uint_test {
    use super::*;

    #[test]
    fn register_test() {
        let mut register = Registers::new();

        register.b = 0x33;
        register.c = 0x34;
        register.d = 0x35;
        register.e = 0x36;
        register.h = 0x37;
        register.l = 0x38;

        assert_eq!(0x3334, register.get_bc());
        assert_eq!(0x3536, register.get_de());
        assert_eq!(0x3738, register.get_hl());

        register.set_bc(0x6968);
        register.set_de(0x7170);
        register.set_hl(0x7372);

        assert_eq!(0x6968, register.get_bc());
        assert_eq!(0x7170, register.get_de());
        assert_eq!(0x7372, register.get_hl());
    }
    #[test]
    fn af_test() {
        let mut register = Registers::new();

        register.a = 0x01;
        assert_eq!(0x100, register.get_af());
        register.flag.z = true;
        assert_eq!(0x180, register.get_af());
        register.flag.n = true;
        assert_eq!(0x1C0, register.get_af());
        register.flag.h = true;
        assert_eq!(0x1E0, register.get_af());
        register.flag.c = true;
        assert_eq!(0x1F0, register.get_af());

        register.set_af(0x390);
        assert!(register.flag.z);
        assert!(register.flag.c);
    }

    #[test]
    fn get_reg_value_and_address_test() {
        let mut register = Registers::new();
        let exp_reg_val: [u8; 7] = [10, 11, 12, 13, 14, 15, 16];

        //SET REGS
        for opcode in exp_reg_val.iter() {
            let mut reg =
                register.get_reg_address_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], *opcode);
            *reg = *opcode;
        }

        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 10),
            register.b
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 11),
            register.c
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 12),
            register.d
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 13),
            register.e
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 14),
            register.h
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 15),
            register.l
        );
        assert_eq!(
            register.get_reg_value_from_opcode_range(&[10, 11, 12, 13, 14, 15, 16], 16),
            register.a
        );
    }
}
