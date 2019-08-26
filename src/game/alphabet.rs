const ALPHABET_0: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_1: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHABET_2: &[u8] = b"@\n0123456789.,!?_#'\"/\\-:()";
const ALPHABET_2_V1: &[u8] = b"@0123456789.,!?_#'\"/\\<-:()";

pub enum Alphabet {
    A0,
    A1,
    A2,
}

impl Alphabet {
    fn value(&self, version: u8) -> &[u8] {
        match self {
            Alphabet::A0 => ALPHABET_0,
            Alphabet::A1 => ALPHABET_1,
            Alphabet::A2 => match version {
                1 => ALPHABET_2_V1,
                _ => ALPHABET_2,
            },
        }
    }
    
    pub fn character(&self, version: u8, i: u8) -> char {
        self.value(version)[i as usize] as char
    }

    pub fn next(&self) -> Self {
        match self {
            Alphabet::A0 => Alphabet::A1,
            Alphabet::A1 => Alphabet::A2,
            Alphabet::A2 => Alphabet::A0,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Alphabet::A0 => Alphabet::A2,
            Alphabet::A1 => Alphabet::A0,
            Alphabet::A2 => Alphabet::A1,
        }
    }
}
