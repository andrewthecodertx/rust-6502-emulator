//! 6502 Addressing Modes
//!
//! The 6502 supports 13 different addressing modes that determine how
//! the operand for an instruction is located.

/// Addressing modes supported by the 6502
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    Indirect,
    Relative,
}

impl AddressingMode {
    pub fn operand_bytes(&self) -> u8 {
        match self {
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            AddressingMode::Immediate
            | AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndirectX
            | AddressingMode::IndirectY
            | AddressingMode::Relative => 1,
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operand_bytes() {
        assert_eq!(AddressingMode::Implied.operand_bytes(), 0);
        assert_eq!(AddressingMode::Accumulator.operand_bytes(), 0);
        assert_eq!(AddressingMode::Immediate.operand_bytes(), 1);
        assert_eq!(AddressingMode::ZeroPage.operand_bytes(), 1);
        assert_eq!(AddressingMode::ZeroPageX.operand_bytes(), 1);
        assert_eq!(AddressingMode::ZeroPageY.operand_bytes(), 1);
        assert_eq!(AddressingMode::Absolute.operand_bytes(), 2);
        assert_eq!(AddressingMode::AbsoluteX.operand_bytes(), 2);
        assert_eq!(AddressingMode::AbsoluteY.operand_bytes(), 2);
        assert_eq!(AddressingMode::IndirectX.operand_bytes(), 1);
        assert_eq!(AddressingMode::IndirectY.operand_bytes(), 1);
        assert_eq!(AddressingMode::Indirect.operand_bytes(), 2);
        assert_eq!(AddressingMode::Relative.operand_bytes(), 1);
    }
}
