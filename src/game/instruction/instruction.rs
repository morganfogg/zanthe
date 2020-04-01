use super::OperandSet;
use crate::game::instruction::{Context, Result as InstructionResult};
use std::error::Error;

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(Context, OperandSet) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Branch(
        &'static dyn Fn(
            Context,
            OperandSet,
            bool,
            i16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    BranchStore(
        &'static dyn Fn(
            Context,
            OperandSet,
            bool,
            i16,
            u8,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Store(
        &'static dyn Fn(Context, OperandSet, u8) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    StringLiteral(
        &'static dyn Fn(Context, String) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
}
