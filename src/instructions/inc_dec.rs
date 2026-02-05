//! Increment and Decrement instructions: INC, DEC, INX, DEX, INY, DEY

use crate::bus::Bus;
use crate::cpu::Cpu;

impl<B: Bus> Cpu<B> {
    /// INC - Increment Memory
    /// M = M + 1
    /// Returns the incremented value to be written back to memory
    /// Affects: N, Z
    pub fn inc_mem(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.status.update_zero_negative(result);
        result
    }

    /// DEC - Decrement Memory
    /// M = M - 1
    /// Returns the decremented value to be written back to memory
    /// Affects: N, Z
    pub fn dec_mem(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.status.update_zero_negative(result);
        result
    }

    /// INX - Increment X Register
    /// X = X + 1
    /// Affects: N, Z
    pub fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.status.update_zero_negative(self.x);
    }

    /// DEX - Decrement X Register
    /// X = X - 1
    /// Affects: N, Z
    pub fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.status.update_zero_negative(self.x);
    }

    /// INY - Increment Y Register
    /// Y = Y + 1
    /// Affects: N, Z
    pub fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.status.update_zero_negative(self.y);
    }

    /// DEY - Decrement Y Register
    /// Y = Y - 1
    /// Affects: N, Z
    pub fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.status.update_zero_negative(self.y);
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

    // INX Tests
    #[test]
    fn test_inx() {
        // LDX #$41, INX = $42
        let mut cpu = setup_cpu(&[0xA2u8, 0x41, 0xE8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.status.get(Flag::Zero));
        assert!(!cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_inx_wraps() {
        // LDX #$FF, INX = $00
        let mut cpu = setup_cpu(&[0xA2u8, 0xFF, 0xE8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    #[test]
    fn test_inx_sets_negative() {
        // LDX #$7F, INX = $80
        let mut cpu = setup_cpu(&[0xA2u8, 0x7F, 0xE8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.status.get(Flag::Negative));
    }

    // DEX Tests
    #[test]
    fn test_dex() {
        // LDX #$42, DEX = $41
        let mut cpu = setup_cpu(&[0xA2u8, 0x42, 0xCA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x41);
    }

    #[test]
    fn test_dex_wraps() {
        // LDX #$00, DEX = $FF
        let mut cpu = setup_cpu(&[0xA2u8, 0x00, 0xCA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0xFF);
        assert!(cpu.status.get(Flag::Negative));
    }

    #[test]
    fn test_dex_sets_zero() {
        // LDX #$01, DEX = $00
        let mut cpu = setup_cpu(&[0xA2u8, 0x01, 0xCA]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.x, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    // INY Tests
    #[test]
    fn test_iny() {
        // LDY #$41, INY = $42
        let mut cpu = setup_cpu(&[0xA0u8, 0x41, 0xC8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_iny_wraps() {
        // LDY #$FF, INY = $00
        let mut cpu = setup_cpu(&[0xA0u8, 0xFF, 0xC8]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    // DEY Tests
    #[test]
    fn test_dey() {
        // LDY #$42, DEY = $41
        let mut cpu = setup_cpu(&[0xA0u8, 0x42, 0x88]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0x41);
    }

    #[test]
    fn test_dey_wraps() {
        // LDY #$00, DEY = $FF
        let mut cpu = setup_cpu(&[0xA0u8, 0x00, 0x88]);
        cpu.execute_instruction();
        cpu.execute_instruction();

        assert_eq!(cpu.y, 0xFF);
        assert!(cpu.status.get(Flag::Negative));
    }

    // INC Memory Tests
    #[test]
    fn test_inc_zero_page() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x41);
        // INC $10
        bus.load(0x8000, &[0xE6, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.bus.read(0x10), 0x42);
    }

    #[test]
    fn test_inc_wraps() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0xFF);
        // INC $10
        bus.load(0x8000, &[0xE6, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.bus.read(0x10), 0x00);
        assert!(cpu.status.get(Flag::Zero));
    }

    // DEC Memory Tests
    #[test]
    fn test_dec_zero_page() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x42);
        // DEC $10
        bus.load(0x8000, &[0xC6, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.bus.read(0x10), 0x41);
    }

    #[test]
    fn test_dec_wraps() {
        let mut bus = SimpleBus::new();
        bus.write(0x10, 0x00);
        // DEC $10
        bus.load(0x8000, &[0xC6, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.bus.read(0x10), 0xFF);
        assert!(cpu.status.get(Flag::Negative));
    }
}
