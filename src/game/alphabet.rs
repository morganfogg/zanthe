use crate::game::error::GameError;
use std::convert::TryFrom;
use std::error::Error;

const ALPHABET_0: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_1: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHABET_2: &[u8; 26] = b"@\n0123456789.,!?_#'\"/\\-:()";
const ALPHABET_2_V1: &[u8; 26] = b"@0123456789.,!?_#'\"/\\<-:()";

pub const DEFAULT_UNICODE_TABLE: &[char; 69] = &[
    'ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß', '»', '«', 'ë', 'ï', 'ÿ', 'Ë', 'Ï', 'á', 'é', 'í', 'ó', 'ú',
    'ý', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ý', 'à', 'è', 'ì', 'ò', 'ù', 'À', 'È', 'Ì', 'Ò', 'Ù', 'â', 'ê',
    'î', 'ô', 'û', 'Â', 'Ê', 'Î', 'Ô', 'Û', 'å', 'Å', 'ø', 'Ø', 'ã', 'ñ', 'õ', 'Ã', 'Ñ', 'Õ', 'æ',
    'Æ', 'ç', 'Ç', 'þ', 'ð', 'Þ', 'Ð', '£', 'œ', 'Œ', '¡', '¿',
];

#[derive(Copy, Clone, PartialEq)]
pub enum AlphabetTable {
    A0,
    A1,
    A2,
}

/// Represents the alphabet used by the game's string parser.
pub struct Alphabet<'a> {
    a0: &'a [u8],
    a1: &'a [u8],
    a2: &'a [u8],
    unicode_table: Option<Vec<char>>,
}

impl<'a> Alphabet<'a> {
    pub fn new(
        a0: &'a [u8],
        a1: &'a [u8],
        a2: &'a [u8],
        unicode_table: Option<Vec<char>>,
    ) -> Alphabet<'a> {
        Alphabet {
            a0,
            a1,
            a2,
            unicode_table,
        }
    }

    pub fn default(version: u8, unicode_table: Option<Vec<char>>) -> Alphabet<'a> {
        Alphabet {
            a0: ALPHABET_0,
            a1: ALPHABET_1,
            a2: match version {
                1 => ALPHABET_2_V1,
                _ => ALPHABET_2,
            },
            unicode_table,
        }
    }

    fn unicode_table(&self) -> &[char] {
        match &self.unicode_table {
            None => DEFAULT_UNICODE_TABLE,
            Some(table) => &table,
        }
    }

    pub fn value(&self, table: AlphabetTable, char: u8) -> char {
        (match table {
            AlphabetTable::A0 => self.a0,
            AlphabetTable::A1 => self.a1,
            AlphabetTable::A2 => self.a2,
        })[char as usize - 6]
            .into()
    }

    pub fn encode_zchar(&self, c: char) -> Option<(u8, AlphabetTable)> {
        match u8::try_from(c as u32) {
            Err(_) => None,
            Ok(c) => {
                if let Some(v) = self.a0.iter().position(|&x| x == c) {
                    Some((v as u8 + 6, AlphabetTable::A0))
                } else if let Some(v) = self.a1.iter().position(|&x| x == c) {
                    Some((v as u8 + 6, AlphabetTable::A1))
                } else if let Some(v) = self.a2.iter().position(|&x| x == c) {
                    Some((v as u8 + 6, AlphabetTable::A2))
                } else {
                    None
                }
            }
        }
    }

    /// Transform a ZSCII output code into a char.
    pub fn decode_zscii(&self, value: u16) -> Result<Option<char>, Box<dyn Error>> {
        match value {
            0 => Ok(None),
            13 => Ok(Some('\n')),
            32..=126 => Ok(Some(char::try_from(value as u32)?)),
            c @ 155..=251 => Ok(Some(self.unicode_table()[c as usize - 155])),
            _ => Err(GameError::InvalidOperation("Invalid ZSCII sequence".into()).into()),
        }
    }

    /// Transform a character into a ZSCII code
    pub fn encode_zscii(&self, value: char) -> Result<u16, Box<dyn Error>> {
        let codepoint = value as u32;

        if codepoint >= 32 && codepoint <= 126 {
            return Ok(codepoint as u16);
        }
        if value == '\n' {
            return Ok(13);
        }
        if let Some(p) = self.unicode_table().iter().position(|&x| x == value) {
            return Ok(p as u16);
        }
        return Err(GameError::InvalidOperation("Invalid input character".into()).into());
    }
}

impl AlphabetTable {
    pub fn next(&mut self) -> AlphabetTable {
        match self {
            AlphabetTable::A0 => AlphabetTable::A1,
            AlphabetTable::A1 => AlphabetTable::A2,
            AlphabetTable::A2 => AlphabetTable::A0,
        }
    }

    pub fn previous(&mut self) -> AlphabetTable {
        match self {
            AlphabetTable::A0 => AlphabetTable::A2,
            AlphabetTable::A1 => AlphabetTable::A0,
            AlphabetTable::A2 => AlphabetTable::A1,
        }
    }
    pub fn default() -> AlphabetTable {
        AlphabetTable::A0
    }
}
