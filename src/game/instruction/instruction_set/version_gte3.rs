use std::error::Error;

use crate::game::instruction::{OperandSet, Result as InstructionResult};
use crate::game::state::GameState;

/// 0OP:189 Verify the file's checksum
pub fn verify(
    state: &mut GameState,
    _: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let condition = state.checksum_valid;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}
