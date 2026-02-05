//! Arithmetic instructions: ADC, SBC, CMP, CPX, CPY

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::status::Flag;

impl<B: Bus> Cpu<B> {
    /// ADC - Add with Carry
    /// A = A + M + C
    /// Affects: N, V, Z, C
    pub fn adc(&mut self, value: u8) {
        let carry = if self.status.get(Flag::Carry) { 1u16 } else { 0u16 };
        let a = self.a as u16;
        let m = value as u16;

        let result = a + m + carry;

        // Overflow: set if sign bit is wrong
        // V = (A^result) & (M^result) & 0x80
        // Overflow occurs when both operands have the same sign,
        // but the result has a different sign
        let overflow = ((self.a ^ result as u8) & (value ^ result as u8) & 0x80) != 0;

        self.a = result as u8;

        self.status.set(Flag::Carry, result > 0xFF);
        self.status.set(Flag::Overflow, overflow);
        self.status.update_zero_negative(self.a);
    }

    /// SBC - Subtract with Carry (Borrow)
    /// A = A - M - (1 - C)
    /// Affects: N, V, Z, C
    pub fn sbc(&mut self, value: u8) {
        // SBC is the same as ADC with the value inverted
        // A - M - B = A + (~M) + C
        self.adc(!value);
    }

    /// CMP - Compare Accumulator
    /// Sets flags as if A - M was performed
    /// Affects: N, Z, C
    pub fn cmp(&mut self, value: u8) {
        self.compare(self.a, value);
    }

    /// CPX - Compare X Register
    /// Sets flags as if X - M was performed
    /// Affects: N, Z, C
    pub fn cpx(&mut self, value: u8) {
        self.compare(self.x, value);
    }

    /// CPY - Compare Y Register
    /// Sets flags as if Y - M was performed
    /// Affects: N, Z, C
    pub fn cpy(&mut self, value: u8) {
        self.compare(self.y, value);
    }

    /// Internal compare operation
    /// Sets C if register >= value
    /// Sets Z if register == value
    /// Sets N if bit 7 of (register - value) is set
    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);

        self.status.set(Flag::Carry, register >= value);
        self.status.update_zero_negative(result);
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

    // ADC Tests
    #[test]
    fn test_adc_simple() {
        // LDA #$10, CLC, ADC #$20 = $30
        let mut cpu = setup_cpu(&[0xA9u8, 0x10, 0x18, 0x69, 0x20]);
        cpu.execute_instruction(); // LDA #$10
        cpu.execute_instruction(); // CLC
        cpu.execute_instruction(); // ADC #$20

        assert_eq!(cpu.a, 0x30);
        assert!(!cpu.status.get(Flag::Carry));
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
        assert!(!cpu.status.get(Flag::Overflow));
    }

    #[test]
    fn test_adc_with_carry_in() {
        // LDA #$10, SEC, ADC #$20 = $31 (with carry)
        let mut cpu = setup_cpu(&[0xA9u8, 0x10, 0x38, 0x69, 0x20]);
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // SEC
        cpu.execute_instruction(); // ADC

        assert_eq!(cpu.a, 0x31);
    }

    #[test]
    fn test_adc_causes_carry() {
        // LDA #$FF, CLC, ADC #$01 = $00 with carry
        let mut cpu = setup_cpu(&[0xA9u8, 0xFF, 0x18, 0x69, 0x01]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_adc_overflow_positive_to_negative() {
        // 0x7F + 0x01 = 0x80 (127 + 1 = -128 in signed)
        // This should set overflow because we went from positive to negative
        let mut cpu = setup_cpu(&[0xA9u8, 0x7F, 0x18, 0x69, 0x01]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.status.get(Flag::Overflow));
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_adc_overflow_negative_to_positive() {
        // 0x80 + 0x80 = 0x00 with carry (-128 + -128 = 0 in signed, overflow)
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0x18, 0x69, 0x80]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Overflow));
        assert!(cpu.status.get(Flag::Zero));
    }

    // SBC Tests
    #[test]
    fn test_sbc_simple() {
        // LDA #$30, SEC, SBC #$10 = $20
        let mut cpu = setup_cpu(&[0xA9u8, 0x30, 0x38, 0xE9, 0x10]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x20);
        assert!(cpu.status.get(Flag::Carry)); // No borrow needed
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_sbc_with_borrow() {
        // LDA #$30, CLC, SBC #$10 = $1F (because of borrow)
        let mut cpu = setup_cpu(&[0xA9u8, 0x30, 0x18, 0xE9, 0x10]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x1F);
        assert!(cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_sbc_causes_borrow() {
        // LDA #$00, SEC, SBC #$01 = $FF (0 - 1 = -1 = 0xFF)
        let mut cpu = setup_cpu(&[0xA9u8, 0x00, 0x38, 0xE9, 0x01]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0xFF);
        assert!(!cpu.status.get(Flag::Carry)); // Borrow occurred
        assert!(cpu.status.get(Flag::Negative));
    }

    // CMP Tests
    #[test]
    fn test_cmp_equal() {
        // LDA #$42, CMP #$42
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0xC9, 0x42]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Zero));
        assert!(cpu.status.get(Flag::Carry)); // A >= M
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_cmp_greater() {
        // LDA #$50, CMP #$40
        let mut cpu = setup_cpu(&[0xA9u8, 0x50, 0xC9, 0x40]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(!cpu.status.get(Flag::Zero));
        assert!(cpu.status.get(Flag::Carry)); // A >= M
        assert!(!cpu.status.get(Flag::Negative)); // 0x10 is positive
    }

    #[test]
    fn test_cmp_less() {
        // LDA #$40, CMP #$50
        let mut cpu = setup_cpu(&[0xA9u8, 0x40, 0xC9, 0x50]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Carry)); // A < M
        assert!(cpu.status.get(Flag::Negative)); // 0xF0 has bit 7 set
    }

    #[test]
    fn test_cpx() {
        // LDX #$42, CPX #$42
        let mut cpu = setup_cpu(&[0xA2u8, 0x42, 0xE0, 0x42]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Zero));
        assert!(cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_cpy() {
        // LDY #$42, CPY #$42
        let mut cpu = setup_cpu(&[0xA0u8, 0x42, 0xC0, 0x42]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert!(cpu.status.get(Flag::Zero));
        assert!(cpu.status.get(Flag::Carry));
    }
}
