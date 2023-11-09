use crate::cpu_data::{ControlFlags, FlagsRegister};

/// # ccf
/// Complement carry flag. If C flag is set, then reset it. If C flag is reset, then set it.
pub fn ccf(flag: &mut FlagsRegister) {
    let old_carry_flag = flag.c;
    flag.h = false;
    flag.n = false;
    flag.c = !old_carry_flag;
}

/// # scf
/// Set Carry flag
pub fn scf(flag: &mut FlagsRegister) {
    flag.h = false;
    flag.n = false;
    flag.c = true;
}

/// # di
/// disable interrupts, IME=0
pub fn di(control_flag: &mut ControlFlags) {
    control_flag.ime = false;
}

/// # ei
/// enable interrupts, IME=1
pub fn ei(control_flag: &mut ControlFlags) {
    control_flag.ime = true;
}

/// # halt
/// halt until interrupt occurs (low power). Set halt flag to true
pub fn halt(control_flag: &mut ControlFlags) {
    control_flag.halted = true;
}

#[cfg(test)]
mod ut {
    use super::*;
    use crate::cpu_data::Registers;

    #[test]
    fn ccf_test() {
        let mut register = Registers::new();
        register.flag.z = true;
        register.flag.h = true;
        register.flag.n = true;
        register.flag.c = true;

        ccf(&mut register.flag);

        assert!(register.flag.z == true);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == false);
    }

    #[test]
    fn scf_test() {
        let mut register = Registers::new();
        register.flag.z = true;
        register.flag.h = true;
        register.flag.n = true;
        register.flag.c = false;

        scf(&mut register.flag);

        assert!(register.flag.z == true);
        assert!(register.flag.n == false);
        assert!(register.flag.h == false);
        assert!(register.flag.c == true);
    }

    #[test]
    fn control_flag_test() {
        let mut cpu_control = ControlFlags::new();

        di(&mut cpu_control);
        assert!(cpu_control.ime == false);

        ei(&mut cpu_control);
        assert!(cpu_control.ime == true);

        halt(&mut cpu_control);
        assert!(cpu_control.halted == true);
    }
}
