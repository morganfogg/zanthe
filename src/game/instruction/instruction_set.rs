use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;

use itertools::Itertools;
use log::info;

use crate::game::error::GameError;
use crate::game::instruction::{
    Context, Instruction,
    OpCode::{self, OneOp, TwoOp, VarOp, ZeroOp},
    Operand, Result as InstructionResult,
};

pub struct InstructionSet {
    instructions: HashMap<OpCode, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<OpCode, Instruction> = [
            (TwoOp(0xD), Instruction::Normal(&common::store)),
            (TwoOp(0x14), Instruction::Store(&common::add)),
            (TwoOp(0x15), Instruction::Store(&common::sub)),
            (TwoOp(0x16), Instruction::Store(&common::mul)),
            (TwoOp(0x17), Instruction::Store(&common::div)),
            (TwoOp(0x18), Instruction::Store(&common::z_mod)),
            (OneOp(0xD), Instruction::Normal(&common::print_paddr)),
            (ZeroOp(0x0), Instruction::Normal(&common::rtrue)),
            (ZeroOp(0x1), Instruction::Normal(&common::rfalse)),
            (ZeroOp(0x2), Instruction::StringLiteral(&common::print)),
            (ZeroOp(0x8), Instruction::Normal(&common::ret_popped)),
            (ZeroOp(0xA), Instruction::Normal(&common::quit)),
            (VarOp(0x6), Instruction::Normal(&common::print_num)),
        ]
        .iter()
        .cloned()
        .collect();
        if version >= 4 {
            instructions.extend(
                [
                    (TwoOp(0x19), Instruction::Store(&version_gte4::call_2s)),
                    (VarOp(0x0), Instruction::Store(&version_gte4::call_vs)),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        if version >= 5 {
            instructions.extend(
                [(OneOp(0xF), Instruction::Normal(&version_gte5::call_1n))]
                    .iter()
                    .cloned()
                    .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: &OpCode) -> Option<&Instruction> {
        self.instructions.get(opcode)
    }
}

mod common {
    use super::*;

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

    /// 2OP:23 Signed 16-bit modulo.
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

    /// 1OP:114 Prints a string stored at a padded address.
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
        return Ok(InstructionResult::Invoke {
            address,
            arguments: Some(arguments),
            store_to: Some(store_to),
        });
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

        return Ok(InstructionResult::Invoke {
            address,
            arguments: Some(arguments),
            store_to: Some(store_to),
        });
    }
}

mod version_gte5 {
    use super::*;

    /// 1OP:143 Calls a routine with no arguments and throws away the result.
    pub fn call_1n(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = ops[0].unsigned(&mut context)?;

        let address = context.memory.unpack_address(address as usize);

        return Ok(InstructionResult::Invoke {
            address,
            arguments: None,
            store_to: None,
        });
    }
}
