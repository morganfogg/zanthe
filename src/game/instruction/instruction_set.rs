use std::collections::HashMap;
use std::error::Error;

use itertools::Itertools;
use log::info;

use crate::game::error::GameError;
use crate::game::instruction::{Context, Instruction, Operand, Result as InstructionResult};

pub struct InstructionSet {
    instructions: HashMap<u8, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<u8, Instruction> = [
            (141, Instruction::Normal(&common::print_paddr)),
            (176, Instruction::Normal(&common::rtrue)),
            (177, Instruction::Normal(&common::rfalse)),
            (178, Instruction::StringLiteral(&common::print)),
            (186, Instruction::Normal(&common::quit)),
        ]
        .iter()
        .cloned()
        .collect();
        if version >= 4 {
            instructions.extend(
                [(224, Instruction::Return(&version_gte4::call_vs))]
                    .iter()
                    .cloned()
                    .collect::<HashMap<u8, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        info!("Code: {0:x} ({0})", opcode);
        self.instructions.get(&opcode)
    }
}

mod common {
    use super::*;

    pub fn rtrue(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(1))
    }

    pub fn rfalse(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(0))
    }

    pub fn quit(_: Context, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Quit)
    }

    pub fn print(context: Context, string: String) -> Result<InstructionResult, Box<dyn Error>> {
        context.interface.print(&string);
        Ok(InstructionResult::Continue)
    }

    pub fn print_paddr(
        mut context: Context,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = ops[0]
            .get_value(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;
        let address = context.memory.unpack_address(address.into());
        context.interface.print(&context.memory.extract_string(address, true)?.0);
        Ok(InstructionResult::Continue)
    }
}

mod version_gte4 {
    use super::*;
    pub fn call_vs(
        mut context: Context,
        ops: Vec<Operand>,
        store_to: u8,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = ops[0]
            .get_value(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;

        let address = context.memory.unpack_address(address as usize);
        let arguments: Vec<u16> = ops[1..]
            .iter()
            .map(|op| op.get_value(&mut context))
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
