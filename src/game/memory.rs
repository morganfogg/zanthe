use std::char;
use std::error::Error;

use log::{error, info, warn};

use crate::game::address;
use crate::game::alphabet::{Alphabet, AlphabetTable};
use crate::game::error::GameError;
use crate::game::instruction::Operand;
use crate::game::property::Property;

/// Represents the game's internal memory.
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(data: Vec<u8>) -> Memory {
        Memory { data }
    }

    /// Returns a 2-byte word from the game memory (most significant byte first).
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

    /// Read a byte from the memory, placing the cursor at the end of the word.
    pub fn read_byte(&self, cursor: &mut usize) -> u8 {
        let result = self.get_byte(*cursor);
        *cursor += 1;
        result
    }

    /// Read a 2-byte word from the memory, placing the cursor at the end of the word.
    pub fn read_word(&self, cursor: &mut usize) -> u16 {
        let result = self.get_word(*cursor);
        *cursor += 2;
        result
    }

    /// Update a byte in memory.
    pub fn set_byte(&mut self, address: usize, content: u8) {
        self.data[address] = content;
    }

    /// Update a word in memory.
    pub fn set_word(&mut self, address: usize, content: u16) {
        self.data[address] = (content >> 8) as u8;
        self.data[address + 1] = content as u8;
    }

    /// Update a byte in memory, placing the cursor after the byte updated.
    pub fn write_byte(&mut self, cursor: &mut usize, content: u8) {
        self.set_byte(*cursor, content);
        *cursor += 1;
    }

    /// Update a word to the memory, placing the cursor after the word updated.
    pub fn write_word(&mut self, cursor: &mut usize, content: u16) {
        self.set_word(*cursor, content);
        *cursor += 2;
    }

    /// Read an operand from a long-form operation.
    pub fn read_operand_long(&self, cursor: &mut usize, op_type: u8) -> Operand {
        match op_type {
            0 => Operand::SmallConstant(self.read_byte(cursor)),
            1 => Operand::Variable(self.read_byte(cursor)),
            _ => unreachable!(),
        }
    }

    /// Read an operand from a short, variable or extended-form operation.
    pub fn read_operand_other(&mut self, cursor: &mut usize, op_type: u8) -> Operand {
        match op_type {
            0 => Operand::LargeConstant(self.read_word(cursor)),
            1 => Operand::SmallConstant(self.read_byte(cursor)),
            2 => Operand::Variable(self.read_byte(cursor)),
            3 => Operand::Omitted,
            _ => unreachable!(),
        }
    }

    /// Extract a string from the memory, placing the cursor at the end of the string.
    pub fn read_string(&mut self, cursor: &mut usize) -> Result<String, Box<dyn Error>> {
        let (string, len) = self.extract_string(*cursor, true)?;
        *cursor += len;

        Ok(string)
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

    /// Return the location of the global variable table.
    fn global_variable_table_location(&self) -> u16 {
        self.get_word(address::GLOBAL_VARIABLE_TABLE_LOCATION)
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

    /// Returns the location of the header extension table.
    fn header_extension_table_location(&self) -> u16 {
        self.get_word(address::HEADER_EXTENSION_TABLE_LOCATION)
    }

    /// Returns the story's unicode translation table, or None if the default table
    /// should be used.
    fn unicode_translation_table(&self) -> Option<Vec<char>> {
        match self.get_word(
            self.header_extension_table_location() as usize
                + (2 * address::UNICODE_TRANSLATION_TABLE_LOCATION),
        ) {
            0 => None,
            addr @ _ => {
                let mut cursor = addr as usize;
                let table_length = self.read_byte(&mut cursor) as usize;
                (0..table_length)
                    .map(|i| char::from_u32(self.get_word(cursor + (i * 2)) as u32))
                    .collect()
            }
        }
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

    /// Returns the actual length of the game data.
    pub fn data_length(&self) -> usize {
        self.data.len()
    }

    /// Return the length of each objects's attribute flags (in bytes).
    fn object_attribute_length(&self) -> u16 {
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

    /// Return the total length of the object property defaults table (in bytes).
    fn property_defaults_length(&self) -> u16 {
        match self.version() {
            1..=3 => 62,
            _ => 126,
        }
    }

    /// Return the total length of each entry in the object table (in bytes)
    fn object_entry_length(&self) -> u16 {
        match self.version() {
            1..=3 => 9,
            _ => 14,
        }
    }

    /// Decompress a packed address.
    pub fn unpack_address(&self, address: usize) -> usize {
        match self.version() {
            1..=3 => 2 * address,
            4..=5 => 4 * address,
            8 => 8 * address,
            _ => panic!("Implement me"), //TODO: Implement this
        }
    }

    /// Extract an encoded Z-Character character sequence from the memory.
    pub fn character_sequence(&self, mut cursor: usize) -> Vec<u8> {
        let mut z_chars = Vec::new();

        loop {
            let word = self.read_word(&mut cursor);
            z_chars.push(((word >> 10) & 0b11111) as u8);
            z_chars.push(((word >> 5) & 0b11111) as u8);
            z_chars.push((word & 0b11111) as u8);

            if word >> 15 != 0 {
                break;
            }
        }
        z_chars
    }

    fn object_relation_length(&self) -> u16 {
        match self.version() {
            1..=3 => 1,
            _ => 2,
        }
    }

    pub fn object_location(&self, object_id: u16) -> u16 {
        self.object_table_location()
            + self.property_defaults_length()
            + ((object_id - 1) * self.object_entry_length())
    }

    pub fn object_attribute(&self, object_id: u16, attribute: u16) -> bool {
        let location = self.object_location(object_id) as usize;
        let offset = attribute as usize / 8;
        let bit = attribute as usize % 8;
        let mask = 1 << (7 - bit);

        self.get_byte(location + offset) & mask != 0
    }

    pub fn update_object_attribute(&mut self, object_id: u16, attribute: u16, set: bool) {
        let location = self.object_location(object_id) as usize;
        let offset = attribute as usize / 8;
        let bit = attribute as usize % 8;

        let mut flags = self.get_byte(location + offset);
        let mask = 1 << (7 - bit);

        if set {
            flags |= mask
        } else {
            flags &= !mask
        };

        self.set_byte(location + offset, flags);
    }

    pub fn object_parent(&self, object: u16) -> u16 {
        let location = self.object_location(object) + self.object_attribute_length();
        match self.version() {
            1..=3 => self.get_byte(location as usize) as u16,
            _ => self.get_word(location as usize),
        }
    }

    pub fn object_sibling(&self, object: u16) -> u16 {
        let location = self.object_location(object)
            + self.object_attribute_length()
            + self.object_relation_length();
        match self.version() {
            1..=3 => self.get_byte(location as usize) as u16,
            _ => self.get_word(location as usize),
        }
    }

    pub fn object_child(&self, object: u16) -> u16 {
        let location = self.object_location(object)
            + self.object_attribute_length()
            + (self.object_relation_length() * 2);
        match self.version() {
            1..=3 => self.get_byte(location as usize) as u16,
            _ => self.get_word(location as usize),
        }
    }

    pub fn object_properties_table_location(&self, object: u16) -> u16 {
        let address = self.object_location(object)
            + self.object_attribute_length()
            + (3 * self.object_relation_length());
        self.get_word(address as usize)
    }

    pub fn object_short_name(&self, object: u16) -> Result<String, Box<dyn Error>> {
        Ok(self
            .extract_string(
                self.object_properties_table_location(object) as usize + 1,
                true,
            )?
            .0)
    }

    pub fn property(&self, object: u16, property: u16) -> Option<Property> {
        let mut cursor = self.object_properties_table_location(object) as usize;
        let short_name_length = self.read_byte(&mut cursor) * 2;
        cursor += short_name_length as usize;
        match self.version() {
            1..=3 => loop {
                let address = cursor as u16;
                let size_byte = self.read_byte(&mut cursor);
                if size_byte == 0 {
                    return None;
                }
                let data_address = cursor as u16;
                let data_length = (size_byte + 1) / 32;
                let property_number = (size_byte + 1) % 32;
                if property_number as u16 == property {
                    return Some(Property {
                        address,
                        data_address,
                        data: self.get_bytes(data_address as usize, data_length as usize),
                    });
                }
                cursor += data_length as usize;
            },
            _ => loop {
                let address = cursor as u16;
                let size_byte = self.read_byte(&mut cursor);
                let mut data_address = cursor as u16;
                let property_number = size_byte & 0b111111;
                if property_number == 0 {
                    return None;
                }
                let has_second_size_byte = size_byte >> 7 != 0;
                let mut data_length;
                if has_second_size_byte {
                    data_address += 1;
                    data_length = self.get_byte(cursor) & 0b111111;
                    if data_length == 0 {
                        data_length = 64;
                    }
                } else {
                    data_length = (size_byte >> 6) + 1;
                }
                if property_number as u16 == property {
                    return Some(Property {
                        address,
                        data_address,
                        data: self.get_bytes(data_address as usize, data_length as usize),
                    });
                }
                cursor += data_length as usize;
            },
        }
    }

    /// Retrieve the location of an abbreviation from the abbreviation tables(s)
    pub fn abbreviation_entry(&self, table: usize, index: usize) -> usize {
        usize::from(
            self.get_word(
                self.abbreviation_table_location() as usize + (32 * (table - 1) + index) * 2,
            ) * 2,
        )
    }

    /// Retrieve a global variable.
    pub fn get_global(&self, number: u8) -> u16 {
        self.get_word(self.global_variable_table_location() as usize + 2 * number as usize)
    }

    /// Set a global variable.
    pub fn set_global(&mut self, number: u8, value: u16) {
        self.set_word(
            self.global_variable_table_location() as usize + 2 * number as usize,
            value,
        );
    }

    /// Retrieve the alphabet table from memory. Should only be used if the game has declared it
    /// includes an alternate alphabet table.
    fn alphabet(&self) -> Alphabet {
        Alphabet::new(
            self.alphabet_table(AlphabetTable::A0),
            self.alphabet_table(AlphabetTable::A1),
            self.alphabet_table(AlphabetTable::A2),
            self.unicode_translation_table(),
        )
    }

    /// Decode a Z-Character-encoded string, strating at the given point in memory.
    pub fn extract_string(
        &self,
        start: usize,
        abbreviations: bool,
    ) -> Result<(String, usize), Box<dyn Error>> {
        let sequence = self.character_sequence(start);
        let byte_length = sequence.len() / 3 * 2;
        let mut sequence = sequence.iter();
        let mut result = Vec::new();
        let mut shift = false;
        let alphabet = match self.alphabet_table_location() {
            0 => Alphabet::default(self.version(), self.unicode_translation_table()),
            _ => self.alphabet(),
        };
        let mut table = AlphabetTable::default();
        while let Some(c) = sequence.next() {
            match c {
                0 => result.push(' '),
                1..=3 if (self.version() >= 3 || *c == 1) => {
                    if !abbreviations {
                        return Err(GameError::InvalidOperation(
                            "Found abbreviation within an abbreviation".into(),
                        )
                        .into());
                    }
                    if let Some(abbreviation_id) = sequence.next() {
                        let abbreviation: String = self
                            .extract_string(
                                self.abbreviation_entry(*c as usize, *abbreviation_id as usize),
                                false,
                            )?
                            .0;
                        result.append(&mut abbreviation.chars().collect());
                    } else {
                        return Err(GameError::InvalidOperation(
                            "String ended unexpectedly".into(),
                        )
                        .into());
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
                    if table == AlphabetTable::A2 && *c == 6 {
                        // Character 6 in Alphabet 2 indicates a 10-bit ZSCII character follows.
                        let b1 = *sequence.next().ok_or_else(|| {
                            GameError::InvalidOperation("String ended unexpectedly".into())
                        })?;
                        let b2 = *sequence.next().ok_or_else(|| {
                            GameError::InvalidOperation("String ended unexpectedly".into())
                        })?;

                        let zscii_code = ((b1 as u16) << 5) | (b2 as u16);
                        if let Some(character) = alphabet.decode_zscii(zscii_code)? {
                            result.push(character);
                        }
                    } else {
                        // Otherwise, the character is found in the alphabet table.
                        result.push(alphabet.value(table, *c));
                    }
                    if shift {
                        table = AlphabetTable::default();
                        shift = false;
                    }
                }
            }
        }
        Ok((result.iter().collect(), byte_length))
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
