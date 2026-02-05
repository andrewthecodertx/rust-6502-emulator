//! Flow Control instructions: JMP, JSR, RTS, BRK, RTI, and branches

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::status::Flag;

impl<B: Bus> Cpu<B> {
    /// JMP - Jump to address
    /// PC = address
    pub fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    /// JSR - Jump to Subroutine
    /// Push (PC - 1), PC = address
    pub fn jsr(&mut self, address: u16) {
        // Push return address - 1 (RTS adds 1)
        let return_addr = self.pc.wrapping_sub(1);
        self.push_word(return_addr);
        self.pc = address;
    }

    /// RTS - Return from Subroutine
    /// Pull PC, PC = PC + 1
    pub fn rts(&mut self) {
        let addr = self.pull_word();
        self.pc = addr.wrapping_add(1);
    }

    /// BRK - Force Interrupt
    /// Push PC+1, Push Status (with B set), PC = IRQ vector
    pub fn brk(&mut self) {
        // PC has already been incremented past BRK opcode
        // BRK pushes PC+1 (accounting for the padding byte)
        let return_addr = self.pc.wrapping_add(1);
        self.push_word(return_addr);

        // Push status with Break flag set
        let status = self.status.to_byte() | 0x10; // Set B flag
        self.push_byte(status);

        // Set interrupt disable flag
        self.status.set(Flag::InterruptDisable, true);

        // Load IRQ vector
        self.pc = self.read_word(0xFFFE);
    }

    /// RTI - Return from Interrupt
    /// Pull Status, Pull PC
    pub fn rti(&mut self) {
        let status = self.pull_byte();
        // Ignore B flag and always set unused flag
        self.status.from_byte((status & 0xEF) | 0x20);
        self.pc = self.pull_word();
    }

    /// Branch helper - returns additional cycles if branch taken
    /// offset is a signed 8-bit value
    pub fn branch(&mut self, condition: bool, offset: u8) -> u8 {
        if condition {
            let old_pc = self.pc;
            // Convert to signed offset
            let signed_offset = offset as i8;
            self.pc = self.pc.wrapping_add(signed_offset as u16);

            // +1 cycle for taken branch, +1 more if page boundary crossed
            if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
                2 // Page boundary crossed
            } else {
                1 // Same page
            }
        } else {
            0 // Branch not taken
        }
    }

    /// BCC - Branch if Carry Clear
    pub fn bcc(&mut self, offset: u8) -> u8 {
        self.branch(!self.status.get(Flag::Carry), offset)
    }

    /// BCS - Branch if Carry Set
    pub fn bcs(&mut self, offset: u8) -> u8 {
        self.branch(self.status.get(Flag::Carry), offset)
    }

    /// BEQ - Branch if Equal (Zero set)
    pub fn beq(&mut self, offset: u8) -> u8 {
        self.branch(self.status.get(Flag::Zero), offset)
    }

    /// BNE - Branch if Not Equal (Zero clear)
    pub fn bne(&mut self, offset: u8) -> u8 {
        self.branch(!self.status.get(Flag::Zero), offset)
    }

    /// BMI - Branch if Minus (Negative set)
    pub fn bmi(&mut self, offset: u8) -> u8 {
        self.branch(self.status.get(Flag::Negative), offset)
    }

    /// BPL - Branch if Plus (Negative clear)
    pub fn bpl(&mut self, offset: u8) -> u8 {
        self.branch(!self.status.get(Flag::Negative), offset)
    }

    /// BVC - Branch if Overflow Clear
    pub fn bvc(&mut self, offset: u8) -> u8 {
        self.branch(!self.status.get(Flag::Overflow), offset)
    }

    /// BVS - Branch if Overflow Set
    pub fn bvs(&mut self, offset: u8) -> u8 {
        self.branch(self.status.get(Flag::Overflow), offset)
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

    // JMP Tests
    #[test]
    fn test_jmp_absolute() {
        // JMP $8010
        let mut cpu = setup_cpu(&[0x4Cu8, 0x10, 0x80]);
        cpu.execute_instruction();

        assert_eq!(cpu.pc, 0x8010);
    }

    #[test]
    fn test_jmp_indirect() {
        let mut bus = SimpleBus::new();
        // Store target address at $1234
        bus.write(0x1234, 0x00);
        bus.write(0x1235, 0x90);
        // JMP ($1234)
        bus.load(0x8000, &[0x6C, 0x34, 0x12]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.pc, 0x9000);
    }

    #[test]
    fn test_jmp_indirect_page_boundary_bug() {
        let mut bus = SimpleBus::new();
        // NMOS 6502 bug: JMP ($10FF) reads high byte from $1000, not $1100
        bus.write(0x10FF, 0x34); // low byte
        bus.write(0x1000, 0x12); // high byte (due to bug)
        // JMP ($10FF)
        bus.load(0x8000, &[0x6C, 0xFF, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction();

        assert_eq!(cpu.pc, 0x1234);
    }

    // JSR/RTS Tests
    #[test]
    fn test_jsr_and_rts() {
        let mut bus = SimpleBus::new();
        // At $8000: JSR $8010
        // At $8010: RTS
        bus.load(0x8000, &[0x20, 0x10, 0x80]); // JSR $8010
        bus.write(0x8010, 0x60); // RTS
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();

        cpu.execute_instruction(); // JSR
        assert_eq!(cpu.pc, 0x8010);

        cpu.execute_instruction(); // RTS
        assert_eq!(cpu.pc, 0x8003); // Return to instruction after JSR
    }

    // Branch Tests
    #[test]
    fn test_beq_taken() {
        // LDA #$00 (sets Z), BEQ +5
        let mut cpu = setup_cpu(&[0xA9u8, 0x00, 0xF0, 0x05]);
        cpu.execute_instruction(); // LDA
        let pc_before = cpu.pc;
        cpu.execute_instruction(); // BEQ

        // Branch should be taken: PC = PC + 2 + 5 = PC + 7
        // After reading BEQ opcode and offset, PC is at $8004
        // Then branch adds 5: $8004 + 5 = $8009
        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    #[test]
    fn test_beq_not_taken() {
        // LDA #$01 (clears Z), BEQ +5
        let mut cpu = setup_cpu(&[0xA9u8, 0x01, 0xF0, 0x05]);
        cpu.execute_instruction(); // LDA
        let pc_before = cpu.pc;
        cpu.execute_instruction(); // BEQ

        // Branch not taken, PC just advances past instruction
        assert_eq!(cpu.pc, pc_before + 2);
    }

    #[test]
    fn test_bne_taken() {
        // LDA #$01 (clears Z), BNE +5
        let mut cpu = setup_cpu(&[0xA9u8, 0x01, 0xD0, 0x05]);
        cpu.execute_instruction(); // LDA
        let pc_before = cpu.pc;
        cpu.execute_instruction(); // BNE

        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    #[test]
    fn test_branch_backward() {
        // BNE -3 (0xFD = -3 in two's complement)
        let mut bus = SimpleBus::new();
        bus.load(0x8000, &[0xA9, 0x01, 0xD0, 0xFD]); // LDA #$01, BNE -3
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu.execute_instruction(); // LDA
        let pc_before = cpu.pc; // $8002
        cpu.execute_instruction(); // BNE

        // PC after reading opcode+operand: $8004
        // Branch offset -3: $8004 - 3 = $8001
        assert_eq!(cpu.pc, pc_before + 2 - 3);
    }

    #[test]
    fn test_bcc_taken() {
        // CLC, BCC +5
        let mut cpu = setup_cpu(&[0x18u8, 0x90, 0x05]);
        cpu.execute_instruction(); // CLC
        let pc_before = cpu.pc;
        cpu.execute_instruction(); // BCC

        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    #[test]
    fn test_bcs_taken() {
        // SEC, BCS +5
        let mut cpu = setup_cpu(&[0x38u8, 0xB0, 0x05]);
        cpu.execute_instruction(); // SEC
        let pc_before = cpu.pc;
        cpu.execute_instruction(); // BCS

        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    #[test]
    fn test_bpl_taken() {
        // LDA #$01 (positive), BPL +5
        let mut cpu = setup_cpu(&[0xA9u8, 0x01, 0x10, 0x05]);
        cpu.execute_instruction();
        let pc_before = cpu.pc;
        cpu.execute_instruction();

        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    #[test]
    fn test_bmi_taken() {
        // LDA #$80 (negative), BMI +5
        let mut cpu = setup_cpu(&[0xA9u8, 0x80, 0x30, 0x05]);
        cpu.execute_instruction();
        let pc_before = cpu.pc;
        cpu.execute_instruction();

        assert_eq!(cpu.pc, pc_before + 2 + 5);
    }

    // BRK/RTI Tests
    #[test]
    fn test_brk() {
        let mut bus = SimpleBus::new();
        // Set up IRQ vector
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x90); // IRQ handler at $9000
        // BRK
        bus.load(0x8000, &[0x00]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();
        let sp_before = cpu.sp;
        cpu.execute_instruction(); // BRK

        assert_eq!(cpu.pc, 0x9000);
        assert!(cpu.status.get(Flag::InterruptDisable));
        // Stack should have 3 bytes pushed (PCH, PCL, Status)
        assert_eq!(cpu.sp, sp_before.wrapping_sub(3));
    }
}
