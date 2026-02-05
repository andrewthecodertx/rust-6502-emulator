//! Shift and Rotate instructions: ASL, LSR, ROL, ROR

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::status::Flag;

impl<B: Bus> Cpu<B> {
    /// ASL - Arithmetic Shift Left (Accumulator)
    /// C <- [76543210] <- 0
    /// Affects: N, Z, C
    pub fn asl_acc(&mut self) {
        let carry = (self.a & 0x80) != 0;
        self.a <<= 1;
        self.status.set(Flag::Carry, carry);
        self.status.update_zero_negative(self.a);
    }

    /// ASL - Arithmetic Shift Left (Memory)
    /// Returns the shifted value to be written back to memory
    pub fn asl_mem(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = value << 1;
        self.status.set(Flag::Carry, carry);
        self.status.update_zero_negative(result);
        result
    }

    /// LSR - Logical Shift Right (Accumulator)
    /// 0 -> [76543210] -> C
    /// Affects: N, Z, C
    pub fn lsr_acc(&mut self) {
        let carry = (self.a & 0x01) != 0;
        self.a >>= 1;
        self.status.set(Flag::Carry, carry);
        self.status.update_zero_negative(self.a);
    }

    /// LSR - Logical Shift Right (Memory)
    /// Returns the shifted value to be written back to memory
    pub fn lsr_mem(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = value >> 1;
        self.status.set(Flag::Carry, carry);
        self.status.update_zero_negative(result);
        result
    }

    /// ROL - Rotate Left (Accumulator)
    /// C <- [76543210] <- C
    /// Affects: N, Z, C
    pub fn rol_acc(&mut self) {
        let old_carry = if self.status.get(Flag::Carry) { 1 } else { 0 };
        let new_carry = (self.a & 0x80) != 0;
        self.a = (self.a << 1) | old_carry;
        self.status.set(Flag::Carry, new_carry);
        self.status.update_zero_negative(self.a);
    }

    /// ROL - Rotate Left (Memory)
    /// Returns the rotated value to be written back to memory
    pub fn rol_mem(&mut self, value: u8) -> u8 {
        let old_carry = if self.status.get(Flag::Carry) { 1 } else { 0 };
        let new_carry = (value & 0x80) != 0;
        let result = (value << 1) | old_carry;
        self.status.set(Flag::Carry, new_carry);
        self.status.update_zero_negative(result);
        result
    }

    /// ROR - Rotate Right (Accumulator)
    /// C -> [76543210] -> C
    /// Affects: N, Z, C
    pub fn ror_acc(&mut self) {
        let old_carry = if self.status.get(Flag::Carry) { 0x80 } else { 0 };
        let new_carry = (self.a & 0x01) != 0;
        self.a = (self.a >> 1) | old_carry;
        self.status.set(Flag::Carry, new_carry);
        self.status.update_zero_negative(self.a);
    }

    /// ROR - Rotate Right (Memory)
    /// Returns the rotated value to be written back to memory
    pub fn ror_mem(&mut self, value: u8) -> u8 {
        let old_carry = if self.status.get(Flag::Carry) { 0x80 } else { 0 };
        let new_carry = (value & 0x01) != 0;
        let result = (value >> 1) | old_carry;
        self.status.set(Flag::Carry, new_carry);
        self.status.update_zero_negative(result);
        result
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

    // ASL Tests
    #[test]
    fn test_asl_accumulator() {
        // LDA #$40, ASL A = $80
        let mut cpu = setup_cpu(&[0xA9u8, 0x40, 0x0A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_asl_sets_carry() {
        // LDA #$80, ASL A = $00 with carry
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0x0A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_asl_zero_page() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x40);
        // ASL $10
        bus.load(0x8000, &[0x06, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.bus.read(0x10), 0x80);
    }

    // LSR Tests
    #[test]
    fn test_lsr_accumulator() {
        // LDA #$02, LSR A = $01
        let mut cpu = setup_cpu(&[0xA9u8, 0x02, 0x4A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x01);
        assert!(!cpu.status.get(Flag::Carry));
        assert!(!cpu.status.get(Flag::Negative)); // LSR always clears N
    }

    #[test]
    fn test_lsr_sets_carry() {
        // LDA #$01, LSR A = $00 with carry
        let mut cpu = setup_cpu(&[0xA9u8, 0x01, 0x4A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_lsr_clears_negative() {
        // LDA #$80, LSR A = $40 (bit 7 shifted out, result is positive)
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0x4A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x40);
        assert!(!cpu.status.get(Flag::Negative));
    }

    // ROL Tests
    #[test]
    fn test_rol_without_carry() {
        // CLC, LDA #$40, ROL A = $80
        let mut cpu = setup_cpu(&[0x18u8, 0xA9, 0x40, 0x2A]);
        cpu.execute_instruction(); // CLC
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // ROL

        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_rol_with_carry_in() {
        // SEC, LDA #$40, ROL A = $81
        let mut cpu = setup_cpu(&[0x38u8, 0xA9, 0x40, 0x2A]);
        cpu.execute_instruction(); // SEC
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // ROL

        assert_eq!(cpu.a, 0x81);
        assert!(!cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_rol_sets_carry() {
        // CLC, LDA #$80, ROL A = $00 with carry
        let mut cpu = setup_cpu(&[0x18u8, 0xA9, 0x80, 0x2A]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Zero));
    }

    // ROR Tests
    #[test]
    fn test_ror_without_carry() {
        // CLC, LDA #$02, ROR A = $01
        let mut cpu = setup_cpu(&[0x18u8, 0xA9, 0x02, 0x6A]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x01);
        assert!(!cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_ror_with_carry_in() {
        // SEC, LDA #$02, ROR A = $81
        let mut cpu = setup_cpu(&[0x38u8, 0xA9, 0x02, 0x6A]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x81);
        assert!(!cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_ror_sets_carry() {
        // CLC, LDA #$01, ROR A = $00 with carry
        let mut cpu = setup_cpu(&[0x18u8, 0xA9, 0x01, 0x6A]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Carry));
        assert!(cpu.status.get(Flag::Zero));
    }
}
