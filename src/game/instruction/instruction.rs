use super::OperandSet;
use super::Result as InstructionResult;
use crate::game::state::GameState;
use std::error::Error;

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(&mut GameState, OperandSet) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Branch(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            bool,
            i16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    BranchStore(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            bool,
            i16,
            u8,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    Store(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            u8,
        ) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
    StringLiteral(
        &'static dyn Fn(&mut GameState, String) -> Result<InstructionResult, Box<dyn Error>>,
        &'static str,
    ),
}
