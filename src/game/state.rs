use crate::game::address;
use crate::game::error::GameError;
use log::{error, info, warn};
use std::vec::Vec;

pub struct GameState {
    pub data: Vec<u8>,
    pub counter: u16,
    pub checksum_valid: bool,
}

impl GameState {
    pub fn new(data: Vec<u8>) -> Result<GameState, GameError> {
        GameState::validate_header(&data)?;
        Ok(GameState {
            counter: GameState::get_word_from_data(&data, address::PROGRAM_COUNTER_STARTS),
            checksum_valid: GameState::calculate_checksum(&data),
            data,
        })
    }

    fn get_word_from_data(data: &[u8], address: usize) -> u16 {
        ((data[address] as u16) << 8) + data[address + 1] as u16
    }

    pub fn _get_word(&self, address: usize) -> u16 {
        GameState::get_word_from_data(&self.data, address)
    }

    fn calculate_checksum(data: &[u8]) -> bool {
        /*! Calculates and checks the checksum of the file, by summing all
            all bytes from the end of the header to the stated end of the
            file as defined in the header, modulo 0x10000. The interpreter
            should continue as normal even if the checksum is incorrect. 
            Refer to `verify` in Chapter 15 of the specification. */

        // The file 
        let factor = match data[address::VERSION] {
            1...3 => 2,
            4...5 => 4,
            6...8 => 8,
            _ => panic!("Not implemented"),
        };
        let mut file_length: usize =
            GameState::get_word_from_data(&data, address::FILE_LENGTH) as usize * factor;
        if file_length > data.len() {
            return false;
        }

        if file_length == 0 {
            file_length = data.len();
        }

        let expected = GameState::get_word_from_data(&data, address::CHECKSUM) as usize;
        let result: usize = data[0x40..file_length.into()]
            .iter()
            .fold(0usize, |acc, x| acc + *x as usize)
            % 0x10000;
        if expected == result {
            info!(
                "Checksum OKAY: Expected {:x}, found {:x}. Stated file length {}",
                expected, result, file_length
            );    
        } else {
            warn!(
                "Checksum ERROR: Expected {:x}, found {:x}. Stated file length {}",
                expected, result, file_length
            )
        }
        expected == result
    }

    /// Does some sanity checking on the header section of the file to ensure the input is valid.
    fn validate_header(data: &[u8]) -> Result<(), GameError> {
        let len = data.len();
        if len < 64 {
            // Header alone must be at least 64 bytes long
            error!("File too small to be valid");
            return Err(GameError::InvalidFile);
        }

        let version = data[address::VERSION];
        if version == 6 {
            error!("Version 6 file provided");
            return Err(GameError::VersionSix);
        }

        if version > 8 || version == 0 {
            // Version byte is outside expected/supported range
            error!("Invalid version byte");
            return Err(GameError::InvalidFile);
        }
        if len > 512 * 1024
            || (version <= 5 && len > 256 * 1024)
            || (version <= 3 && len > 128 * 1024)
        {
            // File is too large for its version
            error!("Invalid file size");
            return Err(GameError::InvalidFile);
        }

        let static_memory_base = GameState::get_word_from_data(&data, address::STATIC_MEMORY_BASE);
        if static_memory_base < 64 || static_memory_base as usize > len - 1 {
            error!("Invalid static memory base");
            return Err(GameError::InvalidFile);
        }

        let high_memory_base = GameState::get_word_from_data(&data, address::HIGH_MEMORY_BASE);
        if high_memory_base < 64
            || high_memory_base as usize > len - 1
            || high_memory_base <= static_memory_base
        {
            error!("Invalid high memory base");
            return Err(GameError::InvalidFile);
        }

        let program_counter_starts =
            GameState::get_word_from_data(&data, address::PROGRAM_COUNTER_STARTS);
        if program_counter_starts < high_memory_base {
            error!("Program counter does not start in high memory");
            return Err(GameError::InvalidFile);
        }
        info!("Header validation OKAY");
        info!(
            "Static Base: {:x}. High base: {:x}. PC starts: {:x}",
            static_memory_base, high_memory_base, program_counter_starts,
        );
        Ok(())
    }
}
