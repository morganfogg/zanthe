use std::fmt::{self, Debug, Display, Formatter};

use crate::game::Result;

use crate::game::error::GameError;
use crate::game::state::GameState;

/// The operands passed to instructions, not including branch, store or string literal operands.
#[derive(Clone, Copy)]
pub enum Operand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
}

impl Operand {
    pub fn try_unsigned(self, state: &mut GameState) -> Result<Option<u16>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(v)),
            Operand::SmallConstant(v) => Ok(Some(u16::from(v))),
            Operand::Variable(v) => Ok(Some(state.get_variable(v)?)),
            Operand::Omitted => Ok(None),
        }
    }

    pub fn try_signed(self, state: &mut GameState) -> Result<Option<i16>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(v as i16)),
            Operand::SmallConstant(v) => Ok(Some(v as i16)),
            Operand::Variable(v) => Ok(Some(state.get_variable(v)? as i16)),
            Operand::Omitted => Ok(None),
        }
    }

    pub fn unsigned(self, state: &mut GameState) -> Result<u16> {
        self.try_unsigned(state)?
            .ok_or_else(|| GameError::invalid_operation("Missing required operand"))
    }

    pub fn signed(self, state: &mut GameState) -> Result<i16> {
        self.try_signed(state)?
            .ok_or_else(|| GameError::invalid_operation("Missing required operand"))
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Operand::LargeConstant(v) => format!("#{:04x}", v),
                Operand::SmallConstant(v) => format!("#{:02x}", v),
                Operand::Variable(v) => match v {
                    0x0 => "(SP)+".to_string(),
                    0x1..=0xf => format!("L{:02x}", v - 0x1),
                    0x10..=0xff => format!("G{:02x}", v - 0x10),
                },
                Operand::Omitted => "".to_string(),
            }
        )
    }
}

impl Debug for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
