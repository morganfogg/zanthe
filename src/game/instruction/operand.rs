use std::convert::TryInto;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use crate::game::error::GameError;
use crate::game::instruction::Context;

/// The operands passed to instructions, not including branch, store or string literal operands.
pub enum Operand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
}

impl Operand {
    pub fn try_unsigned(&self, context: &mut Context) -> Result<Option<u16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v)),
            Operand::SmallConstant(v) => Ok(Some(u16::from(*v))),
            Operand::Variable(v) => Ok(Some(context.get_variable(*v)?)),
            Operand::Omitted => Ok(None),
        }
    }
    pub fn try_signed(&self, context: &mut Context) -> Result<Option<i16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v as i16)),
            Operand::SmallConstant(v) => Ok(Some(i16::from(*v as i8))),
            Operand::Variable(v) => Ok(Some(context.get_variable(*v)? as i16)),
            Operand::Omitted => Ok(None),
        }
    }
    pub fn variable_id(&self, context: &mut Context) -> Result<u8, Box<dyn Error>> {
        self.unsigned(context)?
            .try_into()
            .map_err(|_| GameError::InvalidOperation("Missing required operand".into()).into())
    }
    pub fn unsigned(&self, context: &mut Context) -> Result<u16, Box<dyn Error>> {
        self.try_unsigned(context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()).into())
    }
    pub fn signed(&self, context: &mut Context) -> Result<i16, Box<dyn Error>> {
        self.try_signed(context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()).into())
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Operand::LargeConstant(v) => format!("LargeConstant({:x})", v),
                Operand::SmallConstant(v) => format!("SmallConstant({:x})", v),
                Operand::Variable(v) => format!("Variable({:x})", v),
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
