//! Load and Store instructions: LDA, LDX, LDY, STA, STX, STY

use crate::bus::Bus;
use crate::cpu::Cpu;

impl<B: Bus> Cpu<B> {
    /// LDA - Load Accumulator
    /// Loads a value into the accumulator and sets Z and N flags
    pub fn lda(&mut self, value: u8) {
        self.a = value;
        self.status.update_zero_negative(value);
    }

    /// LDX - Load X Register
    /// Loads a value into the X register and sets Z and N flags
    pub fn ldx(&mut self, value: u8) {
        self.x = value;
        self.status.update_zero_negative(value);
    }

    /// LDY - Load Y Register
    /// Loads a value into the Y register and sets Z and N flags
    pub fn ldy(&mut self, value: u8) {
        self.y = value;
        self.status.update_zero_negative(value);
    }

    /// STA - Store Accumulator
    /// Returns the value of the accumulator to be stored
    pub fn sta(&self) -> u8 {
        self.a
    }

    /// STX - Store X Register
    /// Returns the value of X to be stored
    pub fn stx(&self) -> u8 {
        self.x
    }

    /// STY - Store Y Register
    /// Returns the value of Y to be stored
    pub fn sty(&self) -> u8 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::{Bus, SimpleBus};
    use crate::Cpu;
    use crate::status::Flag;

    #[test]
    fn test_lda_immediate() {
        let mut bus = SimpleBus::new();
        // LDA #$42
        bus.load(0x8000, &[0xA9, 0x42]);
        // Set reset vector
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x42);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_lda_sets_zero_flag() {
        let mut bus = SimpleBus::new();
        // LDA #$00
        bus.load(0x8000, &[0xA9, 0x00]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_lda_sets_negative_flag() {
        let mut bus = SimpleBus::new();
        // LDA #$80 (bit 7 set = negative)
        bus.load(0x8000, &[0xA9, 0x80]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_lda_zero_page() {
        let mut bus = SimpleBus::new();
        // Store value at zero page $42
        bus.write(0x42, 0xAB);
        // LDA $42
        bus.load(0x8000, &[0xA5, 0x42]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0xAB);
    }

    #[test]
    fn test_lda_absolute() {
        let mut bus = SimpleBus::new();
        // Store value at $1234
        bus.write(0x1234, 0xCD);
        // LDA $1234
        bus.load(0x8000, &[0xAD, 0x34, 0x12]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0xCD);
    }

    #[test]
    fn test_ldx_immediate() {
        let mut bus = SimpleBus::new();
        // LDX #$42
        bus.load(0x8000, &[0xA2, 0x42]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_ldy_immediate() {
        let mut bus = SimpleBus::new();
        // LDY #$42
        bus.load(0x8000, &[0xA0, 0x42]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_sta_zero_page() {
        let mut bus = SimpleBus::new();
        // LDA #$42, STA $10
        bus.load(0x8000, &[0xA9, 0x42, 0x85, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // STA

        assert_eq!(cpu.bus.read(0x10), 0x42);
    }

    #[test]
    fn test_sta_absolute() {
        let mut bus = SimpleBus::new();
        // LDA #$42, STA $1234
        bus.load(0x8000, &[0xA9, 0x42, 0x8D, 0x34, 0x12]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // STA

        assert_eq!(cpu.bus.read(0x1234), 0x42);
    }

    #[test]
    fn test_stx_zero_page() {
        let mut bus = SimpleBus::new();
        // LDX #$42, STX $10
        bus.load(0x8000, &[0xA2, 0x42, 0x86, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction(); // LDX
        cpu.execute_instruction(); // STX

        assert_eq!(cpu.bus.read(0x10), 0x42);
    }

    #[test]
    fn test_sty_zero_page() {
        let mut bus = SimpleBus::new();
        // LDY #$42, STY $10
        bus.load(0x8000, &[0xA0, 0x42, 0x84, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction(); // LDY
        cpu.execute_instruction(); // STY

        assert_eq!(cpu.bus.read(0x10), 0x42);
    }
}
