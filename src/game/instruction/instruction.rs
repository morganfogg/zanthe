use std::error::Error;

use crate::game::instruction::{Context, Operand, Result as InstructionResult};

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(&'static dyn Fn(Context, Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>>),
    Branch(
        &'static dyn Fn(
            Context,
            Vec<Operand>,
            bool,
            u16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
    ),
    Store(&'static dyn Fn(Context, Vec<Operand>, u8) -> Result<InstructionResult, Box<dyn Error>>),
    StringLiteral(&'static dyn Fn(Context, String) -> Result<InstructionResult, Box<dyn Error>>),
}
