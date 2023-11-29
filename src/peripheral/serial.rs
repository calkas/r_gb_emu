use super::{HardwareAccessible, IoWorkingCycle};
use crate::constants::gb_memory_map::{address, memory};

/// # SerialDataTransfer
///
/// Only for blargg_tests
///
/// Serial Link: The serial link works one character at a time.
/// If you detect a value of 0x81 written to address 0xFF02, then log the content of address 0xFF01
pub struct SerialDataTransfer {
    data: u8,
    control: u8,
    test_data: Vec<u8>,
    interrupt_req: bool,
}

impl SerialDataTransfer {
    pub fn new() -> Self {
        SerialDataTransfer {
            data: 0,
            control: 0,
            test_data: Vec::new(),
            interrupt_req: false,
        }
    }

    fn write_data_to_test_buff_when_required(&mut self, control_data: u8) {
        if control_data & 0x81 == 0x81 {
            self.test_data.push(self.data);
            self.interrupt_req = true;
        }
    }

    pub fn get_test_buff(&self) -> &[u8] {
        &self.test_data
    }
}

impl HardwareAccessible for SerialDataTransfer {
    fn read_byte_from_hardware_register(&self, address: u16) -> u8 {
        match address {
            address::SERIAL_DATA_REGISTER => self.data,
            address::SERIAL_CONTROL_REGISTER => self.control,
            _ => panic!(
                "Read - This address [{:#02x?}] is not for SerialDataTransfer",
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
                "Write - This address [{:#02x?}] is not for SerialDataTransfer",
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

    fn run_cycle(&mut self, cycle: u32) {}
}
