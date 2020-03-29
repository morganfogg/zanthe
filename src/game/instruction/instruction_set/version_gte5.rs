use std::error::Error;

use itertools::Itertools;

use crate::game::instruction::{Context, Operand, Result as InstructionResult};

/// 2OP:26 Execute a routine with 1 argument and throw away the result.
pub fn call_2n(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address as usize);

    let argument = ops[1].unsigned(&mut context)?;

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(vec![argument]),
        store_to: None,
    })
}

/// 1OP:143 Calls a routine with no arguments and throws away the result.
pub fn call_1n(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address as usize);

    Ok(InstructionResult::Invoke {
        address,
        arguments: None,
        store_to: None,
    })
}

/// VAR:249 Call a routine with up to 3 arguments and throw away the result.
pub fn call_vn(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops[1..]
        .iter()
        .map(|op| op.try_unsigned(&mut context))
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
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops[1..]
        .iter()
        .map(|op| op.try_unsigned(&mut context))
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
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let index = ops[0].unsigned(&mut context)? as usize;

    if (index <= context.frame.arg_count) == condition {
        Ok(context.frame.branch(offset))
    } else {
        Ok(InstructionResult::Continue)
    }
}
