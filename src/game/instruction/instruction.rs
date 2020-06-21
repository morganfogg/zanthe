use anyhow::Result;

use super::OperandSet;
use super::Result as InstructionResult;
use crate::game::state::GameState;

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(&mut GameState, OperandSet) -> Result<InstructionResult>,
        &'static str,
    ),
    Branch(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            bool,
            i16,
        ) -> Result<InstructionResult>,
        &'static str,
    ),
    BranchStore(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            bool,
            i16,
            u8,
        ) -> Result<InstructionResult>,
        &'static str,
    ),
    Store(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            u8,
        ) -> Result<InstructionResult>,
        &'static str,
    ),
    StringLiteral(
        &'static dyn Fn(&mut GameState, String) -> Result<InstructionResult>,
        &'static str,
    ),
}
