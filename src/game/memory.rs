use crate::game::address;
use crate::game::error::GameError;
use log::error;
use log::{info, warn};

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(data: Vec<u8>) -> Memory {
        Memory { data }
    }

    pub fn get_word(&self, address: usize) -> u16 {
        ((self.data[address] as u16) << 8) | self.data[address + 1] as u16
    }

    pub fn _get_byte(&self, address: usize) -> u8 {
        self.data[address]
    }

    /// Calculates and checks the checksum of the file, by summing all
    /// all bytes from the end of the header to the stated end of the
    /// file as defined in the header, modulo 0x10000. The interpreter
    /// should continue as normal even if the checksum is incorrect.
    /// Refer to `verify` in Chapter 15 of the specification.
    pub fn checksum(&self) -> bool {
        // The file length field is divided by a factor, which differs between versions.
        let factor = match self.data[address::VERSION] {
            1...3 => 2,
            4...5 => 4,
            6...8 => 8,
            _ => panic!("Not implemented"),
        };
        let mut file_length: usize = self.get_word(address::FILE_LENGTH) as usize * factor;
        if file_length > self.data.len() {
            warn!("File length header invalid");
            return false;
        }

        // File length of 0 is used by V6/V7 files that exceed the specification's size limits.
        if file_length == 0 {
            file_length = self.data.len();
        }

        let expected: usize = self.get_word(address::CHECKSUM).into();
        let result: usize = self.data[0x40..file_length.into()]
            .iter()
            .fold(0usize, |acc, x| acc + usize::from(*x))
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

    /// Does some sanity checking on the header section of the data to
    /// ensure the input is valid.
    pub fn validate_header(&self) -> Result<(), GameError> {
        let len = self.data.len();
        if len < 64 {
            // Header alone must be at least 64 bytes long
            error!("File too small to be valid");
            return Err(GameError::InvalidFile);
        }

        let version = self.data[address::VERSION];
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

        let static_memory_base: usize = self.get_word(address::STATIC_MEMORY_BASE).into();
        if static_memory_base < 64 || static_memory_base > len - 1 {
            error!("Invalid static memory base");
            return Err(GameError::InvalidFile);
        }

        let high_memory_base: usize = self.get_word(address::HIGH_MEMORY_BASE).into();
        if high_memory_base < 64
            || high_memory_base > len - 1
            || high_memory_base <= static_memory_base
        {
            error!("Invalid high memory base");
            return Err(GameError::InvalidFile);
        }

        let program_counter_starts: usize = self.get_word(address::PROGRAM_COUNTER_STARTS).into();
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
