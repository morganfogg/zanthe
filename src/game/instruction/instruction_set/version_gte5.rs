use std::error::Error;

use itertools::Itertools;

use crate::game::error::GameError;
use crate::game::instruction::{OperandSet, Result as InstructionResult};
use crate::game::state::GameState;

/// 2OP:26 Execute a routine with 1 argument and throw away the result.
pub fn call_2n(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);

    let argument = ops.pull()?.unsigned(state)?;

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(vec![argument]),
        store_to: None,
    })
}

/// 1OP:143 Calls a routine with no arguments and throws away the result.
pub fn call_1n(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address as usize);

    Ok(InstructionResult::Invoke {
        address,
        arguments: None,
        store_to: None,
    })
}

/// 0OP:191 Branch if game is genuine
pub fn piracy(
    state: &mut GameState,
    _: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let is_genuine = true; // TODO: Add a way to toggle this
    Ok(state
        .frame()
        .conditional_branch(offset, is_genuine, expected))
}

/// VAR:249 Call a routine with up to 3 arguments and throw away the result.
pub fn call_vn(
    state: &mut GameState,
    mut ops: OperandSet,
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
        store_to: None,
    })
}

/// VAR:250 Call a routine with up to 7 arguments and throw away the result.
pub fn call_vn2(
    state: &mut GameState,
    mut ops: OperandSet,
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
        store_to: None,
    })
}

/// VAR:255 Branches if the argument number (1-indexed) has been provided.
pub fn check_arg_count(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let index = ops.pull()?.unsigned(state)? as usize;

    let condition = index <= state.frame().arg_count;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// EXT:2 Logical shift
pub fn log_shift(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let number = ops.pull()?.unsigned(state)?;
    let places = ops.pull()?.signed(state)?;
    if places.abs() > 15 {
        return Err(GameError::InvalidOperation("Shift cannot exceed 15".into()).into());
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
pub fn art_shift(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let number = ops.pull()?.signed(state)?;
    let places = ops.pull()?.signed(state)?;
    if places.abs() > 15 {
        return Err(GameError::InvalidOperation("Shift cannot exceed 15".into()).into());
    }

    let result = if places < 0 {
        number.wrapping_shr(places.abs() as u32)
    } else {
        number.wrapping_shl(places as u32)
    };

    state.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}
