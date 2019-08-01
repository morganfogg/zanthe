use std::fmt::{self, Display, Formatter};

pub enum GameError {
    VersionSix,
    InvalidFile,
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                GameError::VersionSix => "Version 6 story files are not supported",
                GameError::InvalidFile => {
                    "The file you have specified is not a supported Z-Code file"
                }
            }
        )
    }
}
