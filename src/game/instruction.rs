use std::collections::HashMap;
use std::error::Error;

use crate::game::cursor::Cursor;
use crate::game::error::GameError;
use crate::game::operand::Operand;
use crate::game::routine::Routine;

#[derive(Clone)]
pub enum Instruction {
    Normal(
        &'static dyn Fn(&mut Routine, Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>>,
    ),
    Branch(
        &'static dyn Fn(
            &mut Routine,
            Vec<Operand>,
            bool,
            u16,
        ) -> Result<InstructionResult, Box<dyn Error>>,
    ),
    Return(
        &'static dyn Fn(
            &mut Routine,
            Vec<Operand>,
            u8,
        ) -> Result<InstructionResult, Box<dyn Error>>,
    ),
    StringLiteral(
        &'static dyn Fn(&mut Routine, String) -> Result<InstructionResult, Box<dyn Error>>,
    ),
}

#[derive(Debug)]
pub enum InstructionResult {
    Continue,
    Return(u16),
    Throw(usize),
    Quit,
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
                [(224, Instruction::Return(&version_gte4::call_vs))]
                    .iter()
                    .cloned()
                    .collect::<HashMap<u8, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        self.instructions.get(&opcode)
    }
}

mod common {
    use super::*;

    pub fn rtrue(_: &mut Routine, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(1))
    }

    pub fn rfalse(_: &mut Routine, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Return(0))
    }

    pub fn quit(_: &mut Routine, _: Vec<Operand>) -> Result<InstructionResult, Box<dyn Error>> {
        Ok(InstructionResult::Quit)
    }

    pub fn print(
        routine: &mut Routine,
        string: String,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        println!("print called with {}", string);
        Ok(InstructionResult::Continue)
    }

    pub fn print_paddr(
        routine: &mut Routine,
        ops: Vec<Operand>,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = match ops[0].get_value(routine)? {
            Some(v) => v,
            None => {
                return Err(GameError::InvalidOperation("Missing required operand".into()).into())
            }
        };

        let address = routine.memory().unpack_address(address.into());
        println!("{}", routine.memory().extract_string(address, true)?.0);
        Ok(InstructionResult::Continue)
    }
}

mod version_gte4 {
    use super::*;
    pub fn call_vs(
        routine: &mut Routine,
        ops: Vec<Operand>,
        variable: u8,
    ) -> Result<InstructionResult, Box<dyn Error>> {
        let address = match ops[0].get_value(routine)? {
            Some(v) => v,
            None => {
                return Err(GameError::InvalidOperation("Missing required operand".into()).into())
            }
        };

        let instruction_set = routine.instruction_set;
        let memory = routine.mut_memory();
        let address = memory.unpack_address(address as usize);
        let subroutine_cursor = Cursor::new(memory, address as usize);

        let mut subroutine = Routine::new(subroutine_cursor, instruction_set);
        subroutine.prepare_locals()?;
        let result = subroutine.invoke()?;

        match result {
            InstructionResult::Continue => unreachable!(),
            InstructionResult::Return(e) => {
                routine.set_variable(variable, e);
                Ok(InstructionResult::Continue)
            }
            InstructionResult::Quit => Ok(result),
            InstructionResult::Throw(_) => unimplemented!(), //TODO: Implement this
        }
    }
}
