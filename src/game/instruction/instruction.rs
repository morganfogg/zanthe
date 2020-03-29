use std::error::Error;

use crate::game::instruction::{Context, Operand, Result as InstructionResult};

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(Context, Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Branch(
        &'static dyn Fn(
            Context,
            Vec<Operand>,
            bool,
            i16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Store(
        &'static dyn Fn(Context, Vec<Operand>, u8) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    StringLiteral(
        &'static dyn Fn(Context, String) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
}
