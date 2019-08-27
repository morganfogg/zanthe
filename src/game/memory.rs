use crate::game::address;
use crate::game::alphabet::Alphabet;
use crate::game::error::GameError;
use log::{error, info, warn};
use std::io::{self, Seek, SeekFrom};

pub struct Memory {
    data: Vec<u8>,
    cursor: usize,
}

impl Memory {
    pub fn new(data: Vec<u8>) -> Memory {
        Memory { data, cursor: 0 }
    }

    pub fn get_word(&self, address: usize) -> u16 {
        ((self.data[address] as u16) << 8) | self.data[address + 1] as u16
    }

    pub fn get_byte(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub fn read_word(&mut self) -> u16 {
        let result = self.get_word(self.cursor);
        self.cursor += 2;
        result
    }

    pub fn version(&self) -> u8 {
        self.data[address::VERSION]
    }

    pub fn checksum(&self) -> u16 {
        self.get_word(address::CHECKSUM)
    }

    pub fn high_memory_base(&self) -> u16 {
        self.get_word(address::HIGH_MEMORY_BASE)
    }

    pub fn program_counter_starts(&self) -> u16 {
        self.get_word(address::PROGRAM_COUNTER_STARTS)
    }

    pub fn static_memory_base(&self) -> u16 {
        self.get_word(address::STATIC_MEMORY_BASE)
    }

    pub fn abbreviation_table_location(&self) -> u16 {
        self.get_word(address::ABBREVIATION_TABLE_LOCATION)
    }

    pub fn dictionary_location(&self) -> u16 {
        self.get_word(address::DICTIONARY_LOCATION)
    }

    pub fn dictionary_word_length(&self) -> usize {
        match self.version() {
            1...3 => 4,
            _ => 6,
        }
    }

    pub fn max_file_length(&self) -> usize {
        match self.version() {
            1...3 => 128 * 1024,
            4...5 => 256 * 1024,
            _ => 512 * 1024,
        }
    }

    /// TODO: Implement custom alphabet tables
    fn ztext_to_string(&self, mut cursor: usize) -> String {
        let mut result: Vec<char> = vec![];
        let mut active = Alphabet::A0;
        let mut shift = false;
        loop {
            let word = self.get_word(cursor);
            cursor += 2;

            let chars = vec![
                ((word >> 10) & 0b11111) as u8,
                ((word >> 5) & 0b11111) as u8,
                (word & 0b11111) as u8,
            ];
            for c in chars.iter() {
                match c {
                    0 => result.push(' '),
                    1...3 => {
                        // TODO: Actually implement this
                        result.push('@');
                    }
                    4 => {
                        active = active.next();
                        if self.version() > 3 {
                            shift = true;
                        }
                    }
                    5 => {
                        active = active.previous();
                        if self.version() > 3 {
                            shift = true;
                        }
                    }
                    _ => {
                        result.push(active.character(self.version(), c - 6));
                        if shift {
                            active = Alphabet::A0;
                            shift = false;
                        }
                    }
                }
            }
            if word >> 15 != 0 {
                break;
            }
        }
        result.iter().collect()
    }

    fn separators(&self) -> Vec<char> {
        let mut cursor: usize = self.dictionary_location().into();
        let num_separators: usize = self.get_byte(cursor).into();
        cursor += 1;
        (0..num_separators)
            .map(|i| {
                let result = self.get_byte(cursor + i);
                if result < 33 || result > 126 {
                    error!("Unexpected word separator");
                    panic!("Unexpected word separator");
                }
                result as char
            })
            .collect()
    }

    fn dictionary_entry(&self, index: usize) -> String {
        let mut cursor: usize = self.dictionary_location().into();
        let num_separators: usize = self.get_byte(cursor).into();
        cursor += num_separators + 1;
        let data_length: usize = self.get_byte(cursor).into();
        cursor += 1;
        let entry_count: usize = self.get_word(cursor).into();
        if index > entry_count {
            panic!("Invalid dictionary entry");
        }
        cursor += 2;
        self.ztext_to_string(cursor + (index - 1) * data_length)
    }

    /// Calculates and checks the checksum of the file. The interpreter
    /// should continue as normal even if the checksum is incorrect.
    /// Should only be run once before program execution, as the data
    /// will change during execution.
    /// Refer to `verify` in Chapter 15 of the specification.
    pub fn verify(&mut self) -> bool {
        // The file length field is divided by a factor, which differs between versions.
        let factor = match self.version() {
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

        // Stated length of 0 is used by V6/V7 files that exceed the specification's size limits.
        if file_length == 0 {
            file_length = self.data.len();
        }

        let expected: usize = self.checksum().into();
        let result: usize = self.data[0x40..file_length.into()]
            .iter()
            .fold(0usize, |acc, x| acc + usize::from(*x))
            % 0x10000;
        if expected == result {
            info!(
                "Checksum OKAY: Expected {:x}, found {:x}. Stated file length {}. Actual file length {}",
                expected, result, file_length, self.data.len()
            );
        } else {
            warn!(
                "Checksum ERROR: Expected {:x}, found {:x}. Stated file length {}. Actual file length {}",
                expected, result, file_length, self.data.len()
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

        if self.version() == 6 {
            error!("Version 6 file provided");
            return Err(GameError::VersionSix);
        }

        if self.version() > 8 || self.version() == 0 {
            // Version byte is outside expected/supported range
            error!("Invalid version byte");
            return Err(GameError::InvalidFile);
        }
        if len > self.max_file_length() {
            // File is too large for its version, this is permitted in Version 7+
            if self.version() < 7 {
                error!("Invalid file size");
                return Err(GameError::InvalidFile);
            } else {
                warn!("File exceeds standard size limit");
            }
        }

        let static_memory_base: usize = self.static_memory_base().into();
        if static_memory_base < 64 || static_memory_base > len - 1 {
            error!("Invalid static memory base");
            return Err(GameError::InvalidFile);
        }

        let high_memory_base: usize = self.high_memory_base().into();
        if high_memory_base < 64
            || high_memory_base > len - 1
            || high_memory_base <= static_memory_base
        {
            error!("Invalid high memory base");
            return Err(GameError::InvalidFile);
        }

        let program_counter_starts: usize = self.program_counter_starts().into();
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

impl Seek for Memory {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        let old_cursor = self.cursor;
        match pos {
            SeekFrom::Start(e) => {
                if e as usize > self.data.len() - 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Seek out of bounds",
                    ));
                }
                self.cursor = e as usize;
            }
            SeekFrom::End(e) => {
                self.cursor = match (self.data.len() - 1).checked_add(e as usize) {
                    Some(i) => i,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Seek out of bounds",
                        ))
                    }
                };
            }
            SeekFrom::Current(e) => {
                self.cursor = match self.cursor.checked_add(e as usize) {
                    Some(i) => i,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Seek out of bounds",
                        ))
                    }
                };
            }
        }
        if self.cursor > self.data.len() - 1 {
            self.cursor = old_cursor;
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Seek out of bounds",
            ));
        }
        Ok(self.cursor as u64)
    }
}
