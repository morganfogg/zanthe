use std::collections::HashMap;
//
use anyhow::Result;
use itertools::Itertools;
use tracing::{error, warn};

use crate::game::error::GameError;
use crate::game::instruction::op_code::OpCode;
use crate::game::instruction::Instruction;
use crate::game::instruction::{OperandSet, Result as InstructionResult};
use crate::game::state::GameState;

pub fn instructions() -> Vec<(OpCode, Instruction)> {
    use crate::game::instruction::instruction_set::common;
    use Instruction::*;
    use OpCode::*;
    vec![
        (TwoOp(0x1A), Normal(&call_2n, "CALL_2N")),
        (TwoOp(0x1C), Normal(&throw, "THROW")),
        (OneOp(0xF), Normal(&call_1n, "CALL_1N")),
        (ZeroOp(0x9), Store(&catch, "CATCH")),
        (ZeroOp(0xF), Branch(&piracy, "PIRACY")),
        (VarOp(0x4), Store(&aread, "AREAD")),
        (VarOp(0x18), Store(&common::not, "NOT")), // Moved from 1OP:143
        (VarOp(0x19), Normal(&call_vn, "CALL_VN")),
        (VarOp(0x1A), Normal(&call_vn2, "CALL_VN2")),
        (VarOp(0x1B), Normal(&tokenise, "TOKENISE")),
        (VarOp(0x1F), Branch(&check_arg_count, "CHECK_ARG_COUNT")),
        (Extended(0x2), Store(&log_shift, "LOG_SHIFT")),
        (Extended(0x3), Store(&art_shift, "ART_SHIFT")),
        (Extended(0x9), Store(&save_undo, "SAVE_UNDO")),
        (Extended(0xA), Store(&restore_undo, "RESTORE_UNDO")),
    ]
}

/// 2OP:26 Execute a routine with 1 argument and throw away the result.
fn call_2n(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);

    let argument = ops.pull()?.unsigned(state)?;

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(vec![argument]),
        store_to: None,
    })
}

/// 2OP:28 Return to the specified stack frame, then return.
fn throw(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let value = ops.pull()?.unsigned(state)?;
    let frame = ops.pull()?.unsigned(state)?;

    state.throw(frame)?;
    Ok(InstructionResult::Return(value))
}

/// 1OP:143 Calls a routine with no arguments and throws away the result.
fn call_1n(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);

    Ok(InstructionResult::Invoke {
        address,
        arguments: None,
        store_to: None,
    })
}

/// 0OP:185 Return the ID of the current stack frame.
fn catch(state: &mut GameState, _: OperandSet, store_to: u8) -> Result<InstructionResult> {
    state.set_variable(store_to, state.frame_id());
    Ok(InstructionResult::Continue)
}

/// 0OP:191 Branch if game is genuine
fn piracy(
    state: &mut GameState,
    _: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult> {
    let is_genuine = true; // TODO: Add a way to toggle this
    Ok(state
        .frame()
        .conditional_branch(offset, is_genuine, expected))
}

/// VAR:228 Read a string from the user
fn aread(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    // TODO: add time routines
    let text_address = ops.pull()?.unsigned(state)?;
    let parse_address = ops.pull()?.try_unsigned(state)?;

    let max_characters = state.memory.get_byte(text_address as usize);
    if max_characters < 3 {
        return Err(
            GameError::InvalidOperation("Text buffer cannot be less than 3 bytes".into()).into(),
        );
    }

    let string = state.interface.read_line(max_characters as usize)?;

    // state
    //     .memory
    //     .set_byte(text_address as usize, string.len() as u8);

    state.set_variable(store_to, 13);
    state
        .memory
        .write_string_array(text_address as usize, &string)?;

    let mut t_addr = text_address as usize;
    let q = state.memory.read_string_array(t_addr);

    if let Some(parse_address) = parse_address {
        let max_words = state.memory.get_byte(parse_address as usize);
        if max_words < 6 {
            return Err(GameError::InvalidOperation(
                "Parse buffer cannot be less than 6 bytes".into(),
            )
            .into());
        }
        state
            .memory
            .parse_string(parse_address as usize, &string, max_words as usize)?;
    }

    Ok(InstructionResult::Continue)
}

/// VAR:249 Call a routine with up to 3 arguments and throw away the result.
fn call_vn(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
        store_to: None,
    })
}

/// VAR:250 Call a routine with up to 7 arguments and throw away the result.
fn call_vn2(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
        store_to: None,
    })
}

/// VAR:251 Tokenise a string.
fn tokenise(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let text_address = ops.pull()?.unsigned(state)?;
    let parse_address = ops.pull()?.unsigned(state)?;
    let dictionary = ops.pull()?.try_unsigned(state)?;
    let flag = ops.pull()?.try_unsigned(state)?;

    if !matches!(flag, None | Some(0)) && !matches!(dictionary, None | Some(0)) {
        todo!("Implement this");
    }

    let max_words = state.memory.get_byte(parse_address as usize);
    if max_words < 6 {
        return Err(
            GameError::InvalidOperation("Parse buffer cannot be less than 6 bytes".into()).into(),
        );
    }

    let mut t_cursor = text_address as usize;

    let string = state.memory.read_string_array(t_cursor)?;

    state
        .memory
        .parse_string(parse_address as usize, &string, max_words as usize)?;

    Ok(InstructionResult::Continue)
}

/// VAR:255 Branches if the argument number (1-indexed) has been provided.
fn check_arg_count(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult> {
    let index = ops.pull()?.unsigned(state)? as usize;

    let condition = index <= state.frame().arg_count;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// EXT:2 Logical shift
fn log_shift(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let number = ops.pull()?.unsigned(state)?;
    let places = ops.pull()?.signed(state)?;
    if places.abs() > 15 {
        warn!("Attempted to bitshift more than 15 places. This is unspecified behaviour.");
        state.set_variable(store_to, 0);
        return Ok(InstructionResult::Continue);
    }

    let result = if places < 0 {
        number.wrapping_shr(places.abs() as u32)
    } else {
        number.wrapping_shl(places as u32)
    };

    state.set_variable(store_to, result);
    Ok(InstructionResult::Continue)
}

/// EXT:3 Artihmetic shift
fn art_shift(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let number = ops.pull()?.signed(state)?;
    let places = ops.pull()?.signed(state)?;
    if places.abs() > 15 {
        warn!("Attempted to bitshift more than 15 places. This is unspecified behaviour.");
        state.set_variable(store_to, if number < 0 { -1i16 as u16 } else { 0 });
        return Ok(InstructionResult::Continue);
    }

    let result = if places < 0 {
        number.wrapping_shr(places.abs() as u32)
    } else {
        number.wrapping_shl(places as u32)
    };

    state.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

/// EXT:9 Save the current game state to the undo buffer.
fn save_undo(
    state: &mut GameState,
    mut _ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    state.save_undo(store_to);
    Ok(InstructionResult::Continue)
}

/// EXT:10 Load the most recent undo state.
fn restore_undo(
    state: &mut GameState,
    mut _ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let success = state.restore_undo();
    if !success {
        state.set_variable(store_to, 0);
    }
    Ok(InstructionResult::Continue)
}
