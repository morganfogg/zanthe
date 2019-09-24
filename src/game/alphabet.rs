const ALPHABET_0: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_1: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHABET_2: &[u8; 26] = b"@\n0123456789.,!?_#'\"/\\-:()";
const ALPHABET_2_V1: &[u8; 26] = b"@0123456789.,!?_#'\"/\\<-:()";

#[derive(Copy, Clone)]
pub enum AlphabetTable {
    A0 = 0,
    A1 = 1,
    A2 = 2,
}

pub struct Alphabet<'a> {
    a0: &'a [u8],
    a1: &'a [u8],
    a2: &'a [u8],
}

impl<'a> Alphabet<'a> {
    pub fn new(a0: &'a [u8], a1: &'a [u8], a2: &'a [u8]) -> Self {
        Self { a0, a1, a2 }
    }

    pub fn default(version: u8) -> Self {
        Self {
            a0: ALPHABET_0,
            a1: ALPHABET_1,
            a2: match version {
                1 => ALPHABET_2_V1,
                _ => ALPHABET_2,
            },
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
}

impl AlphabetTable {
    pub fn next(&mut self) -> Self {
        match self {
            AlphabetTable::A0 => AlphabetTable::A1,
            AlphabetTable::A1 => AlphabetTable::A2,
            AlphabetTable::A2 => AlphabetTable::A0,
        }
    }

    pub fn previous(&mut self) -> Self {
        match self {
            AlphabetTable::A0 => AlphabetTable::A2,
            AlphabetTable::A1 => AlphabetTable::A0,
            AlphabetTable::A2 => AlphabetTable::A1,
        }
    }
    pub fn default() -> Self {
        AlphabetTable::A0
    }
}
