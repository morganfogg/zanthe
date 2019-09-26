use std::vec::Vec;

use crate::game::error::GameError;
use crate::game::memory::Memory;
use crate::game::routine::Routine;

pub struct GameState {
    pub memory: Memory,
    pub checksum_valid: bool,
    pub version: u8,
}

impl GameState {
    pub fn new(data: Vec<u8>) -> Result<GameState, GameError> {
        let memory = Memory::new(data);
        memory.validate_header()?;
        Ok(GameState {
            checksum_valid: memory.verify(),
            version: memory.version(),
            memory,
        })
    }


    pub fn run(&mut self) {
        let mut cursor = self
            .memory
            .cursor(self.memory.program_counter_starts().into());
        Routine::new(&mut cursor).invoke();
    }
}
