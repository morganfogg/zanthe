use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use crate::game::instruction::Context;

pub enum Operand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
}

impl Operand {
    pub fn get_unsigned(&self, context: &mut Context) -> Result<Option<u16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v)),
            Operand::SmallConstant(v) => Ok(Some(u16::from(*v))),
            Operand::Variable(v) => Ok(Some(context.get_variable(*v)?)),
            Operand::Omitted => Ok(None),
        }
    }
    pub fn get_signed(&self, context: &mut Context) -> Result<Option<i16>, Box<dyn Error>> {
        match self {
            Operand::LargeConstant(v) => Ok(Some(*v as i16)),
            Operand::SmallConstant(v) => Ok(Some(i16::from(*v as i8))),
            Operand::Variable(v) => Ok(Some(context.get_variable(*v)? as i16)),
            Operand::Omitted => Ok(None),
        }
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
