use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

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
    pub fn try_unsigned(&self, state: &mut GameState) -> Result<Option<u16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v)),
            Operand::SmallConstant(v) => Ok(Some(u16::from(*v))),
            Operand::Variable(v) => Ok(Some(state.get_variable(*v)?)),
            Operand::Omitted => Ok(None),
        }
    }

    pub fn try_signed(&self, state: &mut GameState) -> Result<Option<i16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v as i16)),
            Operand::SmallConstant(v) => Ok(Some(*v as i16)),
            Operand::Variable(v) => Ok(Some(state.get_variable(*v)? as i16)),
            Operand::Omitted => Ok(None),
        }
    }

    pub fn unsigned(&self, state: &mut GameState) -> Result<u16, Box<dyn Error>> {
        self.try_unsigned(state)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()).into())
    }

    pub fn signed(&self, state: &mut GameState) -> Result<i16, Box<dyn Error>> {
        self.try_signed(state)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()).into())
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Operand::LargeConstant(v) => format!("LargeConstant({0} [0x{0:x}])", v),
                Operand::SmallConstant(v) => format!("SmallConstant({0} [0x{0:x})", v),
                Operand::Variable(v) => match v {
                    0x0 => format!("Variable(SP)"),
                    0x1..=0xf => format!("Variable(L{:x})", v - 0x1),
                    0x10..=0xff => format!("Variable(G{:x})", v - 0x10),
                },
                Operand::Omitted => "Omitted".to_string(),
            }
        )
    }
}

impl Debug for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
