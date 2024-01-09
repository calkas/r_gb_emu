use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::address;

mod joypad_state_register {
    pub const ALL_KEYS_NOT_PRESSED: u8 = 0xFF;
    pub const BUTTONS_MODE_REQUEST: u8 = 0xDF;
    pub const D_PAD_MODE_REQUEST: u8 = 0xEF;
    pub const KEY_0: u8 = 0xFE;
    pub const KEY_1: u8 = 0xFD;
    pub const KEY_2: u8 = 0xFB;
    pub const KEY_3: u8 = 0xF7;
}

#[derive(PartialEq, Clone, Copy)]
pub enum GameBoyKeys {
    Right,
    Left,
    Down,
    Up,
    A,
    B,
    Select,
    Start,
}
pub struct JoypadInput {
    data_register: u8,
    pub interrupt_req: bool,
}

impl JoypadInput {
    pub fn default() -> Self {
        JoypadInput {
            data_register: joypad_state_register::ALL_KEYS_NOT_PRESSED,
            interrupt_req: false,
        }
    }

    fn get_select_mode_value(&self, key: GameBoyKeys) -> u8 {
        let select_button = [
            GameBoyKeys::A,
            GameBoyKeys::B,
            GameBoyKeys::Start,
            GameBoyKeys::Select,
        ];

        let select_dpad = [
            GameBoyKeys::Right,
            GameBoyKeys::Left,
            GameBoyKeys::Down,
            GameBoyKeys::Up,
        ];

        if select_button.contains(&key) {
            return joypad_state_register::BUTTONS_MODE_REQUEST;
        } else if select_dpad.contains(&key) {
            return joypad_state_register::D_PAD_MODE_REQUEST;
        } else {
            panic!("[JOYPAD ERROR] Unsupported key");
        }
    }

    fn get_key_value(&self, key: GameBoyKeys) -> u8 {
        match key {
            GameBoyKeys::A | GameBoyKeys::Right => joypad_state_register::KEY_0,
            GameBoyKeys::B | GameBoyKeys::Left => joypad_state_register::KEY_1,
            GameBoyKeys::Select | GameBoyKeys::Up => joypad_state_register::KEY_2,
            GameBoyKeys::Start | GameBoyKeys::Down => joypad_state_register::KEY_3,
        }
    }

    pub fn key_pressed(&mut self, key: GameBoyKeys) {
        let mode = self.get_select_mode_value(key);
        let key_val = self.get_key_value(key);
        let mut new_key_value = self.data_register;
        new_key_value &= mode & key_val;
        self.update(new_key_value);
    }
    pub fn key_released(&mut self, key: GameBoyKeys) {
        let mode = self.get_select_mode_value(key);
        let key_val = self.get_key_value(key);
        self.data_register |= !mode | !key_val;
    }

    fn update(&mut self, new_key_value: u8) {
        // only request interupt if the button just pressed is
        if self.data_register == joypad_state_register::ALL_KEYS_NOT_PRESSED {
            if self.data_register != new_key_value {
                self.interrupt_req = true;
            }
        } else {
            //But dpad should be treated different
            if !new_key_value & !joypad_state_register::D_PAD_MODE_REQUEST
                == !joypad_state_register::D_PAD_MODE_REQUEST
            {
                self.interrupt_req = true;
            }
        }
        self.data_register = new_key_value;
    }
}

impl HardwareAccessible for JoypadInput {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            address::io_hardware_register::JOYPAD_INPUT => self.data_register,
            _ => panic!(
                "[JOYPAD ERROR][Read] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            address::io_hardware_register::JOYPAD_INPUT => self.update(data | 0xC0),
            _ => panic!(
                "[JOYPAD ERROR][Write] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }
}
#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn button_keys_test() {
        let mut joypad = JoypadInput::default();

        let button_keys = [
            (GameBoyKeys::A, joypad_state_register::KEY_0),
            (GameBoyKeys::B, joypad_state_register::KEY_1),
            (GameBoyKeys::Select, joypad_state_register::KEY_2),
            (GameBoyKeys::Start, joypad_state_register::KEY_3),
        ];

        for key in button_keys {
            joypad.key_pressed(key.0);
            assert_eq!(
                key.1 & joypad_state_register::BUTTONS_MODE_REQUEST,
                joypad.data_register
            );
            joypad.key_released(key.0);
        }
    }

    #[test]
    fn dpad_keys_test() {
        let mut joypad = JoypadInput::default();

        let dpad_keys = [
            (GameBoyKeys::Right, joypad_state_register::KEY_0),
            (GameBoyKeys::Left, joypad_state_register::KEY_1),
            (GameBoyKeys::Up, joypad_state_register::KEY_2),
            (GameBoyKeys::Down, joypad_state_register::KEY_3),
        ];

        for key in dpad_keys {
            joypad.key_pressed(key.0);
            assert_eq!(
                key.1 & joypad_state_register::D_PAD_MODE_REQUEST,
                joypad.data_register
            );
            joypad.key_released(key.0);
        }
    }

    #[test]
    fn interupt_button_select_test() {
        let mut joypad = JoypadInput::default();

        assert_eq!(
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
            joypad.data_register
        );

        // Press A
        joypad.key_pressed(GameBoyKeys::A);
        assert!(joypad.interrupt_req == true);
        assert_eq!(0xDE, joypad.data_register);

        joypad.interrupt_req = false;

        // Press A again - should nothing happen
        joypad.key_pressed(GameBoyKeys::A);
        assert!(joypad.interrupt_req == false);

        // Release A
        joypad.key_released(GameBoyKeys::A);
        assert_eq!(
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
            joypad.data_register
        );

        // Press A
        joypad.key_pressed(GameBoyKeys::A);
        assert!(joypad.interrupt_req == true);
        assert_eq!(0xDE, joypad.data_register);
    }

    #[test]
    fn interupt_d_pad_select_test() {
        let mut joypad = JoypadInput::default();

        assert_eq!(
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
            joypad.data_register
        );

        // Press Up
        joypad.key_pressed(GameBoyKeys::Up);
        assert!(joypad.interrupt_req == true);
        assert_eq!(0xEB, joypad.data_register);

        joypad.interrupt_req = false;

        // Press Up again - should be interrupt
        joypad.key_pressed(GameBoyKeys::Up);
        assert!(joypad.interrupt_req == true);

        joypad.interrupt_req = false;

        // Press Up release
        joypad.key_released(GameBoyKeys::Up);
        assert!(joypad.interrupt_req == false);

        assert_eq!(
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
            joypad.data_register
        );
    }

    #[test]
    fn read_write_test() {
        let mut joypad = JoypadInput::default();
        assert_eq!(
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
            joypad.read_byte_from_hardware_register(address::io_hardware_register::JOYPAD_INPUT)
        );

        let exp_key_value =
            joypad_state_register::KEY_0 & joypad_state_register::D_PAD_MODE_REQUEST;
        joypad.write_byte_to_hardware_register(
            address::io_hardware_register::JOYPAD_INPUT,
            exp_key_value,
        );

        assert_eq!(
            exp_key_value,
            joypad.read_byte_from_hardware_register(address::io_hardware_register::JOYPAD_INPUT)
        );
    }
}
