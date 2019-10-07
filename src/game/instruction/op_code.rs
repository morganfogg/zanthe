use std::fmt::{self, Debug, Display, Formatter};

/// A wrapper for op codes to associate them with their argument counts.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum OpCode {
    ZeroOp(u8),
    OneOp(u8),
    TwoOp(u8),
    VarOp(u8),
    Extended(u8),
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                OpCode::TwoOp(v) => format!("2OP:{}", v),
                OpCode::OneOp(v) => format!("1OP:{}", v + 128),
                OpCode::ZeroOp(v) => format!("0OP:{}", v + 176),
                OpCode::VarOp(v) => format!("VAR:{}", v + 224),
                OpCode::Extended(v) => format!("EXT:{}", v),
            }
        )
    }
}

impl Debug for OpCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
