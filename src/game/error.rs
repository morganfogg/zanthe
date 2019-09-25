use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

pub enum GameError {
    VersionSix,
    InvalidFile,
    InvalidData(String),
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                GameError::VersionSix => "Version 6 story files are not supported".to_string(),
                GameError::InvalidFile => {
                    "The file you have specified is not a supported Z-Code file".to_string()
                }
                GameError::InvalidData(e) => {
                    format!("Error while reading game data: {}", e)
                }
            }
        )
    }
}

impl Debug for GameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Error for GameError {}
