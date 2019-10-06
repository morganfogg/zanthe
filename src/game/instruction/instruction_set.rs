use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;

use itertools::Itertools;
use log::info;

use crate::game::error::GameError;
use crate::game::instruction::{
    Context, Instruction, OpCode, Operand, Result as InstructionResult,
};

pub struct InstructionSet {
    instructions: HashMap<OpCode, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<OpCode, Instruction> = [
            (OpCode::TwoOp(0xD), Instruction::Normal(&common::store)),
            (
                OpCode::OneOp(0xD),
                Instruction::Normal(&common::print_paddr),
            ),
            (OpCode::ZeroOp(0x0), Instruction::Normal(&common::rtrue)),
            (OpCode::ZeroOp(0x1), Instruction::Normal(&common::rfalse)),
            (
                OpCode::ZeroOp(0x2),
                Instruction::StringLiteral(&common::print),
            ),
            (OpCode::ZeroOp(0xA), Instruction::Normal(&common::quit)),
            (OpCode::VarOp(0x6), Instruction::Normal(&common::print_num)),
        ]
        .iter()
        .cloned()
        .collect();
        if version >= 4 {
            instructions.extend(
                [(
                    OpCode::VarOp(0x0),
                    Instruction::Store(&version_gte4::call_vs),
                )]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        if version >= 5 {
            instructions.extend(
                [(
                    OpCode::OneOp(0xF),
                    Instruction::Normal(&version_gte5::call_1n),
                )]
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
        let variable: u8 = ops[0]
            .get_unsigned(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?
            .try_into()
            .map_err(|_| GameError::InvalidOperation("Invalid variable number".into()))?;
        let value = ops[1]
            .get_unsigned(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;

        context.set_variable(variable, value);
        Ok(InstructionResult::Continue)
    }

    /// 1OP:114 Prints a string stored at a padded address.
    pub fn print_paddr(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = ops[0]
            .get_unsigned(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;
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

    /// 0OP:186 Exits the game.
    pub fn quit(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Quit)
    }

    /// VAR:230 Print a signed number.
    pub fn print_num(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let num = ops[0]
            .get_signed(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;
        context.interface.print(&format!("{}", num))?;
        Ok(InstructionResult::Continue)
    }
}

mod version_gte4 {
    use super::*;

    /// VAR:224 Calls a routine with up to 3 arguments and stores the result.
    pub fn call_vs(
        mut context: Context,
        ops: Vec<Operand>,
        store_to: u8,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = ops[0]
            .get_unsigned(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;

        let address = context.memory.unpack_address(address as usize);
        let arguments: Vec<u16> = ops[1..]
            .iter()
            .map(|op| op.get_unsigned(&mut context))
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
        let address = ops[0]
            .get_unsigned(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;

        let address = context.memory.unpack_address(address as usize);

        return Ok(InstructionResult::Invoke {
            address,
            arguments: None,
            store_to: None,
        });
    }
}
