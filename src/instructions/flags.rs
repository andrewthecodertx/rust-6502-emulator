//! Flag instructions: SEC, CLC, SEI, CLI, SED, CLD, CLV

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::status::Flag;

impl<B: Bus> Cpu<B> {
    /// CLC - Clear Carry Flag
    pub fn clc(&mut self) {
        self.status.set(Flag::Carry, false);
    }

    /// SEC - Set Carry Flag
    pub fn sec(&mut self) {
        self.status.set(Flag::Carry, true);
    }

    /// CLI - Clear Interrupt Disable
    pub fn cli(&mut self) {
        self.status.set(Flag::InterruptDisable, false);
    }

    /// SEI - Set Interrupt Disable
    pub fn sei(&mut self) {
        self.status.set(Flag::InterruptDisable, true);
    }

    /// CLD - Clear Decimal Mode
    pub fn cld(&mut self) {
        self.status.set(Flag::DecimalMode, false);
    }

    /// SED - Set Decimal Mode
    pub fn sed(&mut self) {
        self.status.set(Flag::DecimalMode, true);
    }

    /// CLV - Clear Overflow Flag
    pub fn clv(&mut self) {
        self.status.set(Flag::Overflow, false);
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
    fn test_clc() {
        // SEC, CLC
        let mut cpu = setup_cpu(&[0x38u8, 0x18]);
        cpu.execute_instruction(); // SEC
        assert!(cpu.status.get(Flag::Carry));
        cpu.execute_instruction(); // CLC
        assert!(!cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_sec() {
        // CLC, SEC
        let mut cpu = setup_cpu(&[0x18u8, 0x38]);
        cpu.execute_instruction(); // CLC
        assert!(!cpu.status.get(Flag::Carry));
        cpu.execute_instruction(); // SEC
        assert!(cpu.status.get(Flag::Carry));
    }

    #[test]
    fn test_cli() {
        // SEI, CLI
        let mut cpu = setup_cpu(&[0x78u8, 0x58]);
        cpu.execute_instruction(); // SEI
        assert!(cpu.status.get(Flag::InterruptDisable));
        cpu.execute_instruction(); // CLI
        assert!(!cpu.status.get(Flag::InterruptDisable));
    }

    #[test]
    fn test_sei() {
        // CLI, SEI
        let mut cpu = setup_cpu(&[0x58u8, 0x78]);
        cpu.execute_instruction(); // CLI
        assert!(!cpu.status.get(Flag::InterruptDisable));
        cpu.execute_instruction(); // SEI
        assert!(cpu.status.get(Flag::InterruptDisable));
    }

    #[test]
    fn test_cld() {
        // SED, CLD
        let mut cpu = setup_cpu(&[0xF8u8, 0xD8]);
        cpu.execute_instruction(); // SED
        assert!(cpu.status.get(Flag::DecimalMode));
        cpu.execute_instruction(); // CLD
        assert!(!cpu.status.get(Flag::DecimalMode));
    }

    #[test]
    fn test_sed() {
        // CLD, SED
        let mut cpu = setup_cpu(&[0xD8u8, 0xF8]);
        cpu.execute_instruction(); // CLD
        assert!(!cpu.status.get(Flag::DecimalMode));
        cpu.execute_instruction(); // SED
        assert!(cpu.status.get(Flag::DecimalMode));
    }

    #[test]
    fn test_clv() {
        // We need to set overflow first, then clear it
        // ADC that causes overflow, then CLV
        // 0x7F + 0x01 = 0x80 (overflow)
        let mut cpu = setup_cpu(&[0xA9u8, 0x7F, 0x18, 0x69, 0x01, 0xB8]);
        cpu.execute_instruction(); // LDA #$7F
        cpu.execute_instruction(); // CLC
        cpu.execute_instruction(); // ADC #$01
        assert!(cpu.status.get(Flag::Overflow));
        cpu.execute_instruction(); // CLV
        assert!(!cpu.status.get(Flag::Overflow));
    }
}
