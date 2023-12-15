use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::address;

/// # SerialDataTransfer
///
/// Only for blargg_tests
///
/// Serial Link: The serial link works one character at a time.
/// If you detect a value of 0x81 written to address 0xFF02, then log the content of address 0xFF01
#[derive(Default)]
pub struct SerialDataTransfer {
    data: u8,
    control: u8,
    pub test_out_data: Vec<char>,
    interrupt_req: bool,
}

impl SerialDataTransfer {
    fn write_data_to_test_buff_when_required(&mut self, control_data: u8) {
        if control_data == 0x81 {
            self.test_out_data.push(self.data as char);
            self.interrupt_req = true;
        }
    }
}

impl HardwareAccessible for SerialDataTransfer {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            address::SERIAL_DATA_REGISTER => self.data,
            address::SERIAL_CONTROL_REGISTER => self.control,
            _ => panic!(
                "[SERIAL ERROR][Read] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }

    fn write_byte_to_hardware_register(&mut self, address: u16, data: u8) {
        match address {
            address::SERIAL_DATA_REGISTER => {
                self.data = data;
            }
            address::SERIAL_CONTROL_REGISTER => {
                self.control = data;
                self.write_data_to_test_buff_when_required(data);
            }
            _ => panic!(
                "[SERIAL ERROR][Write] Unsupported address: [{:#06x?}]",
                address
            ),
        }
    }
}

impl IoWorkingCycle for SerialDataTransfer {
    fn is_interrupt(&self) -> bool {
        self.interrupt_req
    }

    fn reset_interrupt(&mut self) {
        self.interrupt_req = false;
    }

    fn next(&mut self, _cycle: u32) {}
}
#[cfg(test)]
mod ut {
    use super::*;

    #[test]
    fn serial_output_data_test() {
        let mut serial = SerialDataTransfer::default();
        let exp_word: [char; 6] = ['S', 'E', 'R', 'I', 'A', 'L'];

        for e in exp_word {
            serial.write_byte_to_hardware_register(address::SERIAL_DATA_REGISTER, e as u8);
            assert_eq!(
                e as u8,
                serial.read_byte_from_hardware_register(address::SERIAL_DATA_REGISTER)
            );

            serial.write_byte_to_hardware_register(address::SERIAL_CONTROL_REGISTER, 0x81);
        }

        assert_eq!(exp_word.len(), serial.test_out_data.len());

        for i in 0..exp_word.len() {
            assert_eq!(exp_word[i], serial.test_out_data[i]);
        }
    }
}
