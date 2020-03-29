use std::error::Error;

use itertools::Itertools;

use crate::game::error::GameError;
use crate::game::instruction::{Context, Operand, Result as InstructionResult};

/// 2OP:25 Call a routine with 1 argument and store the result.
pub fn call_2s(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address as usize);
    let arguments = vec![ops[1].unsigned(&mut context)?];

    Ok(InstructionResult::Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// VAR:241 Sets the active text style (bold, emphasis etc.)
pub fn set_text_style(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let format = ops[0].unsigned(&mut context)?;

    match format {
        0 => context.interface.text_style_clear(),
        1 => context.interface.text_style_reverse(),
        2 => context.interface.text_style_bold(),
        4 => context.interface.text_style_emphasis(),
        8 => context.interface.text_style_fixed(),
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
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
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
        store_to: Some(store_to),
    })
}
