pub enum Flags {
    Z,
    N,
    H,
    C,
}

pub struct Registers {
    pub a: u8,
    pub f: u8,

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
            f: 0,
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
    pub fn set_flag(&mut self, flag: Flags) {
        match flag {
            Flags::Z => self.f |= 1_u8.rotate_left(6),
            Flags::N => self.f |= 1_u8.rotate_left(5),
            Flags::H => self.f |= 1_u8.rotate_left(4),
            Flags::C => self.f |= 1_u8.rotate_left(3),
        }
    }
    pub fn unset_flag(&mut self, flag: Flags) {
        match flag {
            Flags::Z => self.f &= !(1_u8.rotate_left(6)),
            Flags::N => self.f &= !(1_u8.rotate_left(5)),
            Flags::H => self.f &= !(1_u8.rotate_left(4)),
            Flags::C => self.f &= !(1_u8.rotate_left(3)),
        }
    }

    pub fn is_flag_set(&mut self, flag: Flags) -> bool {
        match flag {
            Flags::Z => self.f & 1_u8.rotate_left(6) != 0,
            Flags::N => self.f & 1_u8.rotate_left(5) != 0,
            Flags::H => self.f & 1_u8.rotate_left(4) != 0,
            Flags::C => self.f & 1_u8.rotate_left(3) != 0,
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
}

#[cfg(test)]
mod uint_test {
    use super::*;

    #[test]
    fn flag_test() {
        let mut cpu_reg = Registers::new();
        cpu_reg.set_flag(Flags::C);
        assert_eq!(0x08, cpu_reg.f);
        cpu_reg.set_flag(Flags::H);
        assert_eq!(0x18, cpu_reg.f);
        cpu_reg.set_flag(Flags::N);
        assert_eq!(0x38, cpu_reg.f);
        cpu_reg.set_flag(Flags::Z);
        assert_eq!(0x78, cpu_reg.f);

        assert!(cpu_reg.is_flag_set(Flags::C));
        assert!(cpu_reg.is_flag_set(Flags::H));
        assert!(cpu_reg.is_flag_set(Flags::N));
        assert!(cpu_reg.is_flag_set(Flags::Z));

        cpu_reg.unset_flag(Flags::C);
        assert_eq!(0x70, cpu_reg.f);
        cpu_reg.unset_flag(Flags::H);
        assert_eq!(0x60, cpu_reg.f);
        cpu_reg.unset_flag(Flags::N);
        assert_eq!(0x40, cpu_reg.f);
        cpu_reg.unset_flag(Flags::Z);
        assert_eq!(0x00, cpu_reg.f);

        assert!(!cpu_reg.is_flag_set(Flags::C));
        assert!(!cpu_reg.is_flag_set(Flags::H));
        assert!(!cpu_reg.is_flag_set(Flags::N));
        assert!(!cpu_reg.is_flag_set(Flags::Z));
    }

    #[test]
    fn register_test() {
        let mut cpu_reg = Registers::new();

        cpu_reg.set_bc(0x6970);
        assert_eq!(0x69, cpu_reg.b);
        assert_eq!(0x70, cpu_reg.c);

        cpu_reg.set_de(0x6970);
        assert_eq!(0x69, cpu_reg.d);
        assert_eq!(0x70, cpu_reg.e);

        cpu_reg.set_hl(0x6970);
        assert_eq!(0x69, cpu_reg.h);
        assert_eq!(0x70, cpu_reg.l);

        cpu_reg.b = 0x33;
        cpu_reg.c = 0x34;
        cpu_reg.d = 0x35;
        cpu_reg.e = 0x36;
        cpu_reg.h = 0x37;
        cpu_reg.l = 0x38;

        assert_eq!(0x3334, cpu_reg.get_bc());
        assert_eq!(0x3536, cpu_reg.get_de());
        assert_eq!(0x3738, cpu_reg.get_hl());
    }
}
