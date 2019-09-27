use std::fmt::{self, Debug, Display, Formatter};

pub enum Operand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
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
