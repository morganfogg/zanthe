const ALPHABET_0: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_1: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHABET_2: &[u8] = b"@\n0123456789.,!?_#'\"/\\-:()";
const ALPHABET_2_V1: &[u8] = b"@0123456789.,!?_#'\"/\\<-:()";

enum AlphabetTable {
    A0,
    A1,
    A2,
}

pub struct Alphabet {
    version: u8,
    active: AlphabetTable,
}

impl Alphabet {
    pub fn new(version: u8) -> Self {
        Alphabet {
            version,
            active: AlphabetTable::A0,
        }
    }

    fn value(&self) -> &[u8] {
        match self.active {
            AlphabetTable::A0 => ALPHABET_0,
            AlphabetTable::A1 => ALPHABET_1,
            AlphabetTable::A2 => match self.version {
                1 => ALPHABET_2_V1,
                _ => ALPHABET_2,
            },
        }
    }

    pub fn character(&self, i: u8) -> char {
        self.value()[i as usize - 6] as char
    }

    pub fn next(&mut self) {
        self.active = match self.active {
            AlphabetTable::A0 => AlphabetTable::A1,
            AlphabetTable::A1 => AlphabetTable::A2,
            AlphabetTable::A2 => AlphabetTable::A0,
        }
    }

    pub fn previous(&mut self) {
        self.active = match self.active {
            AlphabetTable::A0 => AlphabetTable::A2,
            AlphabetTable::A1 => AlphabetTable::A0,
            AlphabetTable::A2 => AlphabetTable::A1,
        }
    }
    pub fn default(&mut self) {
        self.active = AlphabetTable::A0;
    }
}
