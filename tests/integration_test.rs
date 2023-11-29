use r_gb_emu::cpu::Cpu;
use r_gb_emu::iommu::IOMMU;
use r_gb_emu::peripheral::serial::SerialDataTransfer;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[test]
fn cpu_instruction_behavior_test() {
    // Main test for cpu_instrs.gb
    // At this point, I had not implemented MBC1 yet, so I couldn’t load it
}
