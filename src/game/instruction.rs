mod form;
mod instruction_set;
mod op_code;
mod operand;
mod operand_set;
mod result;

pub use form::Form;
pub use instruction_set::InstructionSet;
pub use op_code::OpCode;
pub use operand::Operand;
pub use operand_set::OperandSet;
pub use result::Result;

use crate::game::state::GameState;
use result::Result as InstructionResult;

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(&mut GameState, OperandSet) -> anyhow::Result<InstructionResult>,
        &'static str,
    ),
    Branch(
        &'static dyn Fn(&mut GameState, OperandSet, bool, i16) -> anyhow::Result<InstructionResult>,
        &'static str,
    ),
    BranchStore(
        &'static dyn Fn(
            &mut GameState,
            OperandSet,
            bool,
            i16,
            u8,
        ) -> anyhow::Result<InstructionResult>,
        &'static str,
    ),
    Store(
        &'static dyn Fn(&mut GameState, OperandSet, u8) -> anyhow::Result<InstructionResult>,
        &'static str,
    ),
    StringLiteral(
        &'static dyn Fn(&mut GameState, String) -> anyhow::Result<InstructionResult>,
        &'static str,
    ),
}
