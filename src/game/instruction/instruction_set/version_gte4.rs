use std::collections::HashMap;
use std::error::Error;

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
        (TwoOp(0x19), Store(&call_2s, "CALL_2S")),
        (OneOp(0x8), Store(&call_1s, "CALL_1S")),
        (VarOp(0x0), Store(&call_vs, "CALL_VS")),
        (VarOp(0xC), Store(&call_vs2, "CALL_VS2")),
        (VarOp(0x11), Normal(&set_text_style, "SET_TEXT_STYLE")),
        (VarOp(0x16), Store(&read_char, "READ_CHAR")),
    ]
    .into_iter()
    .collect()
}

/// 2OP:25 Call a routine with 1 argument and store the result.
pub fn call_2s(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
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
) -> Result<InstructionResult, Box<dyn Error>> {
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
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>, Box<dyn Error>>>()?
        .into_iter()
        .while_some()
        .collect();

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// VAR:241 Sets the active text style (bold, emphasis etc.)
pub fn set_text_style(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
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
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>, Box<dyn Error>>>()?
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
) -> Result<InstructionResult, Box<dyn Error>> {
    let input = state.interface.read_char()?;
    let zscii = state.memory.zscii_from_code(input)?;
    state.set_variable(store_to, zscii.into());
    Ok(InstructionResult::Continue)
}
