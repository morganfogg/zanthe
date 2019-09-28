use std::collections::HashMap;
use std::error::Error;

use crate::game::cursor::Cursor;
use crate::game::error::GameError;
use crate::game::operand::Operand;
use crate::game::routine::Routine;

#[derive(Clone)]
pub enum Instruction {
    Normal(&'static dyn Fn(&mut Routine, Vec<Operand>) -> InstructionResult),
    Branch(&'static dyn Fn(&mut Routine, Vec<Operand>, bool, u16) -> InstructionResult),
    Return(&'static dyn Fn(&mut Routine, Vec<Operand>, u8) -> InstructionResult),
    StringLiteral(&'static dyn Fn(&mut Routine, String) -> InstructionResult),
}

#[derive(Debug)]
pub enum InstructionResult {
    Continue,
    Return(u16),
    Throw(usize),
    Quit,
    Error(Box<dyn Error>),
}

pub struct InstructionSet {
    instructions: HashMap<u8, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<u8, Instruction> = [
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

    pub fn quit(_: &mut Routine, _: Vec<Operand>) -> InstructionResult {
        InstructionResult::Quit
    }

    pub fn print(routine: &mut Routine, string: String) -> InstructionResult {
        println!("print called with {}", string);
        InstructionResult::Continue
    }
}

mod version_gte4 {
    use super::*;
    pub fn call_vs(routine: &mut Routine, ops: Vec<Operand>, variable: u8) -> InstructionResult {
        let address = match ops[0] {
            Operand::LargeConstant(v) => v,
            Operand::SmallConstant(v) => v as u16,
            Operand::Variable(v) => routine.get_variable(v),
            Operand::Omitted => {
                return InstructionResult::Error(
                    GameError::InvalidData("Required operand not present".into()).into(),
                )
            }
        };

        let instruction_set = routine.instruction_set;
        let memory = routine.mut_memory();
        let address = memory.unpack_address(address as usize);
        let mut subroutine_cursor = Cursor::new(memory, address as usize);

        println!("Addr {:x}", address);

        let mut subroutine = Routine::new(subroutine_cursor, instruction_set);
        if let Err(e) = subroutine.prepare_locals() {
            return InstructionResult::Error(e);
        }
        let result = subroutine.invoke();

        match result {
            InstructionResult::Continue => unreachable!(),
            InstructionResult::Return(e) => {
                routine.set_variable(variable, e);
                InstructionResult::Continue
            }
            InstructionResult::Error(_) | InstructionResult::Quit => result,
            InstructionResult::Throw(_) => unimplemented!(), //TODO: Implement this
        }
    }
}
