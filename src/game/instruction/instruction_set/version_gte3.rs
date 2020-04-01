use std::error::Error;

use crate::game::instruction::{Context, OperandSet, Result as InstructionResult};

/// 0OP:189 Verify the file's checksum
pub fn verify(
    context: Context,
    _: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let condition = context.checksum_valid;

    Ok(context
        .frame
        .conditional_branch(offset, condition, expected))
}
