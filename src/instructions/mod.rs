//! 6502 Instruction implementations
//!
//! Instructions are organized by category:
//! - Load/Store: LDA, LDX, LDY, STA, STX, STY
//! - Transfer: TAX, TAY, TXA, TYA, TSX, TXS
//! - Arithmetic: ADC, SBC, CMP, CPX, CPY
//! - Logic: AND, ORA, EOR, BIT
//! - Shift/Rotate: ASL, LSR, ROL, ROR
//! - Inc/Dec: INC, DEC, INX, DEX, INY, DEY
//! - Flow Control: JMP, JSR, RTS, BRK, RTI, branches
//! - Stack: PHA, PLA, PHP, PLP
//! - Flags: SEC, CLC, SEI, CLI, SED, CLD, CLV

// These modules add impl blocks to Cpu
mod arithmetic;
mod flags;
mod flow_control;
mod inc_dec;
mod load_store;
mod logic;
mod shift_rotate;
mod stack;
mod transfer;

use crate::addressing::AddressingMode;

/// Opcode definition containing all metadata for an instruction
#[derive(Debug, Clone, Copy)]
pub struct Opcode {
    /// The opcode byte value
    pub code: u8,
    /// Instruction mnemonic (e.g., "LDA", "STA")
    pub mnemonic: &'static str,
    /// Addressing mode for this opcode variant
    pub mode: AddressingMode,
    /// Number of bytes including opcode
    pub bytes: u8,
    /// Base number of cycles
    pub cycles: u8,
    /// Whether this instruction can take an extra cycle on page boundary crossing
    pub page_boundary_cycle: bool,
}

impl Opcode {
    pub const fn new(
        code: u8,
        mnemonic: &'static str,
        mode: AddressingMode,
        bytes: u8,
        cycles: u8,
        page_boundary_cycle: bool,
    ) -> Self {
        Self {
            code,
            mnemonic,
            mode,
            bytes,
            cycles,
            page_boundary_cycle,
        }
    }
}

/// Lookup table for all 256 possible opcodes
/// Invalid opcodes are represented as NOP with 1 cycle
pub static OPCODES: [Opcode; 256] = create_opcode_table();

const fn create_opcode_table() -> [Opcode; 256] {
    // Start with all NOPs (illegal opcodes will be filled in later)
    let illegal = Opcode::new(0x00, "???", AddressingMode::Implied, 1, 2, false);
    let mut table = [illegal; 256];

    // Load instructions
    // LDA - Load Accumulator
    table[0xA9] = Opcode::new(0xA9, "LDA", AddressingMode::Immediate, 2, 2, false);
    table[0xA5] = Opcode::new(0xA5, "LDA", AddressingMode::ZeroPage, 2, 3, false);
    table[0xB5] = Opcode::new(0xB5, "LDA", AddressingMode::ZeroPageX, 2, 4, false);
    table[0xAD] = Opcode::new(0xAD, "LDA", AddressingMode::Absolute, 3, 4, false);
    table[0xBD] = Opcode::new(0xBD, "LDA", AddressingMode::AbsoluteX, 3, 4, true);
    table[0xB9] = Opcode::new(0xB9, "LDA", AddressingMode::AbsoluteY, 3, 4, true);
    table[0xA1] = Opcode::new(0xA1, "LDA", AddressingMode::IndirectX, 2, 6, false);
    table[0xB1] = Opcode::new(0xB1, "LDA", AddressingMode::IndirectY, 2, 5, true);

    // LDX - Load X Register
    table[0xA2] = Opcode::new(0xA2, "LDX", AddressingMode::Immediate, 2, 2, false);
    table[0xA6] = Opcode::new(0xA6, "LDX", AddressingMode::ZeroPage, 2, 3, false);
    table[0xB6] = Opcode::new(0xB6, "LDX", AddressingMode::ZeroPageY, 2, 4, false);
    table[0xAE] = Opcode::new(0xAE, "LDX", AddressingMode::Absolute, 3, 4, false);
    table[0xBE] = Opcode::new(0xBE, "LDX", AddressingMode::AbsoluteY, 3, 4, true);

    // LDY - Load Y Register
    table[0xA0] = Opcode::new(0xA0, "LDY", AddressingMode::Immediate, 2, 2, false);
    table[0xA4] = Opcode::new(0xA4, "LDY", AddressingMode::ZeroPage, 2, 3, false);
    table[0xB4] = Opcode::new(0xB4, "LDY", AddressingMode::ZeroPageX, 2, 4, false);
    table[0xAC] = Opcode::new(0xAC, "LDY", AddressingMode::Absolute, 3, 4, false);
    table[0xBC] = Opcode::new(0xBC, "LDY", AddressingMode::AbsoluteX, 3, 4, true);

    // Store instructions
    // STA - Store Accumulator
    table[0x85] = Opcode::new(0x85, "STA", AddressingMode::ZeroPage, 2, 3, false);
    table[0x95] = Opcode::new(0x95, "STA", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x8D] = Opcode::new(0x8D, "STA", AddressingMode::Absolute, 3, 4, false);
    table[0x9D] = Opcode::new(0x9D, "STA", AddressingMode::AbsoluteX, 3, 5, false);
    table[0x99] = Opcode::new(0x99, "STA", AddressingMode::AbsoluteY, 3, 5, false);
    table[0x81] = Opcode::new(0x81, "STA", AddressingMode::IndirectX, 2, 6, false);
    table[0x91] = Opcode::new(0x91, "STA", AddressingMode::IndirectY, 2, 6, false);

    // STX - Store X Register
    table[0x86] = Opcode::new(0x86, "STX", AddressingMode::ZeroPage, 2, 3, false);
    table[0x96] = Opcode::new(0x96, "STX", AddressingMode::ZeroPageY, 2, 4, false);
    table[0x8E] = Opcode::new(0x8E, "STX", AddressingMode::Absolute, 3, 4, false);

    // STY - Store Y Register
    table[0x84] = Opcode::new(0x84, "STY", AddressingMode::ZeroPage, 2, 3, false);
    table[0x94] = Opcode::new(0x94, "STY", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x8C] = Opcode::new(0x8C, "STY", AddressingMode::Absolute, 3, 4, false);

    // Transfer instructions
    table[0xAA] = Opcode::new(0xAA, "TAX", AddressingMode::Implied, 1, 2, false);
    table[0xA8] = Opcode::new(0xA8, "TAY", AddressingMode::Implied, 1, 2, false);
    table[0x8A] = Opcode::new(0x8A, "TXA", AddressingMode::Implied, 1, 2, false);
    table[0x98] = Opcode::new(0x98, "TYA", AddressingMode::Implied, 1, 2, false);
    table[0xBA] = Opcode::new(0xBA, "TSX", AddressingMode::Implied, 1, 2, false);
    table[0x9A] = Opcode::new(0x9A, "TXS", AddressingMode::Implied, 1, 2, false);

    // Stack instructions
    table[0x48] = Opcode::new(0x48, "PHA", AddressingMode::Implied, 1, 3, false);
    table[0x68] = Opcode::new(0x68, "PLA", AddressingMode::Implied, 1, 4, false);
    table[0x08] = Opcode::new(0x08, "PHP", AddressingMode::Implied, 1, 3, false);
    table[0x28] = Opcode::new(0x28, "PLP", AddressingMode::Implied, 1, 4, false);

    // Arithmetic - ADC
    table[0x69] = Opcode::new(0x69, "ADC", AddressingMode::Immediate, 2, 2, false);
    table[0x65] = Opcode::new(0x65, "ADC", AddressingMode::ZeroPage, 2, 3, false);
    table[0x75] = Opcode::new(0x75, "ADC", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x6D] = Opcode::new(0x6D, "ADC", AddressingMode::Absolute, 3, 4, false);
    table[0x7D] = Opcode::new(0x7D, "ADC", AddressingMode::AbsoluteX, 3, 4, true);
    table[0x79] = Opcode::new(0x79, "ADC", AddressingMode::AbsoluteY, 3, 4, true);
    table[0x61] = Opcode::new(0x61, "ADC", AddressingMode::IndirectX, 2, 6, false);
    table[0x71] = Opcode::new(0x71, "ADC", AddressingMode::IndirectY, 2, 5, true);

    // Arithmetic - SBC
    table[0xE9] = Opcode::new(0xE9, "SBC", AddressingMode::Immediate, 2, 2, false);
    table[0xE5] = Opcode::new(0xE5, "SBC", AddressingMode::ZeroPage, 2, 3, false);
    table[0xF5] = Opcode::new(0xF5, "SBC", AddressingMode::ZeroPageX, 2, 4, false);
    table[0xED] = Opcode::new(0xED, "SBC", AddressingMode::Absolute, 3, 4, false);
    table[0xFD] = Opcode::new(0xFD, "SBC", AddressingMode::AbsoluteX, 3, 4, true);
    table[0xF9] = Opcode::new(0xF9, "SBC", AddressingMode::AbsoluteY, 3, 4, true);
    table[0xE1] = Opcode::new(0xE1, "SBC", AddressingMode::IndirectX, 2, 6, false);
    table[0xF1] = Opcode::new(0xF1, "SBC", AddressingMode::IndirectY, 2, 5, true);

    // Compare - CMP
    table[0xC9] = Opcode::new(0xC9, "CMP", AddressingMode::Immediate, 2, 2, false);
    table[0xC5] = Opcode::new(0xC5, "CMP", AddressingMode::ZeroPage, 2, 3, false);
    table[0xD5] = Opcode::new(0xD5, "CMP", AddressingMode::ZeroPageX, 2, 4, false);
    table[0xCD] = Opcode::new(0xCD, "CMP", AddressingMode::Absolute, 3, 4, false);
    table[0xDD] = Opcode::new(0xDD, "CMP", AddressingMode::AbsoluteX, 3, 4, true);
    table[0xD9] = Opcode::new(0xD9, "CMP", AddressingMode::AbsoluteY, 3, 4, true);
    table[0xC1] = Opcode::new(0xC1, "CMP", AddressingMode::IndirectX, 2, 6, false);
    table[0xD1] = Opcode::new(0xD1, "CMP", AddressingMode::IndirectY, 2, 5, true);

    // Compare - CPX
    table[0xE0] = Opcode::new(0xE0, "CPX", AddressingMode::Immediate, 2, 2, false);
    table[0xE4] = Opcode::new(0xE4, "CPX", AddressingMode::ZeroPage, 2, 3, false);
    table[0xEC] = Opcode::new(0xEC, "CPX", AddressingMode::Absolute, 3, 4, false);

    // Compare - CPY
    table[0xC0] = Opcode::new(0xC0, "CPY", AddressingMode::Immediate, 2, 2, false);
    table[0xC4] = Opcode::new(0xC4, "CPY", AddressingMode::ZeroPage, 2, 3, false);
    table[0xCC] = Opcode::new(0xCC, "CPY", AddressingMode::Absolute, 3, 4, false);

    // Logic - AND
    table[0x29] = Opcode::new(0x29, "AND", AddressingMode::Immediate, 2, 2, false);
    table[0x25] = Opcode::new(0x25, "AND", AddressingMode::ZeroPage, 2, 3, false);
    table[0x35] = Opcode::new(0x35, "AND", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x2D] = Opcode::new(0x2D, "AND", AddressingMode::Absolute, 3, 4, false);
    table[0x3D] = Opcode::new(0x3D, "AND", AddressingMode::AbsoluteX, 3, 4, true);
    table[0x39] = Opcode::new(0x39, "AND", AddressingMode::AbsoluteY, 3, 4, true);
    table[0x21] = Opcode::new(0x21, "AND", AddressingMode::IndirectX, 2, 6, false);
    table[0x31] = Opcode::new(0x31, "AND", AddressingMode::IndirectY, 2, 5, true);

    // Logic - ORA
    table[0x09] = Opcode::new(0x09, "ORA", AddressingMode::Immediate, 2, 2, false);
    table[0x05] = Opcode::new(0x05, "ORA", AddressingMode::ZeroPage, 2, 3, false);
    table[0x15] = Opcode::new(0x15, "ORA", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x0D] = Opcode::new(0x0D, "ORA", AddressingMode::Absolute, 3, 4, false);
    table[0x1D] = Opcode::new(0x1D, "ORA", AddressingMode::AbsoluteX, 3, 4, true);
    table[0x19] = Opcode::new(0x19, "ORA", AddressingMode::AbsoluteY, 3, 4, true);
    table[0x01] = Opcode::new(0x01, "ORA", AddressingMode::IndirectX, 2, 6, false);
    table[0x11] = Opcode::new(0x11, "ORA", AddressingMode::IndirectY, 2, 5, true);

    // Logic - EOR
    table[0x49] = Opcode::new(0x49, "EOR", AddressingMode::Immediate, 2, 2, false);
    table[0x45] = Opcode::new(0x45, "EOR", AddressingMode::ZeroPage, 2, 3, false);
    table[0x55] = Opcode::new(0x55, "EOR", AddressingMode::ZeroPageX, 2, 4, false);
    table[0x4D] = Opcode::new(0x4D, "EOR", AddressingMode::Absolute, 3, 4, false);
    table[0x5D] = Opcode::new(0x5D, "EOR", AddressingMode::AbsoluteX, 3, 4, true);
    table[0x59] = Opcode::new(0x59, "EOR", AddressingMode::AbsoluteY, 3, 4, true);
    table[0x41] = Opcode::new(0x41, "EOR", AddressingMode::IndirectX, 2, 6, false);
    table[0x51] = Opcode::new(0x51, "EOR", AddressingMode::IndirectY, 2, 5, true);

    // Logic - BIT
    table[0x24] = Opcode::new(0x24, "BIT", AddressingMode::ZeroPage, 2, 3, false);
    table[0x2C] = Opcode::new(0x2C, "BIT", AddressingMode::Absolute, 3, 4, false);

    // Shift/Rotate - ASL
    table[0x0A] = Opcode::new(0x0A, "ASL", AddressingMode::Accumulator, 1, 2, false);
    table[0x06] = Opcode::new(0x06, "ASL", AddressingMode::ZeroPage, 2, 5, false);
    table[0x16] = Opcode::new(0x16, "ASL", AddressingMode::ZeroPageX, 2, 6, false);
    table[0x0E] = Opcode::new(0x0E, "ASL", AddressingMode::Absolute, 3, 6, false);
    table[0x1E] = Opcode::new(0x1E, "ASL", AddressingMode::AbsoluteX, 3, 7, false);

    // Shift/Rotate - LSR
    table[0x4A] = Opcode::new(0x4A, "LSR", AddressingMode::Accumulator, 1, 2, false);
    table[0x46] = Opcode::new(0x46, "LSR", AddressingMode::ZeroPage, 2, 5, false);
    table[0x56] = Opcode::new(0x56, "LSR", AddressingMode::ZeroPageX, 2, 6, false);
    table[0x4E] = Opcode::new(0x4E, "LSR", AddressingMode::Absolute, 3, 6, false);
    table[0x5E] = Opcode::new(0x5E, "LSR", AddressingMode::AbsoluteX, 3, 7, false);

    // Shift/Rotate - ROL
    table[0x2A] = Opcode::new(0x2A, "ROL", AddressingMode::Accumulator, 1, 2, false);
    table[0x26] = Opcode::new(0x26, "ROL", AddressingMode::ZeroPage, 2, 5, false);
    table[0x36] = Opcode::new(0x36, "ROL", AddressingMode::ZeroPageX, 2, 6, false);
    table[0x2E] = Opcode::new(0x2E, "ROL", AddressingMode::Absolute, 3, 6, false);
    table[0x3E] = Opcode::new(0x3E, "ROL", AddressingMode::AbsoluteX, 3, 7, false);

    // Shift/Rotate - ROR
    table[0x6A] = Opcode::new(0x6A, "ROR", AddressingMode::Accumulator, 1, 2, false);
    table[0x66] = Opcode::new(0x66, "ROR", AddressingMode::ZeroPage, 2, 5, false);
    table[0x76] = Opcode::new(0x76, "ROR", AddressingMode::ZeroPageX, 2, 6, false);
    table[0x6E] = Opcode::new(0x6E, "ROR", AddressingMode::Absolute, 3, 6, false);
    table[0x7E] = Opcode::new(0x7E, "ROR", AddressingMode::AbsoluteX, 3, 7, false);

    // Inc/Dec - INC
    table[0xE6] = Opcode::new(0xE6, "INC", AddressingMode::ZeroPage, 2, 5, false);
    table[0xF6] = Opcode::new(0xF6, "INC", AddressingMode::ZeroPageX, 2, 6, false);
    table[0xEE] = Opcode::new(0xEE, "INC", AddressingMode::Absolute, 3, 6, false);
    table[0xFE] = Opcode::new(0xFE, "INC", AddressingMode::AbsoluteX, 3, 7, false);

    // Inc/Dec - DEC
    table[0xC6] = Opcode::new(0xC6, "DEC", AddressingMode::ZeroPage, 2, 5, false);
    table[0xD6] = Opcode::new(0xD6, "DEC", AddressingMode::ZeroPageX, 2, 6, false);
    table[0xCE] = Opcode::new(0xCE, "DEC", AddressingMode::Absolute, 3, 6, false);
    table[0xDE] = Opcode::new(0xDE, "DEC", AddressingMode::AbsoluteX, 3, 7, false);

    // Inc/Dec - Register
    table[0xE8] = Opcode::new(0xE8, "INX", AddressingMode::Implied, 1, 2, false);
    table[0xCA] = Opcode::new(0xCA, "DEX", AddressingMode::Implied, 1, 2, false);
    table[0xC8] = Opcode::new(0xC8, "INY", AddressingMode::Implied, 1, 2, false);
    table[0x88] = Opcode::new(0x88, "DEY", AddressingMode::Implied, 1, 2, false);

    // Flow Control - JMP
    table[0x4C] = Opcode::new(0x4C, "JMP", AddressingMode::Absolute, 3, 3, false);
    table[0x6C] = Opcode::new(0x6C, "JMP", AddressingMode::Indirect, 3, 5, false);

    // Flow Control - JSR/RTS
    table[0x20] = Opcode::new(0x20, "JSR", AddressingMode::Absolute, 3, 6, false);
    table[0x60] = Opcode::new(0x60, "RTS", AddressingMode::Implied, 1, 6, false);

    // Flow Control - BRK/RTI
    table[0x00] = Opcode::new(0x00, "BRK", AddressingMode::Implied, 1, 7, false);
    table[0x40] = Opcode::new(0x40, "RTI", AddressingMode::Implied, 1, 6, false);

    // Branches
    table[0x90] = Opcode::new(0x90, "BCC", AddressingMode::Relative, 2, 2, true);
    table[0xB0] = Opcode::new(0xB0, "BCS", AddressingMode::Relative, 2, 2, true);
    table[0xF0] = Opcode::new(0xF0, "BEQ", AddressingMode::Relative, 2, 2, true);
    table[0xD0] = Opcode::new(0xD0, "BNE", AddressingMode::Relative, 2, 2, true);
    table[0x30] = Opcode::new(0x30, "BMI", AddressingMode::Relative, 2, 2, true);
    table[0x10] = Opcode::new(0x10, "BPL", AddressingMode::Relative, 2, 2, true);
    table[0x50] = Opcode::new(0x50, "BVC", AddressingMode::Relative, 2, 2, true);
    table[0x70] = Opcode::new(0x70, "BVS", AddressingMode::Relative, 2, 2, true);

    // Flag instructions
    table[0x18] = Opcode::new(0x18, "CLC", AddressingMode::Implied, 1, 2, false);
    table[0x38] = Opcode::new(0x38, "SEC", AddressingMode::Implied, 1, 2, false);
    table[0x58] = Opcode::new(0x58, "CLI", AddressingMode::Implied, 1, 2, false);
    table[0x78] = Opcode::new(0x78, "SEI", AddressingMode::Implied, 1, 2, false);
    table[0xD8] = Opcode::new(0xD8, "CLD", AddressingMode::Implied, 1, 2, false);
    table[0xF8] = Opcode::new(0xF8, "SED", AddressingMode::Implied, 1, 2, false);
    table[0xB8] = Opcode::new(0xB8, "CLV", AddressingMode::Implied, 1, 2, false);

    // NOP
    table[0xEA] = Opcode::new(0xEA, "NOP", AddressingMode::Implied, 1, 2, false);

    table
}

/// Get the opcode definition for a given opcode byte
pub fn get_opcode(code: u8) -> &'static Opcode {
    &OPCODES[code as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_lookup() {
        // Test LDA immediate
        let lda = get_opcode(0xA9);
        assert_eq!(lda.mnemonic, "LDA");
        assert_eq!(lda.mode, AddressingMode::Immediate);
        assert_eq!(lda.bytes, 2);
        assert_eq!(lda.cycles, 2);

        // Test JMP absolute
        let jmp = get_opcode(0x4C);
        assert_eq!(jmp.mnemonic, "JMP");
        assert_eq!(jmp.mode, AddressingMode::Absolute);
        assert_eq!(jmp.bytes, 3);
        assert_eq!(jmp.cycles, 3);

        // Test NOP
        let nop = get_opcode(0xEA);
        assert_eq!(nop.mnemonic, "NOP");
        assert_eq!(nop.mode, AddressingMode::Implied);
    }

    #[test]
    fn test_illegal_opcode() {
        // Test an illegal opcode
        let illegal = get_opcode(0x02);
        assert_eq!(illegal.mnemonic, "???");
    }
}
