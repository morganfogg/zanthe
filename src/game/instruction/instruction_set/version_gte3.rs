use std::collections::HashMap;

use anyhow::Result;

use crate::game::instruction::op_code::OpCode;
use crate::game::instruction::Instruction;
use crate::game::instruction::{OperandSet, Result as InstructionResult, Result::*};
use crate::game::state::GameState;

pub fn instructions() -> HashMap<OpCode, Instruction> {
    use Instruction::*;
    use OpCode::*;
    vec![
        (ZeroOp(0xD), Branch(&verify, "VERIFY")),
        (VarOp(0xA), Normal(&split_window, "SPLIT_WINDOW")),
        (VarOp(0xB), Normal(&set_window, "SET_WINDOW")),
        (VarOp(0x13), Normal(&output_stream, "OUTPUT_STREAM")),
    ]
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

/// VAR:234 Split the window so that the upper window has the
/// given number of lines, or destroy the upper window if the
/// given number is zero.
pub fn split_window(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let lines = ops.pull()?.unsigned(state)?;

    state.interface.split_screen(lines)?;
    Ok(Continue)
}

/// VAR:235 Make the given window the active window
pub fn set_window(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let window = ops.pull()?.unsigned(state)?;

    state.interface.set_active(window)?;
    Ok(Continue)
}

/// VAR:243 Change the output stream
pub fn output_stream(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    // TODO: Implment
    Ok(Continue)
}
