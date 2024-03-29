use std::char;
use std::convert::TryInto;
use std::iter::successors;

use crate::game::Result;
use tracing::{error, info, warn};

use crate::game::address;
use crate::game::alphabet::{Alphabet, AlphabetTable};
use crate::game::error::GameError;
use crate::game::instruction::Operand;
use crate::game::property::Property;
use crate::game::InputCode;

/// Represents the game's internal memory.
#[derive(Clone)]
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
        self.data[start..start + length].to_vec()
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

    // Update a series of bytes in memory.
    pub fn set_bytes(&mut self, address: usize, bytes: &[u8]) {
        for (dest, src) in self.data[address..].iter_mut().zip(bytes.iter()) {
            *dest = *src;
        }
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
    pub fn read_string(&mut self, cursor: &mut usize) -> Result<String> {
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
    fn dictionary_location(&self) -> usize {
        self.get_word(address::DICTIONARY_LOCATION).into()
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
            addr => {
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

    fn flag(&self, mut address: usize, mut bit: u16) -> bool {
        if bit >= 8 {
            address += 1;
            bit -= 8;
        }
        (self.data[address] & (1 << bit)) != 0
    }

    fn set_flag(&mut self, mut address: usize, mut bit: u16, set: bool) {
        if bit >= 8 {
            address += 1;
            bit -= 8;
        }
        if set {
            self.data[address] |= 1 << bit;
        } else {
            self.data[address] &= !(1 << bit);
        }
    }

    pub fn transcribing(&self) -> bool {
        self.flag(address::FLAGS_2, address::flags2::TRANSCRIPTING_ON)
    }

    pub fn set_transcribing(&mut self, value: bool) {
        self.set_flag(address::FLAGS_2, address::flags2::TRANSCRIPTING_ON, value);
    }

    pub fn force_fixed_font(&self) -> bool {
        self.flag(address::FLAGS_2, address::flags2::FORCE_FIXED_PITCH)
    }

    pub fn set_force_fixed_font(&mut self, value: bool) {
        self.set_flag(address::FLAGS_2, address::flags2::FORCE_FIXED_PITCH, value);
    }

    /// Set universal headers
    pub fn set_general_headers(&mut self) {
        if self.version() < 4 {
            use address::flags1_bits_pre_v4::*;
            self.set_flag(address::FLAGS_1, STATUS_LINE_UNAVAILABLE, true);
            self.set_flag(address::FLAGS_1, SCREEN_SPLITTING_AVAILABLE, true);
            self.set_flag(address::FLAGS_1, VARIABLE_PITCH_FONT_DEFAULT, true);
        } else {
            use address::flags1_bits_post_v4::*;
            self.set_flag(address::FLAGS_1, COLOR_AVAILABLE, true);
            self.set_flag(address::FLAGS_1, PICTURE_DISPLAYING_AVAILABLE, false);
            self.set_flag(address::FLAGS_1, BOLD_AVAILABLE, true);
            self.set_flag(address::FLAGS_1, ITALICS_AVAILABLE, true);
            self.set_flag(address::FLAGS_1, FIXED_WIDTH_AVAILABLE, true);
            self.set_flag(address::FLAGS_1, SOUND_EFFECTS_AVAILABLE, false);
            self.set_flag(address::FLAGS_1, TIMED_INPUT_AVAILABLE, false);
        }
        use address::flags2::*;
        self.set_flag(address::FLAGS_2, TRANSCRIPTING_ON, false);
        self.set_flag(address::FLAGS_2, PICTURE_SUPPORT, false);
        self.set_flag(address::FLAGS_2, UNDO_SUPPORT, true);
        self.set_flag(address::FLAGS_2, MOUSE_SUPPORT, false);
        self.set_flag(address::FLAGS_2, COLOR_SUPPORT, false);
        self.set_flag(address::FLAGS_2, SOUND_EFFECT_SUPPORT, false);
        self.set_flag(address::FLAGS_2, MENU_SUPPORT, false)
    }

    /// Set the screen size headers
    pub fn set_screen_size(&mut self, width: u16, height: u16) {
        if self.version() >= 5 {
            self.set_word(address::SCREEN_WIDTH_UNITS, width);
            self.set_word(address::SCREEN_HEIGHT_UNITS, height);
        }
        if self.version() >= 4 {
            self.set_byte(address::SCREEN_WIDTH_CHARS, width.try_into().unwrap_or(255));
            self.set_byte(
                address::SCREEN_HEIGHT_CHARS,
                height.try_into().unwrap_or(255),
            );
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

    /// Remove the given object's parent and reflow its siblings
    pub fn detach_object(&mut self, object_id: u16) {
        let parent = self.object_parent(object_id);
        let next_sibling = self.object_sibling(object_id);

        if parent == 0 {
            return;
        }

        self.set_object_parent(object_id, 0);
        self.set_object_sibling(object_id, 0);

        let first_child = self.object_child(parent);
        if first_child == object_id {
            self.set_object_child(parent, next_sibling);
        } else {
            let mut previous_sibling = first_child;
            loop {
                if object_id == self.object_sibling(previous_sibling) {
                    break;
                }
                previous_sibling = self.object_sibling(previous_sibling);
            }
            self.set_object_sibling(previous_sibling, next_sibling);
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

    fn object_relation(&self, location: usize) -> u16 {
        match self.version() {
            1..=3 => self.get_byte(location) as u16,
            _ => self.get_word(location),
        }
    }

    fn set_object_relation(&mut self, location: usize, value: u16) {
        match self.version() {
            1..=3 => self.set_byte(location, value as u8),
            _ => self.set_word(location, value),
        };
    }

    fn object_parent_id_location(&self, object: u16) -> u16 {
        self.object_location(object) + self.object_attribute_length()
    }

    pub fn object_parent(&self, object: u16) -> u16 {
        let location = self.object_parent_id_location(object);
        self.object_relation(location as usize)
    }

    pub fn set_object_parent(&mut self, object: u16, parent: u16) {
        let location = self.object_parent_id_location(object);
        self.set_object_relation(location as usize, parent);
    }

    fn object_sibling_id_location(&self, object: u16) -> u16 {
        self.object_location(object)
            + self.object_attribute_length()
            + self.object_relation_length()
    }

    pub fn object_sibling(&self, object: u16) -> u16 {
        let location = self.object_sibling_id_location(object);
        self.object_relation(location as usize)
    }

    pub fn set_object_sibling(&mut self, object: u16, sibling: u16) {
        let location = self.object_sibling_id_location(object);
        self.set_object_relation(location as usize, sibling);
    }

    fn object_child_id_location(&self, object: u16) -> u16 {
        self.object_location(object)
            + self.object_attribute_length()
            + (self.object_relation_length() * 2)
    }

    pub fn object_child(&self, object: u16) -> u16 {
        let location = self.object_child_id_location(object);
        self.object_relation(location as usize)
    }

    pub fn set_object_child(&mut self, object: u16, child: u16) {
        let location = self.object_child_id_location(object);
        self.set_object_relation(location as usize, child);
    }

    pub fn object_properties_table_location(&self, object: u16) -> u16 {
        let address = self.object_location(object)
            + self.object_attribute_length()
            + (3 * self.object_relation_length());
        self.get_word(address as usize)
    }

    pub fn object_short_name(&self, object: u16) -> Result<String> {
        Ok(self
            .extract_string(
                self.object_properties_table_location(object) as usize + 1,
                true,
            )?
            .0)
    }

    pub fn default_property(&self, property: u16) -> u16 {
        let offset = (property as usize - 1) * 2;
        self.get_word(self.object_table_location() as usize + offset)
    }

    fn property_at_address(&self, address: usize) -> Option<Property> {
        match self.version() {
            1..=3 => {
                let mut cursor = address as usize;
                let size_byte = self.read_byte(&mut cursor);
                if size_byte == 0 {
                    return None;
                }

                let data_address = cursor as u16;
                let data_length = (size_byte + 1) / 32;
                let property_number = (size_byte + 1) % 32;

                Some(Property {
                    number: property_number as u16,
                    address: address as u16,
                    data_address,
                    data: self.get_bytes(data_address as usize, data_length as usize),
                })
            }
            _ => {
                let mut cursor = address as usize;
                let size_byte = self.read_byte(&mut cursor);
                let mut data_address = cursor as u16;
                let property_number = size_byte & 0b11_1111;
                if property_number == 0 {
                    return None;
                }

                let has_second_size_byte = size_byte >> 7 != 0;
                let mut data_length;
                if has_second_size_byte {
                    data_address += 1;
                    data_length = self.get_byte(cursor) & 0b11_1111;
                    if data_length == 0 {
                        data_length = 64;
                    }
                } else {
                    data_length = (size_byte >> 6) + 1;
                }

                Some(Property {
                    number: property_number as u16,
                    address: address as u16,
                    data_address,
                    data: self.get_bytes(data_address as usize, data_length as usize),
                })
            }
        }
    }

    /// Get the length (in bytes) of the property data at a given address.
    pub fn property_data_length(&self, data_addr: usize) -> u16 {
        let size_byte = self.get_byte(data_addr - 1);
        if (size_byte >> 7) == 1 {
            let length = size_byte as u16 & 0b11_1111;
            if length == 0 {
                64
            } else {
                length
            }
        } else if ((size_byte >> 6) & 1) == 1 {
            2
        } else {
            1
        }
    }

    pub fn property_iter(&self, object: u16) -> impl Iterator<Item = Property> + '_ {
        let mut cursor = self.object_properties_table_location(object) as usize;
        let short_name_length = self.read_byte(&mut cursor) as usize * 2;
        cursor += short_name_length;

        successors(self.property_at_address(cursor), move |p| {
            self.property_at_address(p.data_address as usize + p.data.len())
        })
    }

    pub fn property(&self, object: u16, number: u16) -> Option<Property> {
        self.property_iter(object).find(|p| p.number == number)
    }

    pub fn following_property(&self, object: u16, number: u16) -> Option<Property> {
        self.property_iter(object)
            .skip_while(|p| p.number != number)
            .nth(1)
    }

    pub fn word_separators(&self) -> Result<Vec<char>> {
        let alphabet = self.alphabet();
        let mut cursor = self.dictionary_location();
        let count = self.read_byte(&mut cursor);
        let mut result = Vec::new();
        for _ in 0..count {
            let c = alphabet
                .decode_zscii(self.read_byte(&mut cursor).into())?
                .ok_or_else(|| GameError::invalid_operation("Invalid word separator"))?;
            result.push(c);
        }
        Ok(result)
    }

    fn dictionary(&self) -> Result<Vec<(usize, String)>> {
        let mut cursor = self.dictionary_location();
        let separator_count = self.read_byte(&mut cursor) as usize;
        cursor += separator_count;

        let entry_length = self.read_byte(&mut cursor) as usize;
        let entry_count = self.read_word(&mut cursor) as usize;

        let mut result = Vec::new();

        for i in 0..entry_count {
            let address = cursor + (i * entry_length);
            let text = self.extract_string(address, true)?.0;
            result.push((address, text));
        }
        Ok(result)
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

    /// Retrieve the alphabet table from memory.
    pub fn alphabet(&self) -> Alphabet {
        match self.alphabet_table_location() {
            0 => Alphabet::default(self.version(), self.unicode_translation_table()),
            _ => Alphabet::new(
                self.alphabet_table(AlphabetTable::A0),
                self.alphabet_table(AlphabetTable::A1),
                self.alphabet_table(AlphabetTable::A2),
                self.unicode_translation_table(),
            ),
        }
    }

    pub fn read_input_array(&self, mut start: usize) -> Result<String> {
        let mut output = String::new();
        let alphabet = self.alphabet();
        let version = self.version();
        if version >= 5 {
            start += 1;
            let num_chars = self.read_byte(&mut start);
            for _ in 0..num_chars {
                let b = self.read_byte(&mut start);
                let c = alphabet.decode_zscii(b.into())?.unwrap();
                output.push(c);
            }
        } else {
            loop {
                let b = self.read_byte(&mut start);
                if b == 0 {
                    break;
                }
                let c = alphabet.decode_zscii(b.into())?.unwrap();
                output.push(c);
            }
        }
        Ok(output)
    }

    pub fn write_input_array(&mut self, mut start: usize, text: &str) -> Result<()> {
        let alphabet = self.alphabet();
        if self.version() >= 5 {
            // Advance past the 'expected number of input characters'
            start += 1;
            let existing = self.get_byte(start) as i8;
            if existing > 0 {
                start += existing as usize;
            }
            self.write_byte(&mut start, text.chars().count() as u8);
        }
        for c in text.chars() {
            self.write_byte(&mut start, alphabet.zscii_from_char(c)?);
        }
        if self.version() < 5 {
            self.write_byte(&mut start, 0);
        }
        Ok(())
    }

    /// Decode a Z-Character-encoded string, starting at the given point in memory.
    pub fn extract_string(&self, start: usize, abbreviations: bool) -> Result<(String, usize)> {
        let sequence = self.character_sequence(start);
        let byte_length = sequence.len() / 3 * 2;
        let mut sequence = sequence.iter();
        let mut result = Vec::new();
        let mut shift = false;
        let alphabet = self.alphabet();
        let mut table = AlphabetTable::default();
        while let Some(c) = sequence.next() {
            match c {
                0 => result.push(' '),
                1..=3 if (self.version() >= 3 || *c == 1) => {
                    if !abbreviations {
                        return Err(GameError::invalid_operation(
                            "Found abbreviation within an abbreviation",
                        ));
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
                        return Err(GameError::invalid_operation("String ended unexpectedly"));
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
                            GameError::invalid_operation("String ended unexpectedly")
                        })?;
                        let b2 = *sequence.next().ok_or_else(|| {
                            GameError::invalid_operation("String ended unexpectedly")
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

    pub fn zscii_from_code(&self, code: InputCode) -> Result<u8> {
        self.alphabet().zscii_from_code(code)
    }

    pub fn parse_string(&mut self, mut cursor: usize, text: &str, max_words: usize) -> Result<()> {
        let separators = self.word_separators()?;
        let mut new_word = true;
        let words = text
            .chars()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, cur)| {
                if cur == ' ' {
                    new_word = true;
                    return acc;
                }
                if separators.contains(&cur) {
                    acc.push((i, cur.to_string()));
                    new_word = true;
                } else if new_word {
                    new_word = false;
                    acc.push((i, cur.to_string()))
                } else {
                    let len = acc.len();
                    acc[len - 1].1.push(cur);
                }
                acc
            });

        let words = words.iter().take(max_words);

        let dictionary = self.dictionary()?;
        cursor += 1;
        self.write_byte(&mut cursor, words.len() as u8);

        for (i, word) in words {
            let dictionary_entry = dictionary.iter().find(|e| &e.1 == word);
            let dictionary_address = dictionary_entry.map(|e| e.0 as u16).unwrap_or(0);
            let chars = word.chars().count();
            let buffer_offset = i + 1;

            self.write_word(&mut cursor, dictionary_address);
            self.write_byte(&mut cursor, chars as u8);
            self.write_byte(&mut cursor, buffer_offset as u8);
        }

        Ok(())
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
        let result: usize = self.data[0x40..file_length]
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
    pub fn validate_header(&self) -> Result<()> {
        let len = self.data.len();
        if len < 64 {
            // Header alone must be at least 64 bytes long
            error!("File too small to be valid");
            return Err(GameError::invalid_file());
        }

        if self.version() == 6 {
            error!("Version 6 file provided");
            return Err(GameError::version_six());
        }

        if self.version() > 8 || self.version() == 0 {
            // Version byte is outside expected/supported range
            error!("Invalid version byte");
            return Err(GameError::invalid_file());
        }
        if len > self.max_file_length() {
            // File is too large for its version
            error!("Invalid file size");
            return Err(GameError::invalid_file());
        }

        let static_memory_base: usize = self.static_memory_base().into();
        if static_memory_base < 64 || static_memory_base > len - 1 {
            error!("Invalid static memory base");
            return Err(GameError::invalid_file());
        }

        let high_memory_base: usize = self.high_memory_base().into();
        if high_memory_base < 64
            || high_memory_base > len - 1
            || high_memory_base <= static_memory_base
        {
            error!("Invalid high memory base");
            return Err(GameError::invalid_file());
        }

        let program_counter_starts: usize = self.program_counter_starts().into();
        if program_counter_starts < high_memory_base {
            error!("Program counter does not start in high memory");
            return Err(GameError::invalid_file());
        }
        info!("Header validation OKAY");
        info!(
            "Static Base: {:x}. High base: {:x}. PC starts: {:x}",
            static_memory_base, high_memory_base, program_counter_starts,
        );
        Ok(())
    }
}
