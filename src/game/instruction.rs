use std::collections::HashMap;
use std::error::Error;

use itertools::Itertools;

use crate::game::error::GameError;
use crate::game::memory::Memory;
use crate::game::operand::Operand;
use crate::game::stack::StackFrame;
use crate::game::state::GameState;

pub struct Context<'a> {
    frame: &'a mut StackFrame,
    memory: &'a mut Memory,
}

impl<'a> Context<'a> {
    pub fn new(frame: &'a mut StackFrame, memory: &'a mut Memory) -> Context<'a> {
        Context { frame, memory }
    }

    pub fn set_variable(&mut self, variable: u8, value: u16) {
        match variable {
            0 => self.frame.push_stack(value),
            1..=16 => {
                self.frame.set_local(variable as usize - 1, value);
            }
            _ => {
                self.memory.set_global(variable - 16, value);
            }
        }
    }

    pub fn get_variable(&mut self, variable: u8) -> Result<u16, Box<dyn Error>> {
        match variable {
            0 => self.frame.pop_stack(),
            1..=16 => Ok(self.frame.get_local(variable as usize - 1)),
            _ => Ok(self.memory.get_global(variable - 16)),
        }
    }
}

#[derive(Clone)]
pub enum Instruction {
    Normal(&'static dyn Fn(Context, Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>>),
    Branch(
        &'static dyn Fn(
            Context,
            Vec<Operand>,
            bool,
            u16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
    ),
    Return(&'static dyn Fn(Context, Vec<Operand>, u8) -> Result<InstructionResult, Box<dyn Error>>),
    StringLiteral(&'static dyn Fn(Context, String) -> Result<InstructionResult, Box<dyn Error>>),
}

pub enum InstructionResult {
    Continue,
    Return(u16),
    Quit,
    Invoke{address: usize, store_to: Option<u8>, arguments: Option<Vec<u16>>},
}

pub enum InstructionForm {
    Long,
    Short,
    Extended,
    Variable,
}

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
                [
                    (224, Instruction::Return(&version_gte4::call_vs))
                ]
                .iter()
                .cloned()
                .collect::<HashMap<u8, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        println!("Code: {0:x} ({0})", opcode);
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

    pub fn print(_: Context, string: String) -> Result<InstructionResult, Box<dyn Error>> {
        println!("{}", string);
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
        println!("{}", context.memory.extract_string(address, true)?.0);
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
        let address = ops[0].get_value(&mut context)?
            .ok_or_else(|| GameError::InvalidOperation("Missing required operand".into()))?;

        let address = context.memory.unpack_address(address as usize);
        let arguments: Vec<u16> = ops[1..].iter()
            .map(|op| op.get_value(&mut context))
            .collect::<Result<Vec<Option<u16>>, Box<dyn Error>>>()?
            .into_iter()
            .while_some()
            .collect();
        return Ok(InstructionResult::Invoke{
            address,
            arguments: Some(arguments),
            store_to: Some(store_to),
        });
    }
}
