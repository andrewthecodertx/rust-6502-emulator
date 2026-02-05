//! Stack instructions: PHA, PLA, PHP, PLP

use crate::bus::Bus;
use crate::cpu::Cpu;

impl<B: Bus> Cpu<B> {
    /// PHA - Push Accumulator
    pub fn pha(&mut self) {
        self.push_byte(self.a);
    }

    /// PLA - Pull Accumulator
    /// Affects: N, Z
    pub fn pla(&mut self) {
        self.a = self.pull_byte();
        self.status.update_zero_negative(self.a);
    }

    /// PHP - Push Processor Status
    /// Note: B flag is always set when pushing via PHP
    pub fn php(&mut self) {
        // PHP always pushes with B flag set
        let status = self.status.to_byte() | 0x10;
        self.push_byte(status);
    }

    /// PLP - Pull Processor Status
    /// Note: B flag is ignored, Unused flag always set
    pub fn plp(&mut self) {
        let status = self.pull_byte();
        // Ignore B flag (bit 4), always set Unused flag (bit 5)
        self.status.from_byte((status & 0xEF) | 0x20);
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

    #[test]
    fn test_pha_and_pla() {
        // LDA #$42, PHA, LDA #$00, PLA
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0x48, 0xA9, 0x00, 0x68]);
        cpu.execute_instruction(); // LDA #$42
        cpu.execute_instruction(); // PHA
        cpu.execute_instruction(); // LDA #$00
        assert_eq!(cpu.a, 0x00);
        cpu.execute_instruction(); // PLA
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_pla_sets_flags() {
        // LDA #$80, PHA, LDA #$00, PLA
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0x48, 0xA9, 0x00, 0x68]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.status.get(Flag::Negative));
        assert!(!cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_pla_zero_flag() {
        // LDA #$00, PHA, LDA #$42, PLA
        let mut cpu = setup_cpu(&[0xA9u8, 0x00, 0x48, 0xA9, 0x42, 0x68]);
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_php_and_plp() {
        // SEC, PHP, CLC, PLP
        let mut cpu = setup_cpu(&[0x38u8, 0x08, 0x18, 0x28]);
        cpu.execute_instruction(); // SEC
        assert!(cpu.status.get(Flag::Carry));
        cpu.execute_instruction(); // PHP
        cpu.execute_instruction(); // CLC
        assert!(!cpu.status.get(Flag::Carry));
        cpu.execute_instruction(); // PLP
        assert!(cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_php_sets_break_flag() {
        // PHP pushes status with B flag set
        let mut cpu = setup_cpu(&[0x08]); // PHP
        let sp_before = cpu.sp;
        cpu.execute_instruction();

        // Read the pushed value
        let pushed_status = cpu.bus.read(0x0100 + sp_before as u16);
        assert!((pushed_status & 0x10) != 0, "B flag should be set in pushed status");
    }

    #[test]
    fn test_plp_ignores_break_flag() {
        let mut bus = SimpleBus::new();
        // Push a status with B flag set to stack
        bus.write(0x01FF, 0x10); // Only B flag set
        // Set up SP to point to it and PLP
        bus.load(0x8000, &[0x28]); // PLP
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.sp = 0xFE; // Point to 0x01FF

        cpu.execute_instruction();

        // B flag should not be set in status register
        assert!(!cpu.status.get(Flag::Break));
        // But Unused flag should always be set
        assert!(cpu.status.get(Flag::Unused));
    }

    #[test]
    fn test_stack_operations_modify_sp() {
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0x48]); // LDA #$42, PHA
        let sp_before = cpu.sp;
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // PHA

        assert_eq!(cpu.sp, sp_before.wrapping_sub(1));
    }
}
