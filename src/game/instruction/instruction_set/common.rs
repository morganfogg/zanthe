use std::cmp::Ordering;
use std::convert::TryInto;
use std::error::Error;

use itertools::Itertools;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::error::GameError;
use crate::game::instruction::{Context, Operand, Result as InstructionResult};

///20P:1 Branch if the first operand is equal to any subsequent operands
pub fn je(
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let mut result = false;
    for op in ops[1..].iter() {
        if let Some(value) = op.try_signed(&mut context)? {
            if value == first {
                result = true;
                break;
            }
        } else {
            break;
        }
    }

    Ok(context.frame.conditional_branch(offset, result, condition))
}

/// 2OP:2 Jump if a < b (signed).
pub fn jl(
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops[0].signed(&mut context)?;
    let b = ops[1].signed(&mut context)?;

    let result = a < b;

    Ok(context.frame.conditional_branch(offset, result, condition))
}

/// 2OP:3 Jump if a > b (signed).
pub fn jg(
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops[0].signed(&mut context)?;
    let b = ops[1].signed(&mut context)?;

    let result = a > b;

    Ok(context.frame.conditional_branch(offset, result, condition))
}

/// 2OP:6 Jump if object a's parent is object b
pub fn jin(
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_a = ops[0].unsigned(&mut context)?;
    let object_b = ops[0].unsigned(&mut context)?;
    let parent = context.memory.object_parent(object_a);

    let result = object_b == parent;

    Ok(context.frame.conditional_branch(offset, result, condition))
}

/// 2OP:8 Bitwise OR
pub fn or(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let x = ops[0].unsigned(&mut context)?;
    let y = ops[1].unsigned(&mut context)?;

    let result = x | y;

    context.set_variable(store_to, result);

    Ok(InstructionResult::Continue)
}

// 2OP:9 Bitwise AND
pub fn and(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let x = ops[0].unsigned(&mut context)?;
    let y = ops[1].unsigned(&mut context)?;

    let result = x & y;

    context.set_variable(store_to, result);

    Ok(InstructionResult::Continue)
}

/// 2OP:13 Set the variable referenced by the operand to value
pub fn store(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let variable = ops[0].variable_id(&mut context)?;
    let value = ops[1].unsigned(&mut context)?;

    context.set_variable(variable, value);
    Ok(InstructionResult::Continue)
}

/// 2OP:15 Store a word found at the given array and word index.
pub fn loadw(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops[0].unsigned(&mut context)?;
    let word_index = ops[1].unsigned(&mut context)?;
    let word = context
        .memory
        .get_word(usize::from(array + (2 * word_index)));

    context.set_variable(store_to, word);
    Ok(InstructionResult::Continue)
}

/// 2OP:16 Store a byte found at the given array and byte index.
pub fn loadb(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops[0].unsigned(&mut context)?;
    let byte_index = ops[1].unsigned(&mut context)?;
    let byte = context.memory.get_byte(usize::from(array + byte_index));
    context.set_variable(store_to, byte as u16);
    Ok(InstructionResult::Continue)
}

/// 2OP:17 Return the data of the specified property
pub fn get_prop(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops[0].unsigned(&mut context)?;
    let property = ops[1].unsigned(&mut context)?;

    let data = context
        .memory
        .property(object, property)
        .map(|prop| prop.data_to_u16())
        .unwrap()?; // TODO IMPLMENT DEFAULTS.

    context.set_variable(store_to, data);
    Ok(InstructionResult::Continue)
}

/// 2OP:18 Return the byte address of the specified property data
pub fn get_prop_addr(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops[0].unsigned(&mut context)?;
    let property = ops[1].unsigned(&mut context)?;

    let address = context
        .memory
        .property(object, property)
        .map(|prop| prop.data_address)
        .unwrap_or(0);

    context.set_variable(store_to, address);
    Ok(InstructionResult::Continue)
}

/// 2OP:20 Signed 16-bit addition
pub fn add(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let second = ops[1].signed(&mut context)?;
    let result = first + second;

    context.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

// 2OP:21 Signed 16-bit subtraction
pub fn sub(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let second = ops[1].signed(&mut context)?;
    let result = first - second;

    context.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

/// 2OP:22 Signed 16-bit multiplication.
pub fn mul(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let second = ops[1].signed(&mut context)?;

    let result = first * second;

    context.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

/// 2OP:23 Signed 16-bit division.
pub fn div(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let second = ops[1].signed(&mut context)?;

    if second == 0 {
        return Err(GameError::InvalidOperation("Tried to divide by zero".into()).into());
    }

    let result = first / second;

    context.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

/// 2OP:24 Signed 16-bit modulo.
pub fn z_mod(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops[0].signed(&mut context)?;
    let second = ops[1].signed(&mut context)?;

    if second == 0 {
        return Err(GameError::InvalidOperation("Tried to divide by zero".into()).into());
    }

    let result = first % second;

    context.set_variable(store_to, result as u16);
    Ok(InstructionResult::Continue)
}

/// 1OP:133 Increment the provided variable.
pub fn inc(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops[0].unsigned(&mut context)?.try_into()?;
    let value = context.get_variable(variable_id)? as i16;
    context.set_variable(variable_id, (value + 1) as u16);
    Ok(InstructionResult::Continue)
}

/// 1OP:134 Decrement the provided variable.
pub fn dec(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops[0].unsigned(&mut context)?.try_into()?;
    let value = context.get_variable(variable_id)? as i16;
    context.set_variable(variable_id, (value - 1) as u16);
    Ok(InstructionResult::Continue)
}

/// 1OP:128 Jump if the argument equals zero.
pub fn jz(
    mut context: Context,
    ops: Vec<Operand>,
    condition: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops[0].unsigned(&mut context)?;

    let result = a == 0;

    Ok(context.frame.conditional_branch(offset, result, condition))
}

/// 1OP:138 Print the short name of the given object.
pub fn print_obj(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops[0].unsigned(&mut context)?;
    context
        .interface
        .print(&context.memory.object_short_name(object)?)?;

    Ok(InstructionResult::Continue)
}

/// 1OP:139 Returns from the current routine with the given value
pub fn ret(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Return(ops[0].unsigned(&mut context)?))
}

/// 1OP:140 Jump unconditionally
pub fn jump(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let offset = ops[0].signed(&mut context)?;

    Ok(context.frame.branch(offset))
}

/// 1OP:141 Prints a string stored at a padded address.
pub fn print_paddr(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    let address = context.memory.unpack_address(address.into());
    context
        .interface
        .print(&context.memory.extract_string(address, true)?.0)?;

    Ok(InstructionResult::Continue)
}

/// 0OP:176 Returns true (1).
pub fn rtrue(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Return(1))
}

/// 0OP:177 Returns false (0).
pub fn rfalse(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Return(0))
}

/// 0OP:178 Prints a string stored immediately after the instruction.
pub fn print(context: Context, string: String) -> Result<InstructionResult, Box<dyn Error>> {
    context.interface.print(&string)?;
    Ok(InstructionResult::Continue)
}

/// 0OP:179 Prints a literal string, prints a newline then returns from the current routine.
pub fn print_ret(context: Context, string: String) -> Result<InstructionResult, Box<dyn Error>> {
    context.interface.print(&string)?;
    context.interface.print(&"\n")?;

    Ok(InstructionResult::Return(1))
}

/// 0OP:180 Does nothing.
pub fn nop(_context: Context, _ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Continue)
}

/// 0OP:184 Returns the top of the stack.
pub fn ret_popped(context: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Return(context.frame.pop_stack()?))
}

/// 0OP:186 Exits the game.
pub fn quit(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(InstructionResult::Quit)
}

/// 0OP:187 Prints a newline
pub fn new_line(context: Context, _ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    context.interface.print(&"\n")?;

    Ok(InstructionResult::Continue)
}

/// VAR:224 Calls a routine with up to 3 operands and stores the result. If the address is
/// zero, does nothing and returns false.
pub fn call(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops[0].unsigned(&mut context)?;
    if address == 0 {
        context.set_variable(store_to, 0);
        return Ok(InstructionResult::Continue);
    }

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

/// VAR:225 Store a word in the given array and word index.
pub fn storew(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops[0].unsigned(&mut context)?;
    let word_index = ops[1].unsigned(&mut context)?;
    let value = ops[2].unsigned(&mut context)?;

    context
        .memory
        .set_word(usize::from(array + 2 * word_index), value);
    Ok(InstructionResult::Continue)
}

/// VAR:226 Store a byte in the given array and word index
pub fn storeb(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops[0].unsigned(&mut context)?;
    let byte_index = ops[1].unsigned(&mut context)?;
    let value = ops[2].unsigned(&mut context)?;

    context
        .memory
        .set_byte(usize::from(array + byte_index), value as u8);
    Ok(InstructionResult::Continue)
}

/// VAR:230 Print a signed number.
pub fn print_num(
    mut context: Context,
    ops: Vec<Operand>,
) -> Result<InstructionResult, Box<dyn Error>> {
    let num = ops[0].signed(&mut context)?;

    context.interface.print(&format!("{}", num))?;
    Ok(InstructionResult::Continue)
}

/// VAR:231 If the argument is >0, store a random number between 1 and the argument. If it is
/// less than 0, re-seed the RNG using the argument. If it is zero, re-seed the RNG randomly.
pub fn random(
    mut context: Context,
    ops: Vec<Operand>,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let range = ops[0].signed(&mut context)?;
    match range.cmp(&0) {
        Ordering::Less => {
            *context.rng = StdRng::seed_from_u64(-range as u64);
            context.set_variable(store_to, 0);
        }
        Ordering::Equal => {
            *context.rng = StdRng::from_entropy();
            context.set_variable(store_to, 0);
        }
        Ordering::Greater => {
            let result = context.rng.gen_range(1, range + 1);
            context.set_variable(store_to, result as u16);
        }
    };

    Ok(InstructionResult::Continue)
}

/// VAR:232 Pushes a value to the stack.
pub fn push(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let value = ops[0].unsigned(&mut context)?;
    context.frame.push_stack(value);

    Ok(InstructionResult::Continue)
}

/// VAR:233 Pulls a value off the stack and stores it.
pub fn pull(mut context: Context, ops: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
    let store_to = ops[0].unsigned(&mut context)? as u8;
    let value = context.frame.pop_stack()?;
    context.set_variable(store_to, value);

    Ok(InstructionResult::Continue)
}
