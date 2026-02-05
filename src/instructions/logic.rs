//! Logic instructions: AND, ORA, EOR, BIT

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::status::Flag;

impl<B: Bus> Cpu<B> {
    /// AND - Logical AND
    /// A = A & M
    /// Affects: N, Z
    pub fn and(&mut self, value: u8) {
        self.a &= value;
        self.status.update_zero_negative(self.a);
    }

    /// ORA - Logical OR (Inclusive OR)
    /// A = A | M
    /// Affects: N, Z
    pub fn ora(&mut self, value: u8) {
        self.a |= value;
        self.status.update_zero_negative(self.a);
    }

    /// EOR - Exclusive OR
    /// A = A ^ M
    /// Affects: N, Z
    pub fn eor(&mut self, value: u8) {
        self.a ^= value;
        self.status.update_zero_negative(self.a);
    }

    /// BIT - Bit Test
    /// Z = A & M (but result not stored)
    /// N = bit 7 of M
    /// V = bit 6 of M
    pub fn bit(&mut self, value: u8) {
        let result = self.a & value;
        self.status.set(Flag::Zero, result == 0);
        self.status.set(Flag::Negative, (value & 0x80) != 0);
        self.status.set(Flag::Overflow, (value & 0x40) != 0);
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::{Bus, SimpleBus};
    use crate::Cpu;
    use crate::status::Flag;

    fn setup_cpu(program: &[u8]) -> Cpu<SimpleBus> {
        let mut bus = SimpleBus::new();
        bus.load(0x8000, program);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);
        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu
    }

    // AND Tests
    #[test]
    fn test_and() {
        // LDA #$FF, AND #$0F = $0F
        let mut cpu = setup_cpu(&[0xA9u8, 0xFF, 0x29, 0x0F]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x0F);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_and_zero_result() {
        // LDA #$F0, AND #$0F = $00
        let mut cpu = setup_cpu(&[0xA9u8, 0xF0, 0x29, 0x0F]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_and_negative_result() {
        // LDA #$FF, AND #$80 = $80
        let mut cpu = setup_cpu(&[0xA9u8, 0xFF, 0x29, 0x80]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.status.get(Flag::Negative));
    }

    // ORA Tests
    #[test]
    fn test_ora() {
        // LDA #$F0, ORA #$0F = $FF
        let mut cpu = setup_cpu(&[0xA9u8, 0xF0, 0x09, 0x0F]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0xFF);
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_ora_zero() {
        // LDA #$00, ORA #$00 = $00
        let mut cpu = setup_cpu(&[0xA9u8, 0x00, 0x09, 0x00]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    // EOR Tests
    #[test]
    fn test_eor() {
        // LDA #$FF, EOR #$AA = $55
        let mut cpu = setup_cpu(&[0xA9u8, 0xFF, 0x49, 0xAA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x55);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_eor_same_value() {
        // LDA #$42, EOR #$42 = $00
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0x49, 0x42]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    // BIT Tests
    #[test]
    fn test_bit_zero() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x0F); // Value with no overlap with A
        // LDA #$F0, BIT $10 (A & M = 0)
        bus.load(0x8000, &[0xA9, 0xF0, 0x24, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Zero));
        // A is unchanged
        assert_eq!(cpu.a, 0xF0);
    }

    #[test]
    fn test_bit_sets_negative_from_memory() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x80); // bit 7 set
        // LDA #$FF, BIT $10
        bus.load(0x8000, &[0xA9, 0xFF, 0x24, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_bit_sets_overflow_from_memory() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x40); // bit 6 set
        // LDA #$FF, BIT $10
        bus.load(0x8000, &[0xA9, 0xFF, 0x24, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Overflow));
    }

    #[test]
    fn test_bit_clears_flags_appropriately() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x00); // no bits set
        // LDA #$FF, BIT $10
        bus.load(0x8000, &[0xA9, 0xFF, 0x24, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Zero)); // A & M = 0
        assert!(!cpu.status.get(Flag::Negative)); // bit 7 of M is 0
        assert!(!cpu.status.get(Flag::Overflow)); // bit 6 of M is 0
    }
}
