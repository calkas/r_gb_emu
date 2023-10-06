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
            pc: 0, //0x0100,
            sp: 0xFFFE,
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
    }
}
