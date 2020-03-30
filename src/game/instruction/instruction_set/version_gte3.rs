use crate::game::instruction::{Context, Operand, Result as InstructionResult};
use std::error::Error;

/// 0OP:189 Verify the file's checksum
pub fn verify(
    context: Context,
    _ops: Vec<Operand>,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let condition = context.checksum_valid;

    Ok(context
        .frame
        .conditional_branch(offset, condition, expected))
}
