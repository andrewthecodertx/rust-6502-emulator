//! 6502 Processor Status Register (P register)
//!
//! The status register contains 8 flags that indicate various CPU states:
//! - N (Negative): Set if bit 7 of the result is 1
//! - V (Overflow): Set if signed arithmetic overflow occurred
//! - U (Unused): Always set to 1
//! - B (Break): Set if BRK instruction caused the interrupt
//! - D (Decimal): Set to enable BCD arithmetic mode
//! - I (Interrupt Disable): Set to disable IRQ interrupts
//! - Z (Zero): Set if result is zero
//! - C (Carry): Set if carry/borrow occurred
//!
//! Format: NV-BDIZC (bit 7 to bit 0)

/// Flag bit positions in the status register
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Flag {
    Carry = 0,
    Zero = 1,
    InterruptDisable = 2,
    DecimalMode = 3,
    Break = 4,
    Unused = 5,
    Overflow = 6,
    Negative = 7,
}

/// The 6502 processor status register
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusRegister {
    flags: u8,
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusRegister {
    const INITIAL_VALUE: u8 = 0b00110100;

    pub fn new() -> Self {
        Self {
            flags: Self::INITIAL_VALUE,
        }
    }

    #[inline]
    pub fn set(&mut self, flag: Flag, value: bool) {
        if value {
            self.flags |= 1 << (flag as u8);
        } else {
            self.flags &= !(1 << (flag as u8));
        }
    }

    #[inline]
    pub fn get(&self, flag: Flag) -> bool {
        (self.flags & (1 << (flag as u8))) != 0
    }

    #[inline]
    pub fn to_byte(&self) -> u8 {
        self.flags
    }

    #[inline]
    pub fn from_byte(&mut self, value: u8) {
        self.flags = value;
    }

    /// Update the Zero and Negative flags based on a value
    /// This is a common operation after many instructions
    #[inline]
    pub fn update_zero_negative(&mut self, value: u8) {
        self.set(Flag::Zero, value == 0);
        self.set(Flag::Negative, (value & 0x80) != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let status = StatusRegister::new();

        assert!(status.get(Flag::Unused));
        assert!(status.get(Flag::InterruptDisable));
        assert!(status.get(Flag::Break));
        assert!(!status.get(Flag::Carry));
        assert!(!status.get(Flag::Zero));
        assert!(!status.get(Flag::DecimalMode));
        assert!(!status.get(Flag::Overflow));
        assert!(!status.get(Flag::Negative));

        assert_eq!(status.to_byte(), 0b00110100);
    }

    #[test]
    fn test_set_and_get_flags() {
        let mut status = StatusRegister::new();

        status.set(Flag::Carry, true);
        assert!(status.get(Flag::Carry));

        status.set(Flag::Carry, false);
        assert!(!status.get(Flag::Carry));

        status.set(Flag::Zero, true);
        status.set(Flag::Negative, true);
        assert!(status.get(Flag::Zero));
        assert!(status.get(Flag::Negative));
    }

    #[test]
    fn test_to_byte_and_from_byte() {
        let mut status = StatusRegister::new();

        status.from_byte(0b11010101);
        assert_eq!(status.to_byte(), 0b11010101);

        assert!(status.get(Flag::Carry));
        assert!(!status.get(Flag::Zero));
        assert!(status.get(Flag::InterruptDisable));
        assert!(!status.get(Flag::DecimalMode));
        assert!(status.get(Flag::Break));
        assert!(!status.get(Flag::Unused));
        assert!(status.get(Flag::Overflow));
        assert!(status.get(Flag::Negative));
    }

    #[test]
    fn test_update_zero_negative() {
        let mut status = StatusRegister::new();

        status.update_zero_negative(0);
        assert!(status.get(Flag::Zero));
        assert!(!status.get(Flag::Negative));

        status.update_zero_negative(0x42);
        assert!(!status.get(Flag::Zero));
        assert!(!status.get(Flag::Negative));

        status.update_zero_negative(0x80);
        assert!(!status.get(Flag::Zero));
        assert!(status.get(Flag::Negative));

        status.update_zero_negative(0xFF);
        assert!(!status.get(Flag::Zero));
        assert!(status.get(Flag::Negative));
    }
}
