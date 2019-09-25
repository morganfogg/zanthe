use crate::game::address;
use crate::game::alphabet::{Alphabet, AlphabetTable};
use crate::game::cursor::Cursor;
use crate::game::error::GameError;
use log::{error, info, warn};
use std::error::Error;
use std::io::{Seek, SeekFrom};

/// Stores the game's internal memory.
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(data: Vec<u8>) -> Memory {
        Memory { data }
    }

    /// Returns a 2 byte word from the game memory (most significant byte first).
    pub fn get_word(&self, address: usize) -> u16 {
        ((self.data[address] as u16) << 8) | self.data[address + 1] as u16
    }

    /// Returns a single byte from the memory.
    pub fn get_byte(&self, address: usize) -> u8 {
        self.data[address]
    }

    /// Return a series of bytes from the memory.
    pub fn get_bytes(&self, start: usize, length: usize) -> Vec<u8> {
        self.data[start..start + length].iter().cloned().collect()
    }

    pub fn write_byte(&mut self, address: usize, content: u8) {
        self.data[address] = content;
    }

    pub fn write_word(&mut self, address: usize, content: u16) {
        self.data[address] = (content >> 8) as u8;
        self.data[address + 1] = content as u8;
    }

    pub fn cursor(&self, start: usize) -> Cursor<&Memory> {
        Cursor::new(self, start)
    }

    pub fn mut_cursor(&mut self, start: usize) -> Cursor<&mut Memory> {
        Cursor::new(self, start)
    }

    /// Return the story file version.
    pub fn version(&self) -> u8 {
        self.get_byte(address::VERSION)
    }

    /// Return the expected result of the checksum operation.
    fn checksum(&self) -> u16 {
        self.get_word(address::CHECKSUM)
    }

    /// Return the starting point of high memory (containing the game's programming)
    fn high_memory_base(&self) -> u16 {
        self.get_word(address::HIGH_MEMORY_BASE)
    }

    /// Return the initial position of the program counter.
    pub fn program_counter_starts(&self) -> u16 {
        self.get_word(address::PROGRAM_COUNTER_STARTS)
    }

    /// Return the starting point of static memory (containing immutable game data).
    fn static_memory_base(&self) -> u16 {
        self.get_word(address::STATIC_MEMORY_BASE)
    }

    /// Return the location of the abbreviation table.
    fn abbreviation_table_location(&self) -> u16 {
        self.get_word(address::ABBREVIATION_TABLE_LOCATION)
    }

    /// Return the location of the object table
    fn object_table_location(&self) -> u16 {
        self.get_word(address::OBJECT_TABLE_LOCATION)
    }

    /// Return the location of the alphabet table
    /// (Zero indicates the default table should be used.)
    fn alphabet_table_location(&self) -> u16 {
        self.get_word(address::ALPHABET_TABLE_LOCATION)
    }

    /// Returns the content of the nth alphabet table.
    fn alphabet_table(&self, table: AlphabetTable) -> &[u8] {
        let start = self.alphabet_table_location() as usize + (table as usize * 26);
        &self.data[start..start + 26]
    }

    /// Return the location of the dictionary table
    fn dictionary_location(&self) -> u16 {
        self.get_word(address::DICTIONARY_LOCATION)
    }

    /// Return the story file's declared length (in bytes). This may be shorter than its actual
    /// length, as some files are zero-padded.
    fn file_length(&self) -> usize {
        let factor = match self.version() {
            1..=3 => 2,
            4..=5 => 4,
            _ => 8,
        };
        self.get_word(address::FILE_LENGTH) as usize * factor
    }

    pub fn data_length(&self) -> usize {
        self.data.len()
    }

    pub fn abbreviation_count(&self) -> usize {
        match self.version() {
            1 => 0,
            2 => 32,
            _ => 96,
        }
    }

    /// Return the length of each objects's attribute flags (in bytes).
    fn object_attribute_length(&self) -> usize {
        match self.version() {
            1..=3 => 4,
            _ => 6,
        }
    }

    /// Returns the maximum permitted file size for the file's version.
    fn max_file_length(&self) -> usize {
        match self.version() {
            1..=3 => 128 * 1024,
            4..=5 => 256 * 1024,
            6..=7 => 576 * 1024,
            _ => 512 * 1024,
        }
    }

    /// Return the total length of the object property defaults table (in words).
    fn property_defaults_length(&self) -> usize {
        match self.version() {
            1..=3 => 31,
            _ => 63,
        }
    }

    /// Return the length of an object's flag fields (in bytes)
    fn object_flag_length(&self) -> usize {
        match self.version() {
            1..=3 => 4,
            _ => 6,
        }
    }

    /// Return the total length of each entry in the object table (in bytes)
    fn object_entry_length(&self) -> usize {
        match self.version() {
            1..=3 => 9,
            _ => 14,
        }
    }

    /// Extract an encoded ZSCII character sequence from the memory.
    pub fn zscii_sequence(&self, start: usize) -> Vec<u8> {
        let mut z_chars = Vec::new();
        let mut cursor = self.cursor(start);

        loop {
            let word = cursor.read_word();
            z_chars.push(((word >> 10) & 0b11111) as u8);
            z_chars.push(((word >> 5) & 0b11111) as u8);
            z_chars.push((word & 0b11111) as u8);

            if word >> 15 != 0 {
                break;
            }
        }
        z_chars
    }

    /// Return the separator characters used when parsing input
    fn separators(&self) -> Result<Vec<char>, GameError> {
        let mut cursor: usize = self.dictionary_location().into();
        let num_separators: usize = self.get_byte(cursor).into();
        cursor += 1;
        (0..num_separators)
            .map(|i| {
                let result = self.get_byte(cursor + i);
                if result < 33 || result > 126 {
                    return Err(GameError::InvalidData("Unexpected word separator".into()));
                }
                Ok(result as char)
            })
            .collect()
    }

    pub fn dictionary_entry(&self, index: usize) -> Result<String, Box<dyn Error>> {
        if index == 0 {
            return Err(GameError::InvalidData("Dictionary index out of bounds".into()).into());
        }
        let mut cursor = self.cursor(self.dictionary_location().into());
        let num_separators: usize = cursor.read_byte().into();
        cursor.seek(SeekFrom::Current(num_separators as i64))?;
        let data_length: usize = cursor.read_byte().into();
        let entry_count: usize = cursor.read_word().into();
        if index > entry_count {
            return Err(GameError::InvalidData("Dictionary index out of bounds".into()).into());
        }
        self.extract_string(cursor.tell() + (index - 1) * data_length, true)
    }

    /// Look up an object in the object table
    pub fn object_entry(&self, id: usize) {
        let mut cursor: usize = self.object_table_location().into();
        cursor += self.property_defaults_length() * 2;
        let _flags: Vec<u8> = self.get_bytes(cursor, self.object_attribute_length());
        cursor += self.object_attribute_length();
        cursor += (id - 1) * self.object_entry_length();
        let (parent, sibling, child) = match self.version() {
            1..=3 => {
                let result = (
                    self.get_byte(cursor) as u16,
                    self.get_byte(cursor + 1) as u16,
                    self.get_byte(cursor + 2) as u16,
                );
                cursor += 3;
                result
            }
            _ => {
                let result = (
                    self.get_word(cursor),
                    self.get_word(cursor + 2),
                    self.get_word(cursor + 4),
                );
                cursor += 6;
                result
            }
        };
        let properties_address = self.get_word(cursor);
        println!(
            "Parent {} Sibling {} Child {} Properties {:x}",
            parent, sibling, child, properties_address
        );
        cursor = properties_address.into();

        let _short_name_length = self.get_byte(cursor);
        cursor += 1;
        let short_name = self.extract_string(cursor, true).unwrap();
        println!("{}", short_name);
    }

    /// Retrieve the location of an abbreviation from the abbreviation tables(s)
    pub fn abbreviation_entry(&self, table: usize, index: usize) -> usize {
        usize::from(
            self.get_word(
                self.abbreviation_table_location() as usize + (32 * (table - 1) + index) * 2,
            ) * 2,
        )
    }

    ///
    fn alphabet(&self) -> Alphabet {
        Alphabet::new(
            self.alphabet_table(AlphabetTable::A0),
            self.alphabet_table(AlphabetTable::A1),
            self.alphabet_table(AlphabetTable::A2),
        )
    }

    pub fn extract_string(
        &self,
        start: usize,
        abbreviations: bool,
    ) -> Result<String, Box<dyn Error>> {
        let sequence = self.zscii_sequence(start);
        let _byte_length = sequence.len() / 3 * 2;
        let mut sequence = sequence.iter();
        let mut result = Vec::new();
        let mut shift = false;
        let alphabet = Alphabet::default(self.version());
        let mut table = AlphabetTable::default();
        while let Some(c) = sequence.next() {
            match c {
                0 => result.push(' '),
                1..=3 if (self.version() >= 3 || *c == 1) => {
                    if !abbreviations {
                        return Err(GameError::InvalidData(
                            "Found abbreviation within an abbreviation".into(),
                        )
                        .into());
                    }
                    if let Some(abbreviation_id) = sequence.next() {
                        let abbreviation: String = self.extract_string(
                            self.abbreviation_entry(*c as usize, *abbreviation_id as usize),
                            false,
                        )?;
                        result.append(&mut abbreviation.chars().collect());
                    } else {
                        return Err(
                            GameError::InvalidData("String ended unexpectedly".into()).into()
                        );
                    }
                }
                2 => {
                    table = table.next();
                    shift = true;
                }
                3 => {
                    table = table.previous();
                    shift = true;
                }
                4 => {
                    table = table.next();
                    if self.version() >= 3 {
                        shift = true;
                    }
                }
                5 => {
                    table = table.previous();
                    if self.version() >= 3 {
                        shift = true;
                    }
                }
                _ => {
                    result.push(alphabet.value(table, *c));
                    if shift {
                        table = AlphabetTable::default();
                        shift = false;
                    }
                }
            }
        }
        Ok(result.iter().collect())
    }

    /// Calculates and checks the checksum of the file. The interpreter
    /// should continue as normal even if the checksum is incorrect.
    /// Should only be run once before program execution, as the data
    /// will change during execution.
    /// Refer to `verify` in Chapter 15 of the specification.
    pub fn verify(&self) -> bool {
        let mut file_length = self.file_length();
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
            .fold(0usize, |acc, &x| acc + usize::from(x))
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
            );
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
            // File is too large for its version
            error!("Invalid file size");
            return Err(GameError::InvalidFile);
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
