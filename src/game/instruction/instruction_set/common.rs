use std::cmp::Ordering;
use std::convert::TryInto;
use tracing::warn;

use crate::game::Result;
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::error::GameError;
use crate::game::instruction::op_code::OpCode;
use crate::game::instruction::{
    Instruction, OperandSet,
    Result::{self as InstructionResult, *},
};
use crate::game::state::GameState;

pub fn instructions() -> Vec<(OpCode, Instruction)> {
    use Instruction::*;
    use OpCode::*;
    vec![
        (TwoOp(0x1), Branch(&je, "JE")),
        (TwoOp(0x2), Branch(&jl, "JL")),
        (TwoOp(0x3), Branch(&jg, "JG")),
        (TwoOp(0x4), Branch(&dec_chk, "DEC_CHK")),
        (TwoOp(0x5), Branch(&inc_chk, "INC_CHK")),
        (TwoOp(0x6), Branch(&jin, "JIN")),
        (TwoOp(0x7), Branch(&test, "TEST")),
        (TwoOp(0x8), Store(&or, "OR")),
        (TwoOp(0x9), Store(&and, "AND")),
        (TwoOp(0xA), Branch(&test_attr, "TEST_ATTR")),
        (TwoOp(0xB), Normal(&set_attr, "SET_ATTR")),
        (TwoOp(0xC), Normal(&clear_attr, "CLEAR_ATTR")),
        (TwoOp(0xD), Normal(&store, "STORE")),
        (TwoOp(0xE), Normal(&insert_obj, "INSERT_OBJ")),
        (TwoOp(0xF), Store(&loadw, "LOADW")),
        (TwoOp(0x10), Store(&loadb, "LOADB")),
        (TwoOp(0x11), Store(&get_prop, "GET_PROP")),
        (TwoOp(0x12), Store(&get_prop_addr, "GET_PROP_ADDR")),
        (TwoOp(0x13), Store(&get_next_prop, "GET_NEXT_PROP")),
        (TwoOp(0x14), Store(&add, "ADD")),
        (TwoOp(0x15), Store(&sub, "SUB")),
        (TwoOp(0x16), Store(&mul, "MUL")),
        (TwoOp(0x17), Store(&div, "DIV")),
        (TwoOp(0x18), Store(&z_mod, "MOD")),
        (OneOp(0x0), Branch(&jz, "JZ")),
        (OneOp(0x1), BranchStore(&get_sibling, "GET_SIBLING")),
        (OneOp(0x2), BranchStore(&get_child, "GET_CHILD")),
        (OneOp(0x3), Store(&get_parent, "GET_PARENT")),
        (OneOp(0x4), Store(&get_prop_len, "GET_PROP_LEN")),
        (OneOp(0x5), Normal(&inc, "INC")),
        (OneOp(0x6), Normal(&dec, "DEC")),
        (OneOp(0x7), Normal(&print_addr, "PRINT_ADDR")),
        (OneOp(0x9), Normal(&remove_obj, "REMOVE_OBJ")),
        (OneOp(0xA), Normal(&print_obj, "PRINT_OBJ")),
        (OneOp(0xB), Normal(&ret, "RET")),
        (OneOp(0xC), Normal(&jump, "JUMP")),
        (OneOp(0xD), Normal(&print_paddr, "PRINT_PADDR")),
        (OneOp(0xE), Store(&load, "LOAD")),
        (OneOp(0xF), Store(&not, "NOT")), // Moved in V5
        (ZeroOp(0x0), Normal(&rtrue, "RTRUE")),
        (ZeroOp(0x1), Normal(&rfalse, "RFALSE")),
        (ZeroOp(0x2), StringLiteral(&print, "PRINT")),
        (ZeroOp(0x3), StringLiteral(&print_ret, "PRINT_RET")),
        (ZeroOp(0x4), Normal(&nop, "NOP")),
        (ZeroOp(0x7), Normal(&restart, "RESTART")),
        (ZeroOp(0x8), Normal(&ret_popped, "RET_POPPED")),
        (ZeroOp(0xA), Normal(&quit, "QUIT")),
        (ZeroOp(0xB), Normal(&new_line, "NEW_LINE")),
        (VarOp(0x0), Store(&call, "CALL")),
        (VarOp(0x1), Normal(&storew, "STOREW")),
        (VarOp(0x2), Normal(&storeb, "STOREB")),
        (VarOp(0x3), Normal(&put_prop, "PUT_PROP")),
        (VarOp(0x5), Normal(&print_char, "PRINT_CHAR")),
        (VarOp(0x6), Normal(&print_num, "PRINT_NUM")),
        (VarOp(0x7), Store(&random, "RANDOM")),
        (VarOp(0x8), Normal(&push, "PUSH")),
        (VarOp(0x9), Normal(&pull, "PULL")),
    ]
}

///20P:1 Branch if the first operand is equal to any subsequent operands
pub fn je(
    state: &mut GameState,
    mut ops: OperandSet,
    expected: bool,
    offset: i16,
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
    let variable_id: u8 = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
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
) -> Result<InstructionResult> {
    let variable_id: u8 = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
    let bitmap = ops.pull()?.unsigned(state)?;
    let flags = ops.pull()?.unsigned(state)?;

    let condition = bitmap & flags == flags;

    Ok(state
        .frame()
        .conditional_branch(offset, condition, expected))
}

/// 2OP:8 Bitwise OR
pub fn or(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let x = ops.pull()?.unsigned(state)?;
    let y = ops.pull()?.unsigned(state)?;

    let result = x | y;

    state.set_variable(store_to, result);

    Ok(Continue)
}

// 2OP:9 Bitwise AND
pub fn and(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
pub fn set_attr(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
pub fn clear_attr(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
pub fn store(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let variable = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
    let value = ops.pull()?.unsigned(state)?;

    state.poke_variable(variable, value)?;
    Ok(Continue)
}

/// 2OP:14 Move object to be the first child of the destination object
pub fn insert_obj(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
    let array: usize = ops.pull()?.unsigned(state)?.into();
    let word_index: isize = ops.pull()?.signed(state)?.into();
    let word = state.memory.get_word(if word_index < 0 {
        array - ((-word_index as usize) * 2)
    } else {
        array + (word_index as usize * 2)
    });

    state.set_variable(store_to, word);
    Ok(Continue)
}

/// 2OP:16 Store a byte found at the given array and byte index.
pub fn loadb(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
    let array: usize = ops.pull()?.unsigned(state)?.into();
    let byte_index: isize = ops.pull()?.signed(state)?.into();
    let byte = state.memory.get_byte(if byte_index < 0 {
        array - (-byte_index as usize)
    } else {
        array + (byte_index as usize)
    });
    state.set_variable(store_to, byte as u16);
    Ok(Continue)
}

/// 2OP:17 Return the data of the specified property
pub fn get_prop(
    state: &mut GameState,
    mut ops: OperandSet,
    store_to: u8,
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
pub fn add(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;
    let result = first.wrapping_add(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

// 2OP:21 Signed 16-bit subtraction
pub fn sub(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;
    let result = first.wrapping_sub(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 2OP:22 Signed 16-bit multiplication.
pub fn mul(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    let result = first.wrapping_mul(second);

    state.set_variable(store_to, result as u16);
    Ok(Continue)
}

/// 2OP:23 Signed 16-bit division.
pub fn div(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    if second == 0 {
        return Err(GameError::invalid_operation("Tried to divide by zero"));
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
) -> Result<InstructionResult> {
    let first = ops.pull()?.signed(state)?;
    let second = ops.pull()?.signed(state)?;

    if second == 0 {
        return Err(GameError::invalid_operation("Tried to divide by zero"));
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;

    let result = if address == 0 {
        0
    } else {
        state.memory.property_data_length(address as usize)
    };
    state.set_variable(store_to, result);
    Ok(Continue)
}

/// 1OP:133 Increment the provided variable.
pub fn inc(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let variable_id: u8 = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
    let value = state.peek_variable(variable_id)? as i16;

    let result = value.wrapping_add(1) as u16;

    state.poke_variable(variable_id, result)?;
    Ok(Continue)
}

/// 1OP:134 Decrement the provided variable.
pub fn dec(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let variable_id: u8 = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
    let value = state.peek_variable(variable_id)? as i16;

    let result = value.wrapping_sub(1) as u16;

    state.poke_variable(variable_id, result)?;
    Ok(Continue)
}

/// 1OP:135 Prints a string stored at a padded address.
pub fn print_addr(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)? as usize;

    state
        .interface
        .print(&state.memory.extract_string(address, true)?.0)?;

    Ok(Continue)
}

/// 1OP:137 Detach an object from its parents and siblings
pub fn remove_obj(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let object = ops.pull()?.unsigned(state)?;
    if object == 0 {
        warn!("remove_obj called with object 0");
    } else {
        state.memory.detach_object(object);
    }

    Ok(Continue)
}

/// 1OP:138 Print the short name of the given object.
pub fn print_obj(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let object = ops.pull()?.unsigned(state)?;
    state
        .interface
        .print(&state.memory.object_short_name(object)?)?;

    Ok(Continue)
}

/// 1OP:139 Returns from the current routine with the given value
pub fn ret(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    Ok(Return(ops.pull()?.unsigned(state)?))
}

/// 1OP:140 Jump unconditionally
pub fn jump(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let offset = ops.pull()?.signed(state)?;

    Ok(state.frame().branch(offset))
}

/// 1OP:141 Prints a string stored at a padded address.
pub fn print_paddr(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    let address = state.memory.unpack_address(address.into());
    state
        .interface
        .print(&state.memory.extract_string(address, true)?.0)?;

    Ok(Continue)
}

/// 1OP:142 Load the variable referred to by the operand into the result
pub fn load(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let variable_id = ops
        .pull()?
        .unsigned(state)?
        .try_into()
        .map_err(|_| GameError::invalid_operation("Invalid variable ID"))?;
    let value = state.peek_variable(variable_id)?;

    state.set_variable(store_to, value);
    Ok(Continue)
}

/// 1OP:143 (v1-4)
/// VAR:248 (v5+) Bitwise NOT
pub fn not(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let op = ops.pull()?.unsigned(state)?;

    let result = !op;

    state.set_variable(store_to, result);
    Ok(Continue)
}

/// 0OP:176 Returns true (1).
pub fn rtrue(_: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Return(1))
}

/// 0OP:177 Returns false (0).
pub fn rfalse(_: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Return(0))
}

/// 0OP:178 Prints a string stored immediately after the instruction.
pub fn print(state: &mut GameState, string: String) -> Result<InstructionResult> {
    state.interface.print(&string)?;
    Ok(Continue)
}

/// 0OP:179 Prints a literal string, prints a newline then returns from the current routine.
pub fn print_ret(state: &mut GameState, string: String) -> Result<InstructionResult> {
    state.interface.print(&string)?;
    state.interface.print("\n")?;

    Ok(Return(1))
}

/// 0OP:180 Does nothing.
pub fn nop(_state: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Continue)
}

/// 0OP:183 Restart the game. The only preserved information are the 'transcribing to printer' bit
/// and the 'use fixed pitch font' bit.

pub fn restart(_state: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Restart)
}

/// 0OP:184 Returns the top of the stack.
pub fn ret_popped(state: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Return(state.frame().pop_stack()?))
}

/// 0OP:186 Exits the game.
pub fn quit(_: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    Ok(Quit)
}

/// 0OP:187 Prints a newline
pub fn new_line(state: &mut GameState, _: OperandSet) -> Result<InstructionResult> {
    state.interface.print("\n")?;

    Ok(Continue)
}

/// VAR:224 Calls a routine with up to 3 operands and stores the result. If the address is
/// zero, does nothing and returns false.
pub fn call(state: &mut GameState, mut ops: OperandSet, store_to: u8) -> Result<InstructionResult> {
    let address = ops.pull()?.unsigned(state)?;
    if address == 0 {
        state.set_variable(store_to, 0);
        return Ok(Continue);
    }

    let address = state.memory.unpack_address(address as usize);
    let arguments: Vec<u16> = ops
        .map(|op| op.try_unsigned(state))
        .collect::<Result<Vec<Option<u16>>>>()?
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
pub fn storew(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let array: usize = ops.pull()?.unsigned(state)?.into();
    let word_index: isize = ops.pull()?.signed(state)?.into();
    let value = ops.pull()?.unsigned(state)?;

    state.memory.set_word(
        if word_index < 0 {
            array - ((-word_index as usize) * 2)
        } else {
            array + (word_index as usize * 2)
        },
        value,
    );
    Ok(Continue)
}

/// VAR:226 Store a byte in the given array and word index
pub fn storeb(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let array: usize = ops.pull()?.unsigned(state)?.into();
    let byte_index: isize = ops.pull()?.signed(state)?.into();
    let value = ops.pull()?.unsigned(state)?;

    state.memory.set_byte(
        if byte_index < 0 {
            array - (-byte_index as usize)
        } else {
            array + (byte_index as usize)
        },
        value as u8,
    );
    Ok(Continue)
}

/// VAR:227 Update the property data of the goven object
pub fn put_prop(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let object_id = ops.pull()?.unsigned(state)?;
    let property_id = ops.pull()?.unsigned(state)?;
    let value = ops.pull()?.unsigned(state)?;

    let property = state
        .memory
        .property(object_id, property_id)
        .ok_or_else(|| GameError::invalid_operation("Property data doesn't exist"))?;

    match property.data.len() {
        1 => state
            .memory
            .set_byte(property.data_address as usize, value as u8),
        2 => state.memory.set_word(property.data_address as usize, value),
        _ => {
            return Err(GameError::invalid_operation(
                "Cannot assign property with length greater than 2",
            ))
        }
    }
    Ok(Continue)
}

/// VAR:229 Print a ZSCII character
pub fn print_char(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let char_id = ops.pull()?.unsigned(state)?;

    let c = state.memory.alphabet().decode_zscii(char_id)?;
    if let Some(c) = c {
        state.interface.print_char(c)?;
    }

    Ok(Continue)
}

/// VAR:230 Print a signed number.
pub fn print_num(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
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
) -> Result<InstructionResult> {
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
            let result = state.rng.gen_range(1..=range);
            state.set_variable(store_to, result as u16);
        }
    };

    Ok(Continue)
}

/// VAR:232 Pushes a value to the stack.
pub fn push(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let value = ops.pull()?.unsigned(state)?;
    state.frame().push_stack(value);

    Ok(Continue)
}

/// VAR:233 Pulls a value off the stack and stores it.
pub fn pull(state: &mut GameState, mut ops: OperandSet) -> Result<InstructionResult> {
    let store_to = ops.pull()?.unsigned(state)? as u8;
    let value = state.frame().pop_stack()?;
    state.poke_variable(store_to, value)?;

    Ok(Continue)
}
