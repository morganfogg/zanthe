use std::collections::HashMap;

use anyhow::Result;
use itertools::Itertools;

use crate::game::error::GameError;
use crate::game::instruction::op_code::OpCode;
use crate::game::instruction::Instruction;
use crate::game::instruction::{OperandSet, Result as InstructionResult};
use crate::game::state::GameState;

pub fn instructions() -> HashMap<OpCode, Instruction> {
    use Instruction::*;
    use OpCode::*;
    vec![
        (VarOp(0x0), Store(&call_vs, "CALL_VS")),
        (OneOp(0x8), Store(&call_1s, "CALL_1S")),
        (VarOp(0xC), Store(&call_vs2, "CALL_VS2")),
        (VarOp(0xD), Normal(&erase_window, "ERASE_WINDOW")),
        (VarOp(0x11), Normal(&set_text_style, "SET_TEXT_STYLE")),
        (VarOp(0x16), Store(&read_char, "READ_CHAR")),
        (TwoOp(0x19), Store(&call_2s, "CALL_2S")),
    ]
    .into_iter()
    .collect()
}

/// 2OP:25 Call a routine with 1 argument and store the result.
pub fn call_2s(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);
    let arguments = vec![ops.pull()?.unsigned(state)?];

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// 1OP:136 Call the routine with no arguments and store the result.
pub fn call_1s(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);

    Ok(InstructionResult::Invoke {
        address,
        arguments: None,
        store_to: Some(store_to),
    })
}

/// VAR:236 Call a routine with up to 7 arguments and store the result.
pub fn call_vs2(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>>>()?
        .into_iter()
        .while_some()
        .collect();

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// VAR:237 Clear the screen
pub fn erase_window(state: &mut GameState, mut _ops: OperandSet) -> Result<InstructionResult> {
    // TODO: Add multiple windows.
    state.interface.clear()?;
    Ok(InstructionResult::Continue)
}

/// VAR:241 Sets the active text style (bold, emphasis etc.)
pub fn set_text_style(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let format = ops.pull()?.unsigned(state)?;

    match format {
        0 => state.interface.text_style_clear(),
        1 => state.interface.text_style_reverse(),
        2 => state.interface.text_style_bold(),
        4 => state.interface.text_style_emphasis(),
        8 => state.interface.text_style_fixed(),
        _ => {
            return Err(
                GameError::InvalidOperation("Tried to set invalid text style".into()).into(),
            )
        }
    }

    Ok(InstructionResult::Continue)
}

/// VAR:224 Calls a routine with up to 3 arguments and stores the result.
pub fn call_vs(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>>>()?
        .into_iter()
        .while_some()
        .collect();

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// VAR:246 Read a single character of input.
pub fn read_char(
    state: &mut GameState,
    mut _ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let input = state.interface.read_char()?;
    let zscii = state.memory.zscii_from_code(input)?;
    state.set_variable(store_to, zscii.into());
    Ok(InstructionResult::Continue)
}
