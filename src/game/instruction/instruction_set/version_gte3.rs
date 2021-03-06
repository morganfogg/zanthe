use std::collections::HashMap;

use anyhow::Result;

use crate::game::instruction::op_code::OpCode;
use crate::game::instruction::Instruction;
use crate::game::instruction::{OperandSet, Result as InstructionResult};
use crate::game::state::GameState;

pub fn instructions() -> HashMap<OpCode, Instruction> {
    use Instruction::*;
    use OpCode::*;
    vec![(ZeroOp(0xD), Branch(&verify, "VERIFY"))]
        .into_iter()
        .collect()
}

/// 0OP:189 Verify the file's checksum
pub fn verify(
    state: &mut GameState,
    _: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult> {
    let condition = state.checksum_valid;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}
