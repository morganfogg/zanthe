use crate::game::instruction::{Context, Operand, Result as InstructionResult};
use std::error::Error;

/// 0OP:189 Verify the file's checksum
pub fn verify(
    context: Context,
    _ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    if (context.checksum_valid) == condition {
        Ok(context.frame.branch(offset))
    } else {
        Ok(InstructionResult::Continue)
    }
}
