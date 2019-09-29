use std::error::Error;
use std::vec::Vec;

use crate::game::error::GameError;
use crate::game::instruction::{InstructionResult, InstructionSet};
use crate::game::memory::Memory;
use crate::game::routine::Routine;

pub struct GameState {
    pub memory: Memory,
    pub checksum_valid: bool,
    pub version: u8,
    pub instruction_set: InstructionSet,
}

impl GameState {
    pub fn new(data: Vec<u8>) -> Result<GameState, GameError> {
        let memory = Memory::new(data);
        memory.validate_header()?;
        Ok(GameState {
            checksum_valid: memory.verify(),
            version: memory.version(),
            instruction_set: InstructionSet::new(memory.version()),
            memory,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut cursor = self
            .memory
            .mut_cursor(self.memory.program_counter_starts().into());
        match Routine::new(cursor, &self.instruction_set).invoke()? {
            InstructionResult::Quit => Ok(()),
            InstructionResult::Continue => panic!("Unexpeded continue"),
            InstructionResult::Return(_) => {
                Err(GameError::InvalidOperation("Cannot return from main routine".into()).into())
            }
            InstructionResult::Throw(_) => {
                Err(GameError::InvalidOperation("Uncaught throw".into()).into())
            }
        }
    }
}
