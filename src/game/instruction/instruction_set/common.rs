use log::warn;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::error::Error;

use itertools::Itertools;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::error::GameError;
use crate::game::instruction::{
    OperandSet,
    Result::{self as InstructionResult, *},
};
use crate::game::state::GameState;

///20P:1 Branch if the first operand is equal to any subsequent operands
pub fn je(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let mut condition = false;
    for op in ops {
        if let Some(value) = op.try_signed(state)? {
            if value == first {
                condition = true;
                break;
            }
        } else {
            break;
        }
    }

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:2 Jump if a < b (signed).
pub fn jl(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops.pull()?.signed(state)?;
    let b = ops.pull()?.signed(state)?;

    let condition = a < b;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:3 Jump if a > b (signed).
pub fn jg(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops.pull()?.signed(state)?;
    let b = ops.pull()?.signed(state)?;

    let condition = a > b;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:4 Decrement the variable and branch if it is now less than the given value
pub fn dec_chk(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops.pull()?.unsigned(state)?.try_into()?;
    let comparand = ops.pull()?.signed(state)?;
    let value = (state.peek_variable(variable_id)? as i16).wrapping_sub(1);

    state.poke_variable(variable_id, value as u16)?;

    let condition = value < comparand;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:5 Increment the variable and branch if it is now greater than the given value
pub fn inc_chk(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops.pull()?.unsigned(state)?.try_into()?;
    let comparand = ops.pull()?.signed(state)?;
    let value = (state.peek_variable(variable_id)? as i16).wrapping_add(1);

    state.poke_variable(variable_id, value as u16)?;

    let condition = value > comparand;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:6 Jump if object a's parent is object b
pub fn jin(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_a = ops.pull()?.unsigned(state)?;
    let object_b = ops.pull()?.unsigned(state)?;
    let parent = if object_a == 0 || object_b == 0 {
        warn!("@jin called with object 0");
        0
    } else {
        state.memory.object_parent(object_a)
    };

    let condition = object_b == parent;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:7 Jump if `bitmap & flags == flags`
pub fn test(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let bitmap = ops.pull()?.unsigned(state)?;
    let flags = ops.pull()?.unsigned(state)?;

    let condition = bitmap & flags == flags;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:8 Bitwise OR
pub fn or(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let x = ops.pull()?.unsigned(state)?;
    let y = ops.pull()?.unsigned(state)?;

    let result = x | y;

    state.set_variable(store_to, result);

    Ok(Continue)
}

// 2OP:9 Bitwise AND
pub fn and(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let x = ops.pull()?.unsigned(state)?;
    let y = ops.pull()?.unsigned(state)?;

    let result = x & y;

    state.set_variable(store_to, result);

    Ok(Continue)
}

/// 2OP:10 Jump of the object has the given attribute
pub fn test_attr(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;
    let attribute = ops.pull()?.unsigned(state)?;
    let flag_set = if object_id == 0 {
        warn!("test_attr called with object 0");
        false
    } else {
        state.memory.object_attribute(object_id, attribute)
    };

    Ok(state.frame().conditional_branch(offset, flag_set, expected))
}

/// 2OP:11 Set the attribute on the provided object to true
pub fn set_attr(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;
    let attribute = ops.pull()?.unsigned(state)?;
    if object_id == 0 {
        warn!("set_attr called on object 0");
    } else {
        state
            .memory
            .update_object_attribute(object_id, attribute, true);
    }

    Ok(Continue)
}

/// 2OP:12 Set the attribute on the provided object to false
pub fn clear_attr(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;
    let attribute = ops.pull()?.unsigned(state)?;
    if object_id == 0 {
        warn!("@clear_attr called on object 0")
    } else {
        state
            .memory
            .update_object_attribute(object_id, attribute, false);
    }
    Ok(Continue)
}

/// 2OP:13 Set the variable referenced by the operand to value
pub fn store(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable = ops.pull()?.unsigned(state)?;
    let value = ops.pull()?.unsigned(state)?;

    state.poke_variable(variable.try_into()?, value)?;
    Ok(Continue)
}

/// 2OP:14 Move object to be the first child of the destination object
pub fn insert_obj(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;
    let destination = ops.pull()?.unsigned(state)?;
    if object == 0 || destination == 0 {
        warn!("insert_obj called with object 0");
        return Ok(Continue);
    }

    let old_child = state.memory.object_child(destination);

    state.memory.detach_object(object);

    state.memory.set_object_parent(object, destination);
    state.memory.set_object_child(destination, object);
    state.memory.set_object_sibling(object, old_child);

    Ok(Continue)
}

/// 2OP:15 Store a word found at the given array and word index.
pub fn loadw(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops.pull()?.unsigned(state)?;
    let word_index = ops.pull()?.unsigned(state)?;
    let word = state.memory.get_word(usize::from(array + (2 * word_index)));

    state.set_variable(store_to, word);
    Ok(Continue)
}

/// 2OP:16 Store a byte found at the given array and byte index.
pub fn loadb(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops.pull()?.unsigned(state)?;
    let byte_index = ops.pull()?.unsigned(state)?;
    let byte = state.memory.get_byte(usize::from(array + byte_index));

    state.set_variable(store_to, byte as u16);
    Ok(Continue)
}

/// 2OP:17 Return the data of the specified property
pub fn get_prop(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;
    let property = ops.pull()?.unsigned(state)?;

    let data = if object == 0 {
        warn!("get_prop called with object 0");
        0
    } else {
        state
            .memory
            .property(object, property)
            .map(|prop| prop.data_to_u16())
            .transpose()?
            .unwrap_or_else(|| state.memory.default_property(property))
    };
    state.set_variable(store_to, data);
    Ok(Continue)
}

/// 2OP:18 Return the byte address of the specified property data
pub fn get_prop_addr(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;
    let property = ops.pull()?.unsigned(state)?;

    let address = if object == 0 {
        warn!("@get-prop_addr called with 0");
        0
    } else {
        state
            .memory
            .property(object, property)
            .map(|prop| prop.data_address)
            .unwrap_or(0)
    };

    state.set_variable(store_to, address);
    Ok(Continue)
}

/// 2OP:19 Get the number of the next property after the proided one
pub fn get_next_prop(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;

    if object == 0 {
        warn!("@get_next_prop called with object 0");
        state.set_variable(store_to, 0);
        return Ok(Continue);
    }

    let property = ops.pull()?.unsigned(state)?;

    let next_prop = if property == 0 {
        state.memory.property_iter(object).next()
    } else {
        state.memory.following_property(object, property)
    };

    let next_prop_number = next_prop.map(|p| p.number).unwrap_or(0);

    state.set_variable(store_to, next_prop_number);
    Ok(Continue)
}

/// 2OP:20 Signed 16-bit addition
pub fn add(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;
    let result = first.wrapping_add(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

// 2OP:21 Signed 16-bit subtraction
pub fn sub(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;
    let result = first.wrapping_sub(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 2OP:22 Signed 16-bit multiplication.
pub fn mul(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    let result = first.wrapping_mul(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 2OP:23 Signed 16-bit division.
pub fn div(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    if second == 0 {
        return Err(GameError::InvalidOperation("Tried to divide by zero".into()).into());
    }

    let result = first.wrapping_div(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 2OP:24 Signed 16-bit modulo.
pub fn z_mod(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    if second == 0 {
        return Err(GameError::InvalidOperation("Tried to divide by zero".into()).into());
    }

    let result = first.wrapping_rem(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 1OP:128 Jump if the argument equals zero.
pub fn jz(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult, Box<dyn Error>> {
    let a = ops.pull()?.unsigned(state)?;

    let condition = a == 0;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 1OP:129 Store the object's sibling and branch if it exists (is not zero).
pub fn get_sibling(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;

    let result = if object_id == 0 {
        warn!("@get_sibling called with object 0");
        0
    } else {
        state.memory.object_sibling(object_id)
    };

    state.set_variable(store_to, result);

    let condition = result != 0;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 1OP:130 Store the object's child
pub fn get_child(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;
    let result = if object_id == 0 {
        warn!("@get_child called with object 0");
        0
    } else {
        state.memory.object_child(object_id)
    };

    state.set_variable(store_to, result);

    let condition = result != 0;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 1OP:131 Stores the object's parent
pub fn get_parent(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;

    let result = if object_id == 0 {
        warn!("@get_parent called with object 0");
        0
    } else {
        state.memory.object_parent(object_id)
    };

    state.set_variable(store_to, result);
    Ok(Continue)
}

/// 1OP:132 Get the length of propery at the provided address
pub fn get_prop_len(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;

    let result = if address == 0 {
        0
    } else {
        state.memory.property_data_length(address as usize)
    };
    state.set_variable(store_to, result.try_into()?);
    Ok(Continue)
}

/// 1OP:133 Increment the provided variable.
pub fn inc(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops.pull()?.unsigned(state)?.try_into()?;
    let value = state.peek_variable(variable_id)? as i16;

    let result = value.wrapping_add(1) as u16;

    state.poke_variable(variable_id, result)?;
    Ok(Continue)
}

/// 1OP:134 Decrement the provided variable.
pub fn dec(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id: u8 = ops.pull()?.unsigned(state)?.try_into()?;
    let value = state.peek_variable(variable_id)? as i16;

    let result = value.wrapping_sub(1) as u16;

    state.poke_variable(variable_id, result)?;
    Ok(Continue)
}

/// 1OP:135 Prints a string stored at a padded address.
pub fn print_addr(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)? as usize;

    state
        .interface
        .print(&state.memory.extract_string(address, true)?.0)?;

    Ok(Continue)
}

/// 1OP:137 Detach an object from its parents and siblings
pub fn remove_obj(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;
    if object == 0 {
        warn!("remove_obj called with object 0");
    } else {
        state.memory.detach_object(object);
    }

    Ok(Continue)
}

/// 1OP:138 Print the short name of the given object.
pub fn print_obj(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object = ops.pull()?.unsigned(state)?;
    state
        .interface
        .print(&state.memory.object_short_name(object)?)?;

    Ok(Continue)
}

/// 1OP:139 Returns from the current routine with the given value
pub fn ret(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Return(ops.pull()?.unsigned(state)?))
}

/// 1OP:140 Jump unconditionally
pub fn jump(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let offset = ops.pull()?.signed(state)?;

    Ok(state.frame().branch(offset))
}

/// 1OP:141 Prints a string stored at a padded address.
pub fn print_paddr(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address.into());
    state
        .interface
        .print(&state.memory.extract_string(address, true)?.0)?;

    Ok(Continue)
}

/// 1OP:142 Load the variable referred to by the operand into the result
pub fn load(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let variable_id = ops.pull()?.unsigned(state)?;
    let value = state.peek_variable(variable_id.try_into()?)?;

    state.set_variable(store_to, value);
    Ok(Continue)
}

/// 1OP:143 (v1-4)
/// VAR:248 (v5+) Bitwise NOT
pub fn not(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let op = ops.pull()?.unsigned(state)?;

    let result = !op;

    state.set_variable(store_to, result);
    Ok(Continue)
}

/// 0OP:176 Returns true (1).
pub fn rtrue(_: &mut GameState, _: OperandSet) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Return(1))
}

/// 0OP:177 Returns false (0).
pub fn rfalse(_: &mut GameState, _: OperandSet) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Return(0))
}

/// 0OP:178 Prints a string stored immediately after the instruction.
pub fn print(state: &mut GameState, string: String) -> Result<InstructionResult, Box<dyn Error>> {
    state.interface.print(&string)?;
    Ok(Continue)
}

/// 0OP:179 Prints a literal string, prints a newline then returns from the current routine.
pub fn print_ret(
    state: &mut GameState,
    string: String,
) -> Result<InstructionResult, Box<dyn Error>> {
    state.interface.print(&string)?;
    state.interface.print(&"\n")?;

    Ok(Return(1))
}

/// 0OP:180 Does nothing.
pub fn nop(_state: &mut GameState, _: OperandSet) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Continue)
}

/// 0OP:184 Returns the top of the stack.
pub fn ret_popped(
    state: &mut GameState,
    _: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Return(state.frame().pop_stack()?))
}

/// 0OP:186 Exits the game.
pub fn quit(_: &mut GameState, _: OperandSet) -> Result<InstructionResult, Box<dyn Error>> {
    Ok(Quit)
}

/// 0OP:187 Prints a newline
pub fn new_line(state: &mut GameState, _: OperandSet) -> Result<InstructionResult, Box<dyn Error>> {
    state.interface.print(&"\n")?;

    Ok(Continue)
}

/// VAR:224 Calls a routine with up to 3 operands and stores the result. If the address is
/// zero, does nothing and returns false.
pub fn call(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let address = ops.pull()?.unsigned(state)?;
    if address == 0 {
        state.set_variable(store_to, 0);
        return Ok(Continue);
    }

    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>, Box<dyn Error>>>()?
        .into_iter()
        .while_some()
        .collect();

    Ok(Invoke {
        address,
        arguments: Some(arguments),
        store_to: Some(store_to),
    })
}

/// VAR:225 Store a word in the given array and word index.
pub fn storew(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops.pull()?.unsigned(state)?;
    let word_index = ops.pull()?.unsigned(state)?;
    let value = ops.pull()?.unsigned(state)?;

    state
        .memory
        .set_word(usize::from(array + 2 * word_index), value);
    Ok(Continue)
}

/// VAR:226 Store a byte in the given array and word index
pub fn storeb(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let array = ops.pull()?.unsigned(state)?;
    let byte_index = ops.pull()?.unsigned(state)?;
    let value = ops.pull()?.unsigned(state)?;

    state
        .memory
        .set_byte(usize::from(array + byte_index), value as u8);
    Ok(Continue)
}

/// VAR:227 Update the property data of the goven object
pub fn put_prop(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let object_id = ops.pull()?.unsigned(state)?;
    let property_id = ops.pull()?.unsigned(state)?;
    let value = ops.pull()?.unsigned(state)?;

    let property = state
        .memory
        .property(object_id, property_id)
        .ok_or_else(|| GameError::InvalidOperation("Property data doesn't exist".into()))?;

    match property.data.len() {
        1 => state
            .memory
            .set_byte(property.data_address as usize, value as u8),
        2 => state.memory.set_word(property.data_address as usize, value),
        _ => {
            return Err(GameError::InvalidOperation(
                "Cannot assign property with length greater than 2".into(),
            )
            .into())
        }
    }
    Ok(Continue)
}

/// VAR:229 Print a ZSCII character
pub fn print_char(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let char_id = ops.pull()?.unsigned(state)?;

    let char = state.memory.alphabet().decode_zscii(char_id)?;
    if let Some(char) = char {
        state.interface.print_char(char)?;
    }

    Ok(Continue)
}

/// VAR:230 Print a signed number.
pub fn print_num(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let num = ops.pull()?.signed(state)?;

    state.interface.print(&format!("{}", num))?;
    Ok(Continue)
}

/// VAR:231 If the argument is >0, store a random number between 1 and the argument. If it is
/// less than 0, re-seed the RNG using the argument. If it is zero, re-seed the RNG randomly.
pub fn random(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult, Box<dyn Error>> {
    let range = ops.pull()?.signed(state)?;
    match range.cmp(&0) {
        Ordering::Less => {
            state.rng = StdRng::seed_from_u64(-range as u64);
            state.set_variable(store_to, 0);
        }
        Ordering::Equal => {
            state.rng = StdRng::from_entropy();
            state.set_variable(store_to, 0);
        }
        Ordering::Greater => {
            let result = state.rng.gen_range(1, range + 1);
            state.set_variable(store_to, result as u16);
        }
    };

    Ok(Continue)
}

/// VAR:232 Pushes a value to the stack.
pub fn push(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let value = ops.pull()?.unsigned(state)?;
    state.frame().push_stack(value);

    Ok(Continue)
}

/// VAR:233 Pulls a value off the stack and stores it.
pub fn pull(
    state: &mut GameState,
    mut ops: OperandSet,
) -> Result<InstructionResult, Box<dyn Error>> {
    let store_to = ops.pull()?.unsigned(state)? as u8;
    let value = state.frame().pop_stack()?;
    state.poke_variable(store_to, value)?;

    Ok(Continue)
}
