use std::convert::TryFrom;

use anyhow::Result;

use crate::game::error::GameError;
use crate::game::InputCode;

const ALPHABET_0: &[char; 26] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const ALPHABET_1: &[char; 26] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const ALPHABET_2: &[char; 26] = &[
    '@', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '!', '?', '_', '#',
    '\'', '"', '/', '\\', '-', ':', '(', ')',
];
const ALPHABET_2_V1: &[char; 26] = &[
    '@', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '!', '?', '_', '#', '\'', '"',
    '/', '\\', '<', '-', ':', '(', ')',
];

pub const DEFAULT_UNICODE_TABLE: &[char; 69] = &[
    'ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß', '»', '«', 'ë', 'ï', 'ÿ', 'Ë', 'Ï', 'á', 'é', 'í', 'ó', 'ú',
    'ý', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ý', 'à', 'è', 'ì', 'ò', 'ù', 'À', 'È', 'Ì', 'Ò', 'Ù', 'â', 'ê',
    'î', 'ô', 'û', 'Â', 'Ê', 'Î', 'Ô', 'Û', 'å', 'Å', 'ø', 'Ø', 'ã', 'ñ', 'õ', 'Ã', 'Ñ', 'Õ', 'æ',
    'Æ', 'ç', 'Ç', 'þ', 'ð', 'Þ', 'Ð', '£', 'œ', 'Œ', '¡', '¿',
];

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AlphabetTable {
    A0,
    A1,
    A2,
}

/// Represents the alphabet used by the game's string parser.
pub struct Alphabet {
    a0: Vec<char>,
    a1: Vec<char>,
    a2: Vec<char>,
    unicode_table: Option<Vec<char>>,
}

impl Alphabet {
    pub fn new(a0: &[u8], a1: &[u8], a2: &[u8], unicode_table: Option<Vec<char>>) -> Alphabet {
        Alphabet {
            a0: a0.iter().map(|x| *x as char).collect(),
            a1: a1.iter().map(|x| *x as char).collect(),
            a2: a2.iter().map(|x| *x as char).collect(),
            unicode_table,
        }
    }

    pub fn default(version: u8, unicode_table: Option<Vec<char>>) -> Alphabet {
        Alphabet {
            a0: ALPHABET_0.to_vec(),
            a1: ALPHABET_1.to_vec(),
            a2: match version {
                1 => ALPHABET_2_V1.to_vec(),
                _ => ALPHABET_2.to_vec(),
            },
            unicode_table,
        }
    }

    fn unicode_table(&self) -> &[char] {
        match &self.unicode_table {
            None => DEFAULT_UNICODE_TABLE,
            Some(table) => table,
        }
    }

    pub fn value(&self, table: AlphabetTable, char: u8) -> char {
        (match table {
            AlphabetTable::A0 => &self.a0,
            AlphabetTable::A1 => &self.a1,
            AlphabetTable::A2 => &self.a2,
        })[char as usize - 6]
    }

    pub fn encode_zchar(&self, c: char) -> Option<(u8, AlphabetTable)> {
        if let Some(v) = self.a0.iter().position(|&x| x == c) {
            Some((v as u8 + 6, AlphabetTable::A0))
        } else if let Some(v) = self.a1.iter().position(|&x| x == c) {
            Some((v as u8 + 6, AlphabetTable::A1))
        } else {
            self.a2
                .iter()
                .position(|&x| x == c)
                .map(|v| (v as u8 + 6, AlphabetTable::A2))
        }
    }

    /// Transform a ZSCII output code into a char.
    pub fn decode_zscii(&self, value: u16) -> Result<Option<char>> {
        match value {
            0 => Ok(None),
            13 => Ok(Some('\n')),
            32..=126 => Ok(Some(char::try_from(value as u32)?)),
            c @ 155..=251 => Ok(Some(self.unicode_table()[c as usize - 155])),
            _ => Err(GameError::InvalidOperation("Invalid ZSCII sequence".into()).into()),
        }
    }

    /// Transform a character into a ZSCII code
    pub fn zscii_from_char(&self, value: char) -> Result<u8> {
        let codepoint = value as u32;

        if (32..=126).contains(&codepoint) {
            Ok(codepoint as u8)
        } else if value == '\n' {
            Ok(13)
        } else if let Some(p) = self.unicode_table().iter().position(|&x| x == value) {
            Ok(p as u8 + 155)
        } else {
            Err(GameError::InvalidOperation("Invalid input character".into()).into())
        }
    }

    pub fn zscii_from_code(&self, value: InputCode) -> Result<u8> {
        use InputCode::*;
        match value {
            Delete => Ok(8),
            Newline => Ok(13),
            Escape => Ok(27),
            CursorUp => Ok(129),
            CursorDown => Ok(130),
            CursorLeft => Ok(131),
            CursorRight => Ok(132),
            Character(c) => self.zscii_from_char(c),
        }
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
