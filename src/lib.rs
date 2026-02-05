//! MOS 6502 CPU Emulator
//!
//! A cycle-accurate emulator for the MOS Technology 6502 microprocessor,
//! commonly used in systems like the NES, Apple II, and Commodore 64.

pub mod bus;
pub mod status;
pub mod cpu;
pub mod addressing;
pub mod instructions;

pub use bus::Bus;
pub use status::StatusRegister;
pub use cpu::Cpu;
