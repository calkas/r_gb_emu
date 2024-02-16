use std::collections::HashMap;

use super::HardwareAccessible;
use crate::constants::gb_memory_map::address::io_hardware_register;
use crate::emulator_constants::GameBoyKeys;

mod joypad_state_register {
    pub const ALL_KEYS_NOT_PRESSED: u8 = 0xFF;
    pub const BUTTONS_MODE_REQUEST: u8 = 0xDF;
    pub const D_PAD_MODE_REQUEST: u8 = 0xEF;
    pub const KEY_0_VALUE: u8 = 0xFE;
    pub const KEY_1_VALUE: u8 = 0xFD;
    pub const KEY_2_VALUE: u8 = 0xFB;
    pub const KEY_3_VALUE: u8 = 0xF7;
}

pub struct JoypadInput {
    key_data_register: HashMap<u8, u8>,
    select: u8,
    pub interrupt_req: bool,
}

impl JoypadInput {
    pub fn default() -> Self {
        let mut map: HashMap<u8, u8> = HashMap::new();
        map.insert(
            joypad_state_register::D_PAD_MODE_REQUEST,
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
        );
        map.insert(
            joypad_state_register::BUTTONS_MODE_REQUEST,
            joypad_state_register::ALL_KEYS_NOT_PRESSED,
        );
        JoypadInput {
            key_data_register: map,
            select: 0xFF,
            interrupt_req: false,
        }
    }

    fn get_select_mode(&self, key: GameBoyKeys) -> u8 {
        let select_button = [
            GameBoyKeys::A,
            GameBoyKeys::B,
            GameBoyKeys::Start,
            GameBoyKeys::Select,
        ];

        if select_button.contains(&key) {
            joypad_state_register::BUTTONS_MODE_REQUEST
        } else {
            joypad_state_register::D_PAD_MODE_REQUEST
        }
    }

    fn get_key_value(&self, key: GameBoyKeys) -> u8 {
        match key {
            GameBoyKeys::A | GameBoyKeys::Right => joypad_state_register::KEY_0_VALUE,
            GameBoyKeys::B | GameBoyKeys::Left => joypad_state_register::KEY_1_VALUE,
            GameBoyKeys::Select | GameBoyKeys::Up => joypad_state_register::KEY_2_VALUE,
            GameBoyKeys::Start | GameBoyKeys::Down => joypad_state_register::KEY_3_VALUE,
        }
    }

    pub fn key_pressed(&mut self, key: GameBoyKeys) {
        let key_val = self.get_key_value(key);
        let select_mode = self.get_select_mode(key);

        let old_value = self.key_data_register[&select_mode];
        if old_value == joypad_state_register::ALL_KEYS_NOT_PRESSED
            && key_val != joypad_state_register::ALL_KEYS_NOT_PRESSED
        {
            self.interrupt_req = true;
        }
        *self.key_data_register.get_mut(&select_mode).unwrap() &= key_val;
    }

    pub fn key_released(&mut self, key: GameBoyKeys) {
        let mode = self.get_select_mode(key);
        let key_val = self.get_key_value(key);
        *self.key_data_register.get_mut(&mode).unwrap() |= !key_val;
    }
}

impl HardwareAccessible for JoypadInput {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            io_hardware_register::JOYPAD_INPUT => {
                if self.key_data_register.contains_key(&self.select) {
                    self.select & self.key_data_register[&self.select]
                } else {
                    self.select
                }
            }
            _ => panic!(
                "[JOYPAD ERROR][Read] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            io_hardware_register::JOYPAD_INPUT => self.select = 0xCF | (data & 0x30),
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
            (GameBoyKeys::A, joypad_state_register::KEY_0_VALUE),
            (GameBoyKeys::B, joypad_state_register::KEY_1_VALUE),
            (GameBoyKeys::Select, joypad_state_register::KEY_2_VALUE),
            (GameBoyKeys::Start, joypad_state_register::KEY_3_VALUE),
        ];

        joypad.write_byte_to_hardware_register(
            io_hardware_register::JOYPAD_INPUT,
            joypad_state_register::BUTTONS_MODE_REQUEST,
        );

        for key in button_keys {
            joypad.key_pressed(key.0);

            assert_eq!(
                key.1 & joypad_state_register::BUTTONS_MODE_REQUEST,
                joypad.read_byte_from_hardware_register(io_hardware_register::JOYPAD_INPUT)
            );

            assert!(joypad.interrupt_req == true);

            joypad.key_released(key.0);

            assert_eq!(
                joypad_state_register::ALL_KEYS_NOT_PRESSED
                    & joypad_state_register::BUTTONS_MODE_REQUEST,
                joypad.read_byte_from_hardware_register(io_hardware_register::JOYPAD_INPUT)
            );

            joypad.interrupt_req = false;
        }
    }

    #[test]
    fn dpad_keys_test() {
        let mut joypad = JoypadInput::default();

        let dpad_keys = [
            (GameBoyKeys::Right, joypad_state_register::KEY_0_VALUE),
            (GameBoyKeys::Left, joypad_state_register::KEY_1_VALUE),
            (GameBoyKeys::Up, joypad_state_register::KEY_2_VALUE),
            (GameBoyKeys::Down, joypad_state_register::KEY_3_VALUE),
        ];

        joypad.write_byte_to_hardware_register(
            io_hardware_register::JOYPAD_INPUT,
            joypad_state_register::D_PAD_MODE_REQUEST,
        );

        for key in dpad_keys {
            joypad.key_pressed(key.0);

            assert_eq!(
                key.1 & joypad_state_register::D_PAD_MODE_REQUEST,
                joypad.read_byte_from_hardware_register(io_hardware_register::JOYPAD_INPUT)
            );

            assert!(joypad.interrupt_req == true);

            joypad.key_released(key.0);

            assert_eq!(
                joypad_state_register::ALL_KEYS_NOT_PRESSED
                    & joypad_state_register::D_PAD_MODE_REQUEST,
                joypad.read_byte_from_hardware_register(io_hardware_register::JOYPAD_INPUT)
            );

            joypad.interrupt_req = false;
        }
    }
}
