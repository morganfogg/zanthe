use std::collections::HashMap;
use std::error::Error;

use crate::game::operand::Operand;
use crate::game::routine::Routine;
use crate::game::cursor::Cursor;

pub enum Instruction {
    Normal(&'static dyn Fn(&mut Routine, Vec<Operand>) -> InstructionResult),
    Branch(&'static dyn Fn(&mut Routine, Vec<Operand>, bool, u16) -> InstructionResult),
    Return(&'static dyn Fn(&mut Routine, Vec<Operand>, u8) -> InstructionResult),
    StringLiteral(&'static dyn Fn(&mut Routine, String) -> InstructionResult),
}

#[derive(Debug)]
pub enum InstructionResult {
    Continue,
    Return(u8),
    Throw(usize),
    Quit,
    Error(Box<dyn Error>),
}

pub struct InstructionSet {
    instructions: HashMap<u8, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions = HashMap::new();
        instructions.insert(178, Instruction::StringLiteral(&common::print));
        instructions.insert(186, Instruction::Normal(&common::quit));
        if version >= 4 {
            instructions.insert(224, Instruction::Return(&common::call_vs));
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        self.instructions.get(&opcode)
    }
}

mod common {
    use super::{InstructionResult, Operand, Routine, Cursor};

    pub fn quit(_: &mut Routine, _: Vec<Operand>) -> InstructionResult {
        InstructionResult::Quit
    }
    
    pub fn print(routine: &mut Routine, string: String) -> InstructionResult {
        println!("print called with {}", string);
        InstructionResult::Continue
    }

    pub fn call_vs(routine: &mut Routine, ops: Vec<Operand>, result: u8) -> InstructionResult {
        InstructionResult::Continue
    }
}
