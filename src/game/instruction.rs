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

use crate::game::{Result as GameResult};
use crate::game::state::GameState;
use result::Result as InstructionResult;

type NormalHandler = dyn Fn(&mut GameState, OperandSet) -> GameResult<InstructionResult>;
type BranchHandler =
    dyn Fn(&mut GameState, OperandSet, bool, i16) -> GameResult<InstructionResult>;
type BranchStoreHandler =
    dyn Fn(&mut GameState, OperandSet, bool, i16, u8) -> GameResult<InstructionResult>;
type StoreHandler = dyn Fn(&mut GameState, OperandSet, u8) -> GameResult<InstructionResult>;
type StringLiteralHandler = dyn Fn(&mut GameState, String) -> GameResult<InstructionResult>;

/// A wrapper for instruction functions to associate them with their argument types.
#[derive(Clone)]
pub enum Instruction {
    Normal(&'static NormalHandler, &'static str),
    Branch(&'static BranchHandler, &'static str),
    BranchStore(&'static BranchStoreHandler, &'static str),
    Store(&'static StoreHandler, &'static str),
    StringLiteral(&'static StringLiteralHandler, &'static str),
}
