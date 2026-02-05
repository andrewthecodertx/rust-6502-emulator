//! Transfer instructions: TAX, TAY, TXA, TYA, TSX, TXS

use crate::bus::Bus;
use crate::cpu::Cpu;

impl<B: Bus> Cpu<B> {
    /// TAX - Transfer Accumulator to X
    pub fn tax(&mut self) {
        self.x = self.a;
        self.status.update_zero_negative(self.x);
    }

    /// TAY - Transfer Accumulator to Y
    pub fn tay(&mut self) {
        self.y = self.a;
        self.status.update_zero_negative(self.y);
    }

    /// TXA - Transfer X to Accumulator
    pub fn txa(&mut self) {
        self.a = self.x;
        self.status.update_zero_negative(self.a);
    }

    /// TYA - Transfer Y to Accumulator
    pub fn tya(&mut self) {
        self.a = self.y;
        self.status.update_zero_negative(self.a);
    }

    /// TSX - Transfer Stack Pointer to X
    pub fn tsx(&mut self) {
        self.x = self.sp;
        self.status.update_zero_negative(self.x);
    }

    /// TXS - Transfer X to Stack Pointer
    /// Note: Does NOT affect flags
    pub fn txs(&mut self) {
        self.sp = self.x;
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
    fn test_tax() {
        // LDA #$42, TAX
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0xAA]);
        cpu.execute_instruction(); // LDA
        cpu.execute_instruction(); // TAX

        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_tax_zero() {
        // LDA #$00, TAX
        let mut cpu = setup_cpu(&[0xA9u8, 0x00, 0xAA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_tax_negative() {
        // LDA #$80, TAX
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0xAA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_tay() {
        // LDA #$42, TAY
        let mut cpu = setup_cpu(&[0xA9u8, 0x42, 0xA8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_txa() {
        // LDX #$42, TXA
        let mut cpu = setup_cpu(&[0xA2u8, 0x42, 0x8A]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tya() {
        // LDY #$42, TYA
        let mut cpu = setup_cpu(&[0xA0u8, 0x42, 0x98]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tsx() {
        // TSX (should transfer SP to X)
        let mut cpu = setup_cpu(&[0xBA]);
        let sp_before = cpu.sp;
        cpu.execute_instruction();

        assert_eq!(cpu.x, sp_before);
    }

    #[test]
    fn test_txs() {
        // LDX #$42, TXS
        let mut cpu = setup_cpu(&[0xA2u8, 0x42, 0x9A]);
        cpu.execute_instruction(); // LDX
        cpu.execute_instruction(); // TXS

        assert_eq!(cpu.sp, 0x42);
    }
}
