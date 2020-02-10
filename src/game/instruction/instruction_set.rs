use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;

use itertools::Itertools;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::error::GameError;
use crate::game::instruction::OpCode::{OneOp, TwoOp, VarOp, ZeroOp};
use crate::game::instruction::{
    Context, Instruction, OpCode, Operand, Result as InstructionResult,
};

/// Represents all the instructions available to the Z-Machine version specified in the game file.
pub struct InstructionSet {
    instructions: HashMap<OpCode, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<OpCode, Instruction> = [
            (TwoOp(0x1), Instruction::Branch(&common::je)),
            (TwoOp(0x2), Instruction::Branch(&common::jl)),
            (TwoOp(0x3), Instruction::Branch(&common::jg)),
            (TwoOp(0x8), Instruction::Store(&common::or)),
            (TwoOp(0x9), Instruction::Store(&common::and)),
            (TwoOp(0xD), Instruction::Normal(&common::store)),
            (TwoOp(0xF), Instruction::Store(&common::loadw)),
            (TwoOp(0x10), Instruction::Store(&common::loadb)),
            (TwoOp(0x14), Instruction::Store(&common::add)),
            (TwoOp(0x15), Instruction::Store(&common::sub)),
            (TwoOp(0x16), Instruction::Store(&common::mul)),
            (TwoOp(0x17), Instruction::Store(&common::div)),
            (TwoOp(0x18), Instruction::Store(&common::z_mod)),
            (OneOp(0x0), Instruction::Branch(&common::jz)),
            (OneOp(0x5), Instruction::Normal(&common::inc)),
            (OneOp(0x6), Instruction::Normal(&common::dec)),
            (OneOp(0xB), Instruction::Normal(&common::ret)),
            (OneOp(0xC), Instruction::Normal(&common::jump)),
            (OneOp(0xD), Instruction::Normal(&common::print_paddr)),
            (ZeroOp(0x0), Instruction::Normal(&common::rtrue)),
            (ZeroOp(0x1), Instruction::Normal(&common::rfalse)),
            (ZeroOp(0x2), Instruction::StringLiteral(&common::print)),
            (ZeroOp(0x3), Instruction::StringLiteral(&common::print_ret)),
            (ZeroOp(0x4), Instruction::Normal(&common::nop)),
            (ZeroOp(0x8), Instruction::Normal(&common::ret_popped)),
            (ZeroOp(0xA), Instruction::Normal(&common::quit)),
            (VarOp(0x6), Instruction::Normal(&common::print_num)),
            (VarOp(0x7), Instruction::Store(&common::random)),
            (VarOp(0x8), Instruction::Normal(&common::push)),
            (VarOp(0x9), Instruction::Normal(&common::pull)),
        ]
        .iter()
        .cloned()
        .collect();
        if version >= 4 {
            instructions.extend(
                [
                    (TwoOp(0x19), Instruction::Store(&version_gte4::call_2s)),
                    (VarOp(0x0), Instruction::Store(&version_gte4::call_vs)),
                    (
                        VarOp(0x11),
                        Instruction::Normal(&version_gte4::set_text_style),
                    ),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        if version >= 5 {
            instructions.extend(
                [
                    (OneOp(0xF), Instruction::Normal(&version_gte5::call_1n)),
                    (TwoOp(0x1A), Instruction::Normal(&version_gte5::call_2n)),
                    (VarOp(0x19), Instruction::Normal(&version_gte5::call_vn)),
                    (VarOp(0x1A), Instruction::Normal(&version_gte5::call_vn2)),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: &OpCode) -> Option<Instruction> {
        self.instructions.get(opcode).cloned()
    }
}

mod common {
    use super::*;

    ///20P:1 Branch if the first operand is equal to any subsequent operands
    pub fn je(
        mut context: Context,
        ops: Vec<Operand>,
        condition: bool,
        offset: i16,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let first = ops[0].unsigned(&mut context)?;

        for op in ops[1..].iter() {
            if let Some(value) = op.try_unsigned(&mut context)? {
                if (value == first) == condition {
                    return Ok(context.frame.branch(offset));
                }
            } else {
                break;
            }
        }
        Ok(InstructionResult::Continue)
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
        if (a < b) == condition {
            Ok(context.frame.branch(offset))
        } else {
            Ok(InstructionResult::Continue)
        }
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

        if (a > b) == condition {
            Ok(context.frame.branch(offset))
        } else {
            Ok(InstructionResult::Continue)
        }
    }

    // 2OP:8 Bitwise OR
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
    pub fn store(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
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

        let word = context.memory.get_word(usize::from(array + 2 * word_index));

        context.set_variable(store_to, word);
        Ok(InstructionResult::Continue)
    }

    /// 2OP:15 Store a byte found at the given array and byte index.
    pub fn loadb(
        mut context: Context,
        ops: Vec<Operand>,
        store_to: u8,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let array = ops[0].unsigned(&mut context)?;
        let byte_index = ops[1].unsigned(&mut context)?;

        let word = context.memory.get_word(usize::from(array + byte_index));

        context.set_variable(store_to, word);
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
    pub fn inc(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let variable_id: u8 = ops[0].unsigned(&mut context)?.try_into()?;
        let value = context.get_variable(variable_id)? as i16;
        context.set_variable(variable_id, (value + 1) as u16);
        Ok(InstructionResult::Continue)
    }

    /// 1OP:134 Decrement the provided variable.
    pub fn dec(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
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

        if (a == 0) == condition {
            Ok(context.frame.branch(offset))
        } else {
            Ok(InstructionResult::Continue)
        }
    }

    /// 1OP:139 Returns from the current routine with the given value
    pub fn ret(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(ops[0].unsigned(&mut context)?))
    }

    /// 1OP:140 Jump unconditionally
    pub fn jump(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
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
    pub fn print_ret(
        context: Context,
        string: String,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        context.interface.print(&string)?;
        context.interface.print(&"\n")?;
        Ok(InstructionResult::Return(1))
    }
    
    /// 0OP:180 Does nothing.
    pub fn nop (
        _context: Context,
        _ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Continue)
    }

    /// 0OP:184 Returns the top of the stack.
    pub fn ret_popped(
        context: Context,
        _: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(context.frame.pop_stack()?))
    }

    /// 0OP:186 Exits the game.
    pub fn quit(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Quit)
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
    pub fn push(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let value = ops[0].unsigned(&mut context)?;
        context.frame.push_stack(value);
        Ok(InstructionResult::Continue)
    }

    /// VAR:233 Pulls a value off the stack and stores it.
    pub fn pull(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let store_to = ops[0].unsigned(&mut context)? as u8;
        let value = context.frame.pop_stack()?;
        context.set_variable(store_to, value);
        Ok(InstructionResult::Continue)
    }
}

mod version_gte4 {
    use super::*;

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
}

mod version_gte5 {
    use super::*;

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
}
