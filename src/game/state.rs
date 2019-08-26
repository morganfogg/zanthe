use crate::game::address;
use crate::game::error::GameError;
use crate::game::memory::Memory;
use std::vec::Vec;

pub struct GameState {
    pub memory: Memory,
    pub counter: u16,
    pub checksum_valid: bool,
}

impl GameState {
    pub fn new(data: Vec<u8>) -> Result<GameState, GameError> {
        let memory = Memory::new(data);
        memory.validate_header()?;
        Ok(GameState {
            counter: memory.get_word(address::PROGRAM_COUNTER_STARTS),
            checksum_valid: memory.verify(),
            memory,
        })
    }
}
